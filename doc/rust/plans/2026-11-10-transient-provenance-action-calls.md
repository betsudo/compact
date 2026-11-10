# Transient provenance: unit action calls (#612)

Date: 2026-11-10. Parent: signed local `21befe6d9` (stateful sinks).
Branch/worktree: `codex/rust-ast-transient-provenance-action-calls` in an
isolated checkout. No remote publication.

## Scope

Instantiate bounded call-site provenance for typed-IR `StateAction::CircuitCall`
when the callee returns Unit. Formal parameters receive actual origins, and
the callee's action tree is analyzed in the caller report with its own lexical
sink circuit/source. Direct and nested helper calls preserve known producer
routes through ordinary unknown inputs. Name, arity, duplicate formal,
recursion, call depth, and action-node limits fail as `unavailable`. Invalid
callee shapes are checked before visiting arguments, avoiding sink findings
from expressions the renderer would reject. Non-Unit stateful action calls,
value-returning stateful expression calls, and standalone pure action calls
remain unavailable. No state read-after-write provenance is inferred.

## Evidence

The genuine source `examples/rust_backend/transient_provenance_probes/transient_provenance_action_calls.compact`
was compiled using the recorded `compactc`/`compactc-scheme` pair with
`--target rust --skip-zk --emit-rust-ir`. The new source IR fixture is
`tools/compact-rust-backend/tests/transient-provenance-action-calls-source-ir.json`.
It covers a producer passed directly to a helper sink, a producer created by
a nested helper, and an ordinary Field control. Synthetic malformed and
recursive cases test explicit unavailable behavior. No shared fixture was
regenerated.

All focused commands used `CARGO_BUILD_JOBS=2` and a separate Cargo target
under `/Users/ysh/.codex/local-evidence/transient-provenance-action-calls-20261110`.
`cargo check -p compact-rust-backend --lib` passed. Five focused provenance
test targets passed: 16 tests, zero failures. Exact-parent and current
backends compiled the same source with the same Scheme frontend. `cmp`
confirmed byte-identical generated `contract/lib.rs` and
`rust-capabilities.json`: SHA-256
`3c902b5a11b8059fecd13a4174ace19e9c52062025a3fa0e2da16f5a82af4637`
for Rust and
`68e0ee0c6b79a8da581ff9aad51305178f81484665e3b185685f828d673f1cb4`
for capabilities. Outputs are `base-output` and `branch-output` under the
evidence directory. No proof, runtime, broad suite, or Forge work was run.

Independent read-only review found that invalid action calls could analyze
their argument before rejection. Validation was moved before traversal and
a malformed-call regression was added. Re-review found no remaining
actionable blocker in this bounded slice. Full #612 is still open.
