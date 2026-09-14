//! Label each [`ApiDiff`] entry with a break kind + severity.
//!
//! Heuristics are string-based on purpose: real semantic analysis belongs in
//! the producer (rustdoc JSON types), not here. These labels drive the
//! mechanical [`suggest`](crate::suggest) layer and the human triage order.

use crate::diff::{ApiDiff, Changed};
use crate::snapshot::{Item, ItemKind};

/// How bad is this for a downstream consumer?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Additive only (new item, new defaulted param): downstream still builds.
    Compatible,
    /// May break (signature widened, kind shuffled): builds, maybe misbehaves.
    Warning,
    /// Will break a build (removed item, removed param, kind changed).
    Breaking,
}

/// The shape of one break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BreakKind {
    /// Path only in the new snapshot. Additive, compatible.
    Added,
    /// Path only in the old snapshot. Downstream refs fail to resolve.
    Removed,
    /// Same path, signature text moved. Could be rename, new/removed arg,
    /// return-type change — `suggest` refines it.
    SignatureChanged,
    /// Same path, `ItemKind` moved (struct -> enum, fn -> const…).
    KindChanged,
}

/// One diff entry with its label.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassifiedBreak {
    /// Canonical path of the affected item.
    pub path: String,
    /// What shape this break has.
    pub kind: BreakKind,
    /// What kind of item broke (mirrors the `Item`/`Changed` kinds, so
    /// `suggest` can require same-kind renames without re-diffing).
    pub item_kind: ItemKind,
    /// How bad it is for downstream.
    pub severity: Severity,
    /// Old signature (None for pure additions).
    pub old_sig: Option<String>,
    /// New signature (None for pure removals).
    pub new_sig: Option<String>,
    /// One-line human note, e.g. `removed fn arniko::Foo::bar`.
    pub note: String,
}

