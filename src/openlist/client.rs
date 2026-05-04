use cyper::Client;
use snafu::ResultExt;

use super::types::{
    FsGetRequest, FsGetResponse, FsListData, FsListRequest, FsListResponse, FsObject, FsSearchData,
    FsSearchRequest, FsSearchResponse,
};
use crate::error::{AppError, OpenListRequestSnafu, Result};

/// Client wrapper for the OpenList file-system API.
#[derive(Clone)]
pub struct OpenListClient {
    client: Client,
    base_url: String,
    token: String,
}

impl OpenListClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            token: token.into(),
        }
    }

    fn api_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), endpoint)
    }

    /// Execute a JSON POST and return the deserialised envelope.
    /// cyper's `.json::<T>()` handles both sending and deserialising in one
    /// step.
    async fn post_json<T>(
        &self,
        endpoint: &str,
        body: &impl serde::Serialize,
        path: &str,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.api_url(endpoint);

        let mut builder = self
            .client
            .post(&url)
            .context(OpenListRequestSnafu { path })?
            .json(body)
            .context(OpenListRequestSnafu { path })?;

        if !self.token.is_empty() {
            builder = builder
                .header("Authorization", &self.token)
                .context(OpenListRequestSnafu { path })?;
        }

        builder
            .send()
            .await
            .context(OpenListRequestSnafu { path })?
            .json::<T>()
            .await
            .context(OpenListRequestSnafu { path })
    }

    /// List the contents of a directory.
    pub async fn list(&self, path: &str, page: u32, per_page: u32) -> Result<FsListData> {
        let body = FsListRequest {
            path,
            password: "",
            page,
            per_page,
            refresh: false,
        };

        let resp = self
            .post_json::<FsListResponse>("/api/fs/list", &body, path)
            .await?;

        if resp.code != 200 {
            return Err(AppError::OpenListApi {
                path: path.to_string(),
                code: resp.code,
                message: resp.message,
            });
        }

        resp.data.ok_or_else(|| AppError::NotFound {
            path: path.to_string(),
        })
    }

    /// Search files and directories.
    pub async fn search(
        &self,
        parent: &str,
        keywords: &str,
        page: u32,
        per_page: u32,
    ) -> Result<FsSearchData> {
        let body = FsSearchRequest {
            parent,
            keywords,
            scope: 0,
            page,
            per_page,
            password: "",
        };

        let resp = self
            .post_json::<FsSearchResponse>("/api/fs/search", &body, keywords)
            .await?;

        if resp.code != 200 {
            return Err(AppError::OpenListApi {
                path: keywords.to_string(),
                code: resp.code,
                message: resp.message,
            });
        }

        resp.data.ok_or_else(|| AppError::Internal {
            message: "search returned no data".to_string(),
        })
    }

    /// Get metadata for a single file or directory.
    pub async fn get(&self, path: &str) -> Result<FsObject> {
        let body = FsGetRequest { path, password: "" };

        let resp = self
            .post_json::<FsGetResponse>("/api/fs/get", &body, path)
            .await?;

        if resp.code == 404 {
            return Err(AppError::NotFound {
                path: path.to_string(),
            });
        }
        if resp.code != 200 {
            return Err(AppError::OpenListApi {
                path: path.to_string(),
                code: resp.code,
                message: resp.message,
            });
        }

        resp.data.ok_or_else(|| AppError::NotFound {
            path: path.to_string(),
        })
    }
}
