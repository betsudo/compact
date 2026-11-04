# Outer-Vector product proof CI wiring (local implementation)

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

Execution source:5297fca2a3c2ccb54dce0d6d1933de5f9d77cb16. The literal configured
Bash step passes under the existing local Nix compiler environment on macOS
arm64 in877.73s. It uses a fresh current original-recipe receipt, the actual
same-version compiler package, devShell ZKIR and newly resolved TS runtime/Node
identities, plus verified built-in parameters. All91 native/API/ingress commands
pass, all16 intact models are observed, all22 ordered refusals precede key
generation, and all958 physical runner inputs remain bound. Fifteen fresh
keypairs produce15 independently verified standalone proofs, and15 separately
proved funded DEFAULT-strict applications pass with unchanged-ledger replay
refusals. Typed, binding, corruption, CellWrite and wrong-verifier controls remain
intact; empty-result controls retain explicit N/A fields. The whole k9branch
remains unproved and has no keys.

All18 focused workflow controls pass, including literal shell quoting/forwarding
and recipe, ZKIR, runtime and proof-command failure propagation. The other64
Linux workflow blocks are byte-identical and ordered as before; all17 prior
workflow test methods remain. Source review150.63s reports no findings. The
three reviewed CI paths remain byte-identical after adopting the separately
qualified test-only prerequisite repair. A separate receipt review302.62s reports
no findings and recomputes retained evidence hashes; it does not itself rerun
cryptographic verification. Before further builds, all634 current generated
inputs were retained after exact hash matching:16,336,341bytes, within256MiB.
No overwritten historical Cargo log is reconstructed or relabelled.

The initial literal-v1 attempt correctly failed existing-recipe admission10.40s
before native/proof execution: the inherited module-order formatting repair had
changed the original CLI/compiler derivations. The unchanged original recipe
then actually built the current package in201.66s. Current-source realization
8.98s reused that package; this is not another compiler build. All failed
attempts and predecessor outputs remain preserved. These timings are single
local observations, with no benchmark or causal performance claim.

This branch remains LOCAL. Draft PR #701 separately publishes the completed
proof slice and test-only CI prerequisite repairs at ec3531a83. Rust required
checks and the Linux producer pass; its Linux full consumer is still running.
GitHub abandoned the macOS producer before acquiring a hosted arm64 runner,
reporting capacity constraints; the consumer correctly refused missing material.
No gate is waived. Full hosted Linux/macOS qualification and reviewed integration
remain open, including reconciliation of193 preserved prerequisite commits
above the stable delivery target codex/rust-backend-ast. The broader goal remains
incomplete and Forge deliberately deferred. Documentation after the execution
source records this result; it does not claim a new-head execution.
