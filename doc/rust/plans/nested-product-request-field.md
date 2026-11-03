# Nested product request field: baseline repair before widening

Status: compiler 0.31.184 has reviewed, bounded local source/SDK/routes/static
qualification. The required documentation-version correction is closed by owner
regeneration, the literal local version gate and independent correction review.
Full CI/integration and the primary Rust AST goal remain incomplete; Forge is
deferred.

## Baseline and concrete gap

At clean signed `0810a3175`, the complete historical
`nested_product_below_component.compact` source (SHA256
`33c091f6cc2907b05c08156a129a067be8ae15ad062c2c61473538e51a8177c7`)
passes TypeScript emission and refuses Rust/independent IR at Cell.write. Its
nested aggregate widening remains a real semantic gap, not accepted work.

A separately declared same-type control changes only its four return Uint16
leaves to Uint8; its source body and input shape are unchanged. All three
emission lanes pass. Actual all-target/all-feature generated-crate strict Clippy
then rejects the immutable Program's private `input` field for type complexity.
This is an existing baseline defect before widening, so it is repaired first.
The retained complete control fixture is `examples/rust_backend/nested_product_request_field/nested_product_request_field.compact`.

The first external harness attempted locked Cargo before creating its lock;
that failed before compilation. An offline lock-generation successor exposes
the real field diagnostic. Both failed attempts and original outputs are retained;
neither is relabelled as a passing complete baseline.

## Owner, API and bounded repair

`recorded/effect_plan/functional_items.rs::request_field` owns field-local
metadata; the existing signature AST owner already handles method signatures.
Extend the existing reasoned `clippy::type_complexity` attribute only to the
characterized tuple child below a component Vector. Keep the complete nested
`FixedVector<(..., ...), N>` / `BoundedUint<MAX>` carrier and private visibility.
Do not introduce public aliases, erase types, suppress crate-wide lints or change
constructor, Program, context, native/recorded execution or checked conversion.

The public API is unchanged: typed `Program::new(input)` and consuming
`execute_native` / `execute_recorded`. Ordinary wrong bounds, tuple arities and
Vector dimensions must still fail Rust type checking. Existing request/context
ownership and untrusted IR validators remain their current owners.

## Acceptance for this prerequisite

1. Preserve the complete historical widening refusal and its literal authority;
   do not reclassify it until a separate semantic slice actually qualifies it.
2. Add exact AST regression cases for nested product children in either tuple
   position. Removing the one style attribute must recover the identical field
   AST, including type and visibility. Keep prior unmarked/foreign carrier tests.
3. Run focused request-field and existing signature-owner tests and formatting.
   Build the original maintained compiler recipe from a clean signed source,
   then generate the complete control SDK without handwritten output edits.
4. Run locked all-target/all-feature strict Clippy on that SDK; exercise the
   existing four typed routes with asymmetric values, state and recorded replay.
   A matching correct-type control must compile for each misuse; wrong leaf bound, arity and dimension must
   independently fail at marked ordinary-use sites.
5. Source change is metadata/version only: compare generated output against183
   after removing only the declared field attribute and compiler-version metadata;
   do not normalize IR/Programs/proof material. Applicable proof receipts retain
   their literal epochs; no new proof acceptance is inferred from this repair.
6. Obtain fresh authorized Claude CLI review (Codex fallback only on actual
   Claude quota), record local evidence and concurrent-safe knowledge/checkpoint
   updates. Complete required CI/Linux and integration remain open and local-only
   authority remains controlling.

## Following semantic slice, after this baseline passes

Gap: frontend-supplied widening through nested scalar tuple subtrees below a
component Vector. The current scalar-product classifier stops at one product.
Trace actual supplied casts at `compiler/rust-ir-passes.ss`; the existing checked
Coerce, independent validator and structured Rust AST must remain authorities.

Proposed API keeps all tuple arities and Vector dimensions and widens only
already legal Uint leaves / Uint-to-Field, preserving Boolean identity. Inspect
alias, helper, local and witness evaluation boundaries before choosing the exact
structural relation. No Vector newly nested inside the product, nominal Struct,
narrowing, new identity carrier, broad refactor or source-specific admission.

Acceptance must include whole-source live TS and four-route Rust parity,
asymmetric/boundary/empty cases, exact action/witness/error order, independent
compile-fail twins and malformed-IR/dead-branch controls. Observe intact export
models before selecting fresh proofs. Require genuine proof/verify and a separate
funded DEFAULT-strict application/replay for an applicable intact k<=8 source;
retain any over-cap result intact without trimming or raising the automatic cap.
A metadata-only fix does not satisfy this semantic acceptance.

