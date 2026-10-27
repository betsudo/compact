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

## Packaged arm64 qualification — 2026-10-27

At clean source `5c90b80f32fc7193f564b8fbd08a0c8591410f02`, original compiler
0.31.136 package `/nix/store/w60wx0hih9fvwgdymsvsipf1cszhqxv8-compactc` passes
the genuine generated-consumer gate (36 cases / 135 executions / 56 replays),
all API/source refusal and changed-TS controls, and consumer Clippy. The 341
public Python tests pass with 20 explicit existing skips. Six protocol controls
reject the actual earlier producer/consumer format mismatch and false
proof/verification/application/replay success claims. Full header controls pass;
the proof runner builds and strict all-target Clippy passes.

Fresh direct native key generation for `inferred` uses **k=7, 76 rows**. Two fresh
**2,912-byte proofs** cover replacement key 5 and new key 6. Each verifies with
the immutable selected verifier, rejects changed binding and same-geometry
changed result, applies a funded default-strict local transaction, and rejects
`IntentAlreadyExists` replay with unchanged state. Complete regenerated IR and
all five functional program assets match the linked fixture; changed full IR
refuses before keys or state. Source, tool, proof material and generated asset
hashes remain unchanged during the gate. Constructor and unselected branch
proofs are explicitly not claimed.

Claude CLI returned a retained actual session limit; authorized independent
Codex CLI review (`gpt-6.1-sol`, high then medium) found the receipt mismatch,
then accepted its correction and the new controls with no remaining actionable
findings. PR [#691](https://github.com/betsudo/compact/pull/691) targets
`codex/rust-backend-ast`. Full latest-head required Linux/compiler/native/proof/
wallet/package CI and merge are still **OPEN**. This finite local acceptance
does not close all of #516 or qualify later source bytes automatically.

## Integrated scalar baseline — 2026-10-27

PR [#691](https://github.com/betsudo/compact/pull/691) merged into
`codex/rust-backend-ast` at signed `c3a529cb5f4dae3a74f1e4ac0baf94878f245d91`.
All required Linux compiler/native/proof/package/wallet/E2E checks and the
Apple Silicon original consumer gate passed. The first Darwin jobs received no
runner; only that platform was retried, retaining successful Linux work.

Reviewed source is `e568b6064`; CI checks synthetic merge `a84d08158` with parents
`[feefa6577,e568b6064]`. Its complete tracked tree `b20a3b0e5ac7971e5cd5809eb44dd93f2b90370d`
equals the reviewed source and signed canonical merge tree. Actual CI compiler
and artifact identities remain `a84d`; they are not relabelled. GitHub confirms
merged state and a valid GPG signature. The five source commits and merge retain
DCO/Assisted-by trailers. Independent final-head CLI review has no actionable
findings after the receipt-format correction; actual Claude quota failure and
all failed CI/development attempts are retained.

The 341 discovered Python cases in that epoch mean **321 passed /20 skipped**,
not 341 passes. The earlier receipts above retain their own source and tool
bindings. The completed own review checkout was removed only after verifying
preserved review bytes, committed ancestry and no source/process/open-file/lock
use; shared targets and qualification evidence remain needed and retained.

This integrates only the bounded scalar gap. The
[typed-pair follow-up](516-tuple-return-wrappers.md), other #516 initializer/
lexical debt, real-application requalification and the broader backend goal
remain separate and incomplete. Forge remains deliberately deferred.
