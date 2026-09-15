# APIDRIFT-13 — checkstand breadboard + borrows (real-server bar, schemars sig)

| Field | Value |
|-------|--------|
| **Status** | backlog (inputs into 8/9; standalone breadboard later) |
| **Milestone** | M2 |
| **Size** | S–M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

`projects/checkstand` already *practises* the api-drift discipline by hand
(`AGENTS.md` rule #2: exhaustive `StoreError` matches, no `_`, new variant =
compile error in every adapter). It's the manual, single-repo version of the
gate. Its MCP adapter also proves `schemars` schema-hashing is cheap, and its
test bar ("spawn the real server, never call a handler directly") is stricter
than api-drift's current plan.

## Goal

Harvest the three borrows into 8/9, and record checkstand-core as a third
breadboard (alongside bro-acp + arniko) for the ledger demo:

1. **Real-not-asserted consumer check** (→ APIDRIFT-9): usage scan must be a
   genuine `syn` walk; the test drives the real check, not a fixture (mirror
   checkstand rule #3).
2. **Schema-hash protocol sig** (→ APIDRIFT-8): `schemars::JsonSchema` output
   as the `sig` for MCP/ACP methods; checkstand-mcp proves every `*Args`
   struct already derives it.
3. **Reframe `Unrecorded`** (→ APIDRIFT-6 follow-up): the recording note is
   the producer's *decision record* ("added X, maps to 409 in REST / FAILED_
   PRECONDITION in gRPC"), keeping checkstand's "no catch-all" spirit.
4. **Breadboard** (later): checkstand-core embeds `ledger!` over `StoreError`
   (18) + `DomainEvent` (9, `#[serde(tag="type")]`); 7 adapters = the
   consumers APIDRIFT-11's gate reports. New variant → one ledger entry +
   `add-match-arm ×7`.

## Scope

- in: the three borrows inside 8/9/6; breadboard ticket when 8+9 land
- out: depending on checkstand-* crates (borrow patterns + vocabulary only)

## Acceptance

- [ ] APIDRIFT-9's usage scan is a `syn` walk (not a name list)
- [ ] APIDRIFT-8's protocol sig = schemars schema hash
- [ ] `Unrecorded` note is framed as a decision record
- [ ] (later) checkstand-core ledger demo ticket filed

## Notes

Origin: `projects/checkstand` (`AGENTS.md` rules #2/#3, `checkstand-core`
`StoreError`/`DomainEvent`, `checkstand-mcp` `schemars::JsonSchema` tool args,
`#[serde(tag="type")]` DomainEvent). Design doc § "Prior art within the
fleet: checkstand".
