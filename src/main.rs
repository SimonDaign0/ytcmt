use clap::Parser;
use ytcmt::requests::req_comments;

#[derive(Parser, Debug)]
#[command(about = "A tool to pull youtube video comments")]
struct Args {
    /// the youtube video url e.g. -u "https://www.youtube.com/watch?v=dQw4w9WgXcQ&start_radio=1"
    url: String,

    /// max page count
    #[arg(short, long, default_value_t = 6)]
    max_pages: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let video_id = vid_id(&args.url);

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

fn vid_id<'a>(url: &'a String) -> Option<&'a str> {
    let marker = "https://www.youtube.com/watch?v=";
    url.split_once(marker).and_then(|(_, start)| {
        let (id, _) = start.split_once('&')?;
        Some(id)
    })
}
