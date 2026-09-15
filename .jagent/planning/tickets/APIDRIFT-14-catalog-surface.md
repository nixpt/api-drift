# APIDRIFT-14 — data/catalog surface (checkstand products)

| Field | Value |
|-------|--------|
| **Status** | backlog |
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
suggest exactly like `rust-public`:

- Add `ItemKind::Row` (non-code data record; or settle on `Data`).
- `CatalogSurface` (hand-written `Surface` impl + a `catalog_surface!`
  helper): `Item.path = products::<name>`, `Item.sig = canonical row`
  (`name`, `price_cents`, low-stock threshold). A delist-soon product gets
  `attrs = ["deprecated"]` (existing APIDRIFT-5 machinery).
- Severity: `Added` product → Compatible; `SignatureChanged` (re-price) →
  Warning; `Removed` (delisted with pending refs) → Breaking.

## Scope

- in: `ItemKind::Row`, `CatalogSurface` + macro, classify parity for data
  rows, a checkstand-catalog demo snapshot
- out: wiring into broker/pricing rule surfaces (later)

## Acceptance

- [ ] `ItemKind::Row` added, serde + classify handle it
- [ ] `CatalogSurface` snapshots a product list; re-price diffs as Warning
- [ ] A demo catalog snapshot + ledger entry round-trips
- [ ] `cargo test` green (clippy/fmt/rustdoc `-D warnings`)

## Notes

Design doc § "The other way around — checkstand's product catalog is a data
surface". Depends on APIDRIFT-6 (Surface + ledger) only. The `enum_surface!`
macro already covers enum-shaped data; this adds *row*-shaped data.
