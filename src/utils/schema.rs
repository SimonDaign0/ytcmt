pub enum DataType<'a> {
    SectionData(comments_section::SectionData<'a>),
    CommentExtension(initial_comments::ExtensionData<'a>),
    ReplyExtension,
}

pub mod comments_section {
    use crate::schema::token;
    use serde::Deserialize;

    impl<'a> SectionData<'a> {
        pub fn token(&'a self) -> Option<&'a str> {
            self.contents
                .two_column_watch_next_results
                .results
                .results
                .contents
                .iter()
                .filter_map(|res| res.item_section_renderer.as_ref())
                .flat_map(|sec_ren| sec_ren.contents.iter())
                .find_map(|con| {
                    Some(
                        con.continuation_item_renderer
                            .continuation_endpoint
                            .continuation_command
                            .token
                            .as_ref(),
                    )
                })
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SectionData<'a> {
        #[serde(borrow)]
        contents: RootContents<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RootContents<'a> {
        #[serde(borrow)]
        two_column_watch_next_results: TwoColumnWatchNextResults<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TwoColumnWatchNextResults<'a> {
        #[serde(borrow)]
        results: NextResults<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NextResults<'a> {
        #[serde(borrow)]
        results: Results<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Results<'a> {
        #[serde(borrow)]
        contents: Vec<ResultContents<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ResultContents<'a> {
        #[serde(borrow)]
        item_section_renderer: Option<ItemSectionRenderer<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ItemSectionRenderer<'a> {
        #[serde(borrow)]
        contents: Vec<RendererItem<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RendererItem<'a> {
        #[serde(borrow)]
        pub continuation_item_renderer: token::ContinuationItemRenderer<'a>,
    }
}

pub mod initial_comments {
    use serde::Deserialize;

    impl<'a> ExtensionData<'a> {
        pub fn extention_token(&'a self) -> Option<&'a str> {
            let tkn = self
                .on_response_received_endpoints
                .iter()
                .flat_map(|nd_pt| {
                    match nd_pt {
                        ReceivedEndpoint::ContinuationAction {
                            reload_continuation_items_command,
                        } => reload_continuation_items_command,
                        ReceivedEndpoint::AppendAction {
                            append_continuation_items_action,
                        } => append_continuation_items_action,
                    }
                    .continuation_items
                    .iter()
                })
                .filter_map(|it| match it {
                    ContinuationItem::ContinuationRenderer {
                        continuation_item_renderer,
                    } => Some(continuation_item_renderer),
                    _ => None,
                })
                .last()?
                .continuation_endpoint
                .continuation_command
                .token
                .as_ref();
            Some(tkn)
        }

        pub fn replies_extension_header(&'a self) -> Option<&'a str> {
            let tkn = self
                .on_response_received_endpoints
                .iter()
                .flat_map(|nd_pt| {
                    match nd_pt {
                        ReceivedEndpoint::ContinuationAction {
                            reload_continuation_items_command,
                        } => reload_continuation_items_command,
                        ReceivedEndpoint::AppendAction {
                            append_continuation_items_action,
                        } => append_continuation_items_action,
                    }
                    .continuation_items
                    .iter()
                })
                .filter_map(|it| match it {
                    ContinuationItem::HeaderRenderer {
                        comments_header_renderer,
                    } => Some(comments_header_renderer.comments_count.runs.iter().last()?),
                    _ => None,
                })
                .last()?
                .text
                .as_ref();

            Some(tkn)
        }

        pub fn reply_extension_tokens(&'a self) -> impl Iterator<Item = &'a str> {
            self.on_response_received_endpoints
                .iter()
                .flat_map(|nd_pt| {
                    match nd_pt {
                        ReceivedEndpoint::ContinuationAction {
                            reload_continuation_items_command,
                        } => reload_continuation_items_command,
                        ReceivedEndpoint::AppendAction {
                            append_continuation_items_action,
                        } => append_continuation_items_action,
                    }
                    .continuation_items
                    .iter()
                })
                .filter_map(|it| match it {
                    ContinuationItem::ThreadRenderer {
                        comment_thread_renderer,
                    } => Some(comment_thread_renderer.replies.as_ref()?),
                    _ => None,
                })
                .flat_map(|rep| rep.comment_replies_renderer.contents.last())
                .map(|cnt| {
                    cnt.continuation_item_renderer
                        .continuation_endpoint
                        .continuation_command
                        .token
                        .as_ref()
                })
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExtensionData<'a> {
        #[serde(borrow)]
        on_response_received_endpoints: Vec<ReceivedEndpoint<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all_fields = "camelCase")]
    #[serde(untagged)]
    pub enum ReceivedEndpoint<'a> {
        ContinuationAction {
            #[serde(borrow)]
            reload_continuation_items_command: Continuation<'a>,
        },
        AppendAction {
            #[serde(borrow)]
            append_continuation_items_action: Continuation<'a>,
        },
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Continuation<'a> {
        #[serde(borrow)]
        pub continuation_items: Vec<ContinuationItem<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all_fields = "camelCase")]
    #[serde(untagged)]
    pub enum ContinuationItem<'a> {
        HeaderRenderer {
            #[serde(borrow)]
            comments_header_renderer: continuation_item::CommentsHeaderRenderer<'a>,
        },
        ThreadRenderer {
            #[serde(borrow)]
            comment_thread_renderer: continuation_item::CommentsThreadRenderer<'a>,
        },
        ContinuationRenderer {
            #[serde(borrow)]
            continuation_item_renderer: continuation_item::CommentsItemRenderer<'a>,
        },
    }

    pub mod continuation_item {
        pub use header_renderer::CommentsHeaderRenderer;
        pub use thread_renderer::CommentsThreadRenderer;

        use crate::schema::token;
        use serde::Deserialize;

        pub mod header_renderer {
            use serde::Deserialize;
            use std::borrow::Cow;

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentsHeaderRenderer<'a> {
                #[serde(borrow)]
                pub comments_count: CommentsCount<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentsCount<'a> {
                #[serde(borrow)]
                pub runs: Vec<Run<'a>>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Run<'a> {
                #[serde(borrow)]
                pub text: Cow<'a, str>,
            }
        }

        pub mod thread_renderer {
            use crate::schema::token;
            use serde::Deserialize;

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentsThreadRenderer<'a> {
                #[serde(borrow)]
                pub replies: Option<Replies<'a>>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Replies<'a> {
                #[serde(borrow)]
                pub comment_replies_renderer: CommentRepliesRenderer<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentRepliesRenderer<'a> {
                #[serde(borrow)]
                pub contents: Vec<Content<'a>>,
                pub view_replies: replies_header::ViewReplies<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Content<'a> {
                #[serde(borrow)]
                pub continuation_item_renderer: ContinuationItemRenderer<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct ContinuationItemRenderer<'a> {
                #[serde(borrow)]
                pub continuation_endpoint: token::ContinuationEndpoint<'a>,
            }
            pub mod replies_header {
                use serde::Deserialize;
                use std::borrow::Cow;

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct ViewReplies<'a> {
                    #[serde(borrow)]
                    pub button_renderer: ButtonRenderer<'a>,
                }

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct ButtonRenderer<'a> {
                    #[serde(borrow)]
                    pub text: Text<'a>,
                }

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct Text<'a> {
                    #[serde(borrow)]
                    pub runs: Vec<Run<'a>>,
                }

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct Run<'a> {
                    #[serde(borrow)]
                    pub text: Cow<'a, str>,
                }
            }
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct CommentsItemRenderer<'a> {
            #[serde(borrow)]
            pub continuation_endpoint: token::ContinuationEndpoint<'a>,
        }
    }
}

pub mod token {
    use serde::Deserialize;
    use std::borrow::Cow;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinuationItemRenderer<'a> {
        #[serde(borrow)]
        pub continuation_endpoint: ContinuationEndpoint<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinuationEndpoint<'a> {
        #[serde(borrow)]
        pub continuation_command: ContinuationCommand<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinuationCommand<'a> {
        #[serde(borrow)]
        pub token: Cow<'a, str>,
    }
}
