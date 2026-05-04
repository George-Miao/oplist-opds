use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use cyper::Client;

use crate::{
    AppState,
    error::{AppError, Result},
};

/// Reverse-proxy a file download from OpenList, injecting the auth token.
///
/// The client downloads from `/opds/raw/{*path}`; this handler fetches the
/// corresponding `raw_url` from OpenList (via `fs/get`) and streams it back
/// chunk-by-chunk without buffering the full body in memory.
pub async fn handle_proxy(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response> {
    let full_path = format!(
        "{}/{}",
        state.config.root_path.trim_end_matches('/'),
        path.trim_start_matches('/')
    );

    // Resolve the actual download URL via the OpenList metadata endpoint.
    let obj = state.openlist.get(&full_path).await?;

    if obj.raw_url.is_empty() {
        return Err(AppError::NotFound { path: full_path });
    }

    // Fetch the file from OpenList, injecting auth.
    let client = Client::new();
    let mut req = client
        .get(&obj.raw_url)
        .map_err(|e| AppError::OpenListRequest {
            path: full_path.clone(),
            source: e,
        })?;

    if !state.config.oplist_token.is_empty() {
        req = req
            .header("Authorization", &state.config.oplist_token)
            .map_err(|e| AppError::OpenListRequest {
                path: full_path.clone(),
                source: e,
            })?;
    }

    let upstream = req.send().await.map_err(|e| AppError::OpenListRequest {
        path: full_path,
        source: e,
    })?;

    // Forward status + key headers to the OPDS client.
    let status = StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut headers = HeaderMap::new();
    for (name, value) in upstream.headers() {
        if matches!(
            name.as_str(),
            "content-type" | "content-length" | "content-disposition" | "last-modified" | "etag"
        ) {
            headers.insert(name.clone(), value.clone());
        }
    }

    // Stream the upstream body directly to the client without buffering.
    let stream = upstream.bytes_stream();
    let body = Body::from_stream(stream);

    Ok((status, headers, body).into_response())
}
