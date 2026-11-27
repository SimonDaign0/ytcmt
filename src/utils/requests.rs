use crate::schema::Comment;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::{Arc, mpsc::Sender};
use wreq::Client;

use crate::schema::comments_extension::ExtensionData;
use crate::schema::reply::ReplyData;

use crate::error::Error;

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

pub async fn get_html_page(client: Client, video_id: &str) -> Result<String, Error> {
    let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);

    let html = client.get(watch_url.as_str()).send().await?.text().await?;

    Ok(html)
}

pub async fn process_tokens(
    cmt_section_tkn: Option<String>,
    tx: Sender<String>,
    client: Client,
    max_pages: usize,
    endpoint_url: Arc<str>,
    video_id: Arc<str>,
) -> Result<(), Error> {
    let mut set = tokio::task::JoinSet::new();
    let mut current_token = cmt_section_tkn;
    let mut current_page = 1;

    while let Some(token) = current_token.as_mut()
        && current_page <= max_pages
    {
        let data =
            Arc::from(call_endpoint(client.clone(), &endpoint_url, &video_id, &token).await?);

        let Ok(ext_data) = serde_json::from_str::<ExtensionData>(&data) else {
            break;
        };

        let client_cl = client.clone();
        let url_cl = Arc::clone(&endpoint_url);
        let id_cl = Arc::clone(&video_id);
        let data_cl = Arc::clone(&data);
        let tx_cl = tx.clone();

        set.spawn(
            async move { process_extension_data(client_cl, url_cl, id_cl, data_cl, tx_cl) }.await,
        );

        // cleans up some handles if they finished
        while set.try_join_next().is_some() {}

        let comments = ext_data.comments();

        write_batch(&tx.clone(), comments);

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

    while set.join_next().await.is_some() {}
    Ok(())
}

fn write_batch<'a>(tx: &Sender<String>, comments: impl Iterator<Item = Comment<'a>>) {
    let mut batch = String::new();

    for comment in comments {
        if let Ok(json) = serde_json::to_string(&comment) {
            batch.push_str(",");
            batch.push_str(&json);
        }
    }

    if !batch.is_empty() {
        let _ = tx.send(batch);
    }
}

async fn process_extension_data(
    client: Client,
    url: Arc<str>,
    video_id: Arc<str>,
    data: Arc<str>,
    tx: Sender<String>,
) {
    let Ok(ext_data) = serde_json::from_str::<ExtensionData>(&data) else {
        return;
    };
    let mut in_flight = FuturesUnordered::new();

    for tkn in ext_data.reply_extension_tokens() {
        in_flight.push(process_reply(
            &tx,
            tkn.to_string(),
            client.clone(),
            &url,
            &video_id,
        ));
    }

    while let Some(next_batch) = in_flight.next().await {
        if let Some(tkns) = next_batch {
            for tkn in tkns {
                in_flight.push(process_reply(&tx, tkn, client.clone(), &url, &video_id));
            }
        }
    }
}

async fn process_reply<'a>(
    tx: &Sender<String>,
    tkn: String,
    client: Client,
    url: &str,
    video_id: &str,
) -> Option<Vec<String>> {
    let body = call_endpoint(client.clone(), &url, &video_id, &tkn)
        .await
        .ok()?;
    let reply_data = serde_json::from_str::<ReplyData<'_>>(&body).ok()?;

    write_batch(&tx, reply_data.comments());

    Some(reply_data.subthread_tokens())
}
