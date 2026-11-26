use tokio::task::JoinHandle;

use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::sync::mpsc::Receiver;

pub fn vid_id<'a>(url: &'a String) -> Option<&'a str> {
    let marker = "https://www.youtube.com/watch?v=";
    url.split_once(marker).and_then(|(_, start)| {
        let (id, _) = start.split_once('&')?;
        Some(id)
    })
}

pub fn extract_innertube_key<'a>(html: &'a str) -> Option<&'a str> {
    let key = r#"INNERTUBE_API_KEY":"#;
    let start = html.find(key)? + key.len();
    let len = html.get(start..)?.find('"')?;
    html.get(start..start + len)
}

/// Extracts and parses `ytInitialData` from raw HTML, ignoring trailing garbage.
pub fn parse_initial_data<'a>(html: &'a str) -> Option<&'a str> {
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

pub async fn writer(rx: Receiver<String>) -> JoinHandle<io::Result<()>> {
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("comments.json")?;
        let mut w = BufWriter::new(file);
        w.write_all(b"[{}")?;

        while let Ok(data) = rx.recv() {
            w.write_all(data.as_bytes())?;
        }
        w.write_all(b"]")?;
        Ok(())
    })
}
