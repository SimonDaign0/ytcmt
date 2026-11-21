pub use comment_section::ExtensionData;

pub mod comment_section {
    use serde::Deserialize;
    use std::borrow::Cow;

    pub struct Comment<'a> {
        id: &'a str,
        author: &'a str,
        text: &'a str,
    }

    impl<'a> ExtensionData<'a> {
        pub fn into_token(self) -> Option<Cow<'a, str>> {
            self.on_response_received_endpoints
                .into_iter()
                .filter_map(|end_pt| {
                    end_pt
                        .append_continuation_items_action
                        .or(end_pt.reload_continuation_items_command)
                })
                .last()?
                .continuation_items
                .into_iter()
                .next_back()
                .and_then(|item| {
                    item.continuation_item_renderer
                        .map(|r| r.continuation_endpoint.continuation_command.token)
                })
        }

        pub fn comment_ids(self) -> impl Iterator<Item = Cow<'a, str>> {
            let ids = self
                .on_response_received_endpoints
                .into_iter()
                .filter_map(|end_pt| {
                    end_pt
                        .append_continuation_items_action
                        .or(end_pt.reload_continuation_items_command)
                })
                .flat_map(|con| {
                    con.continuation_items
                        .into_iter()
                        .filter_map(|it| it.comment_thread_renderer)
                        .into_iter()
                        .map(|ren| ren.comment_view_model.comment_view_model.comment_id)
                });
            ids
        }

        pub fn comments(self) {

            /*  let mut payloads = data
                .pointer("/frameworkUpdates/entityBatchUpdate/mutations")?
                .as_array()
                .and_then(|muts| {
                    Some(muts.iter().filter_map(|mu| {
                        mu.pointer("/payload/commentEntityPayload").and_then(|cep| {
                            let id = cep.pointer("/properties/commentId")?.as_str()?;
                            let author = cep.pointer("/author/displayName")?.as_str()?;
                            let text = cep.pointer("/properties/content/content")?.as_str()?;

                            Some(Comment { id, author, text })
                        })
                    }))
                })?;

            let comments = comment_ids
                .iter()
                .filter_map(|id| payloads.find(|pd| &pd.id == id))
                .collect();

            Some(comments) */
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExtensionData<'a> {
        #[serde(borrow)]
        on_response_received_endpoints: Vec<ReceivedEndpoint<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ReceivedEndpoint<'a> {
        #[serde(borrow)]
        append_continuation_items_action: Option<Continuation<'a>>,
        reload_continuation_items_command: Option<Continuation<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Continuation<'a> {
        #[serde(borrow)]
        continuation_items: Vec<ContinuationItem<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ContinuationItem<'a> {
        #[serde(borrow)]
        continuation_item_renderer: Option<ContinuationItemRenderer<'a>>,
        comment_thread_renderer: Option<comments::CommentThreadRenderer<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ContinuationItemRenderer<'a> {
        #[serde(borrow)]
        continuation_endpoint: ContinuationEndpoint<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ContinuationEndpoint<'a> {
        #[serde(borrow)]
        continuation_command: ContinuationCommand<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ContinuationCommand<'a> {
        #[serde(borrow)]
        token: Cow<'a, str>,
    }

    pub mod comments {
        pub use header::*;

        pub mod header {
            use serde::Deserialize;
            use std::borrow::Cow;

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentThreadRenderer<'a> {
                #[serde(borrow)]
                pub comment_view_model: CommentViewModel<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentViewModel<'a> {
                #[serde(borrow)]
                pub comment_view_model: ViewModelContent<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct ViewModelContent<'a> {
                #[serde(borrow)]
                pub comment_id: Cow<'a, str>,
            }
        }
    }
}
