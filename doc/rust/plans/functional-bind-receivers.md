# Functional bind receiver syntax

Status: bounded implementation and local qualification in progress. Canonical
parent is signed `1d0288846`, the verified integration of PR #694.

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
