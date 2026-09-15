//! Ledger test over a `rust-public` surface: arniko 0.2.99's public API as
//! synthetic rustdoc JSON, committed to `api-drift/rust-public.snapshot.json`.
//!
//! Proves APIDRIFT-7's loop: the producer (rustdoc-JSON → `ApiSnapshot`)
//! feeds a `RustPublicSurface`, and the embedded ledger records its shape. If
//! arniko's surface drifts and isn't recorded, this test fails with the
//! classified breaks + the recording command.
//!
//! Uses synthetic rustdoc JSON (not a real `cargo rustdoc` run) so the demo
//! stays self-contained and deterministic — real rustdoc output drifts per
//! toolchain and needs nightly + a full build.

#![cfg(feature = "producer")]

use api_drift::RustPublicSurface;
use api_drift_ledger::ledger;

/// A small arniko-shaped rustdoc JSON doc: `Alert::new` (fn) + `Badge` (struct).
const ARNIKO_099: &str = r#"{
  "index": {
    "1": {"visibility": "public", "inner": {"function": {"sig": {"inputs": [["message", {"resolved_path": {"path": "String", "args": null, "id": 0}}]], "output": null}}}},
    "2": {"visibility": "public", "inner": {"struct": {"kind": "plain", "fields": [], "impls": []}}}
  },
  "paths": {
    "1": {"crate_id": 0, "path": ["arniko", "Alert", "new"]},
    "2": {"crate_id": 0, "path": ["arniko", "Badge"]}
  }
}"#;

ledger! {
    dir: "../../api-drift";
    RustPublicSurface::from_str(ARNIKO_099, "arniko 0.2.99").unwrap();
}
