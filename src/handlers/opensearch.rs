use axum::{extract::State, http::header, response::IntoResponse};

use crate::AppState;

/// Return an OpenSearch description document so OPDS clients can discover the
/// search endpoint automatically.
pub async fn handle_opensearch(State(state): State<AppState>) -> impl IntoResponse {
    let search_url = format!("{}/opds/search?q={{searchTerms}}", state.config.base_url);
    let title = &state.config.catalog_title;

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/">
  <ShortName>{title}</ShortName>
  <Description>Search {title}</Description>
  <Url type="application/atom+xml;profile=opds-catalog;kind=acquisition"
       template="{search_url}"/>
  <Language>*</Language>
  <InputEncoding>UTF-8</InputEncoding>
  <OutputEncoding>UTF-8</OutputEncoding>
</OpenSearchDescription>
"#
    );

    (
        [(
            header::CONTENT_TYPE,
            "application/opensearchdescription+xml;charset=utf-8",
        )],
        xml,
    )
}
