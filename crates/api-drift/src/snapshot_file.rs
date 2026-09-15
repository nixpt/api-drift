//! Snapshot file format v1: the committed contract.
//!
//! Requires the `serde` feature. A `*.snapshot.json` file is what the
//! embedded ledger (APIDRIFT-6) commits per surface: a versioned envelope
//! around an [`ApiSnapshot`] plus the sha256 content hash that ledger entries
//! and consumer pins (`api-drift.toml`) reference.
//!
//! ```json
//! {"format": "api-drift/snapshot/v1",
//!  "surface": "rust-public",
//!  "version": "arniko 0.2.99",
//!  "items": [{"path": "…", "kind": "Function", "sig": "…", "attrs": []}],
//!  "content_hash": "sha256:…"}
//! ```
//!
//! Rules: `format` must equal [`SNAPSHOT_FORMAT_V1`] on parse; `items` are
//! sorted by path on render; `content_hash` is recomputed by
//! [`content_hash`] and verified by [`parse_snapshot_file`] (mismatch =
//! error, not warning — a tampered contract must fail closed).

use crate::snapshot::{ApiSnapshot, Item};
use std::fmt;

/// The only accepted `format` string for v1 files.
pub const SNAPSHOT_FORMAT_V1: &str = "api-drift/snapshot/v1";

/// Committed envelope: format tag + surface name + snapshot + hash.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnapshotFile {
    /// Always [`SNAPSHOT_FORMAT_V1`].
    pub format: String,
    /// Surface name, e.g. `rust-public`, `acp-contract`.
    pub surface: String,
    /// Version label from the snapshot (e.g. `arniko 0.2.99`).
    pub version: String,
    /// Sorted-by-path items.
    pub items: Vec<Item>,
    /// `sha256:` hex of the canonical content (see [`content_hash`]).
    pub content_hash: String,
}

/// What went wrong reading or writing a snapshot file.
#[derive(Debug)]
pub enum SnapshotFileError {
    /// Underlying JSON error.
    Json(serde_json::Error),
    /// `format` tag mismatch (expected, found).
    BadFormat(String, String),
    /// `content_hash` does not recompute (expected, found).
    HashMismatch(String, String),
}

impl fmt::Display for SnapshotFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "snapshot file JSON: {e}"),
            Self::BadFormat(exp, got) => {
                write!(f, "snapshot format: expected `{exp}`, found `{got}`")
            }
            Self::HashMismatch(exp, got) => {
                write!(
                    f,
                    "snapshot content hash mismatch: expected `{exp}`, found `{got}`"
                )
            }
        }
    }
}

impl From<serde_json::Error> for SnapshotFileError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl std::error::Error for SnapshotFileError {}

/// sha256 over the canonical content: for each item (sorted by path)
/// `path \0 kind \0 sig \0 attr1,attr2(sorted) \0`, joined by `\n`.
/// Returns the `sha256:<hex>` string stored in the file.
pub fn content_hash(items: &[Item]) -> String {
    use sha2::{Digest, Sha256};
    let mut sorted: Vec<&Item> = items.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut h = Sha256::new();
    for (i, it) in sorted.iter().enumerate() {
        if i > 0 {
            h.update(b"\n");
        }
        h.update(it.path.as_bytes());
        h.update(b"\0");
        h.update(format!("{:?}", it.kind).as_bytes());
        h.update(b"\0");
        h.update(it.sig.as_bytes());
        h.update(b"\0");
        let mut attrs = it.attrs.clone();
        attrs.sort();
        h.update(attrs.join(",").as_bytes());
        h.update(b"\0");
    }
    format!("sha256:{}", hex::encode(h.finalize()))
}

/// Build the committed envelope for `surface` from a snapshot.
pub fn render_snapshot_file(surface: &str, snap: &ApiSnapshot) -> SnapshotFile {
    let mut items = snap.items.clone();
    items.sort_by(|a, b| a.path.cmp(&b.path));
    let content_hash = content_hash(&items);
    SnapshotFile {
        format: SNAPSHOT_FORMAT_V1.to_owned(),
        surface: surface.to_owned(),
        version: snap.version.clone(),
        items,
        content_hash,
    }
}

