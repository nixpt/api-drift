//! Versioned inventory of a crate's public surface.
//!
//! A [`ApiSnapshot`] is what a producer (rustdoc JSON reader, `cargo
//! public-api` wrapper, tree-sitter pass…) hands to [`crate::diff`]. It is
//! deliberately coarse: path + kind + signature string. Deep semantic
//! comparison lives in `classify`, not here.

/// What kind of public item this is. Used by `classify` to tell a
/// struct-field removal apart from a function-signature change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Field,
    Variant,
    Const,
    Static,
    Module,
    /// Anything else (type alias, macro…). Kept so producers never have to drop an item.
    Other,
}

/// One public item in a crate's surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    /// Canonical path, e.g. `arniko::Alert::new`.
    pub path: String,
    /// What kind of item it is.
    pub kind: ItemKind,
    /// Rendered signature, e.g. `pub fn new(message: &str) -> Self`.
    /// Compared verbatim by `diff`; interpreted by `classify`.
    pub sig: String,
}

impl Item {
    /// Build an item from anything string-like.
    pub fn new(path: &str, kind: ItemKind, sig: &str) -> Self {
        Self {
            path: path.to_owned(),
            kind,
            sig: sig.to_owned(),
        }
    }
}

/// A versioned inventory of one crate release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSnapshot {
    /// Human label, e.g. `arniko 0.2.99` or a git rev.
    pub version: String,
    /// The public surface. Order is insignificant; `diff` indexes by `path`.
    /// Duplicate paths: last one wins in `diff` (producers should dedupe).
    pub items: Vec<Item>,
}

impl ApiSnapshot {
    /// Build a snapshot from a version label and an item list.
    pub fn new(version: &str, items: Vec<Item>) -> Self {
        Self {
            version: version.to_owned(),
            items,
        }
    }

    /// Look up one item by path.
    pub fn get(&self, path: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.path == path)
    }

    /// Number of inventoried items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when the snapshot inventories nothing.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_finds_by_path() {
        let s = ApiSnapshot::new(
            "arniko 0.2.99",
            vec![Item::new("arniko::Alert::new", ItemKind::Function, "pub fn new")],
        );
        assert!(s.get("arniko::Alert::new").is_some());
        assert!(s.get("arniko::Missing").is_none());
    }

    #[test]
    fn len_and_is_empty() {
        assert!(ApiSnapshot::new("v0", vec![]).is_empty());
        let s = ApiSnapshot::new(
            "v1",
            vec![Item::new("a::b", ItemKind::Struct, "pub struct B")],
        );
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
    }
}
