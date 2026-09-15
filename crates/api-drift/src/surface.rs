//! Surfaces: any contract that can produce an [`ApiSnapshot`].
//!
//! A crate's Rust `pub` items are one surface. The embedded-ledger design
//! (`docs/DESIGN-embedded-ledger.md`) names others that matter as much to
//! consumers: ACP/MCP methods and their schemas, enum variants (the
//! `BackendEvent` case), CLI flags, config keys. Every one is a [`Surface`]
//! that produces an `ApiSnapshot`; the core does not care which.
//!
//! This module is pure std — no producer machinery, no I/O. Producers live
//! behind features (see [`crate::producer`]) or in hand-written
//! `contract_surface()` fns like the design doc's `bro_tui::acp` example.

use crate::snapshot::{ApiSnapshot, Item, ItemKind};

/// A named contract whose current shape can be inventoried.
///
/// `name` is the stable identifier the ledger keys files on
/// (`<name>.snapshot.json`); it must not change once committed. `snapshot`
/// regenerates the current contract shape at test time.
pub trait Surface {
    /// Stable ledger key, e.g. `rust-public`, `acp-contract`, `item-kind`.
    fn name(&self) -> &str;
    /// The contract's current shape.
    fn snapshot(&self) -> ApiSnapshot;
}

/// A [`Surface`] over the variants of one public enum — the
/// `bro_tui::backend::BackendEvent` case. Each variant becomes a
/// [`ItemKind::Variant`] item with path `<enum>::<variant>`; the sig is the
/// variant's constructor text when the caller supplies one, else the bare
/// name.
///
/// Not reflection: `enum_surface!` is applied where the variant list is
/// written. A new variant without updating the list (or the enum) is exactly
/// the drift the ledger test then catches — the enum and its surface list
/// disagree, which is a real finding, not a false positive. Keep the list
/// next to the enum.
#[derive(Debug, Clone)]
pub struct EnumSurface {
    surface_name: &'static str,
    enum_path: &'static str,
    variants: Vec<Item>,
}

impl EnumSurface {
    /// Build from a variant list: `(variant_name, constructor_sig)`. Empty
    /// sigs render as the bare variant name.
    pub fn new(
        surface_name: &'static str,
        enum_path: &'static str,
        variants: &[(&str, &str)],
    ) -> Self {
        Self {
            surface_name,
            enum_path,
            variants: variants
                .iter()
                .map(|(n, sig)| {
                    let sig = if sig.is_empty() {
                        (*n).to_owned()
                    } else {
                        (*sig).to_owned()
                    };
                    Item::new(&format!("{enum_path}::{n}"), ItemKind::Variant, &sig)
                })
                .collect(),
        }
    }

    /// The enum path this surface covers.
    pub fn enum_path(&self) -> &str {
        self.enum_path
    }
}

impl Surface for EnumSurface {
    fn name(&self) -> &str {
        self.surface_name
    }

    fn snapshot(&self) -> ApiSnapshot {
        ApiSnapshot::new(self.enum_path, self.variants.clone())
    }
}

/// List one public enum's variants as a ledger [`Surface`].
///
/// Applied next to the enum definition (or wherever the variant list is
/// maintained); a drift between the enum and this list is a real finding
/// the ledger test catches. See [`EnumSurface`] for why this is not
/// reflection.
///
/// ```ignore
/// enum_surface!("backend-events", bro_tui::backend::BackendEvent,
///     [Output: "Output(String)", Disconnected, Partial]);
/// ```
#[macro_export]
macro_rules! enum_surface {
    ($name:literal, $path:path, [ $($variant:ident : $sig:literal),* $(,)? ]) => {
        $crate::surface::EnumSurface::new(
            $name,
            stringify!($path),
            &[$( (stringify!($variant), $sig) ),*],
        )
    };
    ($name:literal, $path:path, [ $($variant:ident),* $(,)? ]) => {
        $crate::surface::EnumSurface::new(
            $name,
            stringify!($path),
            &[$( (stringify!($variant), "") ),*],
        )
    };
}

/// A [`Surface`] over rows of a data catalog — the checkstand product-catalog
/// case. Each row becomes an [`ItemKind::Row`] item; `path` is a stable key
/// (e.g. `products::widget`), `sig` is the canonical row rendering (name +
/// price + threshold, or whatever the row's contract is).
///
/// Classification falls out of the existing rules untouched: a new row is
/// `Added` (Compatible), a changed row is `SignatureChanged` (Warning — a
/// re-price is semantic drift, not a build break), a removed row is
/// `Removed` (Breaking, for any pending reference), and `attrs =
/// ["deprecated"]` marks delist-soon.
#[derive(Debug, Clone, Default)]
pub struct CatalogSurface {
    surface_name: &'static str,
    rows: Vec<Item>,
}

impl CatalogSurface {
    /// Build from rows: `(path, sig, attrs)`. Empty `attrs` allowed.
    pub fn new(surface_name: &'static str, rows: &[(&str, &str, &[&str])]) -> Self {
        Self {
            surface_name,
            rows: rows
                .iter()
                .map(|(path, sig, attrs)| Item::new(path, ItemKind::Row, sig).with_attrs(attrs))
                .collect(),
        }
    }
}

