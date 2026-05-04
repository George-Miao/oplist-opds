use serde::{Deserialize, Serialize};

// ── Request bodies
// ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FsListRequest<'a> {
    pub path: &'a str,
    #[serde(default)]
    pub password: &'a str,
    pub page: u32,
    pub per_page: u32,
    pub refresh: bool,
}

#[derive(Debug, Serialize)]
pub struct FsSearchRequest<'a> {
    pub parent: &'a str,
    pub keywords: &'a str,
    pub scope: u8,
    pub page: u32,
    pub per_page: u32,
    #[serde(default)]
    pub password: &'a str,
}

#[derive(Debug, Serialize)]
pub struct FsGetRequest<'a> {
    pub path: &'a str,
    #[serde(default)]
    pub password: &'a str,
}

// ── Shared data types
// ─────────────────────────────────────────────────────────

/// A single file or directory entry as returned by OpenList.
#[derive(Debug, Deserialize)]
pub struct FsObject {
    pub name: String,
    /// Parent directory path; only present in search results.
    pub parent: Option<String>,
    /// File size in bytes; 0 for directories.
    pub size: u64,
    pub is_dir: bool,
    /// RFC 3339 modification timestamp; absent in search results.
    pub modified: Option<String>,
    /// RFC 3339 creation timestamp; absent in search results.
    pub created: Option<String>,
    /// Auth sign required for protected downloads; may be empty.
    #[serde(default)]
    pub sign: String,
    /// Thumbnail URL; may be empty.
    #[serde(default)]
    pub thumb: String,
    /// File type code: 1 = folder, others vary.
    #[serde(rename = "type")]
    pub file_type: u8,
    /// Direct download URL; populated for files, empty for directories.
    #[serde(default)]
    pub raw_url: String,
}

// ── Response envelopes
// ────────────────────────────────────────────────────────

/// Generic OpenList API envelope.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    #[serde(default)]
    pub message: String,
    pub data: Option<T>,
}

/// `data` payload for `/api/fs/list`.
#[derive(Debug, Deserialize)]
pub struct FsListData {
    pub content: Vec<FsObject>,
    pub total: u64,
}

/// `data` payload for `/api/fs/search`.
#[derive(Debug, Deserialize)]
pub struct FsSearchData {
    pub content: Vec<FsObject>,
    pub total: u64,
}

// Type aliases for the full response types.
pub type FsListResponse = ApiResponse<FsListData>;
pub type FsSearchResponse = ApiResponse<FsSearchData>;
pub type FsGetResponse = ApiResponse<FsObject>;
