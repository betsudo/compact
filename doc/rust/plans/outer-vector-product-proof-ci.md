# Next bounded integration slice (proposal; no workflow changed)

Gap: the full outer-Vector proof/application gate now executes locally, while the
required Linux compiler workflow currently invokes only the native91-command
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
integration remain unaccepted until actually executed. Current local-only work
does not authorize backend pushes, hosted trials or merges.

Exclusions: broader CI optimization, caching proofs/verdicts, source trimming,
cap changes, new features/refactors, Forge reconstruction, deployment/wallet or
network claims, and cleanup before verified merge. The intact k9branch remains
whole and unproved. No local-only result is relabelled native Linux or hosted CI.
