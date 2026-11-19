//! The lore guide (`/lore`): the being atlas, the design and research documents and the
//! reference images, read from the repository on every request so the page follows the files
//! as they change. Read-only, and only from the folders in [`SHARED`].
use crate::Reply;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

pub(crate) const PAGE: &str = include_str!("../web/lore.html");
pub(crate) const MARKED: &str = include_str!("../web/vendor/marked.min.js");

/// What the page may read, relative to the repository root: whole folders end in `/`.
const SHARED: &[&str] = &[
    "NARRATIVE.md",
    "docs/design/",
    "docs/art/",
    "docs/research/beings/",
    "docs/research/music-mythologies.md",
    "docs/research/biome-map-generation.md",
];
/// Documents listed in the page, in this order, then any other `.md` under `docs/art/`.
const DOCUMENTS: &[&str] = &[
    "NARRATIVE.md",
    "docs/design/design-guide.md",
    "docs/art/biome-characters.md",
    "docs/research/beings/README.md",
    "docs/research/music-mythologies.md",
    "docs/research/beings/catalogue.md",
    "docs/research/biome-map-generation.md",
];
const IMAGES: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

/// A repository-relative path the page may read, or `None`.
fn shared(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel_path = Path::new(rel);
    if rel.is_empty()
        || !rel_path
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return None;
    }
    if !SHARED.iter().any(|s| {
        if s.ends_with('/') {
            rel.starts_with(s)
        } else {
            rel == *s
        }
    }) {
        return None;
    }
    let path = root.join(rel_path).canonicalize().ok()?;
    path.starts_with(root.canonicalize().ok()?).then_some(path)
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "md" => "text/markdown; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "text/plain; charset=utf-8",
    }
}

/// Every file under `dir` (repository-relative), sorted.
fn walk(root: &Path, dir: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = format!("{dir}{name}");
        if entry.path().is_dir() {
            walk(root, &format!("{rel}/"), out);
        } else {
            out.push(rel);
        }
    }
}

fn read_json(root: &Path, rel: &str) -> Value {
    std::fs::read_to_string(root.join(rel))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

/// The atlas, its layouts, the documents and the reference images.
pub(crate) fn index(root: &Path) -> Reply {
    let mut files = Vec::new();
    for dir in SHARED.iter().filter(|s| s.ends_with('/')) {
        walk(root, dir, &mut files);
    }
    let is = |f: &str, exts: &[&str]| {
        Path::new(f)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e.to_ascii_lowercase().as_str()))
    };
    let mut documents: Vec<String> = DOCUMENTS
        .iter()
        .filter(|d| root.join(d).is_file())
        .map(|d| d.to_string())
        .collect();
    let more: Vec<String> = files
        .iter()
        .filter(|f| f.starts_with("docs/art/") && is(f, &["md"]) && !documents.contains(f))
        .cloned()
        .collect();
    documents.extend(more);
    let layouts: Vec<Value> = files
        .iter()
        .filter(|f| f.starts_with("docs/research/beings/") && f.ends_with("/layout.json"))
        .map(|f| json!({ "path": f, "layout": read_json(root, f) }))
        .collect();
    Reply::json(
        200,
        &json!({
            "atlas": read_json(root, "docs/research/beings/beings.json"),
            "layouts": layouts,
            "documents": documents,
            "images": files.iter().filter(|f| is(f, IMAGES)).collect::<Vec<_>>(),
        }),
    )
}

/// One shared file, bytes as they are on disk.
pub(crate) fn file(root: &Path, rel: &str) -> Reply {
    match shared(root, rel).and_then(|path| std::fs::read(&path).ok().map(|bytes| (path, bytes))) {
        Some((path, bytes)) => Reply {
            status: 200,
            content_type: content_type(&path),
            body: bytes,
        },
        None => Reply::error(404, format!("not a shared file: {rel}")),
    }
}

/// `%XX` and `+` decoding for a query value.
pub(crate) fn decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
                match hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(b) => {
                        out.push(b);
                        i += 3;
                        continue;
                    }
                    None => out.push(b'%'),
                }
            }
            b'+' => out.push(b' '),
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn shares_the_atlas_documents_and_images_only() {
        let index: Value = serde_json::from_slice(&super::index(&root()).body).unwrap();
        assert!(
            index["atlas"]["beings"]
                .as_array()
                .is_some_and(|b| !b.is_empty())
        );
        assert!(
            index["documents"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d == "docs/design/design-guide.md")
        );
        assert!(
            index["images"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| i.as_str().unwrap().starts_with("docs/"))
        );
        assert_eq!(file(&root(), "docs/design/design-guide.md").status, 200);
        for refused in [
            "Cargo.toml",
            "docs/../Cargo.toml",
            "/etc/passwd",
            "docs/research/audio-engine.md",
        ] {
            assert_eq!(file(&root(), refused).status, 404, "{refused}");
        }
        assert_eq!(decode("docs%2Fart%2Fomar.md"), "docs/art/omar.md");
    }
}