impl Surface for CatalogSurface {
    fn name(&self) -> &str {
        self.surface_name
    }

    fn snapshot(&self) -> ApiSnapshot {
        ApiSnapshot::new(self.surface_name, self.rows.clone())
    }
}

/// List one data catalog's rows as a ledger [`Surface`].
///
/// ```ignore
/// catalog_surface!("catalog",
///     ["products::widget": "Widget 500 0.10", "products::gadget": "Gadget 1200 0.05"]);
/// ```
#[macro_export]
macro_rules! catalog_surface {
    ($name:literal, [ $($key:literal : $sig:literal),* $(,)? ]) => {
        $crate::surface::CatalogSurface::new(
            $name,
            &[$( ($key, $sig, &[] as &[&str]) ),*],
        )
    };
}

/// A [`Surface`] over a protocol's methods — ACP/MCP tool calls. Each method
/// becomes an [`ItemKind::Method`] item; `path` is the wire method name
/// (e.g. `bro.checkout`, `_bro/extension`), `sig` is the request/response
/// schema hash the producer computed (`crate::schema_hash`), so a schema
/// change flips the sig and `classify` flags it `SignatureChanged`.
///
/// Classification uses the existing `Method` arm: a schema change is
/// `Breaking` (a request/response contract move breaks every caller), an
/// added method is `Compatible`, a removed method is `Breaking`.
#[derive(Debug, Clone, Default)]
pub struct ProtocolSurface {
    surface_name: &'static str,
    methods: Vec<Item>,
}

impl ProtocolSurface {
    /// Build from methods: `(wire_name, schema_hash_sig)`.
    pub fn new(surface_name: &'static str, methods: &[(&str, &str)]) -> Self {
        Self {
            surface_name,
            methods: methods
                .iter()
                .map(|(name, sig)| Item::new(name, ItemKind::Method, sig))
                .collect(),
        }
    }
}

impl Surface for ProtocolSurface {
    fn name(&self) -> &str {
        self.surface_name
    }

    fn snapshot(&self) -> ApiSnapshot {
        ApiSnapshot::new(self.surface_name, self.methods.clone())
    }
}

/// List one protocol's methods as a ledger [`Surface`].
///
/// ```ignore
/// protocol_surface!("acp-contract",
///     ["bro.checkout": "sha256:…", "_bro/extension": "sha256:…"]);
/// ```
#[macro_export]
macro_rules! protocol_surface {
    ($name:literal, [ $($method:literal : $sig:literal),* $(,)? ]) => {
        $crate::surface::ProtocolSurface::new(
            $name,
            &[$( ($method, $sig) ),*],
        )
    };
}

/// A [`Surface`] over a crate's Rust `pub` API, produced from rustdoc JSON
/// (`cargo +nightly rustdoc -- -Z unstable-options --output-format json`).
///
/// Requires the `producer` feature (pulls `serde_json`). The snapshot is
/// parsed **eagerly** at construction (a broken rustdoc file fails then and
/// there, not silently at test time); `Surface::snapshot` clones it.
///
/// ```ignore
/// let s = api_drift::RustPublicSurface::from_json_file(
///     std::path::Path::new("target/doc/arniko.json"), "arniko 0.2.99",
/// )?;
/// ```
#[cfg(feature = "producer")]
#[derive(Debug, Clone)]
pub struct RustPublicSurface {
    snapshot: ApiSnapshot,
}

#[cfg(feature = "producer")]
impl RustPublicSurface {
    /// Parse an in-memory rustdoc JSON document, labelled `version`.
    pub fn from_str(doc: &str, version: &str) -> Result<Self, crate::producer::ProducerError> {
        Ok(Self {
            snapshot: crate::producer::snapshot_from_rustdoc_str(doc, version)?,
        })
    }

    /// Read + parse a rustdoc JSON file, labelled `version`.
    pub fn from_json_file(
        path: &std::path::Path,
        version: &str,
    ) -> Result<Self, crate::producer::ProducerError> {
        Ok(Self {
            snapshot: crate::producer::snapshot_from_rustdoc_json(path, version)?,
        })
    }
}

