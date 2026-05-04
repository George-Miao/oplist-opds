pub mod browse;
pub mod opensearch;
pub mod proxy;
pub mod root;
pub mod search;

use axum::{Router, routing::get};

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/opds", get(root::handle_root))
        .route("/opds/browse/", get(browse::handle_browse))
        .route("/opds/browse/{*path}", get(browse::handle_browse))
        .route("/opds/search", get(search::handle_search))
        .route("/opds/opensearch.xml", get(opensearch::handle_opensearch))
        .route("/opds/raw/{*path}", get(proxy::handle_proxy))
        .with_state(state)
}
