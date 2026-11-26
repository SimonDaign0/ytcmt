use crate::schema::Comment;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::{Arc, mpsc::Sender};
use tokio::task::JoinSet;
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
    let mut set: JoinSet<Result<(), serde_json::Error>> = tokio::task::JoinSet::new();
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

        set.spawn(async move {
            let ext_data = serde_json::from_str::<ExtensionData>(&data_cl)?;
            let reply_tokens = ext_data.reply_extension_tokens().into_iter();

            let calls =
                reply_tokens.map(|tkn| call_endpoint(client_cl.clone(), &url_cl, &id_cl, tkn));

            let mut in_flight: FuturesUnordered<_> = calls.into_iter().collect();
            while let Some(res) = in_flight.next().await {
                if let Ok(body) = res {
                    if let Ok(comments) = serde_json::from_str::<ReplyData<'_>>(&body) {
                        write_batch(&tx_cl, comments.comments());
                    }
                }
            }

            Ok(())
        });

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
