//! api-drift: snapshot crate surfaces, diff versions, classify breaks, suggest downstream patches.
//!
//! Motivating case: `arniko` is consumed by many sibling repos. When an API
//! changes upstream (rename, new required arg, removed item), downstream
//! crates break and someone has to hand-edit every call site. This crate is
//! the policy/type layer for that workflow:
//!
//! 1. [`snapshot`] — a versioned inventory of a crate's public surface
//!    (`ApiSnapshot`: items with path + kind + signature).
//! 2. [`diff`] — compare two snapshots (`ApiDiff`: added / removed / changed).
//! 3. [`classify`] — label each diff entry with a [`BreakKind`] and severity.
//! 4. [`suggest`] — turn a classified break into a mechanical downstream
//!    [`Suggestion`] (rename call, add/remove arg, handle new variant…).
//!
//! The crate is intentionally I/O-free (pure std, no deps): producers parse
//! real code (rustdoc JSON, `cargo public-api`, tree-sitter…) into
//! `ApiSnapshot`, consumers render `Suggestion`s as patches. See `README.md`
//! for the end-to-end picture.
//!
//! # Example
//!
//! ```
//! use api_drift::{snapshot::{ApiSnapshot, Item, ItemKind}, diff::diff_snapshots};
//!
//! let old = ApiSnapshot::new("arniko 0.2.98", vec![
//!     Item::new("arniko::Alert::new", ItemKind::Function, "pub fn new(message: &str) -> Self"),
//! ]);
//! let new = ApiSnapshot::new("arniko 0.2.99", vec![
//!     Item::new("arniko::Alert::new", ItemKind::Function, "pub fn new(message: impl Into<String>) -> Self"),
//! ]);
//! let d = diff_snapshots(&old, &new);
//! assert_eq!(d.changed.len(), 1);
//! assert!(d.added.is_empty() && d.removed.is_empty());
//! ```

pub mod classify;
pub mod diff;
#[cfg(feature = "producer")]
pub mod producer;
pub mod snapshot;
#[cfg(feature = "serde")]
pub mod snapshot_file;
pub mod suggest;

pub use classify::{BreakKind, ClassifiedBreak, Severity};
pub use diff::ApiDiff;
pub use snapshot::{ApiSnapshot, Item, ItemKind};
#[cfg(feature = "serde")]
pub use snapshot_file::{
    content_hash, parse_snapshot_file, render_snapshot_file, to_string_pretty, SnapshotFile,
    SnapshotFileError,
};
pub use suggest::{Suggestion, SuggestionAction};
