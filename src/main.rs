use ytcmt::requests::req_comments;

#[tokio::main]
async fn main() {
    let video_id = option_env!("VID_ID").unwrap_or("zsCD5XCu6CM");
    let max_pages = 6;

    if let Err(err) = req_comments(video_id, max_pages).await {
        eprint!("{}", err);
    };
}
