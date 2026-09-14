# Invariants


## 2026-09-14T18:01:04-05:00

Core default feature set has zero dependencies; cargo tree with default features shows api-drift alone.

## 2026-09-14T18:01:04-05:00

diff: identical path+kind+sig is never reported; duplicate paths resolve last-wins on both old and new sides.

## 2026-09-14T18:01:05-05:00

Snapshot file parse rejects unknown format strings and content-hash mismatches (fail closed).
