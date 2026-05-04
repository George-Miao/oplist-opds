use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{
    now_rfc3339,
    opds::entry::{
        Entry, Feed, Link, REL_ACQUISITION, REL_THUMBNAIL, TYPE_ACQUISITION, TYPE_NAVIGATION,
        mime_for_ext,
    },
    openlist::types::FsObject,
};

/// Build a catalog-root navigation feed with links to Browse and Search.
pub fn root_feed(base_url: &str, title: &str, updated: &str) -> Feed {
    let self_url = format!("{base_url}/opds");
    let browse_url = format!("{base_url}/opds/browse/");
    let search_url = format!("{base_url}/opds/search");
    let opensearch_url = format!("{base_url}/opds/opensearch.xml");

    Feed {
        id: format!("urn:oplist-opds:root"),
        title: title.to_string(),
        updated: updated.to_string(),
        links: vec![
            Link::new("self", &self_url, TYPE_NAVIGATION),
            Link::new("start", &self_url, TYPE_NAVIGATION),
            Link::new(
                "search",
                &opensearch_url,
                "application/opensearchdescription+xml",
            ),
        ],
        entries: vec![
            Entry {
                id: "urn:oplist-opds:browse".to_string(),
                title: "Browse Files".to_string(),
                updated: updated.to_string(),
                links: vec![
                    Link::new("subsection", &browse_url, TYPE_ACQUISITION)
                        .with_title("Browse Files"),
                ],
                summary: Some("Browse the file collection by directory.".to_string()),
            },
            Entry {
                id: "urn:oplist-opds:search".to_string(),
                title: "Search".to_string(),
                updated: updated.to_string(),
                links: vec![
                    Link::new("search", &search_url, TYPE_ACQUISITION).with_title("Search"),
                ],
                summary: Some("Search for files by name.".to_string()),
            },
        ],
    }
}

/// Build a directory-listing feed from a slice of `FsObject`s.
///
/// Directories become navigation entries (subsection links).
/// Files become acquisition entries.
/// The feed `type` is determined by the content mix per OPDS 1.2 §2.3.
pub fn browse_feed(
    base_url: &str,
    dir_path: &str,
    objects: Vec<FsObject>,
    updated: &str,
    proxy_enabled: bool,
) -> Feed {
    let self_url = format!(
        "{base_url}/opds/browse/{}",
        dir_path.trim_start_matches('/')
    );
    let root_url = format!("{base_url}/opds");

    let _has_dirs = objects.iter().any(|o| o.is_dir);
    let has_files = objects.iter().any(|o| !o.is_dir);
    let feed_type = if has_files {
        TYPE_ACQUISITION
    } else {
        TYPE_NAVIGATION
    };

    // Parent path for "up" link
    let parent_path = parent_of(dir_path);
    let up_url = if parent_path.is_empty() || parent_path == "/" {
        format!("{base_url}/opds/browse/")
    } else {
        format!(
            "{base_url}/opds/browse/{}",
            parent_path.trim_start_matches('/')
        )
    };

    let mut links = vec![
        Link::new("self", &self_url, feed_type),
        Link::new("start", &root_url, TYPE_NAVIGATION),
    ];
    // Only add "up" if we're not already at the root
    if dir_path != "/" && !dir_path.is_empty() {
        links.push(Link::new("up", &up_url, feed_type));
    }

    let entries = objects
        .into_iter()
        .map(|obj| fs_object_to_entry(base_url, dir_path, obj, proxy_enabled))
        .collect();

    Feed {
        id: format!("urn:oplist-opds:browse:{}", stable_id(dir_path)),
        title: last_segment(dir_path).unwrap_or("Root").to_string(),
        updated: updated.to_string(),
        links,
        entries,
    }
}

/// Build a search-results acquisition feed.
pub fn search_feed(
    base_url: &str,
    query: &str,
    objects: Vec<FsObject>,
    updated: &str,
    proxy_enabled: bool,
) -> Feed {
    let self_url = format!("{base_url}/opds/search?q={}", urlencoded(query));
    let root_url = format!("{base_url}/opds");

    let entries = objects
        .into_iter()
        .map(|obj| {
            // Search results include the parent directory; construct the full
            // path so browse links and proxy URLs are correct.
            let dir_path = obj.parent.as_deref().unwrap_or("/").to_string();
            fs_object_to_entry(base_url, &dir_path, obj, proxy_enabled)
        })
        .collect();

    Feed {
        id: format!("urn:oplist-opds:search:{}", stable_id(query)),
        title: format!("Search: {query}"),
        updated: updated.to_string(),
        links: vec![
            Link::new("self", &self_url, TYPE_ACQUISITION),
            Link::new("start", &root_url, TYPE_NAVIGATION),
        ],
        entries,
    }
}

// ── Helpers
// ───────────────────────────────────────────────────────────────────

fn fs_object_to_entry(base_url: &str, dir_path: &str, obj: FsObject, proxy_enabled: bool) -> Entry {
    let full_path = if dir_path.ends_with('/') {
        format!("{}{}", dir_path, obj.name)
    } else {
        format!("{}/{}", dir_path, obj.name)
    };

    if obj.is_dir {
        let browse_url = format!(
            "{}/opds/browse/{}",
            base_url,
            full_path.trim_start_matches('/')
        );
        Entry {
            id: format!("urn:oplist-opds:{}", stable_id(&full_path)),
            title: obj.name,
            updated: obj.modified.unwrap_or_else(now_rfc3339),
            links: vec![Link::new("subsection", &browse_url, TYPE_NAVIGATION)],
            summary: None,
        }
    } else {
        // When proxy is enabled, route through our proxy endpoint so the
        // OpenList token is never exposed to the OPDS client.
        // When disabled, link directly to OpenList's raw_url if available,
        // falling back to the proxy URL if raw_url is empty.
        let download_url = if proxy_enabled || obj.raw_url.is_empty() {
            format!(
                "{}/opds/raw/{}",
                base_url,
                full_path.trim_start_matches('/')
            )
        } else {
            obj.raw_url
        };
        let mime = mime_for_ext(&obj.name);

        let mut links = vec![Link::new(REL_ACQUISITION, &download_url, mime).with_length(obj.size)];

        if !obj.thumb.is_empty() {
            links.push(Link::new(REL_THUMBNAIL, &obj.thumb, "image/jpeg"));
        }

        Entry {
            id: format!("urn:oplist-opds:{}", stable_id(&full_path)),
            title: obj.name,
            updated: obj.modified.unwrap_or_else(now_rfc3339),
            links,
            summary: None,
        }
    }
}

fn stable_id(s: &str) -> String {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn parent_of(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(idx) if idx == 0 => "/",
        Some(idx) => &trimmed[..idx],
        None => "",
    }
}

fn last_segment(path: &str) -> Option<&str> {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}
