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
