//! Turn a [`ClassifiedBreak`] into a mechanical downstream [`Suggestion`].
//!
//! Suggestions are *edits a script could apply*, not free text: a rename map,
//! an arg add/remove, a new match arm. The consumer (codemod, agent, human)
//! decides how to render them as patches.

use crate::classify::{BreakKind, ClassifiedBreak};

/// A mechanical downstream edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Canonical path of the affected upstream item.
    pub path: String,
    /// What the downstream should do, in verbs (`rename-call`,
    /// `add-arg`, `remove-item`, `review-signature`, …).
    pub action: String,
    /// Human-readable detail, e.g. `rename arniko::Foo -> arniko::Bar`.
    pub detail: String,
    /// True when a script can apply this without judgment (pure rename with
    /// a known target). False means a human/agent must review first.
    pub auto_appliable: bool,
}

/// Suggest one edit per classified break.
///
/// Rename detection is conservative: a `Removed` + an `Added` fold into one
/// auto-appliable `rename-call` only when they share the parent module,
/// share the `ItemKind`, and carry a similarity signal (same sig modulo the
/// last path segment, or small sig edit distance). Anything else stays two
/// separate suggestions, or a `review-rename` when the kind matches but the
/// signal is weak.
pub fn suggest_for_breaks(breaks: &[ClassifiedBreak]) -> Vec<Suggestion> {
    let renames = detect_renames(breaks);
    let mut out: Vec<Suggestion> = Vec::with_capacity(breaks.len());
    for b in breaks {
        if renames.iter().any(|(from, _, _)| from == &b.path) {
            continue; // folded into the rename target's suggestion
        }
        if let Some((from, to, confident)) =
            renames.iter().find(|(_, to, _)| to == &b.path)
        {
            if *confident {
                out.push(Suggestion {
                    path: b.path.clone(),
                    action: "rename-call".to_owned(),
                    detail: format!("rename {from} -> {to} at call sites"),
                    auto_appliable: true,
                });
            } else {
                out.push(rename_review_for(b, from, to));
            }
            continue;
        }
        out.push(suggest_single(b));
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// A same-kind, same-parent pair with only a weak similarity signal: tell
/// the human it *might* be a rename, but never auto-apply.
fn rename_review_for(b: &ClassifiedBreak, from: &str, to: &str) -> Suggestion {
    Suggestion {
        path: b.path.clone(),
        action: "review-rename".to_owned(),
        detail: format!(
            "possible rename {from} -> {to}; verify sigs then rename call sites"
        ),
        auto_appliable: false,
    }
}

fn suggest_single(b: &ClassifiedBreak) -> Suggestion {
    match b.kind {
        BreakKind::Added => {
            // New variants/fields need a match arm / struct update downstream.
            let (action, detail, auto) = match b.item_kind {
                crate::snapshot::ItemKind::Variant => (
                    "add-match-arm",
                    format!("add a match arm for new variant {}", b.path),
                    false,
                ),
                crate::snapshot::ItemKind::Field => (
                    "review-field",
                    format!("new field {} may break struct literals", b.path),
                    false,
                ),
                _ => (
                    "no-op",
                    format!("new item {}; no downstream change needed", b.path),
                    true,
                ),
            };
            Suggestion {
                path: b.path.clone(),
                action: action.to_owned(),
                detail,
                auto_appliable: auto,
            }
        }
        BreakKind::Removed => Suggestion {
            path: b.path.clone(),
            action: "remove-item".to_owned(),
            detail: format!("drop or gate uses of removed {}", b.path),
            auto_appliable: false,
        },
        BreakKind::SignatureChanged => Suggestion {
            path: b.path.clone(),
            action: "review-signature".to_owned(),
            detail: format!(
                "update call sites of {}: `{}` -> `{}`",
                b.path,
                b.old_sig.as_deref().unwrap_or("?"),
                b.new_sig.as_deref().unwrap_or("?"),
            ),
            auto_appliable: false,
        },
        BreakKind::KindChanged => Suggestion {
            path: b.path.clone(),
            action: "review-kind".to_owned(),
            detail: format!("item {} changed kind; rework usage", b.path),
            auto_appliable: false,
        },
    }
}

/// Find (removed, added) pairs that look like renames.
/// Requires: same parent module, same `ItemKind`, plus a similarity signal —
/// same sig modulo the last path segment, or sig edit distance ≤ 8. Returns
/// (from, to, confident) triples; unconfident same-kind pairs surface as
/// `review-rename` (not auto) via [`rename_review_for`].
fn detect_renames(breaks: &[ClassifiedBreak]) -> Vec<(String, String, bool)> {
    let removed: Vec<&ClassifiedBreak> = breaks
        .iter()
        .filter(|b| b.kind == BreakKind::Removed)
        .collect();
    let added: Vec<&ClassifiedBreak> = breaks
        .iter()
        .filter(|b| b.kind == BreakKind::Added)
        .collect();
    let mut pairs: Vec<(String, String, bool)> = Vec::new();
    for r in removed {
        let parent = parent_of(&r.path);
        // Deterministic: smallest path among same-parent, same-kind, unused.
        let cands: Vec<&&ClassifiedBreak> = added
            .iter()
            .filter(|a| parent_of(&a.path) == parent)
            .filter(|a| a.item_kind == r.item_kind)
            .filter(|a| !pairs.iter().any(|(_, to, _)| to == &a.path))
            .collect();
        if cands.is_empty() {
            continue;
        }
        let old_sig = r.old_sig.as_deref().unwrap_or("");
        // Confident first: sig equal modulo the renamed leaf segment.
        let confident = cands.iter().find(|a| {
            sigs_match_modulo_rename(old_sig, a.new_sig.as_deref().unwrap_or(""), &r.path, &a.path)
        });
        if let Some(a) = confident {
            pairs.push((r.path.clone(), a.path.clone(), true));
            continue;
        }
        // Weak signal: small sig edit distance → review, not auto.
        if let Some(a) = cands
            .iter()
            .filter(|a| {
                edit_distance(old_sig, a.new_sig.as_deref().unwrap_or("")) <= 8
            })
            .min_by(|a, b| a.path.cmp(&b.path))
        {
            pairs.push((r.path.clone(), a.path.clone(), false));
        }
    }
    pairs
}

/// True when both sigs are identical after replacing the leaf path segment
/// (the renamed item's own name) with a placeholder. Catches
/// `pub fn badge(..)` → `pub fn tag(..)` while rejecting Field→Method noise
/// (those never reach here: kinds must already match).
fn sigs_match_modulo_rename(old_sig: &str, new_sig: &str, from: &str, to: &str) -> bool {
    fn leaf(p: &str) -> &str {
        p.rsplit("::").next().unwrap_or(p)
    }
    old_sig.replace(leaf(from), "\u{0}") == new_sig.replace(leaf(to), "\u{0}")
}

/// Character-level Levenshtein distance, capped early at 9 (we only care
/// about ≤ 8). Small inputs (sigs); O(m·n) time, O(min) space.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len() > b.len() {
        return edit_distance_inner(&b, &a);
    }
    edit_distance_inner(&a, &b)
}

