use clap::Parser;
use serde::Serialize;

#[derive(Serialize)]
pub struct Comment<'a> {
    pub id: &'a str,
    pub author: &'a str,
    pub text: &'a str,
}

#[derive(Parser, Debug)]
#[command(about = "A tool to pull youtube video comments")]
pub struct Args {
    /// the youtube video url e.g. -u "https://www.youtube.com/watch?v=dQw4w9WgXcQ&start_radio=1"
    pub url: String,

    /// max page count
    #[arg(short, long, default_value_t = 6)]
    pub max_pages: usize,
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

pub mod comments_extension {
    use crate::schema::Comment;
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

        pub fn comments(&'a self) -> impl Iterator<Item = Comment<'a>> {
            self.framework_updates
                .entity_batch_update
                .mutations
                .iter()
                .filter_map(|muta| {
                    let pl = muta.payload.comment_entity_payload.as_ref()?;
                    Some(Comment {
                        id: pl.properties.comment_id.as_ref(),
                        author: pl.author.display_name.as_ref(),
                        text: pl.properties.content.content.as_ref(),
                    })
                })
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExtensionData<'a> {
        #[serde(borrow)]
        on_response_received_endpoints: Vec<ReceivedEndpoint<'a>>,
        framework_updates: comments::FrameworkUpdates<'a>,
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
                pub sub_threads: Option<Vec<subthread::SubThread<'a>>>,
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
            pub mod subthread {
                use crate::schema::comments_extension::ContinuationItem;
                use serde::Deserialize;

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct SubThread<'a> {
                    #[serde(borrow)]
                    pub continuation: ContinuationItem<'a>,
                }
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

    pub mod comments {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct FrameworkUpdates<'a> {
            #[serde(borrow)]
            pub entity_batch_update: EntityBatchUpdate<'a>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct EntityBatchUpdate<'a> {
            #[serde(borrow)]
            pub mutations: Vec<Mutation<'a>>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Mutation<'a> {
            #[serde(borrow)]
            pub payload: Payload<'a>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Payload<'a> {
            #[serde(borrow)]
            pub comment_entity_payload: Option<comment_payload::CommentEntityPayload<'a>>,
        }

        pub mod comment_payload {
            use serde::Deserialize;
            use std::borrow::Cow;

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct CommentEntityPayload<'a> {
                #[serde(borrow)]
                pub author: Author<'a>,
                pub properties: Properties<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Author<'a> {
                #[serde(borrow)]
                pub display_name: Cow<'a, str>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Properties<'a> {
                #[serde(borrow)]
                pub comment_id: Cow<'a, str>,
                pub content: Content<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Content<'a> {
                #[serde(borrow)]
                pub content: Cow<'a, str>,
            }
        }
    }
}

pub mod reply {
    use crate::schema::{Comment, comments_extension::comments, token};
    use serde::Deserialize;

    impl<'a> ReplyData<'a> {
        pub fn extention_token(&'a self) -> Option<&'a str> {
            let tkn = self
                .on_response_received_endpoints
                .iter()
                .flat_map(|nd_pt| {
                    nd_pt
                        .append_continuation_items_action
                        .continuation_items
                        .iter()
                })
                .last()?
                .continuation_item_renderer
                .as_ref()?
                .button
                .button_renderer
                .command
                .continuation_command
                .token
                .as_ref();
            Some(tkn)
        }
        pub fn comments(&'a self) -> impl Iterator<Item = Comment<'a>> {
            self.framework_updates
                .entity_batch_update
                .mutations
                .iter()
                .filter_map(|muta| {
                    let pl = muta.payload.comment_entity_payload.as_ref()?;
                    Some(Comment {
                        id: pl.properties.comment_id.as_ref(),
                        author: pl.author.display_name.as_ref(),
                        text: pl.properties.content.content.as_ref(),
                    })
                })
        }
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReplyData<'a> {
        #[serde(borrow)]
        pub on_response_received_endpoints: Vec<ReceivedEndpoint<'a>>,
        pub framework_updates: comments::FrameworkUpdates<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReceivedEndpoint<'a> {
        #[serde(borrow)]
        pub append_continuation_items_action: AppendContinuationItemsAction<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppendContinuationItemsAction<'a> {
        #[serde(borrow)]
        pub continuation_items: Vec<ContinuationItem<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinuationItem<'a> {
        #[serde(borrow)]
        pub continuation_item_renderer: Option<ContinuationItemRenderer<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinuationItemRenderer<'a> {
        #[serde(borrow)]
        pub button: Button<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Button<'a> {
        #[serde(borrow)]
        pub button_renderer: ButtonRenderer<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ButtonRenderer<'a> {
        #[serde(borrow)]
        pub command: Command<'a>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Command<'a> {
        #[serde(borrow)]
        pub continuation_command: token::ContinuationCommand<'a>,
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
