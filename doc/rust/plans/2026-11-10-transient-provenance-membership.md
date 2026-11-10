# Transient provenance: collection membership control (#612)

Date: 2026-11-10. Parent: signed local `0be5e7e0c` (state assertions).
Branch/worktree: `codex/rust-ast-transient-provenance-membership` in a
separate checkout. No remote publication.

## Scope

The opt-in bounded typed-IR pass now preserves a known transient-derived
operand through `Expr::SetMember` and `Expr::MapMember`. The Boolean query
result also carries unknown ledger state: a known producer remains visible
when an unknown state input contributes. Branch-controlled persistent sinks
receive the route as control evidence and set `has_unknown_control`. The
same policy applies to typed `StateReturn::SetMember` and `MapMember` in a
stateful value helper. The pass does not infer persisted collection contents
or reconstruct prior transactions. Map lookup values remain an unavailable
or unknown boundary, since the selected stored value's origin is not known.

## Source and evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_membership.compact` compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. The source-produced schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-membership-source-ir.json`
(SHA-256 `dc0a53d9d1708d3bc3af0c2c437e52b0df6036eddd667917b9e81e10ddc9ca7f`).
It emits direct set and map membership branch conditions, a stateful helper
returning membership, and an ordinary Field membership control. The focused
test checks deterministic producer, membership dependency, unknown ledger
contribution, and `cell_write` control evidence. Synthetic typed-IR variants
exercise the two `StateReturn` membership forms not emitted in this source.
No shared fixture was regenerated.

Three focused provenance test targets passed: 10 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-membership-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`8b6aca14dcf345397bba4477871940d3b791c5f5ef2ee91467ebafe5bea1d47d`;
capabilities
`08544d30fb5b4eeb8e845b25c678f4d7a9ee9d197eb1efc6c182eba36f55e92c`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review found no correctness blocker.

## Limits

This is possible-path origin evidence, not a proof of branch reachability or
collection state. Unknown ledger and external inputs are not certified
stable. Unsupported query/value forms remain unavailable or unknown. This
optional pass has no frontend warning transport or generated API effect. No
runtime, FAB-byte equivalence, proof, broad suite, or Forge claim is made.