## Restart and evidence

Branch: `codex/rust-ast-nested-product-request-field`, based on SDK documentation
head `0810a3175`. Canonical, protected checkout and Forge refs remain untouched.
Evidence: `/Users/ysh/.codex/local-evidence/rust-ast-nested-component-product-20261103`.
`source-inventory-v1.json`, `baseline-v1.json` and `baseline-lock-repair-v1.json`
retain original183 characterization and both the harness and real lint failure.
No push, hosted trial, merge, release, shared installation or cleanup is part of
this local slice. Prior accepted SDK/application receipts are not reopened merely
because this prerequisite has a new compiler identity.


## Actual local compiler184 baseline qualification

The narrow implementation starts at signed `29bbc7be3`. Four request-field AST
methods (including both nested-child positions) and two existing signature-owner
methods pass. All three earlier request-field test bodies remain byte unchanged.

The original maintained local compiler build exits successfully in 322.75 seconds.
Its controller honestly records `same_head: false`: the separately committed
consumer fixture and its test-only import correction were added during that
build. Those are the only successor paths and are outside the explicit Nix CLI
and Scheme source filters. This is not claimed as a frozen whole-source pass.
Before qualification, the clean `f872753e2` existing-only original recipe passes
in 10.22 seconds with builds, remote builders and substitution downloads disabled,
selecting that exact immutable compiler184 package. Both the controller and the
source/recipe bridge are retained.

At clean `f872753e2`, fourteen focused commands pass their expected outcomes in
45.03 seconds. The complete unedited generated SDK passes all-target/all-feature
strict Clippy; the retained in-tree consumer fixture passes strict library Clippy. A separate byte-identical copy of its source, manifest
and lock (without the deliberately failing binaries) also passes all-target
strict Clippy in 1.20 seconds, including cfg(test) replay and behavior code. Its actual
Rust method executes the ordinary native, ordinary recorded, functional native
and functional recorded routes on asymmetric nested values including 0 and255.
Both recorded results replay through the VM. Exact typed values, state/effects,
private state, empty private outputs and gas agree; the retained initial state
stays unchanged. Three independently marked E0308 refusals cover a wrong inner
leaf bound, inner tuple arity and outer Vector dimension, with one byte-identical matching positive signature compiled separately for each
misuse. These are three specific negative controls and one positive signature,
not three distinct positive behaviors.

A separate same-physical-source comparison uses the same runtime root as the
183 baseline. Complete IR, raw origins, checked Program, capabilities, ZKIR and
`contract/Cargo.toml` are byte identical. The generated Rust differs only by the one
reasoned private-field attribute. Compiler metadata keeps its separate184 epoch;
no IR/Program normalization or proof substitution is used.

These are focused trusted local results, not complete transitive environment
attestation, full CI/Linux, MSRV, universal carrier coverage, profile freeze or
integration acceptance. No new proof was executed for this same-type regression;
proof applicability is unchanged, and byte equality is not a proof receipt.
Earlier selected proof/verification/strict-application receipts retain their
literal source/tool epochs. The separate nested widening still refuses and has
not been implemented or accepted by this metadata repair.

Actual records: `focused-tests-v1.json`, `original-build-v1.json`,
`build-test-successor-bridge-v1.json`, `current-sdk-v1/recipe/receipt.json`,
`current-sdk-v1/receipt.json`, `actual-receipt-inspection-v1.json` and the genuine
CLI review result under the evidence directory above. The old183 refusal,
missing-lock attempt, actual lint failure and original full source stay retained.


### Independent review correction and integration limits

Authentic Claude review at `f872753e2` (165.59 seconds) independently rehashes
actual logs, binaries, source and generated identities. It finds no implementation
blocker, and one P2: the required version check also reads generated ledger docs.
The maintained `compiler/ledger.ss` documentation macro now regenerates the full
raw document in an isolated tracked-source copy (28.21 seconds); the only byte
change is compiler183 to184. The literal existing workflow version step then
passes locally in the arm64 compiler Nix shell (9.36 seconds). This is not hosted
CI. Compiler/runtime implementation and all qualified input/source bytes stay
unchanged. The first review and its required correction remain retained.

