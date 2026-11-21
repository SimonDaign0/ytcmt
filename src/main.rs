use ytcmt::requests::req_comments;

#[tokio::main]
async fn main() {
    let video_id = "zsCD5XCu6CM";
    let max_pages = 6;

    if let Err(err) = req_comments(video_id, max_pages).await {
        eprint!("{}", err);
    };
}
