//! rustdoc-JSON → [`ApiSnapshot`](crate::snapshot::ApiSnapshot) producer.
//!
//! Requires the `producer` cargo feature (`serde_json`). Input is a rustdoc
//! JSON file as emitted by:
//!
//! ```bash
//! cargo +nightly rustdoc -- -Z unstable-options --output-format json
//! ```
//!
//! Only in-crate (`crate_id == 0`) public items listed in the `paths` table
//! become [`Item`](crate::snapshot::Item)s. Signature strings are the raw
//! `inner` JSON payload (compact form), so any semantic move flips the sig.
//!
//! # Limitations (documented, not silent)
//!
//! - Methods surface as `ItemKind::Function` (rustdoc lists them as
//!   `function` nodes; no Method split yet).
//! - `sig` is compact JSON, not `pub fn …` rendering — machine-diffable,
//!   human-ugly.
//! - Unknown `inner` shapes are skipped, never error (newer rustdoc
//!   degrades to fewer items, not failure). Built against format v61.

use crate::snapshot::{ApiSnapshot, Item, ItemKind};
use std::collections::HashMap;
use std::fmt;

#[cfg(feature = "producer")]
use serde_json::Value;

/// What went wrong reading a rustdoc JSON document.
#[derive(Debug)]
pub enum ProducerError {
    /// File could not be read.
    Io(std::io::Error),
    /// File is not valid JSON.
    Json(serde_json::Error),
    /// JSON parses but has no usable `index`/`paths` tables.
    BadShape(String),
}

impl fmt::Display for ProducerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "read rustdoc JSON: {e}"),
            Self::Json(e) => write!(f, "parse rustdoc JSON: {e}"),
            Self::BadShape(m) => write!(f, "unexpected rustdoc JSON shape: {m}"),
        }
    }
}

impl From<std::io::Error> for ProducerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ProducerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Read a rustdoc JSON file and build an [`ApiSnapshot`] labelled `version`.
pub fn snapshot_from_rustdoc_json(
    path: &std::path::Path,
    version: &str,
) -> Result<ApiSnapshot, ProducerError> {
    let text = std::fs::read_to_string(path)?;
    snapshot_from_rustdoc_str(&text, version)
}

/// Same as [`snapshot_from_rustdoc_json`] but from an in-memory document.
pub fn snapshot_from_rustdoc_str(
    doc: &str,
    version: &str,
) -> Result<ApiSnapshot, ProducerError> {
    let root: Value = serde_json::from_str(doc)?;
    parse_doc(&root, version)
}
fn parse_doc(root: &Value, version: &str) -> Result<ApiSnapshot, ProducerError> {
    let index = root
        .get("index")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::BadShape("missing `index` object".to_owned()))?;
    let paths = root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::BadShape("missing `paths` object".to_owned()))?;

    let mut id_path: HashMap<&str, (String, bool)> = HashMap::new();
    for (id, p) in paths {
        let dotted = p
            .get("path")
            .and_then(Value::as_array)
            .map(|segs| {
                segs.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("::")
            })
            .unwrap_or_default();
        let in_crate = p.get("crate_id").and_then(Value::as_u64) == Some(0);
        id_path.insert(id.as_str(), (dotted, in_crate));
    }

    let mut items: Vec<Item> = Vec::new();
    for (id, node) in index {
        if node.get("visibility").and_then(Value::as_str) != Some("public") {
            continue;
        }
        let Some((path, in_crate)) = id_path.get(id.as_str()) else {
            continue;
        };
        if !in_crate || path.is_empty() {
            continue;
        }
        let Some(inner) = node.get("inner").and_then(Value::as_object) else {
            continue;
        };
        let mut inner_iter = inner.iter();
        let Some((kind_key, payload)) = inner_iter.next() else {
            continue;
        };
        if inner_iter.next().is_some() {
            continue;
        }
        let kind = match kind_key.as_str() {
            "function" => ItemKind::Function,
            "struct" => ItemKind::Struct,
            "enum" => ItemKind::Enum,
            "trait" => ItemKind::Trait,
            "struct_field" => ItemKind::Field,
            "variant" => ItemKind::Variant,
            "constant" => ItemKind::Const,
            _ => continue,
        };
        items.push(Item {
            path: path.clone(),
            kind,
            sig: compact(payload),
        });
    }

    items.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ApiSnapshot::new(version, items))
}

