# Transient provenance: stateful sinks (#612)

Date: 2026-11-10. Parent: signed local `8275f72dd` (aggregate and branch slice).
Branch/worktree: `codex/rust-ast-transient-provenance-state-sinks`, isolated from
the main session. No remote publication.

## Scope and policy

Extend the explicitly invoked schema 20 typed-IR analysis through bounded
stateful action and effectful-return traversal. Lexical `Let`, `Sequence`,
`If`, and return-plan scopes retain producer routes. A terminal action-local
binding reaches a separate expression return, matching the renderer's
terminal lexical suffix. Source-backed direct ledger sinks include cell write,
set insert, and map key/value insert; the pass also recognizes corresponding
typed-IR remove/list/merkle write forms and static position operands. Each
sink records the value route separately from a transient-dependent branch
execution route (`control_routes`). An ordinary or unknown branch condition
is not certified stable. Branch diagnostics are possible-path evidence, not
reachability or state-equivalence proofs.

Unsupported state actions, standalone action calls, stateful circuit calls,
constructor flow, and unsupported expression forms produce `unavailable`.
The pass does not follow state across a write/read boundary. A malformed
effectful return with separate actions is rejected before traversing actions.
Node, depth, route, dependency, finding, and unavailable limits remain in
force. Absence of a finding never certifies stability. Expression-level
source spans are unavailable in schema 20, so circuit source locations and
IR paths identify producer and sink occurrences.

## Evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_state_sinks.compact` was compiled
with the recorded `compactc`/`compactc-scheme` pair using `--target rust
--skip-zk --emit-rust-ir`; the new fixture is
`tools/compact-rust-backend/tests/transient-provenance-state-sinks-source-ir.json`.
It covers positive and negative cell writes, set and map writes, a branch
write, a terminal binding used by a persistent hash return, and a
transient-dependent branch controlling an ordinary write. A synthetic
effectful return plan and malformed-shape/budget cases cover typed-IR forms
that this frontend source did not emit. No shared fixture was regenerated.

All checks used `CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-state-sinks-20261110`.
`cargo check -p compact-rust-backend --lib` passed. Four focused provenance
test targets passed: 13 tests, zero failures. Exact-parent and current
backends compiled the same source using the same Scheme frontend. `cmp`
confirmed byte-identical generated `contract/lib.rs` and
`rust-capabilities.json`: SHA-256
`6d55903e375c01f481f7f352b23485a89c58f10144f43c43f49c64749e1c244b`
for Rust and
`75aba995270f99287b016c9e7ccbd51a57bb5fa1e4fb138aa49ba705742ed97b`
for capabilities. Outputs are in `base-output` and `branch-output` under the
evidence directory. No proof, runtime, broad suite, or Forge work was run.

Independent read-only review identified a lost terminal action binding at a
return and missing branch-execution provenance. Both were fixed with
source-backed regression cases. Re-review identified malformed effectful
return actions being analyzed before rejection; an early guard and test fix
that case.

This is a reviewable local continuation, not full issue #612 completion.
