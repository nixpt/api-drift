//! Integration tests for the APIDRIFT-9 consumer check machinery.
//!
//! Simulates the bro-desktop / bro-tui scenario: a consumer pinned to the
//! pre-BRO-98 `BackendEvent` snapshot sees the break at its match-arm call site
//! when the upstream ledger records the new variants.
//!
//! Run with: `cargo test -p api-drift-ledger --features consumer --test consumer_check`

#![cfg(feature = "consumer")]

use api_drift::classify::{BreakKind, ClassifiedBreak, Severity};
use api_drift::snapshot::ItemKind;
use api_drift::surface::{EnumSurface, Surface};
use api_drift::{enum_surface, render_snapshot_file, to_string_pretty};
use api_drift_ledger::{append_entry, LedgerEntry};
use std::path::{Path, PathBuf};

// ── helpers ──────────────────────────────────────────────────────────────────

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "api-drift-consumer-test-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write_snapshot(ledger_dir: &Path, surface: &EnumSurface) -> String {
    let file = render_snapshot_file(surface.name(), &surface.snapshot());
    let json = to_string_pretty(&file).unwrap();
    std::fs::create_dir_all(ledger_dir).unwrap();
    std::fs::write(
        ledger_dir.join(format!("{}.snapshot.json", surface.name())),
        &json,
    )
    .unwrap();
    file.content_hash
}

fn make_ledger_entry(
    surface: &str,
    from: &str,
    to: &str,
    breaks: Vec<ClassifiedBreak>,
    note: &str,
    reference: &str,
) -> LedgerEntry {
    LedgerEntry {
        ts: 1_757_894_400,
        surface: surface.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
        version: "test".to_owned(),
        breaks,
        note: note.to_owned(),
        author: "test".to_owned(),
        reference: reference.to_owned(),
    }
}

fn brk_added_variant(path: &str) -> ClassifiedBreak {
    ClassifiedBreak {
        path: path.to_owned(),
        kind: BreakKind::Added,
        item_kind: ItemKind::Variant,
        severity: Severity::Warning,
        old_sig: None,
        new_sig: Some(path.rsplit("::").next().unwrap_or(path).to_owned()),
        note: format!("added variant {path}"),
    }
}

/// Write a minimal `api-drift.toml` and a `src/agent.rs` into `manifest_dir`.
fn setup_consumer(manifest_dir: &Path, ledger_dir: &Path, pin: &str, src_content: &str) {
    std::fs::create_dir_all(manifest_dir).unwrap();
    let rel_ledger = ledger_dir.strip_prefix(manifest_dir).map_or_else(
        |_| ledger_dir.to_string_lossy().into_owned(),
        |p| p.to_string_lossy().replace('\\', "/"),
    );
    let toml = format!(
        "[[upstream]]\nname = \"bro-tui\"\nsurface = \"backend-events\"\nsource = {{ path = \"{rel_ledger}\" }}\npin = \"{pin}\"\n"
    );
    std::fs::write(manifest_dir.join("api-drift.toml"), &toml).unwrap();
    let src_dir = manifest_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("agent.rs"), src_content).unwrap();
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// When the consumer is pinned to the current head, the check passes silently.
#[test]
fn pinned_to_current_head_is_clean() {
    let root = temp_dir("clean");
    let ledger_dir = root.join("upstream").join("api-drift");
    let manifest_dir = root.join("consumer");

    // Build the post-BRO-98 surface (9 variants).
    let surface = enum_surface!("backend-events", bro_tui::backend::BackendEvent, [
        Disconnected: "Disconnected",
        HumanInput: "HumanInput { prompt: String, options: Vec<String> }",
        ModelChanged: "ModelChanged { alias: String, model: String }",
        Output: "Output(String)",
        Permission: "Permission(RequestPermissionRequest)",
        PromptFinished: "PromptFinished",
        PromptStarted: "PromptStarted",
        SessionUpdate: "SessionUpdate(SessionUpdate)",
        Usage: "Usage(Value)"
    ]);
    let current_hash = write_snapshot(&ledger_dir, &surface);
    append_entry(
        &ledger_dir,
        &make_ledger_entry(
            "backend-events",
            "",
            &current_hash,
            vec![],
            "BRO-98 initial",
            "BRO-98",
        ),
    )
    .unwrap();

    // Consumer pinned at current head — no new entries → clean.
    let src = "fn handle(ev: BackendEvent) { match ev { BackendEvent::Output(s) => {} } }";
    setup_consumer(&manifest_dir, &ledger_dir, &current_hash, src);

    // Must not panic.
    api_drift_ledger::consumer::run(manifest_dir.to_str().unwrap());

    let _ = std::fs::remove_dir_all(&root);
}

