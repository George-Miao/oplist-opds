/// Maps a file extension (lowercase, with leading dot) to its OPDS MIME type.
pub fn mime_for_ext(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "epub" => "application/epub+zip",
        "mobi" => "application/x-mobipocket-ebook",
        "azw" | "azw3" => "application/x-mobi8-ebook",
        "pdf" => "application/pdf",
        "cbz" => "application/vnd.comicbook+zip",
        "cbr" => "application/vnd.comicbook-rar",
        "fb2" => "application/x-fictionbook+xml",
        "djvu" | "djv" => "image/vnd.djvu",
        "txt" => "text/plain",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

// ── Link relation constants
// ───────────────────────────────────────────────────

pub const REL_ACQUISITION: &str = "http://opds-spec.org/acquisition";
#[allow(dead_code)]
pub const REL_IMAGE: &str = "http://opds-spec.org/image";
pub const REL_THUMBNAIL: &str = "http://opds-spec.org/image/thumbnail";

pub const TYPE_NAVIGATION: &str = "application/atom+xml;profile=opds-catalog;kind=navigation";
pub const TYPE_ACQUISITION: &str = "application/atom+xml;profile=opds-catalog;kind=acquisition";
#[allow(dead_code)]
pub const TYPE_OPENSEARCH: &str = "application/opensearchdescription+xml";

// ── Feed / Entry data model
// ───────────────────────────────────────────────────

/// A single link element within a feed or entry.
#[derive(Debug, Clone)]
pub struct Link {
    pub rel: String,
    pub href: String,
    pub mime_type: String,
    /// Optional human-readable title.
    pub title: Option<String>,
    /// Byte length, used on acquisition links.
    pub length: Option<u64>,
}

impl Link {
    pub fn new(
        rel: impl Into<String>,
        href: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            rel: rel.into(),
            href: href.into(),
            mime_type: mime_type.into(),
            title: None,
            length: None,
        }
    }

    pub fn with_length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// An entry within a feed (either navigation or acquisition).
#[derive(Debug, Clone)]
pub struct Entry {
    /// Stable unique ID, e.g. `urn:oplist-opds:<sha256>`.
    pub id: String,
    pub title: String,
    /// RFC 3339 last-updated timestamp.
    pub updated: String,
    pub links: Vec<Link>,
    /// Short human-readable description (plain text).
    pub summary: Option<String>,
}

/// The top-level OPDS Atom feed.
#[derive(Debug)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub updated: String,
    /// `self` link and navigation links (start, up, search, …).
    pub links: Vec<Link>,
    pub entries: Vec<Entry>,
}
