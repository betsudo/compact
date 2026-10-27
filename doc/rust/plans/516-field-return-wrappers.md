# #516 slice: Field conversion around effectful terminal returns

## Gap and baseline

At canonical `feefa6577`, the retained `frontend-return-plans/cast_wrapped_mutation.compact` compiles for TypeScript and fails for Rust with nested Set.insert. Its inferred Uint branches are wrapped in a Uint-to-Field safe cast, hiding the terminal storage-mutation conditional from Scheme return decomposition. Existing checked ReturnPlan, operation owners, conversion validation and Rust AST emitters already support the required semantics.

## Owners and public surface

- `compiler/rust-ir-passes.ss`: recognize supported Field-widening wrappers around terminal mutation conditionals; decompose through existing sequence/Let/conditional return plans and apply the conversion once to each reached value after preceding effects.
- Existing checked IR/lowering/validation/native/recorded/functional emission owners retain their authority. No new IR tag, runtime ABI, generated API or shape fallback.
- Generated circuit methods keep their declared Field return type; the ordinary argument types and consuming functional Program stay unchanged.
- Genuine Compact sources and a maintained external consumer gate compare independent expected results with live TypeScript, native, recorded and both functional routes. Existing proof smoke owns selected genuine proof/verification/application controls.

## Bounded scope

Uint-to-Field safe casts and compiler-inferred monotone Uint branch joins wrapping effectful terminal returns, including nested branches and intervening lexical scope/sequences. Preserve branch selection, exactly-once condition/witness execution, conversion timing, mutations before subsequent failure, witness prefixes and checked subtraction failures. Keep accepted expression-only output unchanged.

Exclude general narrowing/cast normalization, every effect-bearing initializer/aggregate, replacement of Cell-specific lexical discovery, new targets, new SDK/distribution work, retired recording admission, broad AST refactors and Forge. This slice does not close all of #516.

## Acceptance

1. Reproduce genuine Rust refusal and TypeScript source acceptance at the exact clean baseline; retain failed artifacts.
2. Preserve existing accepted source compilation and capability signatures; compare the registered source census with the same Rust CLI and original/candidate Scheme source. Do not repin historical authorities.
3. Execute selected/unselected and nested branches, inferred versus explicit Field casts, lexical captures, zero/nonzero/replaced Set entries and effects/witnesses around failure. Verify canonical output, public state/effects, private state/output, query-summed cost and successful recorded VM replay across all four routes against live TS and independent expectations.
4. Wrong ordinary argument types and consumed Program reuse must fail Rust compilation at intended diagnostics. Unknown/forged/inactive-arm checked IR and conversion domains retain current explicit refusals.
5. Fresh selected keys/proofs, immutable verification, binding/result negative controls, funded default-strict application and unchanged replay refusal. Constructor execution and unselected exports remain separate.
6. Focused local iteration; full existing CI/proof/package/wallet gates before reviewed PR merge into rust-ast. Claude CLI review, Codex CLI fallback only on limits. Signed Conventional/DCO/Assisted-by commits; preserve concurrent knowledge-base edits and CAS checkpoint. Clean only completed own unneeded worktrees/targets after verified merge.

## Development checkpoint — 2026-10-27

The focused candidate passes 279 registered fixture outputs with zero stale/failures, five frontend return regressions (including this formerly refused source), and the live TS external consumer gate: 36 cases, 135 Rust executions, 56 successful recorded VM replays, E0308/E0382 controls, four both-target source refusals and strict consumer Clippy. The three pure controls use the pure API; they are not four-route cases. All 335 discovered Python tests pass with 20 explicit existing skips. Historical declaration authorities were preserved; the new source adds nine declarations.

Canonical baseline is the four-PR merged `feefa6577`. New exact-head original compiler package, selected proofs, complete required CI and independent PR review are still open. Focused development receipts do not qualify later bytes or complete #516.
