//! Turn a [`ClassifiedBreak`] into a mechanical downstream [`Suggestion`].
//!
//! Suggestions are *edits a script could apply*, not free text: a rename map,
//! an arg add/remove, a new match arm. The consumer (codemod, agent, human)
//! decides how to render them as patches.

use crate::classify::{BreakKind, ClassifiedBreak};
use crate::snapshot::ItemKind;

/// A mechanical downstream edit, in machine-matchable form.
///
/// `action` is the stable enum downstream matches on; `detail` is the
/// rendered human form; `auto_appliable` gates script application; `confidence`
/// is how sure the engine is that the auto-apply is *correct* (a rename with a
/// verified target is 1.0; a speculative one is lower). The ledger's
/// auto-applier must run [`autoimmune_guard`] before applying — a
/// low-confidence auto-apply stays a review item. The string `name()`s are
/// frozen (serde + ledger files depend on them) — add variants, never rename.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Suggestion {
    /// Canonical path of the affected upstream item.
    pub path: String,
    /// Structured edit verb. See [`SuggestionAction`].
    pub action: SuggestionAction,
    /// Human-readable detail, e.g. `rename arniko::Foo -> arniko::Bar`.
    pub detail: String,
    /// True when a script can apply this without judgment (pure rename with
    /// a known target). False means a human/agent must review first.
    pub auto_appliable: bool,
    /// Confidence in `[0, 1]` that the suggestion is correct. Drives
    /// [`autoimmune_guard`]; default 1.0.
    #[cfg_attr(feature = "serde", serde(default = "default_confidence"))]
    pub confidence: f64,
}

/// Default confidence when a producer doesn't state one.
pub fn default_confidence() -> f64 {
    1.0
}

/// The edit verb, with the fields the applier needs. Covers every action the
/// core emits today:
///
/// - `rename-call` / `review-rename` (rename fold, confident or not)
/// - `remove-item` (removal), `review-kind` (kind change)
/// - `review-signature` (sig change), `review-field` (new field)
/// - `add-match-arm` (new variant — the `BackendEvent` case)
/// - `no-op` (pure addition)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuggestionAction {
    /// Confident rename: rewrite call sites `from` → `to`. Auto-appliable.
    RenameCall {
        /// Removed path.
        from: String,
        /// Added path.
        to: String,
    },
    /// Possible rename, weak signal: verify sigs first. Never auto.
    ReviewRename {
        /// Removed path.
        from: String,
        /// Added path.
        to: String,
    },
    /// Upstream item is gone: drop or gate downstream uses.
    RemoveItem,
    /// Same path, new signature: update call sites `old` → `new`.
    ReviewSignature {
        /// Previous signature.
        old: String,
        /// New signature.
        new: String,
    },
    /// Same path, new `ItemKind`: rework usage.
    ReviewKind,
    /// New enum variant: add a match arm for `variant` of `enum_path`.
    AddMatchArm {
        /// Path of the enum, e.g. `bro::BackendEvent`.
        enum_path: String,
        /// New variant name, e.g. `Partial`.
        variant: String,
    },
    /// New struct field: check struct literals for `field`.
    ReviewField {
        /// New field path.
        field: String,
    },
    /// Pure addition: nothing to do downstream.
    NoOp,
}

