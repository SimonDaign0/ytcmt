use clap::Parser;
use wreq::Client;
use wreq_util::Emulation;
use ytcmt::parsing;
use ytcmt::schema::{self, comments_section::SectionData};
use ytcmt::{error::Error, requests};

use std::sync::{Arc, mpsc};

#[tokio::main]
async fn main() {
    let args = schema::Args::parse();

    let video_id = parsing::vid_id(&args.url);

    match video_id {
        Some(id) if id.len() > 0 => {
            if let Err(err) = req_comments(id, args.max_pages).await {
                eprintln!("{}", err);
            };
        }
        _ => {
            eprintln!("invalid url")
        }
    }
}

pub async fn req_comments(video_id: &str, max_pages: usize) -> Result<(), Error> {
    let client = Client::builder().emulation(Emulation::Firefox139).build()?;
    let video_id = Arc::from(video_id);

    let html = requests::get_html_page(client.clone(), &video_id).await?;
    let innertube_key = parsing::extract_innertube_key(&html).ok_or(Error::ParsingError)?;

    let json_slice = parsing::parse_initial_data(html.as_str()).ok_or(Error::ParsingError)?;
    let cmt_section = serde_json::from_str::<SectionData>(json_slice)?;

    let (tx, rx) = mpsc::channel::<String>();
    let w_handle = parsing::writer(rx);

    let endpoint_url: Arc<str> = format!(
        "https://www.youtube.com/youtubei/v1/next?key={}",
        innertube_key,
    )
    .into();

    let token = cmt_section.token().map(|s| s.to_string());
    requests::process_tokens(token, tx, client, max_pages, endpoint_url, video_id).await?;
    let _ = w_handle.await;
    Ok(())
}
//Eg0SC3YtdEZfX2VweVR3GAYy0gEKIWdldF9jb21tZW50X3dpdGhfcmVwbGllc19zdHJlYW0tLRpjEjFVZ3p5anFnUGUtWUQ2eDNUaVZKNEFhQUJBZy5BUmI3Q1BZRDd2QkFWVERtZjNvSm04IgIIACoYVUMydVJmanRVam9CRlJlcl9IbjllSlp3Mgt2LXRGX19lcHlUd4IBAggBKAhCRmNvbW1lbnQtcmVwbGllcy1pdGVtLVVnenlqcWdQZS1ZRDZ4M1RpVko0QWFBQkFnLkFSYjdDUFlEN3ZCQVZURG1mM29KbTg%3D
/*
let payload = serde_json::json!({
    "context": {
        "client": {
            "clientName": "WEB",
            "clientVersion": "2.20261116.00.00",
            "hl": "en",
            "gl": "US"
        }
    },
    "continuation": reply_token(video_id, channel_id, comment_id, blob)
});

 */
