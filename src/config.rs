use std::path::Path;

use figment::{
    Figment,
    providers::{Env, Format, Json, Toml, Yaml},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Base URL of the OpenList instance, e.g. `https://files.example.com`
    pub oplist_url: String,

    /// Bearer token for authenticated OpenList; leave empty for public
    /// instances
    #[serde(default)]
    pub oplist_token: String,

    /// Address the OPDS server listens on
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    /// Public URL of this server (used in feed `self` links)
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// `<title>` shown in the root OPDS catalog feed
    #[serde(default = "default_catalog_title")]
    pub catalog_title: String,

    /// OpenList path to expose as OPDS root
    #[serde(default = "default_root_path")]
    pub root_path: String,

    /// When true, file downloads are proxied through `/opds/raw/*path` so the
    /// OpenList token is never exposed to the OPDS client.
    /// When false (default), acquisition links point directly to the OpenList
    /// `raw_url`.
    #[serde(default)]
    pub proxy_enabled: bool,
}

fn default_bind_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_catalog_title() -> String {
    "OpenList OPDS".to_string()
}

fn default_root_path() -> String {
    "/".to_string()
}

impl Config {
    /// Load configuration from an optional config file path plus environment
    /// variables (env always wins).
    ///
    /// - `Some(path)` — load the file at `path`; the format is inferred from
    ///   the file extension (`.toml`, `.yaml`/`.yml`, `.json`).
    /// - `None` — skip file loading entirely; only env vars are used.
    pub fn load(config_path: Option<&Path>) -> Result<Self, figment::Error> {
        let mut figment = Figment::new();

        if let Some(path) = config_path {
            figment = match path.extension().and_then(|e| e.to_str()) {
                Some("toml") => figment.merge(Toml::file(path)),
                Some("yaml" | "yml") => figment.merge(Yaml::file(path)),
                Some("json") => figment.merge(Json::file(path)),
                _ => {
                    return Err(figment::Error::from(format!(
                        "unsupported config file extension: {}",
                        path.display()
                    )));
                }
            };
        }

        figment.merge(Env::raw()).extract()
    }
}
