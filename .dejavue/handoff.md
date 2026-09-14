# Handoff

Updated: 2026-09-14T18:01:06-05:00

## Summary
dejavue initialized properly (was timeline-only). Release pass merged as PR #1; CI green on main.

## Next Steps
- Captain: tag v0.1.0, cargo publish, flip visibility, apply fleet-default ruleset
- Agent: APIDRIFT-6 Surface trait + api-drift-ledger crate; dogfood the ledger! test on this repo first
- Run stable cargo fmt before every commit; CI checks stable formatting

## Boot Instructions
Read `.dejavue/handoff.md`, `.dejavue/state.md`, `.dejavue/decisions.md`, and `.dejavue/timeline.jsonl` before making changes.
