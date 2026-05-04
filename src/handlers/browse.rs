use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};

use crate::{
    AppState,
    error::Result,
    now_rfc3339,
    opds::{feed::browse_feed, xml::feed_to_xml},
};

/// Handle `GET /opds/browse/` (root directory) and `GET /opds/browse/*path`.
pub async fn handle_browse(
    State(state): State<AppState>,
    path: Option<Path<String>>,
) -> Result<impl IntoResponse> {
    // Resolve the OpenList path: prefix with config root_path.
    let rel = path.as_deref().map(|p| p.as_str()).unwrap_or("");
    let dir_path = join_paths(&state.config.root_path, rel);

    let listing = state.openlist.list(&dir_path, 1, 0).await?;
    let updated = listing
        .content
        .first()
        .and_then(|o| o.modified.as_deref())
        .map(str::to_string)
        .unwrap_or_else(now_rfc3339);

    let feed = browse_feed(
        &state.config.base_url,
        &dir_path,
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

fn join_paths(root: &str, rel: &str) -> String {
    let root = root.trim_end_matches('/');
    let rel = rel.trim_start_matches('/');
    if rel.is_empty() {
        if root.is_empty() {
            "/".to_string()
        } else {
            root.to_string()
        }
    } else {
        format!("{root}/{rel}")
    }
}
