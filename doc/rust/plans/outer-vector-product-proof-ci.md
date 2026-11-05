# Outer-Vector product proof CI wiring

Current status: publication candidate reconciled with PR #703. Historical local
qualification and publication restrictions below retain their original dates.

Gap: the full outer-Vector proof/application gate now executes locally, while the
required Linux compiler workflow previously invoked only the native91-command
outer-Vector gate. Publishing a local result does not establish hosted coverage.

Owners: .github/workflows/build-compiler.yml original authenticated compiler
consumer and its existing outer_vector_product_semantics step/artifact owner;
.github/scripts/test_compiler_closure.py; maintained native/proof gates. Public
Rust/Compact APIs, compiler/runtime/lowering and proof cohort stay unchanged.

Scope: wire the complete maintained outer-Vector proof gate into the required
Linux consumer after verified built-in k5/7/8 material is available. Resolve the
exact same-run imported compiler and devShell ZKIR/TS runtime through existing
owners. Retain all91native checks and all15proofs/strict apps plus22pre-key
refusals, without --prepare-only, continue-on-error or waived gates. Keep existing
component-Vector14proof and every other previous required gate. Retain full
success/failure raw diagnostics independently of later runtime-package outcomes.
Avoid adding an extra native91run when the combined full gate already runs it;
make any replacement explicit and prove the native prerequisite is retained.

Acceptance: focused exact workflow forwarding/order/non-weakening/artifact
success/failure/skipped controls; literal configured shell passes locally on
arm64 under the existing Nix compiler environment with exact current recipe and
fresh15directproofs/independentverification plus15separateDEFAULTstrictapps/replays;
fresh source and separate receipt reviews. Hosted Linux/fullCI and reviewed GitHub
integration remain unaccepted until actually executed. This follow-up stays local. The owner separately authorized publication of the
completed proof slice as draft PR #701; that authorization does not declare this
workflow follow-up published or accepted.

Exclusions: broader CI optimization, caching proofs/verdicts, source trimming,
cap changes, new features/refactors, Forge reconstruction, deployment/wallet or
network claims, and cleanup before verified merge. The intact k9branch remains
whole and unproved. No local-only result is relabelled native Linux or hosted CI.


## Local implementation; qualification pending

Replace the native-only workflow invocation with the combined maintained proof
gate, moving it after the existing verified built-in material step. The combined
gate itself first executes the full native91-command owner, so no native check
is removed or redundantly added. Its ZKIR path comes from the existing compiler
devShell; its compiler receipt uses the authenticated same-run imported binary.
Keep the component-Vector14 proof gate and all other required gates unchanged.

The adjacent always() upload captures both success and failure outcomes without
relying on later runtime-package gates; skipped/cancelled steps do not upload.
Retain recipe plus complete proof output including nested native protocol raw
logs; exclude only Cargo target directories from this diagnostic artifact.
Focused controls execute the literal configured shell with observable tool
boundaries and failing recipe/ZKIR/runtime/gate commands. These shell controls
are not proof evidence. A clean-head literal Nix execution and independent
source/receipt reviews remain required before local acceptance.


## Bounded local qualification accepted — 2026-11-04

Execution source: `5297fca2a3c2ccb54dce0d6d1933de5f9d77cb16`. The literal configured
Bash step passes under the existing local Nix compiler environment on macOS
arm64 in 877.73 seconds. It uses a fresh current original-recipe receipt, the actual
same-version compiler package, devShell ZKIR and newly resolved TS runtime/Node
identities, plus verified built-in parameters. All 91 native/API/ingress commands
pass, all 16 intact models are observed, all 22 ordered refusals precede key
generation, and all 958 physical runner inputs remain bound. Fifteen fresh
keypairs produce 15 independently verified standalone proofs, and 15 separately
proved funded DEFAULT-strict applications pass with unchanged-ledger replay
refusals. Typed, binding, corruption, CellWrite and wrong-verifier controls remain
intact; empty-result controls retain explicit N/A fields. The whole k = 9 branch
remains unproved and has no keys.