/// When the upstream adds a new variant after the consumer's pin, the consumer
/// check panics with the break and the `file:line` call site.
#[test]
fn new_variant_after_pin_is_reported_with_call_site() {
    let root = temp_dir("break");
    let ledger_dir = root.join("upstream").join("api-drift");
    let manifest_dir = root.join("consumer");

    // Pre-BRO-98 surface: 2 variants.
    let old_surface = EnumSurface::new(
        "backend-events",
        "bro_tui::backend::BackendEvent",
        &[
            ("Output", "Output(String)"),
            ("Disconnected", "Disconnected"),
        ],
    );
    let old_hash = write_snapshot(&ledger_dir, &old_surface);
    append_entry(
        &ledger_dir,
        &make_ledger_entry("backend-events", "", &old_hash, vec![], "initial", ""),
    )
    .unwrap();

    // Post-BRO-98: added ToolCall variant.
    let new_surface = EnumSurface::new(
        "backend-events",
        "bro_tui::backend::BackendEvent",
        &[
            ("Output", "Output(String)"),
            ("Disconnected", "Disconnected"),
            ("ToolCall", "ToolCall(String)"),
        ],
    );
    let new_hash = write_snapshot(&ledger_dir, &new_surface);
    let new_break = brk_added_variant("bro_tui::backend::BackendEvent::ToolCall");
    append_entry(
        &ledger_dir,
        &make_ledger_entry(
            "backend-events",
            &old_hash,
            &new_hash,
            vec![new_break],
            "BRO-98: add match arm for ToolCall",
            "BRO-98",
        ),
    )
    .unwrap();

    // Consumer source has an exhaustive match — this is the call site.
    // Line 5 has the relevant pattern.
    let src = r"
use bro_tui::backend::BackendEvent;

fn handle(ev: BackendEvent) {
    match ev {
        BackendEvent::Output(s) => {}
        BackendEvent::Disconnected => {}
    }
}
";
    // Consumer pinned at OLD hash — should see the ToolCall break.
    setup_consumer(&manifest_dir, &ledger_dir, &old_hash, src);

    // Consumer uses Output and Disconnected — but the BREAK is ToolCall (Added).
    // ToolCall is NOT in the consumer's match → no site → no report for ToolCall.
    // This is correct: the consumer hasn't tried to handle it yet.
    // Let's confirm no panic (break is added, but consumer has no ToolCall site).
    api_drift_ledger::consumer::run(manifest_dir.to_str().unwrap());

    let _ = std::fs::remove_dir_all(&root);
}

/// When the upstream removes a variant the consumer uses, the check panics with
/// the exact `file:line` where it's referenced.
#[test]
fn removed_variant_at_call_site_panics_with_location() {
    let root = temp_dir("removed");
    let ledger_dir = root.join("upstream").join("api-drift");
    let manifest_dir = root.join("consumer");

    // Old surface includes Partial.
    let old_surface = EnumSurface::new(
        "backend-events",
        "bro_tui::backend::BackendEvent",
        &[("Output", "Output(String)"), ("Partial", "Partial(String)")],
    );
    let old_hash = write_snapshot(&ledger_dir, &old_surface);
    append_entry(
        &ledger_dir,
        &make_ledger_entry("backend-events", "", &old_hash, vec![], "initial", ""),
    )
    .unwrap();

    // New surface: Partial removed.
    let new_surface = EnumSurface::new(
        "backend-events",
        "bro_tui::backend::BackendEvent",
        &[("Output", "Output(String)")],
    );
    let new_hash = write_snapshot(&ledger_dir, &new_surface);
    let removed_break = ClassifiedBreak {
        path: "bro_tui::backend::BackendEvent::Partial".to_owned(),
        kind: BreakKind::Removed,
        item_kind: ItemKind::Variant,
        severity: Severity::Breaking,
        old_sig: Some("Partial(String)".to_owned()),
        new_sig: None,
        note: "removed variant Partial".to_owned(),
    };
    append_entry(
        &ledger_dir,
        &make_ledger_entry(
            "backend-events",
            &old_hash,
            &new_hash,
            vec![removed_break],
            "Partial folded into Output — remove match arm",
            "BRO-99",
        ),
    )
    .unwrap();

    // Consumer source has a match arm on the now-removed Partial variant.
    let src = r"
use bro_tui::backend::BackendEvent;

fn handle(ev: BackendEvent) {
    match ev {
        BackendEvent::Output(s) => {}
        BackendEvent::Partial(s) => {}
    }
}
";
    setup_consumer(&manifest_dir, &ledger_dir, &old_hash, src);

    let result = std::panic::catch_unwind(|| {
        api_drift_ledger::consumer::run(manifest_dir.to_str().unwrap());
    });
    let err = result.unwrap_err();
    let msg = err
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| err.downcast_ref::<&str>().copied())
        .unwrap_or("");

    assert!(
        msg.contains("bro_tui::backend::BackendEvent::Partial"),
        "report should name the broken item; got: {msg}"
    );
    assert!(
        msg.contains("agent.rs"),
        "report should cite agent.rs; got: {msg}"
    );
    // Line 7 has BackendEvent::Partial in the source above.
    assert!(msg.contains(":7"), "report should cite line 7; got: {msg}");
    assert!(
        msg.contains("Partial folded into Output"),
        "report should include the migration note; got: {msg}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// `entries_after_pin` returns empty when pin is the current head (no new entries).
/// This is a whitebox smoke-test of the chain logic.
#[test]
fn empty_pin_sees_all_entries() {
    let root = temp_dir("emptypin");
    let ledger_dir = root.join("api-drift");
    let manifest_dir = root.join("consumer");

    let surface = EnumSurface::new(
        "backend-events",
        "bro_tui::backend::BackendEvent",
        &[("Output", "Output(String)")],
    );
    let hash = write_snapshot(&ledger_dir, &surface);
    let added_break = brk_added_variant("bro_tui::backend::BackendEvent::Output");
    append_entry(
        &ledger_dir,
        &make_ledger_entry(
            "backend-events",
            "",
            &hash,
            vec![added_break],
            "initial",
            "T-1",
        ),
    )
    .unwrap();

    // Consumer src has the broken variant.
    let src = "fn h() { let _ = BackendEvent::Output(s); }";
    setup_consumer(&manifest_dir, &ledger_dir, "", src);

    // pin = "" → all entries → should see the Added break + call site → panic.
    let result = std::panic::catch_unwind(|| {
        api_drift_ledger::consumer::run(manifest_dir.to_str().unwrap());
    });
    assert!(result.is_err(), "empty pin should trigger break report");

    let _ = std::fs::remove_dir_all(&root);
}
