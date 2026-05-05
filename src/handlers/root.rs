use axum::{extract::State, http::header, response::IntoResponse};

use crate::{
    AppState,
    error::Result,
    now_rfc3339,
    opds::{feed::root_listing_feed, xml::feed_to_xml},
};

pub async fn handle_root(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let listing = state.openlist.list(&state.config.root_path, 1, 0).await?;
    let updated = listing
        .content
        .first()
        .and_then(|o| o.modified.as_deref())
        .map(str::to_string)
        .unwrap_or_else(now_rfc3339);

    let feed = root_listing_feed(
        &state.config.base_url,
        &state.config.catalog_title,
        &state.config.root_path,
        listing.content,
        &updated,
        state.config.proxy_enabled,
    );
    let xml = feed_to_xml(&feed)?;
    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml;charset=utf-8")],
        xml,
    ))
}
