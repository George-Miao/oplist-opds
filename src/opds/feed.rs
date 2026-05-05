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

/// Build the OPDS catalog root as the configured OpenList root listing.
pub fn root_listing_feed(
    base_url: &str,
    title: &str,
    dir_path: &str,
    objects: Vec<FsObject>,
    updated: &str,
    proxy_enabled: bool,
) -> Feed {
    directory_feed(
        base_url,
        title,
        dir_path,
        "",
        &format!("{base_url}/opds"),
        objects,
        updated,
        proxy_enabled,
        false,
    )
}

/// Build a directory-listing feed from a slice of `FsObject`s.
///
/// Directories become navigation entries (subsection links).
/// Files become acquisition entries.
/// The feed `type` is determined by the content mix per OPDS 1.2 §2.3.
pub fn browse_feed(
    base_url: &str,
    dir_path: &str,
    route_path: &str,
    objects: Vec<FsObject>,
    updated: &str,
    proxy_enabled: bool,
) -> Feed {
    let self_url = browse_url(base_url, route_path);
    let title = last_segment(dir_path).unwrap_or("Root").to_string();

    directory_feed(
        base_url,
        &title,
        dir_path,
        route_path,
        &self_url,
        objects,
        updated,
        proxy_enabled,
        !route_path.trim_matches('/').is_empty(),
    )
}

/// Build a search-results acquisition feed.
pub fn search_feed(
    base_url: &str,
    root_path: &str,
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
            let route_path = route_path_for_openlist_dir(&dir_path, root_path);
            fs_object_to_entry(base_url, &dir_path, &route_path, obj, proxy_enabled)
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

fn directory_feed(
    base_url: &str,
    title: &str,
    dir_path: &str,
    route_path: &str,
    self_url: &str,
    objects: Vec<FsObject>,
    updated: &str,
    proxy_enabled: bool,
    include_up: bool,
) -> Feed {
    let root_url = format!("{base_url}/opds");
    let has_files = objects.iter().any(|o| !o.is_dir);
    let feed_type = if has_files {
        TYPE_ACQUISITION
    } else {
        TYPE_NAVIGATION
    };

    let mut links = vec![
        Link::new("self", self_url, feed_type),
        Link::new("start", &root_url, TYPE_NAVIGATION),
    ];
    if include_up {
        links.push(Link::new(
            "up",
            &browse_url(base_url, parent_of(route_path)),
            feed_type,
        ));
    }

    let entries = objects
        .into_iter()
        .map(|obj| fs_object_to_entry(base_url, dir_path, route_path, obj, proxy_enabled))
        .collect();

    Feed {
        id: format!("urn:oplist-opds:browse:{}", stable_id(dir_path)),
        title: title.to_string(),
        updated: updated.to_string(),
        links,
        entries,
    }
}

fn fs_object_to_entry(
    base_url: &str,
    dir_path: &str,
    route_path: &str,
    obj: FsObject,
    proxy_enabled: bool,
) -> Entry {
    let full_path = if dir_path.ends_with('/') {
        format!("{}{}", dir_path, obj.name)
    } else {
        format!("{}/{}", dir_path, obj.name)
    };
    let entry_route_path = join_route_path(route_path, &obj.name);

    if obj.is_dir {
        let browse_url = browse_url(base_url, &entry_route_path);
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
            format!("{}/opds/raw/{}", base_url, entry_route_path)
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

fn browse_url(base_url: &str, route_path: &str) -> String {
    let route_path = route_path.trim_matches('/');
    if route_path.is_empty() {
        format!("{base_url}/opds/browse/")
    } else {
        format!("{base_url}/opds/browse/{route_path}")
    }
}

fn join_route_path(route_path: &str, name: &str) -> String {
    let route_path = route_path.trim_matches('/');
    let name = name.trim_matches('/');
    if route_path.is_empty() {
        name.to_string()
    } else {
        format!("{route_path}/{name}")
    }
}

fn route_path_for_openlist_dir(dir_path: &str, root_path: &str) -> String {
    let root = normalize_abs_path(root_path);
    let dir = normalize_abs_path(dir_path);
    if root == "/" {
        return dir.trim_start_matches('/').to_string();
    }

    if dir == root {
        String::new()
    } else if let Some(child) = dir.strip_prefix(&format!("{root}/")) {
        child.trim_matches('/').to_string()
    } else {
        dir.trim_matches('/').to_string()
    }
}

fn normalize_abs_path(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}")
    }
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
