mod config;
mod error;
mod handlers;
mod opds;
mod openlist;

use std::sync::OnceLock;

use compio::net::TcpListener;
use config::Config;
use openlist::OpenListClient;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Returns the current UTC time formatted as an RFC 3339 string.
pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC 3339 formatting is infallible")
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Shared application state passed to every handler via axum `State`.
#[derive(Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub openlist: OpenListClient,
}

#[compio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config_arg = std::env::args_os().nth(1);
    let config_path = config_arg.as_deref().map(std::path::Path::new);

    if let Some(p) = config_path {
        info!(path = %p.display(), "loading config from file");
    } else {
        info!("no config file given — using environment variables only");
    }

    let config = Config::load(config_path).unwrap_or_else(|e| {
        eprintln!("Configuration error: {e}");
        std::process::exit(1);
    });

    let config: &'static Config = CONFIG.get_or_init(|| config);

    info!(
        bind_addr  = %config.bind_addr,
        base_url   = %config.base_url,
        oplist_url = %config.oplist_url,
        "Starting oplist-opds"
    );

    let openlist = OpenListClient::new(&config.oplist_url, &config.oplist_token);
    let state = AppState { config, openlist };

    let app = handlers::router(state);

    let listener = TcpListener::bind(&config.bind_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {e}", config.bind_addr);
            std::process::exit(1);
        });

    info!("Listening on {}", listener.local_addr().unwrap());

    cyper_axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    });
}
