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
