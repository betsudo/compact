# Transient provenance: pure action helpers (#612)

Date: 2026-11-10. Parent: signed local `8c9684ce3` (effectful returns).
Branch/worktree: `codex/rust-ast-transient-provenance-pure-actions` in a
separate checkout. No remote publication.

## Scope

The explicitly invoked, bounded typed-IR pass now traverses a valid
`StateAction::PureCall` into a Unit-returning pure helper. It validates the
callee, arity, parameter count and names, recursion, and call depth before
visiting arguments. Actual origins bind to formals without cloning argument
expression trees; the callee retains lexical source and the caller remains
the finding's entry circuit. The existing node, route, step, finding, and
depth bounds apply during traversal. An `Expr::Assert` traverses its condition
so a nested persistent sink is visible; a known transient route affecting
assertion execution is marked unavailable for control interpretation.

## Evidence

`examples/rust_backend/transient_provenance_pure_actions.compact` compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. Its new schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-pure-actions-source-ir.json`.
The source produces a transient value directly in a stateful caller and in a
pure helper, forwards them through standalone pure action calls to a
`persistentHash` sink in another pure helper, and includes an ordinary Field
control. Ledger writes keep the exported source circuits effectful so the
frontend emits `StateAction::PureCall`. The source-backed test checks caller,
producer, sink and negative evidence, plus deterministic repetition. Focused
malformed-IR tests cover missing target, wrong arity, recursion, and call
depth. They verify invalid calls do not create findings from unreachable
arguments.

Ten focused provenance test targets passed: 31 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-pure-actions-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`e1a1974eace94dad3d5dcd5306668240d96caf93edf746ebf90a795d7ce052c5`;
capabilities
`63aa302d80ccaccf4dba395609b04f90375aa48c24d397973fe8dea2aac34ad6`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review, including the added guard tests, found no
correctness blocker.

## Limits

This is optional analysis, with no frontend warning transport or generated
Rust/capability changes. Known transient routes through assertion conditions
are reported at their sinks, while assertion execution control itself is
unavailable. Unsupported expression and action forms remain unavailable;
unknown origin and no known producer cannot certify stability. No runtime,
FAB-byte equivalence, proof, broad suite, or Forge claim is made.
