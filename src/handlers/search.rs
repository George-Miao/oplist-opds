use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    AppState,
    error::{AppError, Result},
    now_rfc3339,
    opds::{feed::search_feed, xml::feed_to_xml},
};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn handle_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse> {
    let query = params.q.unwrap_or_default();
    if query.trim().is_empty() {
        return Err(AppError::Internal {
            message: "search query `q` is required".to_string(),
        });
    }

    let results = state
        .openlist
        .search(&state.config.root_path, &query, 1, 100)
        .await?;

    let updated = results
        .content
        .first()
        .and_then(|o| o.modified.as_deref())
        .map(str::to_string)
        .unwrap_or_else(now_rfc3339);

    let feed = search_feed(
        &state.config.base_url,
        &query,
        results.content,
        &updated,
        state.config.proxy_enabled,
    );
    let xml = feed_to_xml(&feed)?;

    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml;charset=utf-8")],
        xml,
    ))
}
