use super::*;
use crate::snapshot::ItemKind;

fn item(path: &str, sig: &str) -> Item {
    Item::new(path, ItemKind::Function, sig)
}

#[test]
fn identical_snapshots_diff_empty() {
    let a = ApiSnapshot::new("v1", vec![item("a::f", "pub fn f()")]);
    let b = ApiSnapshot::new("v2", vec![item("a::f", "pub fn f()")]);
    let d = diff_snapshots(&a, &b);
    assert!(d.is_empty());
    assert_eq!(d.len(), 0);
}

#[test]
fn detects_added_removed_changed() {
    let old = ApiSnapshot::new(
        "v1",
        vec![
            item("a::gone", "pub fn gone()"),
            item("a::same", "pub fn same()"),
            item("a::moved", "pub fn moved(x: u8)"),
        ],
    );
    let neo = ApiSnapshot::new(
        "v2",
        vec![
            item("a::same", "pub fn same()"),
            item("a::moved", "pub fn moved(x: u8, y: u8)"),
            item("a::fresh", "pub fn fresh()"),
        ],
    );
    let d = diff_snapshots(&old, &neo);
    assert_eq!(d.added.len(), 1);
    assert_eq!(d.removed.len(), 1);
    assert_eq!(d.changed.len(), 1);
    assert_eq!(d.added[0].path, "a::fresh");
    assert_eq!(d.removed[0].path, "a::gone");
    assert_eq!(d.changed[0].path, "a::moved");
}

#[test]
fn kind_change_reports_changed() {
    let old = ApiSnapshot::new(
        "v1",
        vec![Item::new("a::T", ItemKind::Struct, "pub struct T")],
    );
    let neo = ApiSnapshot::new("v2", vec![Item::new("a::T", ItemKind::Enum, "pub enum T")]);
    let d = diff_snapshots(&old, &neo);
    assert_eq!(d.changed.len(), 1);
    assert_eq!(d.changed[0].old_kind, ItemKind::Struct);
    assert_eq!(d.changed[0].new_kind, ItemKind::Enum);
}

#[test]
fn output_is_sorted_by_path() {
    let old = ApiSnapshot::new("v1", vec![]);
    let neo = ApiSnapshot::new(
        "v2",
        vec![item("z::f", "s"), item("a::f", "s"), item("m::f", "s")],
    );
    let d = diff_snapshots(&old, &neo);
    let paths: Vec<&str> = d.added.iter().map(|i| i.path.as_str()).collect();
    assert_eq!(paths, vec!["a::f", "m::f", "z::f"]);
}

#[test]
fn old_side_duplicates_last_wins() {
    // The reported bug: an old-side dupe with a stale sig double-reported
    // a change. Only the winning (last) entry may report.
    let old = ApiSnapshot::new(
        "v1",
        vec![item("a::f", "sig-stale"), item("a::f", "sig-final")],
    );
    let neo = ApiSnapshot::new("v2", vec![item("a::f", "sig-final")]);
    let d = diff_snapshots(&old, &neo);
    assert!(d.is_empty(), "stale old dupe must not report: {d:?}");
}
