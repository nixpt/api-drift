# APIDRIFT-14 — data/catalog surface (checkstand products)

| Field | Value |
|-------|--------|
| **Status** | done |
| **Milestone** | M2 |
| **Size** | S |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

checkstand's drift is not only code (`StoreError`/`DomainEvent` variants) —
its **product catalog is periodic data** (`Product { id, name, price_cents,
stock }`), re-typed independently in seven `main.rs` files, none of which is
the catalog-of-record. api-drift currently models *code* contracts; a data
contract should flow through the identical pipeline.

## Goal

Generalize a `catalog` (data) surface so product rows snapshot/diff/classify/
suggest exactly like `rust-public`.

## Acceptance

- [x] `ItemKind::Row` added, serde + classify handle it
- [x] `CatalogSurface` snapshots a product list; re-price diffs as Warning
- [x] A demo catalog snapshot + ledger entry round-trips
- [x] `cargo test` green: 35 core lib / 40 all-features; clippy + rustdoc `-D warnings` + `cargo fmt --check` clean

## Notes

Design doc § "The other way around — checkstand's product catalog is a data
surface". Depends on APIDRIFT-6 (Surface + ledger) only.

## Resolution

- `snapshot.rs`: added `ItemKind::Row` (doc says changed-row → Warning,
  removed-row → Breaking; serde derives already on the enum).
- `surface.rs`: `CatalogSurface` (rows `(path, sig, attrs)`) implementing
  `Surface`, plus `catalog_surface!` macro (string-literal keys — `path`
  fragment can't cleanly precede `:`). Classification needs no change:
  `Added` Row → Compatible, `SignatureChanged` Row → Warning (existing
  `_ => Warning` arm), `Removed` Row → Breaking — exactly the re-price /
  new-SKU / delist semantics from the design doc.
- `lib.rs`: re-export `CatalogSurface`.
- Tests (6 new): Row items + attrs, new-row Compatible, re-price Warning,
  delist Breaking, macro build, snapshot-file serde round-trip.