fn edit_distance_inner(short: &[char], long: &[char]) -> usize {
    let mut prev: Vec<usize> = (0..=long.len()).collect();
    let mut curr = vec![0; long.len() + 1];
    for (i, &ca) in short.iter().enumerate() {
        curr[0] = i + 1;
        let mut row_min = curr[0];
        for (j, &cb) in long.iter().enumerate() {
            curr[j + 1] = if ca == cb {
                prev[j]
            } else {
                1 + prev[j].min(curr[j]).min(prev[j + 1])
            };
            row_min = row_min.min(curr[j + 1]);
        }
        // Early cap: callers only test ≤ 8.
        if row_min > 8 {
            return 9;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[long.len()]
}

fn parent_of(path: &str) -> &str {
    match path.rfind("::") {
        Some(i) => &path[..i],
        None => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{BreakKind, ClassifiedBreak, Severity};
    use crate::snapshot::ItemKind;

    fn brk(path: &str, kind: BreakKind, item_kind: ItemKind) -> ClassifiedBreak {
        ClassifiedBreak {
            path: path.to_owned(),
            kind,
            item_kind,
            severity: Severity::Warning,
            old_sig: None,
            new_sig: None,
            note: path.to_owned(),
        }
    }

    fn brk_sig(
        path: &str,
        kind: BreakKind,
        item_kind: ItemKind,
        old_sig: &str,
        new_sig: &str,
    ) -> ClassifiedBreak {
        ClassifiedBreak {
            path: path.to_owned(),
            kind,
            item_kind,
            severity: Severity::Warning,
            old_sig: Some(old_sig.to_owned()),
            new_sig: Some(new_sig.to_owned()),
            note: path.to_owned(),
        }
    }

    #[test]
    fn rename_pair_folds_to_one_auto_suggestion() {
        let breaks = vec![
            brk_sig(
                "arniko::Alert::old",
                BreakKind::Removed,
                ItemKind::Function,
                "pub fn old(&self)",
                "",
            ),
            brk_sig(
                "arniko::Alert::new",
                BreakKind::Added,
                ItemKind::Function,
                "",
                "pub fn new(&self)",
            ),
        ];
        let out = suggest_for_breaks(&breaks);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].action, "rename-call");
        assert!(out[0].auto_appliable);
    }

    #[test]
    fn lone_removal_needs_review() {
        let out = suggest_for_breaks(&[brk(
            "arniko::Gone::f",
            BreakKind::Removed,
            ItemKind::Function,
        )]);
        assert_eq!(out[0].action, "remove-item");
        assert!(!out[0].auto_appliable);
    }

    #[test]
    fn addition_is_noop() {
        let out = suggest_for_breaks(&[brk(
            "arniko::Fresh::f",
            BreakKind::Added,
            ItemKind::Function,
        )]);
        assert_eq!(out[0].action, "no-op");
        assert!(out[0].auto_appliable);
    }

    #[test]
    fn different_parents_do_not_fold() {
        let breaks = vec![
            brk("arniko::A::old", BreakKind::Removed, ItemKind::Function),
            brk("arniko::B::new", BreakKind::Added, ItemKind::Function),
        ];
        let out = suggest_for_breaks(&breaks);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|s| s.action != "rename-call"));
    }

    #[test]
    fn field_removed_plus_method_added_does_not_auto_rename() {
        // The reported bug: a removed Field + an added Method under one
        // parent used to auto-apply as rename-call. Kinds must match.
        let breaks = vec![
            brk_sig(
                "bro::Event::old_field",
                BreakKind::Removed,
                ItemKind::Field,
                "pub old_field: u8",
                "",
            ),
            brk_sig(
                "bro::Event::new_method",
                BreakKind::Added,
                ItemKind::Method,
                "",
                "pub fn new_method(&self)",
            ),
        ];
        let out = suggest_for_breaks(&breaks);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|s| s.action != "rename-call"));
        // The Field removal needs review; the lone added Method stays no-op.
        let rm = out.iter().find(|s| s.action == "remove-item").unwrap();
        assert!(!rm.auto_appliable);
    }

    #[test]
    fn weak_signal_same_kind_is_review_not_auto() {
        // Same kind + parent, but sigs differ beyond the leaf rename:
        // review-rename, never auto.
        let breaks = vec![
            brk_sig(
                "m::old",
                BreakKind::Removed,
                ItemKind::Function,
                "pub fn old(x: u8, y: u8, z: u8) -> Vec<String>",
                "",
            ),
            brk_sig(
                "m::new",
                BreakKind::Added,
                ItemKind::Function,
                "",
                "pub fn new() -> u32",
            ),
        ];
        let out = suggest_for_breaks(&breaks);
        let folded: Vec<&Suggestion> =
            out.iter().filter(|s| s.action == "review-rename").collect();
        if folded.is_empty() {
            // Edit distance > 8: stays two separate suggestions. Also safe.
            assert_eq!(out.len(), 2);
        } else {
            assert_eq!(folded.len(), 1);
            assert!(!folded[0].auto_appliable);
        }
        assert!(out.iter().all(|s| s.action != "rename-call"));
    }

    #[test]
    fn added_variant_suggests_match_arm() {
        let out = suggest_for_breaks(&[brk(
            "bro::BackendEvent::Partial",
            BreakKind::Added,
            ItemKind::Variant,
        )]);
        assert_eq!(out[0].action, "add-match-arm");
        assert!(!out[0].auto_appliable);
    }

    #[test]
    fn added_field_suggests_review() {
        let out = suggest_for_breaks(&[brk(
            "bro::Opts::extra",
            BreakKind::Added,
            ItemKind::Field,
        )]);
        assert_eq!(out[0].action, "review-field");
        assert!(!out[0].auto_appliable);
    }
}

