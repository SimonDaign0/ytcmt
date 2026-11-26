use futures::future::join_all;
use tokio::task::{JoinHandle, JoinSet};
use wreq::Client;
use wreq_util::Emulation;

use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use crate::schema::reply::ReplyData;
use crate::schema::{comments_extension::ExtensionData, comments_section::SectionData};

use crate::error::Error;

pub async fn req_comments(video_id: &str, max_pages: usize) -> Result<(), Error> {
    let client = Client::builder().emulation(Emulation::Firefox139).build()?;
    let video_id = Arc::from(video_id);

    let html = get_html_page(client.clone(), &video_id).await?;
    let innertube_key = extract_innertube_key(&html).ok_or(Error::ParsingError)?;

    let json_slice = parse_initial_data(html.as_str()).ok_or(Error::ParsingError)?;
    let cmt_section = serde_json::from_str::<SectionData>(json_slice)?;

    let (tx, rx) = mpsc::channel::<String>();
    let w_handle = writer(rx);

    let endpoint_url: Arc<str> = format!(
        "https://www.youtube.com/youtubei/v1/next?key={}",
        innertube_key,
    )
    .into();

    let current_token = cmt_section.token().map(|s| s.to_string());
    process_tokens(current_token, tx, client, max_pages, endpoint_url, video_id).await?;
    let _ = w_handle.await;
    Ok(())
}

fn extract_innertube_key<'a>(html: &'a str) -> Option<&'a str> {
    let key = r#"INNERTUBE_API_KEY":"#;
    let start = html.find(key)? + key.len();
    let len = html.get(start..)?.find('"')?;
    html.get(start..start + len)
}

async fn call_endpoint<'a>(
    client: Client,
    url: &str,
    video_id: &str,
    token: &'a str,
) -> Result<String, Error> {
    let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let payload = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20261116.00.00",
                "hl": "en",
                "gl": "US"
            }
        },
        "videoId": video_id,
        "continuation": token
    });

    let res = client
        .post(url)
        .header("Referer", watch_url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let json = res.text().await?;
    Ok(json)
}

async fn get_html_page(client: Client, video_id: &str) -> Result<String, Error> {
    let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);

    let html = client.get(watch_url.as_str()).send().await?.text().await?;

    Ok(html)
}

/// Extracts and parses `ytInitialData` from raw HTML, ignoring trailing garbage.
fn parse_initial_data<'a>(html: &'a str) -> Option<&'a str> {
    let obj_key = "var ytInitialData = ";
    let brace_start = html.find(obj_key)? + obj_key.len();

    wrap_brace(html.get(brace_start..)?)
}
fn wrap_brace<'a>(slice: &'a str) -> Option<&'a str> {
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in slice.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' if in_string => {
                escape = true;
            }
            '"' => {
                in_string = !in_string;
            }
            '{' if !in_string => {
                brace_count += 1;
            }
            '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                    return slice.get(..i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

async fn writer(rx: Receiver<String>) -> JoinHandle<io::Result<()>> {
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("comments.json")?;
        let mut w = BufWriter::new(file);
        w.write_all(b"[")?;

        while let Ok(data) = rx.recv() {
            w.write_all(data.as_bytes())?;
        }
        w.write_all(b"{}]")?;
        Ok(())
    })
}

async fn process_tokens(
    token: Option<String>,
    tx: Sender<String>,
    client: Client,
    max_pages: usize,
    endpoint_url: Arc<str>,
    video_id: Arc<str>,
) -> Result<(), Error> {
    let mut handles: JoinSet<Result<(), serde_json::Error>> = tokio::task::JoinSet::new();
    let mut current_token = token;
    let mut current_page = 1;
    // tx
    while let Some(token) = current_token.as_mut()
        && current_page <= max_pages
    {
        let data =
            Arc::from(call_endpoint(client.clone(), &endpoint_url, &video_id, &token).await?);

        /* Temporary before all cases are handled. Currently not all comments are pulled */
        let ext_data = serde_json::from_str::<ExtensionData>(&data);
        if ext_data.is_err() {
            break;
        }
        let ext_data = ext_data?;
        /* ------------------------------------------- */

        let client_cl = client.clone();
        let url_cl = Arc::clone(&endpoint_url);
        let id_cl = Arc::clone(&video_id);
        let data_cl = Arc::clone(&data);
        let tx_cl = tx.clone();

        handles.spawn(async move {
            let ext_data = serde_json::from_str::<ExtensionData>(&data_cl)?;
            let reply_tokens = ext_data.reply_extension_tokens().into_iter();

            let calls =
                reply_tokens.map(|tkn| call_endpoint(client_cl.clone(), &url_cl, &id_cl, tkn));

            let raw_responses: Vec<String> = join_all(calls)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect();

            let mut buf: Vec<u8> = Vec::new();
            for body in raw_responses {
                let Ok(rp_data) = serde_json::from_str::<ReplyData<'_>>(body.as_str()) else {
                    continue;
                };

                for comment in rp_data.comments() {
                    if serde_json::to_writer(&mut buf, &comment).is_ok() {
                        buf.push(b',');
                    }
                }
            }

            if !buf.is_empty() {
                if let Ok(batch) = String::from_utf8(buf) {
                    let _ = tx_cl.send(batch);
                }
            }
            Ok(())
        });

        // cleans up some handles if they finished
        while handles.try_join_next().is_some() {}

        let comments = ext_data.comments();

        let mut buf: Vec<u8> = Vec::new();
        for comment in comments {
            if serde_json::to_writer(&mut buf, &comment).is_ok() {
                buf.push(b',');
            }
        }

        if !buf.is_empty() {
            if let Ok(batch) = String::from_utf8(buf) {
                let _ = tx.send(batch);
            }
        }

        let next_token = ext_data.extention_token();
        match next_token {
            Some(tkn) => {
                token.clear();
                token.push_str(tkn);
            }
            None => current_token = None,
        }
        current_page += 1;
    }

    while handles.join_next().await.is_some() {}
    Ok(())
}
