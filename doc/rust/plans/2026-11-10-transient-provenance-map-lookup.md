# Transient provenance: map lookup selection (#612)

Date: 2026-11-10. Parent: signed local `1f40c9a8f` (membership control).
Branch/worktree: `codex/rust-ast-transient-provenance-map-lookup` in a
separate checkout. No remote publication.

## Scope

The opt-in bounded typed-IR pass now retains a known transient-derived key
as a possible selection dependency through `Expr::MapLookup` and
`StateReturn::MapLookup`. The returned ledger value also has unknown origin;
`has_unknown_input` remains true when it reaches a persistent hash or state
write. This is possible-dependence evidence: different keys may select
different values. It does not claim the map contents were transient-derived
or recover their cross-transaction origin. The existing depth, node, route,
dependency, finding and unavailable bounds apply.

## Source and evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_map_lookup.compact` compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. The source-produced schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-map-lookup-source-ir.json`
(SHA-256 `1ce40e3e0457f8b30cd289073dd1abcf2115e32064167c9db41f7bb318e58115`).
It emits a transient-key lookup reaching a cell write, a direct persistent
hash, and a persistent hash through a stateful value helper, plus an ordinary
Field key control. A synthetic typed-IR case covers `StateReturn::MapLookup`,
which this source emitted as an expression return. The existing genuine
nested lookup corpus now checks both its inner and outer persistent hashes:
the inner sink remains visible, and the outer sink has an unknown ledger
contribution plus a known key-selection route. No shared fixture was
regenerated.

Four focused provenance test targets passed: 11 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-map-lookup-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`3ee6feff610a22a74223e54383f26871de091465507e0c5fe0bf6e9ce4d114fe`;
capabilities
`11b72e80d71add4b7ea0becb13f1e43ebf3acd4484b1f48646ef5c73fea149d1`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review found no correctness blocker.

## Limits

This does not trace provenance of stored map values across transactions, or
project aggregate lookup results precisely. Ordinary keys and externally
supplied values remain valid unknown-origin inputs; absence of a known route
does not certify stability. This optional pass has no frontend warning
transport or generated API effect. No runtime, FAB-byte equivalence, proof,
broad suite, or Forge claim is made.
