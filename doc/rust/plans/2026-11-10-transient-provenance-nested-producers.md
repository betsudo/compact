# Transient provenance: nested producers and lookup keys (#612)

Date: 2026-11-10. Parent: signed local `d79de7051` (acknowledgements).
Branch/worktree: `codex/rust-ast-transient-provenance-nested-producers`
in a separate checkout. No remote publication.

## Cumulative-review fixes

The cumulative independent audit found that a transient hash/commit created
a new producer while discarding known producers in its value or opening.
Nested transient operations now retain both the direct producer and upstream
routes. Dependency steps identify `transient_value` and
`transient_opening`. The existing route/step limits still bound evidence;
exhaustion reports unavailable/truncated, never stability.

The same audit found that expression `MapLookup` returned unknown without
visiting its key. A nested persistent sink in that key was silently missed.
The pass now visits the key, reports any nested sink, and marks the lookup
result origin unavailable when a known transient route reaches the key.
The returned ledger value remains an unknown cross-transaction boundary.

## Evidence and limits

`examples/rust_backend/transient_provenance_nested.compact` was compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`; its new schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-nested-source-ir.json`.
Source tests cover nested transient hashes, transient commitment value and
opening routes, and a persistent sink inside a map lookup key. A tight route
limit test verifies deterministic truncation. No shared fixture was
regenerated.

Eight focused provenance test targets passed: 26 tests, zero failures, with
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-nested-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend; `cmp` found generated `contract/lib.rs` and
`rust-capabilities.json` byte-identical. SHA-256:
`cbd3422b54d02c01233fc51f794c7f24514728c51bfddc1771f649323b1de86d`
for Rust and
`b60a8e36949760bc5b63eea061ec34975d64d179d17780f99071a0f52e1918a7`
for capabilities. No runtime/FAB equivalence, proof, broad suite, or Forge
claim is made. The cumulative reviewer rechecked both fixes and found no
remaining actionable blocker in the bounded prototype.

Unsupported fold, dynamic/opaque projection, standalone pure action call,
effectful value-helper return, and constructor flows remain explicit
`unavailable` limits. This is opt-in advisory typed-IR analysis; it does not
change generated public APIs or certify values with no known producer.