/// Label every entry of `diff`. Output sorted by path for stable goldens.
pub fn classify_diff(diff: &ApiDiff) -> Vec<ClassifiedBreak> {
    let mut out: Vec<ClassifiedBreak> = Vec::with_capacity(diff.len());
    for item in &diff.added {
        out.push(classify_added(item));
    }
    for item in &diff.removed {
        out.push(classify_removed(item));
    }
    for c in &diff.changed {
        out.push(classify_changed(c));
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

fn classify_added(item: &Item) -> ClassifiedBreak {
    // New enum variants / new public fields break exhaustive matches and
    // struct literals downstream: Warning, not Compatible — unless the enum
    // is `non_exhaustive` (downstream must already wildcard) or the item is
    // `deprecated` (signals don't-use, not don't-compile). Everything else
    // additive stays Compatible.
    let severity = match item.kind {
        ItemKind::Variant | ItemKind::Field => {
            if item.has_attr("non_exhaustive") || item.has_attr("deprecated") {
                Severity::Compatible
            } else {
                Severity::Warning
            }
        }
        _ => Severity::Compatible,
    };
    ClassifiedBreak {
        path: item.path.clone(),
        kind: BreakKind::Added,
        item_kind: item.kind,
        severity,
        old_sig: None,
        new_sig: Some(item.sig.clone()),
        note: if item.attrs.is_empty() {
            format!("added {:?} {}", item.kind, item.path)
        } else {
            format!(
                "added {:?} {} [{}]",
                item.kind,
                item.path,
                item.attrs.join(", ")
            )
        },
    }
}

fn classify_removed(item: &Item) -> ClassifiedBreak {
    // Deprecated removals were announced; still Breaking (refs fail to
    // resolve) but the note says so, so triage reads it first.
    let note = if item.has_attr("deprecated") {
        format!("removed deprecated {:?} {}", item.kind, item.path)
    } else {
        format!("removed {:?} {}", item.kind, item.path)
    };
    ClassifiedBreak {
        path: item.path.clone(),
        kind: BreakKind::Removed,
        item_kind: item.kind,
        severity: Severity::Breaking,
        old_sig: Some(item.sig.clone()),
        new_sig: None,
        note,
    }
}

fn classify_changed(c: &Changed) -> ClassifiedBreak {
    let (kind, severity) = if c.old_kind == c.new_kind {
        // Rust has no default args: any fn/method signature move is Breaking
        // unless it matches the widening allowlist (purely more permissive
        // input, e.g. `&str` -> `impl Into<String>`).
        let sev = match c.old_kind {
            ItemKind::Function | ItemKind::Method => {
                if is_widening(&c.old_sig, &c.new_sig) {
                    Severity::Compatible
                } else {
                    Severity::Breaking
                }
            }
            _ => Severity::Warning,
        };
        (BreakKind::SignatureChanged, sev)
    } else {
        (BreakKind::KindChanged, Severity::Breaking)
    };
    ClassifiedBreak {
        path: c.path.clone(),
        kind,
        item_kind: c.new_kind,
        severity,
        old_sig: Some(c.old_sig.clone()),
        new_sig: Some(c.new_sig.clone()),
        note: format!("changed {}: `{}` -> `{}`", c.path, c.old_sig, c.new_sig),
    }
}

/// Purely-more-permissive input widenings: old call sites keep compiling.
/// Substring-normalized (`&str` vs `& str` both match); anything else is
/// Breaking by default. Conservative on purpose: the ledger auto-applies
/// what `suggest` marks auto, so unknown shapes must fail closed.
fn is_widening(old_sig: &str, new_sig: &str) -> bool {
    // (old-fragment, new-fragment) pairs, whitespace-insensitive.
    const ALLOW: &[(&str, &str)] = &[
        ("&str", "implInto<String>"),
        ("&str", "implAsRef<str>"),
        ("&String", "implAsRef<str>"),
        ("&T", "implAsRef<T>"),
        ("T", "implInto<T>"),
        ("&Vec<T>", "&[T]"),
        ("Vec<T>", "&[T]"),
    ];
    let old_n: String = old_sig.split_whitespace().collect();
    let new_n: String = new_sig.split_whitespace().collect();
    ALLOW
        .iter()
        .any(|(o, n)| old_n.contains(o) && new_n.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::Changed;
    use crate::snapshot::{ApiSnapshot, Item, ItemKind};

    fn snap(items: Vec<Item>) -> ApiSnapshot {
        ApiSnapshot::new("v", items)
    }

    #[test]
    fn added_is_compatible() {
        let d = crate::diff::diff_snapshots(
            &snap(vec![]),
            &snap(vec![Item::new("a::f", ItemKind::Function, "pub fn f()")]),
        );
        let out = classify_diff(&d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, BreakKind::Added);
        assert_eq!(out[0].severity, Severity::Compatible);
    }

    #[test]
    fn removed_is_breaking() {
        let d = crate::diff::diff_snapshots(
            &snap(vec![Item::new("a::f", ItemKind::Function, "pub fn f()")]),
            &snap(vec![]),
        );
        let out = classify_diff(&d);
        assert_eq!(out[0].kind, BreakKind::Removed);
        assert_eq!(out[0].severity, Severity::Breaking);
    }

    #[test]
    fn sig_change_breaking_unless_widening() {
        // New required arg: Breaking (the reported bug — was Warning).
        let d = crate::diff::diff_snapshots(
            &snap(vec![Item::new(
                "a::f",
                ItemKind::Function,
                "pub fn f(x: u8)",
            )]),
            &snap(vec![Item::new(
                "a::f",
                ItemKind::Function,
                "pub fn f(x: u8, y: u8)",
            )]),
        );
        let out = classify_diff(&d);
        assert_eq!(out[0].kind, BreakKind::SignatureChanged);
        assert_eq!(out[0].severity, Severity::Breaking);

        let kind = classify_diff(&ApiDiff {
            added: vec![],
            removed: vec![],
            changed: vec![Changed {
                path: "a::T".to_owned(),
                old_kind: ItemKind::Struct,
                new_kind: ItemKind::Enum,
                old_sig: "pub struct T".to_owned(),
                new_sig: "pub enum T".to_owned(),
            }],
        });
        assert_eq!(kind[0].kind, BreakKind::KindChanged);
        assert_eq!(kind[0].severity, Severity::Breaking);
    }

    #[test]
    fn widening_allowlist_stays_compatible() {
        let d = crate::diff::diff_snapshots(
            &snap(vec![Item::new(
                "arniko::Alert::new",
                ItemKind::Function,
                "pub fn new(message: &str) -> Self",
            )]),
            &snap(vec![Item::new(
                "arniko::Alert::new",
                ItemKind::Function,
                "pub fn new(message: impl Into<String>) -> Self",
            )]),
        );
        let out = classify_diff(&d);
        assert_eq!(out[0].kind, BreakKind::SignatureChanged);
        assert_eq!(out[0].severity, Severity::Compatible);
    }

    #[test]
    fn added_variant_and_field_are_warning() {
        let d = crate::diff::diff_snapshots(
            &snap(vec![]),
            &snap(vec![
                Item::new("e::E::V", ItemKind::Variant, "V"),
                Item::new("s::S::f", ItemKind::Field, "pub f: u8"),
                Item::new("m::f", ItemKind::Function, "pub fn f()"),
            ]),
        );
        let out = classify_diff(&d);
        let sev = |p: &str| out.iter().find(|b| b.path == p).unwrap().severity;
        // The bro-desktop case: new BackendEvent variants must warn, not pass
        // as Compatible.
        assert_eq!(sev("e::E::V"), Severity::Warning);
        assert_eq!(sev("s::S::f"), Severity::Warning);
        assert_eq!(sev("m::f"), Severity::Compatible);
    }

    #[test]
    fn output_sorted_by_path() {
        let d = crate::diff::diff_snapshots(
            &snap(vec![]),
            &snap(vec![
                Item::new("z::f", ItemKind::Function, "s"),
                Item::new("a::f", ItemKind::Function, "s"),
            ]),
        );
        let out = classify_diff(&d);
        assert_eq!(out[0].path, "a::f");
        assert_eq!(out[1].path, "z::f");
    }
}
