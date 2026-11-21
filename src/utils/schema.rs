pub use comment_section_container::ContainerData;

pub mod comment_section_container {
    use std::borrow::Cow;

    use serde::Deserialize;

    impl<'a> ContainerData<'a> {
        pub fn into_token(self) -> Option<Cow<'a, str>> {
            self.contents
                .two_column_watch_next_results
                .results
                .results
                .contents
                .into_iter()
                .filter_map(|res| res.item_section_renderer)
                .flat_map(|sec_ren| sec_ren.contents)
                .find_map(|con| {
                    con.continuation_item_renderer
                        .map(|it| it.continuation_endpoint.continuation_command.token)
                })
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContainerData<'a> {
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

pub mod comment_section_extension {
    use serde::Deserialize;
    use std::borrow::Cow;

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