/// Compact JSON rendering of a rustdoc `inner` payload (the sig).
fn compact(v: &Value) -> String {
    let mut buf = Vec::new();
    write_compact(v, &mut buf);
    String::from_utf8(buf).unwrap_or_default()
}

fn write_compact(v: &Value, out: &mut Vec<u8>) {
    // Structural chars by hand so key order follows the input document;
    // string escaping delegated to serde_json.
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Value::String(s) => {
            let esc = serde_json::Value::String(s.clone()).to_string();
            out.extend_from_slice(esc.as_bytes());
        }
        Value::Array(a) => {
            out.push(b'[');
            for (i, e) in a.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_compact(e, out);
            }
            out.push(b']');
        }
        Value::Object(m) => {
            out.push(b'{');
            for (i, (k, e)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                let esc = serde_json::Value::String(k.clone()).to_string();
                out.extend_from_slice(esc.as_bytes());
                out.push(b':');
                write_compact(e, out);
            }
            out.push(b'}');
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal rustdoc-JSON-shaped doc: index + paths, mixed visibility.
    fn mini_doc() -> String {
        serde_json::json!({
            "index": {
                "10": {"visibility": "public", "inner": {"function": {"sig": "fn a()"}}},
                "20": {"visibility": "public", "inner": {"struct": {"fields": []}}},
                "30": {"visibility": "default", "inner": {"function": {"sig": "fn priv()"}}},
                "40": {"visibility": "public", "inner": {"module": {"items": []}}},
                "50": {"visibility": "public", "inner": {"function": {"sig": "fn ext()"}}},
                "60": {"visibility": "public", "inner": {"weird_future_kind": {}}},
                "70": {"visibility": "public"}
            },
            "paths": {
                "10": {"crate_id": 0, "path": ["demo", "a"]},
                "20": {"crate_id": 0, "path": ["demo", "T"]},
                "30": {"crate_id": 0, "path": ["demo", "priv"]},
                "40": {"crate_id": 0, "path": ["demo", "sub"]},
                "50": {"crate_id": 99, "path": ["std", "ext"]},
                "60": {"crate_id": 0, "path": ["demo", "future"]},
                "70": {"crate_id": 0, "path": ["demo", "noshape"]}
            }
        })
        .to_string()
    }

    #[test]
    fn keeps_public_in_crate_known_kinds() {
        let s = snapshot_from_rustdoc_str(&mini_doc(), "demo 0.1.0").unwrap();
        let paths: Vec<&str> = s.items.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["demo::T", "demo::a"]);
        assert_eq!(s.version, "demo 0.1.0");
    }

    #[test]
    fn sig_change_in_payload_flips_diff() {
        let a = snapshot_from_rustdoc_str(&mini_doc(), "v1").unwrap();
        let evolved = mini_doc().replace("fn a()", "fn a(x: u8)");
        let b = snapshot_from_rustdoc_str(&evolved, "v2").unwrap();
        let d = crate::diff::diff_snapshots(&a, &b);
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].path, "demo::a");
        assert!(d.added.is_empty() && d.removed.is_empty());
    }

    #[test]
    fn bad_shape_errors_not_panics() {
        let err = snapshot_from_rustdoc_str(r#"{"no": "tables"}"#, "v").unwrap_err();
        assert!(matches!(err, ProducerError::BadShape(_)));
        assert!(snapshot_from_rustdoc_str("not json", "v").is_err());
    }

    #[test]
    fn missing_file_errors() {
        let err =
            snapshot_from_rustdoc_json(std::path::Path::new("/nonexistent/x.json"), "v")
                .unwrap_err();
        assert!(matches!(err, ProducerError::Io(_)));
    }
}