All 18 focused workflow controls pass, including literal shell quoting/forwarding
and recipe, ZKIR, runtime and proof-command failure propagation. The other 64
Linux workflow blocks are byte-identical and ordered as before; all 17 prior
workflow test methods remain. Source review (150.63 seconds) reports no findings. The
three reviewed CI paths remain byte-identical after adopting the separately
qualified test-only prerequisite repair. A separate receipt review (302.62 seconds) reports
no findings and recomputes retained evidence hashes; it does not itself rerun
cryptographic verification. Before further builds, all 634 current generated
inputs were retained after exact hash matching: 16,336,341 bytes, within 256 MiB.
No overwritten historical Cargo log is reconstructed or relabelled.

The initial literal-v1 attempt correctly failed existing-recipe admission after 10.40 seconds
before native/proof execution: the inherited module-order formatting repair had
changed the original CLI/compiler derivations. The unchanged original recipe
then actually built the current package in 201.66 seconds. Current-source realization
8.98 seconds reused that package; this is not another compiler build. All failed
attempts and predecessor outputs remain preserved. These timings are single
local observations, with no benchmark or causal performance claim.

At that November 4 checkpoint this branch remained LOCAL. Draft PR #701 separately published the completed
proof slice and test-only CI prerequisite repairs at ec3531a83. Rust required
checks and the Linux producer pass; its Linux full consumer is still running.
GitHub abandoned the macOS producer before acquiring a hosted arm64 runner,
reporting capacity constraints; the consumer correctly refused missing material.
No gate is waived. Full hosted Linux/macOS qualification and reviewed integration
remain open, including reconciliation of 193 preserved prerequisite commits
above the stable delivery target codex/rust-backend-ast. The broader goal remains
incomplete and Forge deliberately deferred. Documentation after the execution
source records this result; it does not claim a new-head execution.


## Local successor after diagnostic transport repair — 2026-11-05

`codex/rust-ast-outer-vector-product-proof-ci-v3` reconciles the preserved
`ad7462937` follow-up with PR #701 repair `8d100a1b5`, retaining both histories.
The only merge conflict was the source-map artifact test: retain its new archive
assertions and the full fifteen-proof gate ordering assertion together. All 18
previous workflow controls remain, plus the physical archive regression control.
The combined proof gate shell is byte-identical to execution source `5297fca2a`;
its compiler/runtime/SDK/proof/source owners are also byte-identical. The
source-map packaging and upload blocks are byte-identical to reviewed `8d100a1b5`.
This is a source bridge, not a fresh compiler build or cryptographic execution.
Retain the actual original `5297fca2a` receipts and independent source/receipt
reviews; full current-head hosted coverage and integration remain open.

Focused successor controls: 19 methods pass with native and GNU tar, including
literal full-gate forwarding and source-map archive membership/byte/failure
propagation. The successor remains local while PR #701's full platform consumers
run. On the published repair, Rust required checks and both compiler producers
pass; the macOS capacity problem from the older run is resolved in this run.
PR #701 is currently non-draft but remains unaccepted for integration. No gates
are waived, no predecessor is overwritten, and the 193-commit prerequisite
integration and zero-width follow-up #702 remain separate obligations.

## Publication reconciliation — 2026-11-05

The owner now authorizes publication and stabilization of the available Rust AST
slices. Preserve `58c1e5083` and PR #703 repair `cb39def70` through a forward merge.
The comparison base is PR #703's branch; the stable delivery target remains
`codex/rust-backend-ast`. No Forge work, release, shared installation, history
rewrite or merge is implied by publishing this follow-up.

This slice changes only workflow forwarding, its controls and documentation.
The complete proof gate still owns all 91 native commands before its 15 proofs,
strict applications and replay controls; built-in material must be ready first.
Preserve the canonical wide-zero gate and every other required qualification.
Nineteen focused workflow controls pass on the reconciled candidate. Current-head
Codex CLI review and all required hosted checks remain required for integration.

The fresh PR #703 local qualification at `cb39def70` separately passes the same
maintained outer-Vector gate, including all 15 eligible exports. Neither that
receipt nor the older literal-shell execution is relabeled as a new execution
of this publication candidate. The intact over-cap export remains unproved.
