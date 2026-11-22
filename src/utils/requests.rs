use colored::Colorize;
use futures::future::join_all;
use wreq::Client;
use wreq_util::Emulation;

use crate::schema::reply::ReplyData;
use crate::schema::{comments_extension::ExtensionData, comments_section::SectionData};

use crate::error::Error;

pub async fn req_comments(video_id: &str, max_pages: usize) -> Result<(), Error> {
    let client = Client::builder().emulation(Emulation::Firefox139).build()?;

    let html = get_html_page(client.clone(), video_id).await?;
    let innertube_key = extract_innertube_key(&html).ok_or(Error::ParsingError)?;

    let endpoint_url = format!(
        "https://www.youtube.com/youtubei/v1/next?key={}",
        innertube_key
    );
    let json_slice = parse_initial_data(html.as_str()).ok_or(Error::ParsingError)?;
    let cmt_section = serde_json::from_str::<SectionData>(json_slice)?;

    let mut current_token = cmt_section.token().map(|s| s.to_string());
    let mut current_page = 1;

    while let Some(token) = current_token.as_mut()
        && current_page <= max_pages
    {
        let data = call_endpoint(client.clone(), &endpoint_url, video_id, &token).await?;
        let ext_data = serde_json::from_str::<ExtensionData>(&data)?;

        let reply_tokens = ext_data.reply_extension_tokens();

        let calls = reply_tokens
            .into_iter()
            .map(|tkn| call_endpoint(client.clone(), &endpoint_url, video_id, tkn));

        let res = join_all(calls).await.into_iter().filter_map(|res| res.ok());

        for data in res {
            if let Ok(data) = serde_json::from_str::<ReplyData>(&data) {
                data.print_comments();
            };
        }

        println!("{}{}", "## Page: ".red(), current_page);
        ext_data.print_comments();

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
