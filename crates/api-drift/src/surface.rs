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
}
