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
