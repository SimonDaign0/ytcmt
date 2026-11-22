// .json files need to be created by the tester
use ytcmt::schema::{comments_extension::ExtensionData, comments_section::SectionData};

#[test]
fn get_comment_section_token() {
    let initial_data = include_str!("initial_data.json");
    let cmt_section = serde_json::from_str::<SectionData>(initial_data).unwrap();
    assert!(cmt_section.token().is_some());
}

#[test]
fn get_extension_token() {
    let cmt_section_data = include_str!("comment_section.json");
    let ext_data = serde_json::from_str::<ExtensionData>(cmt_section_data).unwrap();
    assert!(ext_data.extention_token().is_some());
}

#[test]
fn get_reply_header() {
    let comments_data = include_str!("comment_section.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.replies_extension_header().is_some());

    let comments_data = include_str!("comments.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.replies_extension_header().is_none())
}

#[test]
fn get_reply_extension_tokens() {
    let comments_data = include_str!("comment_section.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.reply_extension_tokens().count() > 0);

    let comments_data = include_str!("comments.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.reply_extension_tokens().count() > 0);
}

#[test]
fn get_comments() {
    let comments_data = include_str!("comment_section.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.comments().count() > 0);

    let comments_data = include_str!("comments.json");
    let ext_data = serde_json::from_str::<ExtensionData>(comments_data).unwrap();
    assert!(ext_data.comments().count() > 0);
}