/// sha256 over a canonical schema (or any contract text) — the sig a
/// [`ProtocolSurface`](crate::surface::ProtocolSurface) uses for a
/// request/response schema. Caller must canonicalize first (stable key
/// order + formatting): two logically-identical schemas with different
/// formatting hash differently, on purpose — drift should be visible.
pub fn schema_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("sha256:{}", hex::encode(h.finalize()))
}

/// Serialize the envelope to pretty JSON (the committed file bytes).
pub fn to_string_pretty(file: &SnapshotFile) -> Result<String, SnapshotFileError> {
    Ok(serde_json::to_string_pretty(file)?)
}

/// Parse + verify: format tag must match, hash must recompute.
/// Returns the envelope (use `.items` / `.version` for the snapshot).
pub fn parse_snapshot_file(text: &str) -> Result<SnapshotFile, SnapshotFileError> {
    let file: SnapshotFile = serde_json::from_str(text)?;
    if file.format != SNAPSHOT_FORMAT_V1 {
        return Err(SnapshotFileError::BadFormat(
            SNAPSHOT_FORMAT_V1.to_owned(),
            file.format.clone(),
        ));
    }
    let recomputed = content_hash(&file.items);
    if recomputed != file.content_hash {
        return Err(SnapshotFileError::HashMismatch(
            recomputed,
            file.content_hash.clone(),
        ));
    }
    Ok(file)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{ApiSnapshot, ItemKind};

    fn snap() -> ApiSnapshot {
        ApiSnapshot::new(
            "demo 0.1.0",
            vec![
                Item::new("demo::b", ItemKind::Function, "pub fn b()"),
                Item::new("demo::a", ItemKind::Struct, "pub struct A"),
            ],
        )
    }

    #[test]
    fn round_trip_verifies() {
        let file = render_snapshot_file("rust-public", &snap());
        // Items sorted by path regardless of input order.
        assert_eq!(file.items[0].path, "demo::a");
        assert!(file.content_hash.starts_with("sha256:"));
        let text = to_string_pretty(&file).unwrap();
        let back = parse_snapshot_file(&text).unwrap();
        assert_eq!(back, file);
        assert_eq!(back.version, "demo 0.1.0");
    }

    #[test]
    fn tampered_items_fail_hash() {
        let file = render_snapshot_file("rust-public", &snap());
        let mut text = to_string_pretty(&file).unwrap();
        text = text.replace("pub fn b()", "pub fn b(x: u8)");
        let err = parse_snapshot_file(&text).unwrap_err();
        assert!(matches!(err, SnapshotFileError::HashMismatch(_, _)));
    }

    #[test]
    fn wrong_format_rejected() {
        let mut file = render_snapshot_file("rust-public", &snap());
        file.format = "api-drift/snapshot/v0".to_owned();
        // Re-hash so the failure is the format tag, not the hash.
        file.content_hash = content_hash(&file.items);
        let text = serde_json::to_string_pretty(&file).unwrap();
        let err = parse_snapshot_file(&text).unwrap_err();
        assert!(matches!(err, SnapshotFileError::BadFormat(_, _)));
    }

    #[test]
    fn hash_stable_across_item_order_and_attr_order() {
        let a = ApiSnapshot::new(
            "v",
            vec![
                Item::new("m::f", ItemKind::Function, "s").with_attrs(&["b", "a"]),
                Item::new("m::g", ItemKind::Function, "s"),
            ],
        );
        let b = ApiSnapshot::new(
            "v",
            vec![
                Item::new("m::g", ItemKind::Function, "s"),
                Item::new("m::f", ItemKind::Function, "s").with_attrs(&["a", "b"]),
            ],
        );
        assert_eq!(content_hash(&a.items), content_hash(&b.items));
    }

    #[test]
    fn schema_hash_is_stable_and_responsive() {
        let a = schema_hash(r#"{"type":"object","properties":{"id":{"type":"string"}}}"#);
        let b = schema_hash(r#"{"type":"object","properties":{"id":{"type":"string"}}}"#);
        let c = schema_hash(r#"{"type":"object","properties":{"id":{"type":"integer"}}}"#);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with("sha256:"));
    }
}
