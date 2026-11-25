# Transient provenance: typed-IR schema guard (#612)

Date: 2026-11-10. Parent: GPG-signed local `fa429b909` (cumulative artifact
parity). Branch/worktree: `codex/rust-ast-transient-provenance-schema-guard`
in an isolated checkout. No remote publication.

## Change

`transient_provenance::analyze` now checks `Contract.schema_version` against
`ir::SCHEMA_VERSION` before building helper maps, summaries or visiting any
expression/action. A mismatch returns one structured unavailable record at
`contract/schema_version`, with actual and expected versions in its reason,
no finding or summary, and no resource-limit truncation claim. `advise`
preserves the unavailable record. This aligns direct library callers with the
standalone CLI's existing schema-20 gate and the renderer's schema check.
Valid schema-20 analysis follows the pre-existing path. An absence of known
producer evidence still does not certify stability.

## Evidence

The focused test starts from the genuine source-produced
`transient-provenance-source-ir.json`, confirms that valid schema-20 IR has
findings, then changes only the schema number to adjacent older/newer values.
It checks deterministic unavailable evidence, no findings/summaries and
advisory propagation. Independent read-only review found no blocker.

All 21 `transient_provenance*.rs` integration-test targets passed: 71 tests,
zero failures, with `CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-schema-guard-20261110`.
The retained `provenance-tests.log` contains the full focused run. The pinned
pre-existing frontend compiled `transient_provenance.compact` with exact-parent
and current backends into separate evidence paths. All 113 generated files
excluding per-run `compile.log` matched byte-for-byte. SHA-256: generated Rust
`d8b3bdccea10b5f59eff19570d7be89bd0746a38fef16b91a9065a13aedcd6e7`;
capabilities
`940e7e8089134b4c92361f0b7cd230a6f093c5f5c3d74600225d9b1b95cb3721`.
No shared fixture was regenerated.

## Limits

This is an analyzer input-contract check. It does not transport warnings through
the Compact compiler, execute native/recorded calls, compare runtime result or
FAB bytes, exercise proof behavior or expand supported IR expression forms.
