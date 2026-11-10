# Transient provenance: state assertion control (#612)

Date: 2026-11-10. Parent: signed local `2fa228d54` (pure actions).
Branch/worktree: `codex/rust-ast-transient-provenance-assertions` in a
separate checkout. No remote publication.

## Scope

The opt-in typed-IR pass now applies the same assertion-control boundary to
`StateAction::Assert` as to `Expr::Assert`. It visits the condition, preserving
findings for nested persistent sinks. When a known transient route can affect
whether execution continues past the assertion, the pass records
`assertion execution control unavailable`. It does not attribute that route
to later sinks as a proven control dependency. An ordinary Field argument
remains a valid unknown-origin input and does not produce a known-transient
finding. Existing action/expression depth, node, finding, and unavailable
record bounds apply.

## Source and tests

`examples/rust_backend/transient_provenance_probes/transient_provenance_assertions.compact` was compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. Its new schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-assertions-source-ir.json`
(SHA-256 `5633a6a46108dca557dd766e4ece81a98885371c66f10560449879332c4e64ef`).
The source emits three genuine state action assertions: a transient-dependent
gate before an ordinary write, an ordinary Field gate, and a transient value
inside a persistent hash in an assertion condition. The new focused tests
check deterministic producer/sink and unavailable evidence, the ordinary
control, and truncation when unavailable records reach their limit.

Three focused provenance test targets passed: 10 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-assertions-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`15b802a739792490c20a10186cf716994ae407558e3f4f9040f02c2cdb1638a5`;
capabilities
`01d06a45fd9ba36f2e6d7ffa212e333dc7ab3979e73fb5d4b4f7214207aac194`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review found no correctness blocker and confirmed the
asserted limit is honest.

## Limits

Assertion-gated execution after a known transient condition remains
unavailable; the pass does not claim precise control flow or reachability.
No finding for an ordinary or unknown-origin input is a stability
certificate. This optional pass has no frontend warning transport or
generated API effect. No runtime, FAB-byte equivalence, proof, broad suite,
or Forge claim is made.
