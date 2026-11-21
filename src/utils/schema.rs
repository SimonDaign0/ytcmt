pub mod comments_extension {
    use serde::Deserialize;
    use std::borrow::Cow;

    pub struct Comment<'a> {
        pub id: &'a str,
        pub author: &'a str,
        pub text: &'a str,
    }

    impl<'a> ExtensionData<'a> {
        pub fn token(&'a self) -> Option<&'a str> {
            self.on_response_received_endpoints
                .iter()
                .filter_map(|end_pt| {
                    end_pt
                        .append_continuation_items_action
                        .as_ref()
                        .or(end_pt.reload_continuation_items_command.as_ref())
                })
                .next_back()?
                .continuation_items
                .iter()
                .next_back()
                .and_then(|item| {
                    item.continuation_item_renderer
                        .as_ref()
                        .map(|r| r.continuation_endpoint.continuation_command.token.as_ref())
                })
        }

        pub fn comments<'b>(&'b self) -> impl Iterator<Item = Comment<'b>> + 'b {
            self.framework_updates
                .entity_batch_update
                .mutations
                .iter()
                .filter_map(|mutation| {
                    let payload = mutation.payload.comment_entity_payload.as_ref()?;
                    Some(Comment {
                        id: payload.properties.comment_id.as_ref(),
                        author: payload.author.display_name.as_ref(),
                        text: payload.properties.content.content.as_ref(),
                    })
                })
        }
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExtensionData<'a> {
        #[serde(borrow)]
        on_response_received_endpoints: Vec<ReceivedEndpoint<'a>>,
        framework_updates: comments::payload::FrameworkUpdates<'a>,
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

        pub mod payload {
            use serde::Deserialize;
            use std::borrow::Cow;

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
                pub comment_entity_payload: Option<CommentEntityPayload<'a>>,
            }

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
                pub content: Message<'a>,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Message<'a> {
                #[serde(borrow)]
                pub content: Cow<'a, str>,
            }
        }
    }
}

pub mod comments_container {
    use std::borrow::Cow;

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
                    con.continuation_item_renderer
                        .as_ref()
                        .map(|it| it.continuation_endpoint.continuation_command.token.as_ref())
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
    struct ItemSectionRenderer<'a> {
        #[serde(borrow)]
        contents: Vec<RendererItem<'a>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RendererItem<'a> {
        #[serde(borrow)]
        continuation_item_renderer: Option<ContinuationItemRenderer<'a>>,
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
}
