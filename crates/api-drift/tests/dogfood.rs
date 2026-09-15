//! Dogfood: api-drift embeds its own ledger over `snapshot::ItemKind`.
//!
//! Proves APIDRIFT-6's loop on the project itself: the committed
//! `api-drift/item-kind.snapshot.json` + `api-drift/ledger.jsonl` record the
//! enum surface; if a variant is added/renamed and not recorded, this test
//! fails with the classified break + the recording command.

use api_drift::enum_surface;
use api_drift_ledger::ledger;

ledger! {
    dir: "../../api-drift";
    enum_surface!("item-kind", api_drift::snapshot::ItemKind,
        [Function, Method, Struct, Enum, Trait, Field, Variant, Const, Static, Module, Other]);
}
