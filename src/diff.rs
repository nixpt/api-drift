//! Compare two [`ApiSnapshot`]s item-by-item.
//!
//! Indexing is by `Item::path`. A path present in both snapshots with a
//! different `sig` or `kind` becomes a [`Changed`]; paths only on one side
//! become added/removed. Same path + same sig + same kind = unchanged and
//! is not reported.

use crate::snapshot::{ApiSnapshot, Item, ItemKind};
use std::collections::HashMap;

/// One item whose path exists in both snapshots but whose shape moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changed {
    /// Canonical path, e.g. `arniko::Alert::new`.
    pub path: String,
    /// Kind before / after (equal when only the signature moved).
    pub old_kind: ItemKind,
    /// Kind before / after (equal when only the signature moved).
    pub new_kind: ItemKind,
    /// Rendered signature before / after.
    pub old_sig: String,
    /// Rendered signature before / after.
    pub new_sig: String,
}

/// The raw old-vs-new comparison.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApiDiff {
    /// Paths only in the new snapshot.
    pub added: Vec<Item>,
    /// Paths only in the old snapshot.
    pub removed: Vec<Item>,
    /// Paths in both, with a different kind or signature.
    pub changed: Vec<Changed>,
}

impl ApiDiff {
    /// True when nothing was added, removed, or changed.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Total number of reported entries.
    pub fn len(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

/// Compare `old` against `new`. Duplicate paths: last item wins.
pub fn diff_snapshots(old: &ApiSnapshot, new: &ApiSnapshot) -> ApiDiff {
    let old_by_path: HashMap<&str, &Item> =
        old.items.iter().map(|i| (i.path.as_str(), i)).collect();
    let new_by_path: HashMap<&str, &Item> =
        new.items.iter().map(|i| (i.path.as_str(), i)).collect();

    let mut diff = ApiDiff::default();
    emit_removed_changed(old, &new_by_path, &old_by_path, &mut diff);
    emit_added(new, &old_by_path, &new_by_path, &mut diff);

    diff.added.sort_by(|a, b| a.path.cmp(&b.path));
    diff.removed.sort_by(|a, b| a.path.cmp(&b.path));
    diff.changed.sort_by(|a, b| a.path.cmp(&b.path));
    diff
}

fn is_winner(by_path: &HashMap<&str, &Item>, item: &Item) -> bool {
    match by_path.get(item.path.as_str()) {
        Some(w) => std::ptr::eq(*w, item),
        None => false,
    }
}

fn emit_removed_changed(
    old: &ApiSnapshot,
    new_by_path: &HashMap<&str, &Item>,
    old_by_path: &HashMap<&str, &Item>,
    diff: &mut ApiDiff,
) {
    for item in &old.items {
        match new_by_path.get(item.path.as_str()) {
            None => {
                if is_winner(old_by_path, item) {
                    diff.removed.push(item.clone());
                }
            }
            Some(n) => {
                // Old-side duplicate guard: only the winning (last) entry may
                // report a change — earlier dupes compare against the loser's
                // sig and would double-report. `n` is the new-side winner by
                // construction (HashMap last-write-wins), so compare the old
                // winner against it.
                if is_winner(old_by_path, item)
                    && (item.sig != n.sig || item.kind != n.kind)
                {
                    diff.changed.push(Changed {
                        path: item.path.clone(),
                        old_kind: item.kind,
                        new_kind: n.kind,
                        old_sig: item.sig.clone(),
                        new_sig: n.sig.clone(),
                    });
                }
            }
        }
    }
}

fn emit_added(
    new: &ApiSnapshot,
    old_by_path: &HashMap<&str, &Item>,
    new_by_path: &HashMap<&str, &Item>,
    diff: &mut ApiDiff,
) {
    for item in &new.items {
        if !old_by_path.contains_key(item.path.as_str()) && is_winner(new_by_path, item) {
            diff.added.push(item.clone());
        }
    }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;