The original consumer driver and the type-control bins live in maintainer-local
evidence. The checked-in example and consumer are retained regression fixtures;
only the AST test is currently invoked automatically in-tree. A maintained driver
and integration wiring for this consumer remain open before claiming that CI
protects its E2E outcome. Extra adjacent empty/one-tuple style controls and a test
name cleanup are optional hardening, not executed coverage. Existing negative
carrier controls retain their original bodies. No runtime refusal or new semantic
admission was added by this metadata-only prerequisite.


### Reviewed bounded closeout

The first Claude source/receipt review at `f872753e2` remains its literal epoch,
including its actual P2 finding. Claude's correction review hit a genuine quota;
it is not a completed review. Authorized Codex CLI fallback (`gpt-6.1-sol`, medium)
then reviews clean `c0264f73c` in 105.20 seconds and finds no remaining actionable
P1/P2 for this local metadata repair. It independently checks 197 file hashes,
the raw owner regeneration, literal version gate, all-target test-code lint and
source/documentation bridge. Current-clean correction evidence also retains a
separate four-test run (0.78 seconds) and existing-only recipe (7.45 seconds);
this does not supply the missing raw log for the first test invocation.

The exact comparison applies to `contract/Cargo.toml` and the listed IR/origin/
Program/capability/ZKIR assets. `compiler/contract-manifest.json` is a distinct
epoch with version and derived hashes/sizes, including the changed Rust file;
it is not claimed byte-equal. New proof execution and universal coverage remain
unclaimed. Consumer-driver/workflow integration stays open. Begin the separately
planned nested-product widening from this repaired baseline without reopening
accepted SDK, Passport, DID or historical recording work.

Final review records are `local-claude-request-field-review-v1.json`, the genuine
quota `local-claude-request-field-review-v2.json`,
`local-codex-request-field-correction-review-v1.json` and
`correction-focused-v1/receipt.json`. This paragraph is a documentation successor
to reviewed `c0264f73c`; it creates no new independent execution/review epoch.


### Root inventory correction

A later semantic-slice source review found that this unpublished example was
missing from the top-level fixture registry. Preserve the earlier local source
and evidence epochs; do not call their full CI green. The example now lives in
its dedicated cohort directory, with its full bytes and basename unchanged.
The actual existing fixture_inventory.source_registry() gate is rerun on this
parent before later integration. No registry authority or old fixture is removed.
A new exact recipe/consumer validation remains separate from the old184 epoch.


### Maintained whole-example consumer follow-up — 2026-11-03

The remaining gap is regression protection: the complete same-type source and
consumer have actual local184 receipts, but their driver was external evidence.
Maintain `check_nested_product_request_field.py` with the complete source cohort,
exact current clean original-recipe/compiler identity and unedited whole SDK.
Keep the original consumer unchanged: four ordinary/functional native/recorded
routes, exact typed result/private state/public state/effects/gas and two VM
replays. Strictly lint the whole SDK at its own crate root, including all features,
and the consumer's actual test code. Three marked wrong bound/arity/dimension
ordinary calls must fail E0308 at their intended spans; run the same independently
checked positive signature beside each misuse. These are three controls, not
three distinct positive behaviors.

Wire that maintained gate into the existing imported-compiler Linux workflow,
including retained failure diagnostics. Local workflow tests do not imply a
hosted execution. Keep the source body, typed public API, frontend semantics,
request-field metadata fix, recording validation and historical source/proof
identities unchanged. Do not add TS parity or proof claims to this prerequisite.
The separately qualified nested-widening stage keeps its own compiler185 epoch.
Current gate execution and independent review are required before closing this
follow-up; full CI/Linux/integration and local ancestry reconciliation stay open.


The first maintained clean-source run at `1dbd53f2c` passes all13commands:
original184 recipe20.76s and SDK/API/static gate16.77s. Independent actual Claude
source/receipt review (148.87s) finds no P1/P2 and accepts the maintained consumer
coverage in source. It independently rehashes1423repository inputs,18build
inputs, both resolved locks, all13command logs and the unedited10file SDK plus
its Cargo.lock. Those counts describe the retained first run, not later epochs.

The review identifies a portability follow-up: without CARGO_TARGET_DIR, Cargo
would create a target inside the generated SDK and violate its immutability
check. Default to a separate output-local cargo-target, retain explicit shared
absolute targets, reject targets inside the SDK, and record relevant build
environment. Also validate the original recipe format and store/package relation.
The unchanged consumer source fixes its four routes/two replay calls; headline
counts are source-derived literals, not separately measured output fields.
The three positive commands remain one distinct signature. No TS/proof/MSRV or
hosted result is claimed. The guard successor requires its own focused execution
and correction review; preserve first-run source, logs and review unchanged.
