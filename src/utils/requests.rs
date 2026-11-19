use colored::Colorize;
use serde_json::Value;
use wreq::Client;
use wreq_util::Emulation;

use crate::error::Error;

struct Comment<'a> {
    id: &'a str,
    author: &'a str,
    text: &'a str,
}

pub async fn req_comments(video_id: &str, max_pages: usize) -> Result<(), Error> {
    let client = Client::builder().emulation(Emulation::Firefox139).build()?;

    let html = get_html_page(client.clone(), video_id).await?;
    let innertube_key = extract_innertube_key(&html).ok_or(Error::ParsingError)?;

    let endpoint_url = format!(
        "https://www.youtube.com/youtubei/v1/next?key={}",
        innertube_key
    );

    let mut current_token = comments_section_token(html.as_str());
    let mut current_page = 1;

    while let Some(token) = current_token
        && current_page <= max_pages
    {
        let data = call_endpoint(client.clone(), &endpoint_url, video_id, &token).await?;

        if let Some(comments) = parse_comments(&data) {
            println!("{}{}", "## Page: ".red(), current_page);
            for comment in comments {
                println!(
                    "- {}{}{}{}",
                    "Author: ".red(),
                    comment.author,
                    "Comment: ".red(),
                    comment.text
                );
            }
        };

        current_token = comments_extension_token(&data);
        current_page += 1;
    }

    Ok(())
}

fn parse_comments<'a>(data: &'a Value) -> Option<Vec<Comment<'a>>> {
    let comment_ids: Vec<&serde_json::Value> = data
        .get("onResponseReceivedEndpoints")?
        .as_array()?
        .iter()
        .flat_map(|res| {
            res.get("reloadContinuationItemsCommand")
                .or_else(|| res.get("appendContinuationItemsAction"))
                .and_then(|c| c.get("continuationItems"))
                .and_then(|ci| ci.as_array())
                .into_iter()
                .flatten()
                .filter_map(|item| {
                    item.get("commentThreadRenderer")
                        .and_then(|th| th.get("commentViewModel"))
                        .and_then(|vm| vm.get("commentViewModel"))
                        .and_then(|vm| vm.get("commentId"))
                })
        })
        .collect();

    let mut payloads = data
        .get("frameworkUpdates")?
        .get("entityBatchUpdate")?
        .get("mutations")?
        .as_array()
        .and_then(|muts| Some(muts.iter().flat_map(|mu| mu.get("payload"))))
        .into_iter()
        .flatten()
        .filter_map(|pl| pl.get("commentEntityPayload"))
        .filter_map(|cep| {
            let properties = cep.get("properties")?;

            let id = properties.get("commentId")?.as_str()?;
            let author = cep.get("author")?.get("displayName")?.as_str()?;
            let text = properties.get("content")?.get("content")?.as_str()?;
            Some(Comment { id, author, text })
        });

    let comments = comment_ids
        .iter()
        .filter_map(|id| payloads.find(|pd| &pd.id == id))
        .collect();

    Some(comments)
}

async fn get_html_page(client: Client, video_id: &str) -> Result<String, Error> {
    let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);

    let html = client.get(watch_url.as_str()).send().await?.text().await?;

    Ok(html)
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
) -> Result<Value, Error> {
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
    })
    .to_string();

    let res = client
        .post(url)
        .header("Referer", watch_url)
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await?;

    let json = res.text().await?;
    let data: Value = serde_json::from_str(&json).map_err(|_| Error::ParsingError)?;
    Ok(data)
}

fn comments_extension_token(data: &Value) -> Option<String> {
    let token = data
        .get("onResponseReceivedEndpoints")?
        .as_array()?
        .last()
        .and_then(|item| {
            item.pointer("/appendContinuationItemsAction/continuationItems")
                .or_else(|| item.pointer("/reloadContinuationItemsCommand/continuationItems"))
        })?
        .as_array()?
        .last()?
        .pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")?
        .as_str()
        .and_then(|v| Some(v.to_string()));

    token
}

/// Extracts and parses `ytInitialData` from raw HTML, ignoring trailing garbage.
fn comments_section_token(html: &str) -> Option<String> {
    let obj_key = "var ytInitialData = ";
    let brace_start = html.find(obj_key)? + obj_key.len();

    let slice = wrap_brace(html.get(brace_start..)?)?;

    let data: Value = serde_json::from_str(slice).ok()?;

    let token = data
        .pointer("/contents/twoColumnWatchNextResults/results/results/contents")?
        .as_array()?
        .iter()
        .find_map(|item| item.pointer("/itemSectionRenderer/contents"))?
        .as_array()?
        .iter()
        .find_map(|section_item| {
            section_item
                .pointer(
                    "/continuationItemRenderer/continuationEndpoint/continuationCommand/token",
                )?
                .as_str()
        })?
        .to_string();

    Some(token)
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
