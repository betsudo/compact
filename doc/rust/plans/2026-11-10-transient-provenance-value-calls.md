# Transient provenance: stateful value calls and standard conversion (#612)

Date: 2026-11-10. Parent: signed local `3249b92fe` (unit action calls).
Branch/worktree: `codex/rust-ast-transient-provenance-value-calls` in a
separate checkout. No remote publication.

## Scope

Bounded `Expr::Call` analysis now handles value-returning stateful helpers.
Call-site actual origins instantiate formal parameters; supported action
sinks are traversed in the callee's lexical source context, and an expression
return carries its known producer route back to the caller. A terminal
action-local binding can reach that return. Valid non-expression returns keep
their action-sink evidence, but their result origin is unknown and explicitly
unavailable. Effectful returns with separate actions are malformed and are
rejected before argument traversal. Unknown, ambiguous, recursive, wrong-
arity, or over-budget calls remain unavailable. A name present in both pure
and stateful maps is treated as ambiguous. This is opt-in analysis; no
rendering or capability route calls it.

## Standard-library audit and evidence

`compiler/standard-library.compact` intentionally returns
`upgradeFromTransient(transientHash(...))` from `evolveNonce` (line 144), and
uses the same conversion in shielded send/change and merge nonce construction
(lines 171, 187, 218). The conversion is a representation step; the diagnostic
must retain its transient producer. The genuine source corpus
`examples/rust_backend/transient_provenance_value_calls.compact` invokes
`evolveNonce` and passes its result to a persistent hash. Its source-produced
schema 20 IR fixture is
`tools/compact-rust-backend/tests/transient-provenance-value-calls-source-ir.json`.
The source test confirms a finding with producer in `evolveNonce` and a
conversion dependency, alongside stateful-helper production, pure-helper
forwarding, and an ordinary Field control. This test does not designate
the protocol use as acknowledged: no source annotation or acknowledgement
policy is supplied by schema 20. A narrow explicit acknowledgement mechanism
is a remaining #612 task; conversion itself must never silently suppress a
warning.

The recorded frontend pair compiled the corpus with `--target rust --skip-zk
--emit-rust-ir`; no shared fixture was regenerated. All focused commands used
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-value-calls-20261110`.
Six focused provenance test targets passed: 20 tests, zero failures. Exact-
parent and current backends compiled the same source with the same Scheme
frontend. `cmp` confirmed byte-identical generated `contract/lib.rs` and
`rust-capabilities.json`: SHA-256
`a2acccac18ba0a753c366150d3355e103cc6be667d5f512b2070c6c2dcae7b59`
for Rust and
`072d0af0641bcf3adf9825ca0b7a1976b7b82b1c3c931dab5dde5e20f1ed5565`
for capabilities. Outputs are `base-output` and `branch-output` under the
evidence directory. No proof, runtime, broad suite, or Forge work was run.

Independent read-only review identified malformed call arguments being
visited before validation, unbounded formal-name collection, and a synthetic
test result-type mismatch. These were fixed and re-reviewed with no remaining
actionable blocker in this bounded slice. Full #612 remains open.
