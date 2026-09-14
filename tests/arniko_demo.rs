#![cfg(feature = "producer")]
//! End-to-end demo: rustdoc JSON in, classified breaks + suggestions out.
//!
//! Uses two synthetic rustdoc-shaped documents (arniko 0.2.98 → 0.2.99):
//! a widened `Alert::new` signature, a renamed `badge` → `tag` under the
//! same parent, and a removed `legacy` fn. Requires the `producer` feature.

use api_drift::classify::classify_diff;
use api_drift::diff::diff_snapshots;
use api_drift::producer::snapshot_from_rustdoc_str;
use api_drift::suggest::suggest_for_breaks;

fn fn_node(sig: &str) -> serde_json::Value {
    serde_json::json!({"visibility": "public", "inner": {"function": {"sig": sig}}})
}

fn doc(items: &[(&str, &str, serde_json::Value)]) -> String {
    let mut index = serde_json::Map::new();
    let mut paths = serde_json::Map::new();
    for (id, path, node) in items {
        index.insert((*id).to_owned(), node.clone());
        let segs: Vec<serde_json::Value> =
            path.split("::").map(|s| serde_json::Value::String(s.to_owned())).collect();
        paths.insert(
            (*id).to_owned(),
            serde_json::json!({"crate_id": 0, "path": segs}),
        );
    }
    serde_json::json!({"index": index, "paths": paths}).to_string()
}

fn v98() -> String {
    doc(&[
        ("1", "arniko::Alert::new", fn_node("pub fn new(message: &str) -> Self")),
        ("2", "arniko::badge", fn_node("pub fn badge(text: &str) -> String")),
        ("3", "arniko::legacy", fn_node("pub fn legacy()")),
    ])
}

fn v99() -> String {
    doc(&[
        (
            "1",
            "arniko::Alert::new",
            fn_node("pub fn new(message: impl Into<String>) -> Self"),
        ),
        ("2", "arniko::tag", fn_node("pub fn tag(text: &str) -> String")),
        ("4", "arniko::Sparkline::render", fn_node("pub fn render(&self) -> String")),
    ])
}

#[test]
fn arniko_098_to_099_demo() {
    let old = snapshot_from_rustdoc_str(&v98(), "arniko 0.2.98").unwrap();
    let new = snapshot_from_rustdoc_str(&v99(), "arniko 0.2.99").unwrap();
    assert_eq!(old.len(), 3);
    assert_eq!(new.len(), 3);

    let diff = diff_snapshots(&old, &new);
    assert_eq!(diff.changed.len(), 1); // Alert::new widened
    assert_eq!(diff.removed.len(), 2); // badge + legacy gone…
    assert_eq!(diff.added.len(), 2); // …tag + Sparkline::render appear

    let breaks = classify_diff(&diff);
    assert_eq!(breaks.len(), 5);

    let fixes = suggest_for_breaks(&breaks);
    // badge→tag folds into one auto rename-call; legacy needs review;
    // Alert::new needs signature review; Sparkline::render is a no-op add.
    assert_eq!(fixes.len(), 4);
    let rename = fixes.iter().find(|s| s.action == "rename-call").unwrap();
    assert!(rename.auto_appliable);
    assert!(rename.detail.contains("arniko::badge -> arniko::tag"));

    // Human-readable demo output (visible with --nocapture).
    for b in &breaks {
        println!("[{:?}/{:?}] {} — {}", b.severity, b.kind, b.path, b.note);
    }
    for s in &fixes {
        println!("suggest {} [{}]: {}", s.path, s.action, s.detail);
    }
}