#[cfg(feature = "producer")]
impl Surface for RustPublicSurface {
    fn name(&self) -> &'static str {
        // One rust-public surface per crate; the ledger keys on this fixed name.
        "rust-public"
    }

    fn snapshot(&self) -> ApiSnapshot {
        self.snapshot.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{classify_diff, BreakKind, Severity};
    use crate::diff::diff_snapshots;

    fn cat() -> CatalogSurface {
        CatalogSurface::new(
            "catalog",
            &[
                ("products::widget", "Widget 500 0.10", &[]),
                ("products::gadget", "Gadget 1200 0.05", &[]),
            ],
        )
    }

    #[test]
    fn catalog_rows_are_row_items_with_attrs() {
        let s = cat().snapshot();
        assert_eq!(s.len(), 2);
        let w = s.get("products::widget").unwrap();
        assert_eq!(w.kind, ItemKind::Row);
        assert_eq!(w.sig, "Widget 500 0.10");

        let deprecated = CatalogSurface::new(
            "catalog",
            &[("products::old", "Old 100 0.0", &["deprecated"])],
        )
        .snapshot();
        assert!(deprecated
            .get("products::old")
            .unwrap()
            .has_attr("deprecated"));
    }

    #[test]
    fn new_row_is_compatible_added() {
        let old = cat().snapshot();
        let new = CatalogSurface::new(
            "catalog",
            &[
                ("products::widget", "Widget 500 0.10", &[]),
                ("products::gadget", "Gadget 1200 0.05", &[]),
                ("products::sprocket", "Sprocket 700 0.02", &[]),
            ],
        )
        .snapshot();
        let breaks = classify_diff(&diff_snapshots(&old, &new));
        let b = breaks
            .iter()
            .find(|b| b.path == "products::sprocket")
            .unwrap();
        assert_eq!(b.kind, BreakKind::Added);
        assert_eq!(b.severity, Severity::Compatible);
    }

    #[test]
    fn reprice_is_signature_changed_warning() {
        let old = cat().snapshot();
        let new = CatalogSurface::new(
            "catalog",
            &[
                ("products::widget", "Widget 650 0.10", &[]), // repriced
                ("products::gadget", "Gadget 1200 0.05", &[]),
            ],
        )
        .snapshot();
        let breaks = classify_diff(&diff_snapshots(&old, &new));
        let b = breaks
            .iter()
            .find(|b| b.path == "products::widget")
            .unwrap();
        assert_eq!(b.kind, BreakKind::SignatureChanged);
        assert_eq!(b.severity, Severity::Warning); // semantic drift, not a build break
    }

    #[test]
    fn delist_is_removed_breaking() {
        let old = cat().snapshot();
        let new = CatalogSurface::new("catalog", &[("products::widget", "Widget 500 0.10", &[])])
            .snapshot();
        let breaks = classify_diff(&diff_snapshots(&old, &new));
        let b = breaks
            .iter()
            .find(|b| b.path == "products::gadget")
            .unwrap();
        assert_eq!(b.kind, BreakKind::Removed);
        assert_eq!(b.severity, Severity::Breaking); // pending refs can't resolve
    }

    #[test]
    fn catalog_surface_macro_builds() {
        let s = catalog_surface!(
            "catalog",
            ["products::a": "A 100 0.0", "products::b": "B 200 0.0"]
        )
        .snapshot();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get("products::a").unwrap().kind, ItemKind::Row);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn catalog_surface_roundtrips_through_snapshot_file() {
        let file = crate::snapshot_file::render_snapshot_file("catalog", &cat().snapshot());
        let text = crate::snapshot_file::to_string_pretty(&file).unwrap();
        let back = crate::snapshot_file::parse_snapshot_file(&text).unwrap();
        assert_eq!(back.items, file.items);
        assert_eq!(back.items[0].kind, ItemKind::Row);
    }

    fn proto() -> ProtocolSurface {
        ProtocolSurface::new(
            "acp-contract",
            &[
                ("bro.checkout", "sha256:aaaa"),
                ("bro.refund", "sha256:bbbb"),
            ],
        )
    }

    #[test]
    fn protocol_methods_are_method_items() {
        let s = proto().snapshot();
        assert_eq!(s.len(), 2);
        let c = s.get("bro.checkout").unwrap();
        assert_eq!(c.kind, ItemKind::Method);
        assert_eq!(c.sig, "sha256:aaaa");
    }

    #[test]
    fn protocol_schema_change_is_breaking() {
        let old = proto().snapshot();
        let new = ProtocolSurface::new(
            "acp-contract",
            &[
                ("bro.checkout", "sha256:changed"),
                ("bro.refund", "sha256:bbbb"),
            ],
        )
        .snapshot();
        let breaks = classify_diff(&diff_snapshots(&old, &new));
        let b = breaks.iter().find(|b| b.path == "bro.checkout").unwrap();
        assert_eq!(b.kind, BreakKind::SignatureChanged);
        assert_eq!(b.severity, Severity::Breaking); // a schema move breaks every caller
    }

    #[test]
    fn protocol_added_method_is_compatible() {
        let old = proto().snapshot();
        let new = ProtocolSurface::new(
            "acp-contract",
            &[
                ("bro.checkout", "sha256:aaaa"),
                ("bro.refund", "sha256:bbbb"),
                ("bro.void", "sha256:cccc"),
            ],
        )
        .snapshot();
        let breaks = classify_diff(&diff_snapshots(&old, &new));
        let b = breaks.iter().find(|b| b.path == "bro.void").unwrap();
        assert_eq!(b.kind, BreakKind::Added);
        assert_eq!(b.severity, Severity::Compatible);
    }

    #[test]
    fn protocol_surface_macro_builds() {
        let s = protocol_surface!(
            "acp-contract",
            ["bro.checkout": "sha256:aaaa", "_bro/extension": "sha256:bbbb"]
        )
        .snapshot();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get("_bro/extension").unwrap().kind, ItemKind::Method);
    }
}