impl SuggestionAction {
    /// Frozen string name (serde + ledger files depend on it).
    pub fn name(&self) -> &'static str {
        match self {
            Self::RenameCall { .. } => "rename-call",
            Self::ReviewRename { .. } => "review-rename",
            Self::RemoveItem => "remove-item",
            Self::ReviewSignature { .. } => "review-signature",
            Self::ReviewKind => "review-kind",
            Self::AddMatchArm { .. } => "add-match-arm",
            Self::ReviewField { .. } => "review-field",
            Self::NoOp => "no-op",
        }
    }
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
        if let Some((from, to, confident)) = renames.iter().find(|(_, to, _)| to == &b.path) {
            if *confident {
                let detail = format!("rename {from} -> {to} at call sites");
                out.push(Suggestion {
                    path: b.path.clone(),
                    action: SuggestionAction::RenameCall {
                        from: from.clone(),
                        to: to.clone(),
                    },
                    detail,
                    auto_appliable: true,
                    confidence: 1.0,
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
        action: SuggestionAction::ReviewRename {
            from: from.to_owned(),
            to: to.to_owned(),
        },
        detail: format!("possible rename {from} -> {to}; verify sigs then rename call sites"),
        auto_appliable: false,
        confidence: 0.5,
    }
}

fn suggest_single(b: &ClassifiedBreak) -> Suggestion {
    match b.kind {
        BreakKind::Added => {
            // New variants/fields need a match arm / struct update downstream.
            let (action, detail, auto) = match b.item_kind {
                ItemKind::Variant => {
                    let (enum_path, variant) = split_variant(&b.path);
                    (
                        SuggestionAction::AddMatchArm {
                            enum_path: enum_path.to_owned(),
                            variant: variant.to_owned(),
                        },
                        format!("add a match arm for new variant {}", b.path),
                        false,
                    )
                }
                ItemKind::Field => (
                    SuggestionAction::ReviewField {
                        field: b.path.clone(),
                    },
                    format!("new field {} may break struct literals", b.path),
                    false,
                ),
                _ => (
                    SuggestionAction::NoOp,
                    format!("new item {}; no downstream change needed", b.path),
                    true,
                ),
            };
            Suggestion {
                path: b.path.clone(),
                action,
                detail,
                auto_appliable: auto,
                confidence: if auto { 1.0 } else { 0.5 },
            }
        }
        BreakKind::Removed => Suggestion {
            path: b.path.clone(),
            action: SuggestionAction::RemoveItem,
            detail: format!("drop or gate uses of removed {}", b.path),
            auto_appliable: false,
            confidence: 0.0,
        },
        BreakKind::SignatureChanged => {
            let old = b.old_sig.clone().unwrap_or_else(|| "?".to_owned());
            let new = b.new_sig.clone().unwrap_or_else(|| "?".to_owned());
            Suggestion {
                path: b.path.clone(),
                action: SuggestionAction::ReviewSignature {
                    old: old.clone(),
                    new: new.clone(),
                },
                detail: format!("update call sites of {}: `{old}` -> `{new}`", b.path),
                auto_appliable: false,
                confidence: 0.0,
            }
        }
        BreakKind::KindChanged => Suggestion {
            path: b.path.clone(),
            action: SuggestionAction::ReviewKind,
            detail: format!("item {} changed kind; rework usage", b.path),
            auto_appliable: false,
            confidence: 0.0,
        },
    }
}

/// Anti-autoimmune check (mirrors evorium's `immune::autoimmune_guard`): a
/// suggestion marked auto-appliable but with low confidence must be
/// downgraded to review — never auto-applied on a hunch. Returns the
/// corrected suggestion (a clone when unchanged).
///
/// The ledger's auto-applier (APIDRIFT-3/9) must run this before applying
/// anything; producers that don't state a `confidence` default to 1.0 and
/// pass through unchanged.
pub fn autoimmune_guard(suggestion: &Suggestion) -> Suggestion {
    if suggestion.auto_appliable && suggestion.confidence < 0.5 {
        let mut out = suggestion.clone();
        out.auto_appliable = false;
        out.detail = format!(
            "{} (low confidence {:.2} — auto-apply kept for review)",
            suggestion.detail, suggestion.confidence,
        );
        out
    } else {
        suggestion.clone()
    }
}

/// Split `enum::Path::Variant` into (`enum::Path`, `Variant`).
/// No `::` → (`""`, whole path); never panics.
fn split_variant(path: &str) -> (&str, &str) {
    match path.rfind("::") {
        Some(i) => (&path[..i], &path[i + 2..]),
        None => ("", path),
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
            sigs_match_modulo_rename(
                old_sig,
                a.new_sig.as_deref().unwrap_or(""),
                &r.path,
                &a.path,
            )
        });
        if let Some(a) = confident {
            pairs.push((r.path.clone(), a.path.clone(), true));
            continue;
        }
        // Weak signal: small sig edit distance → review, not auto.
        if let Some(a) = cands
            .iter()
            .filter(|a| edit_distance(old_sig, a.new_sig.as_deref().unwrap_or("")) <= 8)
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
    use crate::classify::Severity;
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
        assert!(matches!(out[0].action, SuggestionAction::RenameCall { .. }));
        assert!(out[0].auto_appliable);
    }

    #[test]
    fn lone_removal_needs_review() {
        let out = suggest_for_breaks(&[brk(
            "arniko::Gone::f",
            BreakKind::Removed,
            ItemKind::Function,
        )]);
        assert!(matches!(out[0].action, SuggestionAction::RemoveItem));
        assert!(!out[0].auto_appliable);
    }

    #[test]
    fn addition_is_noop() {
        let out = suggest_for_breaks(&[brk(
            "arniko::Fresh::f",
            BreakKind::Added,
            ItemKind::Function,
        )]);
        assert!(matches!(out[0].action, SuggestionAction::NoOp));
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
        assert!(out
            .iter()
            .all(|s| !matches!(s.action, SuggestionAction::RenameCall { .. })));
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
        assert!(out
            .iter()
            .all(|s| !matches!(s.action, SuggestionAction::RenameCall { .. })));
        // The Field removal needs review; the lone added Method stays no-op.
        let rm = out
            .iter()
            .find(|s| matches!(s.action, SuggestionAction::RemoveItem))
            .unwrap();
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
        let folded: Vec<&Suggestion> = out
            .iter()
            .filter(|s| matches!(s.action, SuggestionAction::ReviewRename { .. }))
            .collect();
        if folded.is_empty() {
            // Edit distance > 8: stays two separate suggestions. Also safe.
            assert_eq!(out.len(), 2);
        } else {
            assert_eq!(folded.len(), 1);
            assert!(!folded[0].auto_appliable);
        }
        assert!(out
            .iter()
            .all(|s| !matches!(s.action, SuggestionAction::RenameCall { .. })));
    }

    #[test]
    fn added_variant_suggests_match_arm() {
        let out = suggest_for_breaks(&[brk(
            "bro::BackendEvent::Partial",
            BreakKind::Added,
            ItemKind::Variant,
        )]);
        assert!(matches!(
            out[0].action,
            SuggestionAction::AddMatchArm { .. }
        ));
        assert!(!out[0].auto_appliable);
    }

    #[test]
    fn added_field_suggests_review() {
        let out = suggest_for_breaks(&[brk("bro::Opts::extra", BreakKind::Added, ItemKind::Field)]);
        assert!(matches!(
            out[0].action,
            SuggestionAction::ReviewField { .. }
        ));
        assert!(!out[0].auto_appliable);
    }

    #[test]
    fn action_names_frozen() {
        // Ledger files + downstream match on these strings. If this fails,
        // you renamed a verb: add a variant instead.
        let cases = vec![
            (
                SuggestionAction::RenameCall {
                    from: "a".to_owned(),
                    to: "b".to_owned(),
                },
                "rename-call",
            ),
            (
                SuggestionAction::ReviewRename {
                    from: "a".to_owned(),
                    to: "b".to_owned(),
                },
                "review-rename",
            ),
            (SuggestionAction::RemoveItem, "remove-item"),
            (
                SuggestionAction::ReviewSignature {
                    old: "o".to_owned(),
                    new: "n".to_owned(),
                },
                "review-signature",
            ),
            (SuggestionAction::ReviewKind, "review-kind"),
            (
                SuggestionAction::AddMatchArm {
                    enum_path: "e::E".to_owned(),
                    variant: "V".to_owned(),
                },
                "add-match-arm",
            ),
            (
                SuggestionAction::ReviewField {
                    field: "s::S::f".to_owned(),
                },
                "review-field",
            ),
            (SuggestionAction::NoOp, "no-op"),
        ];
        for (action, name) in cases {
            assert_eq!(action.name(), name);
        }
    }

    #[test]
    fn match_arm_carries_enum_and_variant() {
        let out = suggest_for_breaks(&[brk(
            "bro::BackendEvent::Partial",
            BreakKind::Added,
            ItemKind::Variant,
        )]);
        assert!(
            matches!(
                &out[0].action,
                SuggestionAction::AddMatchArm { enum_path, variant }
                if enum_path == "bro::BackendEvent" && variant == "Partial"
            ),
            "unexpected: {:?}",
            out[0].action
        );
    }

    #[test]
    fn rename_call_carries_from_to() {
        let breaks = vec![
            brk_sig(
                "m::old",
                BreakKind::Removed,
                ItemKind::Function,
                "pub fn old(&self)",
                "",
            ),
            brk_sig(
                "m::new",
                BreakKind::Added,
                ItemKind::Function,
                "",
                "pub fn new(&self)",
            ),
        ];
        let out = suggest_for_breaks(&breaks);
        assert!(
            matches!(
                &out[0].action,
                SuggestionAction::RenameCall { from, to }
                if from == "m::old" && to == "m::new"
            ),
            "unexpected: {:?}",
            out[0].action
        );
    }

    #[test]
    fn attrs_builder_and_lookup() {
        let it = crate::snapshot::Item::new("m::f", ItemKind::Function, "s")
            .with_attr("deprecated")
            .with_attrs(&["a", "b"]);
        assert!(it.has_attr("deprecated"));
        assert!(it.has_attr("a"));
        assert!(!it.has_attr("non_exhaustive"));
    }

    fn sug(auto: bool, confidence: f64) -> Suggestion {
        Suggestion {
            path: "m::f".to_owned(),
            action: SuggestionAction::RenameCall {
                from: "m::f".to_owned(),
                to: "m::g".to_owned(),
            },
            detail: "rename m::f -> m::g".to_owned(),
            auto_appliable: auto,
            confidence,
        }
    }

    #[test]
    fn confident_rename_has_full_confidence() {
        let breaks = vec![
            brk_sig(
                "m::old",
                BreakKind::Removed,
                ItemKind::Function,
                "pub fn old(&self)",
                "",
            ),
            brk_sig(
                "m::new",
                BreakKind::Added,
                ItemKind::Function,
                "",
                "pub fn new(&self)",
            ),
        ];
        let out = suggest_for_breaks(&breaks);
        assert!((out[0].confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn autoimmune_guard_downgrades_low_confidence_auto() {
        let guarded = autoimmune_guard(&sug(true, 0.3));
        assert!(!guarded.auto_appliable);
        assert!(guarded.detail.contains("kept for review"));
        assert!((guarded.confidence - 0.3).abs() < f64::EPSILON); // recorded, not edited
    }

    #[test]
    fn autoimmune_guard_leaves_high_confidence_and_review_alone() {
        let confident = autoimmune_guard(&sug(true, 1.0));
        assert!(confident.auto_appliable);

        let review = autoimmune_guard(&sug(false, 0.3));
        assert!(!review.auto_appliable); // already review; detail untouched
        assert!(!review.detail.contains("kept for review"));
    }
}
