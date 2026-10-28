# Functional bind receiver syntax

Status: PR #695 is verified merged at signed canonical `709a86230`.
Reviewed head `7134fb240`, actual CI checkout `9534bac96` and that merge share
complete tree `94c48222a1102938c99d3a94766d568ea6ca2749`. Exact-head Claude
review, all eleven CI checks and authenticated retained runtime-artifact/source
verification pass. Clean local qualification at `fd7b28259` retains its literal
source epoch; later CI supplies the current published-source qualification.
The completed worktree was recoverably archived after all ignored evidence was
preserved. The statement-action follow-up and broader Rust AST goal remain open.

## Gap and owners

The structured functional renderer places a complete step, including its bind
closure, in the receiver of the following bind. A valid mixed Field/Unit local
scope exposed excessive formatting work under pinned Rust 1.99.0. The same
underlying syntax is reachable from an already admitted canonical return scope;
this fix does not depend on admitting the separately preserved statement scope.

The owner is `Renderer::sequence` in
`tools/compact-rust-backend/src/recorded/effect_plan/functional_render.rs`.
Each expanded step now initializes a private block-local Circuit variable before
the existing consuming bind. Compact names still lower to the existing typed
SSA identifiers; repeated step names are confined to their own blocks. The
success-boundary bind in `step` continues to protect operand evaluation. No
continuation, parameter, result pattern or call order changes.

## Public API and scope

There is no public API, runtime ABI, checked graph, serialization or admission
change. Native and recorded interfaces and immutable consuming Programs retain
their existing contracts. The Circuit is evaluated and moved once. An error
still bypasses the continuation; inactive branches remain lazy. This is a
structured AST change, not a generated-source text rewrite.

Excluded: optimizer/framework work, capability expansion, new targets, CI
optimization, changed resource limits, historical oracle repinning and Forge.
The separate statement-action slice keeps its mixed binding case and review
controls; it follows this prerequisite through its own PR and qualification.

## Acceptance

- A structural regression must fail against the old renderer and cover value
  bindings, Unit bindings and branches without depending on whitespace or the
  spelling of the new local.
- Run the actual functional frontend/emitter on the preserved slow input;
  compare complete input IR and checked program bytes. Format with pinned
  Rust 1.99.0. A diagnostic AST-copy experiment is not accepted output.
- Regenerate current functional Rust snapshots through the actual compiler.
  Account for every changed fixture; retain source, IR, origins, capabilities
  and complete checked programs. Do not repin historical authority records.
- Run focused backend tests/Clippy and the maintained source E2E covering native,
  recorded, functional native/recorded, witnesses, first failures, owned values,
  replay, ordinary typing refusals and consuming-use refusals.
- Requalify selected genuine proofs, immutable verification and mutation
  negatives, plus separately proved default-strict ledger application/replay.
  Selected proof cases do not prove constructors, all exports or all fixtures.
- Use original clean compiler140, then a logical PR with actual Claude CLI
  review and complete required CI, including native Linux. No local macOS
  receipt substitutes for Linux CI. Merge only after those gates pass.

## Development evidence

The regression fails against canonical emission with seven nested bind
receivers. The modified actual functional compiler emits the preserved mixed
cohort and pinned rustfmt completes in 50.350 seconds. The old formatter was
intentionally stopped after 36m37s of CPU work; this is a failed/settled attempt,
not an exact timing baseline or whole-CI performance claim. These development
observations do not replace clean source, runtime or proof qualification.

Clean original compiler140 generated all 27 current functional snapshots. Every
retained complete IR, origin report, capability report and checked-program file
compares byte-for-byte equal. The exact generated native/recorded prefix before
each functional module also remains unchanged. Only current functional Rust
syntax snapshots change; historical authorities and source registrations do not.
The four-owner local all-feature run passes 2,520 tests with seven existing
ignored tests. Public Python selects 369 methods with no failures/errors and
20 existing manual skip events; these controls are not proof qualification.


## Clean source E2E and selected proofs

At exact clean `fd7b28259`, original compiler140 realizes the same immutable
package built at `bc8e91766`, with all 2,327 declared input records unchanged.
The maintained generic functional gate passes 160 scenarios across four routes.
The initializer gate passes 58 live TS cases, 196 Rust route checks, 62 recording
replays and three independent argument/result/consuming-reuse refusals. Selected
DID deactivation and late-witness failure compare all four Rust APIs with live TS.
These gates retain complete state, effects, witness outputs and query-cost checks.

Fresh direct native key generation produces two 2,912-byte initializer proofs
and one 3,296-byte DID deactivation proof. Immutable verification and applicable
same-geometry input/result/binding mutation controls pass. Separately proved
funded transactions pass default-strict application, exact poststate checks and
unchanged replay refusal. Constructors, seeded prestates and all exports are
not proved. Explicit all-284 registered-source emission checks pass with no
declared input drift; all 27 refreshed fixture crates compile. Emission is not
runtime execution or proof coverage for all registered sources.

The focused AST regression covers value/Unit bindings and branches. The current
Call/Fold source snapshots and maintained source E2E/full CI cover those existing
owners; the focused regression does not claim every graph shape or universal
formatting bounds. Review hardening recursively checks receivers and includes
syntax-only block, cast and parenthesis counterexamples. Flattening all
continuations or replacing the effect framework is outside this slice.
