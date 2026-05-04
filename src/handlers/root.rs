use axum::{extract::State, http::header, response::IntoResponse};

use crate::{
    AppState,
    error::Result,
    now_rfc3339,
    opds::{feed::root_feed, xml::feed_to_xml},
};

pub async fn handle_root(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let feed = root_feed(
        &state.config.base_url,
        &state.config.catalog_title,
        &now_rfc3339(),
    );
    let xml = feed_to_xml(&feed)?;
    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml;charset=utf-8")],
        xml,
    ))
}
