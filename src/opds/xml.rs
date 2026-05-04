use std::fmt::Write as _;

use crate::{
    error::Result,
    opds::entry::{Entry, Feed, Link},
};

/// Serialise a `Feed` to an OPDS 1.2 Atom XML string.
pub fn feed_to_xml(feed: &Feed) -> Result<String> {
    let mut out = String::with_capacity(4096);

    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(
        r#"<feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="http://opds-spec.org/2010/catalog" xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">"#,
    );
    out.push('\n');

    write_elem(&mut out, "id", &feed.id);
    write_elem(&mut out, "title", &feed.title);
    write_elem(&mut out, "updated", &feed.updated);

    for link in &feed.links {
        write_link(&mut out, link);
    }

    for entry in &feed.entries {
        write_entry(&mut out, entry);
    }

    out.push_str("</feed>\n");
    Ok(out)
}

// ── Helpers
// ───────────────────────────────────────────────────────────────────

fn write_elem(out: &mut String, tag: &str, value: &str) {
    let _ = write!(out, "  <{tag}>{}</{tag}>\n", escape_xml(value));
}

fn write_link(out: &mut String, link: &Link) {
    let _ = write!(
        out,
        r#"  <link rel="{}" href="{}" type="{}""#,
        escape_attr(&link.rel),
        escape_attr(&link.href),
        escape_attr(&link.mime_type),
    );
    if let Some(title) = &link.title {
        let _ = write!(out, r#" title="{}""#, escape_attr(title));
    }
    if let Some(len) = link.length {
        let _ = write!(out, r#" length="{len}""#);
    }
    out.push_str("/>\n");
}

fn write_entry(out: &mut String, entry: &Entry) {
    out.push_str("  <entry>\n");
    let _ = write!(out, "    <id>{}</id>\n", escape_xml(&entry.id));
    let _ = write!(out, "    <title>{}</title>\n", escape_xml(&entry.title));
    let _ = write!(out, "    <updated>{}</updated>\n", &entry.updated);
    if let Some(summary) = &entry.summary {
        let _ = write!(
            out,
            r#"    <summary type="text">{}</summary>"#,
            escape_xml(summary)
        );
        out.push('\n');
    }
    for link in &entry.links {
        out.push_str("  "); // extra indent inside <entry>
        write_link(out, link);
    }
    out.push_str("  </entry>\n");
}

/// Escape XML text content (& < > ' ").
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape XML attribute values.
fn escape_attr(s: &str) -> String {
    escape_xml(s).replace('"', "&quot;").replace('\'', "&apos;")
}
