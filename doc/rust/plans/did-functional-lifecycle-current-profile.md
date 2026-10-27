# Next bounded application slice: current-profile DID lifecycle

## Concrete gap and current baseline

PR #692 is verified merged at canonical `acf529effc5d4b9bc8428d91dcaf21e641965a13`
after full CI and exact-head review. Qualify the unchanged midnight-did v0.7.0
source (`4e7f6b0f69bf4e2c8506a9693f8d0c3dfe68e550`) on the current
Ledger8.1.3 / runtime ABI53 / IR20 graph. Prior original DID acceptance used
Ledger8.0.3; the newer standalone tutorial proves only its stated ordinary
scenarios, not current functional end-to-end compatibility.

Provisional source characterization on the PR #692 candidate passes ordinary
and functional Rust plus TS generation. The source-manifest hash is
`a5386c2d3cc8d2f3aa090599a6038efc638f36f20ede19a7067a1ba416610106`;
both original Compact files match their accepted hashes. Capability metadata
lists all 12 recorded/observed, proof-applicable exports, and functional output
contains 12 checked programs. These are generation facts, not 12 executed or
proved circuits. The checkout has advanced onto the actual canonical merge with all 62 working-source files preserved; final clean compiler qualification remains separate.

## Bounded behavior and public API

Use the existing constructor-derived deactivation lifecycle from the retained
DID tutorial and its genuine Schnorr/reduction witnesses. Compare ordinary
native, ordinary recorded, functional native and functional recorded execution
from identical initialized context and private state. Use existing generated
`Program` and typed argument/witness interfaces; no new public API or codegen
option is planned. If an actual regression blocks this flow, reproduce it
against the clean canonical baseline and fix the existing semantic owner only.

## Owners

- Pinned source: examples/rust_backend/did_adoption/source-manifest.json and
  unchanged source closure.
- Existing DID lifecycle tests and support: doc/rust/guides/examples/did-lifecycle
  and tests-rust-backend/did-adoption. Preserve real independent TS inputs.
- Existing checked lowering/validation and generated functional Program owner;
  change only on a demonstrated semantic defect.
- Existing Ledger8 preparation/proof-smoke and funded default-strict application
  helpers; retain immutable source/tool/key/parameter/verifier identities.
- DID guide, qualification/delivery crosswalk, midnight knowledge base and
  main checkpoint: record fresh current acceptance separately from old receipts.

## Acceptance

1. Freeze the actual merged baseline, two-file source/import closure, SDK/runtime
   and lock identities. Generate complete outputs/programs without hand edits or
   stripping/normalization. Confirm actual tools and ledger profile.
2. Compile the genuine generated consumer with the matching runtime identity,
   then execute the constructor-derived valid lifecycle through all four routes
   against live pinned TS and independent expected results. Compare full aligned
   result/state/effects, private state/output, witness order, query-summed gas
   and successful recorded replay from identical prestate.
3. Preserve genuine authorization/signature and stale-version failures, inactive
   state controls and callback-prefix/first-error behavior. Do not claim rollback
   of inaccessible failed contexts or replace cryptographic witnesses with true.
4. Compile-fail wrong ordinary signature/argument shape and consumed Program
   reuse where applicable, plus strict generated-consumer Clippy. Preserve all
   source/capability/refusal and historical authority rows.
5. For the selected proof-applicable deactivation call, generate fresh keys and
   proof, verify with immutable verifier identity, reject changed public binding
   and applicable same-geometry result/state changes, apply with separately funded
   default-strict policy, and reject replay without mutation. Constructor data is
   deployment input; constructor execution is not proved by a call proof.
6. Focused local iteration, then the complete required CI and Claude CLI exact-PR
   review before signed Conventional/DCO/Assisted-by integration. Retain failed
   attempts and exact source epochs. Cleanup only own unused completed targets
   and worktrees after verified GitHub merge.

## Exclusions

No claim to prove all 12 exports, no artificial stateful Passport export, no
network submission, Ledger9/ZKIRv3 migration, Forge work, publication/release
changes, broader CI project, generic new framework, unrelated ergonomic refactor
or speculative aggregate/initializer expansion. If the selected flow reveals
a concrete gap, characterize that gap before changing its scope.

## Stacked development contract

This checkout originally started at reviewed PR #692 head `98d6b141c` before
its full CI completed; that historical development contract was honored. After
all gates passed and the signed merge was verified in GitHub, it fast-forwarded
onto actual canonical `acf529eff`, whose complete tree equals that parent head.
All 62 uncommitted source files matched the preserved archive before and after;
no reset, stash or history rewrite was used. The stable integration target is
`codex/rust-backend-ast`. Re-establish final source/tool premises with the clean
original compiler138, then complete this slice's PR review and full CI.

## Fixture and test design

Preserve the existing ordinary DID flat fixture and all historical registry
authorities unchanged. A separate functional source view copies the complete
original two-file closure byte-for-byte, retaining its original manifest and
relative imports. Its complete generated fixture keeps all IR, origins,
capabilities and 12 checked programs. This is another qualified generation
mode for the same pinned contract, not a new contract or claim to prove every
export. Only manifest placement and ordinary Rust formatting may differ from
fresh generated output.

Reuse the existing DID tutorial witnesses and constructor-derived lifecycle
observations. The source-level public API is unchanged: real SchnorrSignature
and `BoundedUint<u64::MAX>` arguments enter the generated consuming
`functional::deactivate::Program`, with ordinary calls retained as comparison
paths. Reuse existing official Ledger8 preparation/proof/application owners,
adding only the selected functional execution seam and its explicit receipt.
No common-gate framework or generic new public builder is required.

## Provisional candidate results

Before creating this checkout, the candidate's unchanged DID source generated
12 checked programs and its crate compiled. Four local lifecycle tests passed
for valid deactivation and late-witness failure through ordinary/functional
native and recorded routes. Fresh actual TypeScript execution produced 12 DID
rows, all equal to the retained observations; only the two selected case IDs
have the four-route Rust comparisons so far. The capture pins the exact runtime
production closure (219 files), Node and external shared-library bytes; host
system-library paths remain metadata, not a byte qualification claim. Strict
consumer Clippy passes. Wrong ordinary version and functional signature refuse
with E0308; consumed Program reuse refuses with E0382.

These are candidate development receipts, not actual-merged-baseline or proof
acceptance. Current-profile fresh selected proof/immutable verification,
funded default-strict application, binding/applicable result negatives, replay
refusal, complete integration CI and actual PR review remain open.

## Concrete generated-source regression and candidate repair

The complete functional DID fixture exposed six `let_unit_value` errors under
strict Clippy. The existing `Operation::PureCall` renderer bound a fallible
Unit-valued call to a local before returning that local. A checked `Type::Unit`
branch now emits the call once as a fallible statement inside the existing
`try_returning` closure, then returns `Ok(())`. Other checked result types retain
the existing returning helper. No source-name exception, lint suppression,
new IR/ABI or handwritten generated-code change is involved.

The focused regression genuinely failed before the fix and all seven functional
call tests pass afterward. Fresh complete generation changes exactly the six
Unit call sites while full IR/origins/capabilities/all twelve programs remain
byte-identical. The first maintained gate failure is retained; its next run
passes live TS, four selected tests, Clippy and three typed refusals using the
development emitter and immutable compiler137 Scheme frontend. These provisional
results do not replace clean compiler qualification after canonical reconciliation.

Failure coverage now also checks all ten captured invalid deactivation inputs
through ordinary native/recorded and functional native/recorded routes, including
stale-before-signature, signature-before-inactive, invalid signature/reduction,
quotient, timestamp and version/operation overflow. Every route must retain the
exact witness callback prefix and first error. The altered failure prestates are
explicit captured test inputs; inaccessible failed contexts are not claimed to
roll back. Timestamp-zero is a TS control and is not a selected Rust success case.

The selected proof seam shares the existing DID strict funded transaction owner;
it does not duplicate the transaction/proving implementation. It checks complete
IR and all twelve program bytes before keys/state, then uses functional recorded
execution and compares all four routes and ordinary typed observed preparation.
One fresh deactivation call proof must reject changed binding and a same-geometry
version input against the original verifier, followed by default-strict separately
funded application, exact final state and replay refusal without ledger mutation.
The Unit result has zero atoms, so no changed-result component is invented.
No fresh functional proof acceptance is claimed until that gate actually passes.

## Development proof and current validation result

The uncommitted prototype selected proof gate passes with the Unit-fixed
development Rust emitter and immutable compiler137 Scheme frontend. It generates
one fresh `deactivate` key (`k=11`, 1,804 rows) and a 3,296-byte functional-recorded
call proof. Positive verification, changed binding, same-geometry version input,
complete IR/program refusal before keys/state, funded default-strict application,
exact final state, replay refusal without mutation and stale-version first-error
controls pass. The original verifier and all source/tool/material/output premise
guards remain unchanged at completion. This is one call, not twelve proved exports;
constructor execution is not proved. Preserve these development receipts separately
from final clean compiler and actual canonical qualification.

The expanded maintained API gate passes five Rust test methods, all ten invalid
inputs through four routes (40 refusal executions), all twelve exact live TS DID
rows, strict Clippy and three compile-fail controls. Python discovery currently
reports 352 methods: 332 passes and 20 existing explicit skips, zero failures.
The new proof-runner seam passes strict all-target Clippy. Full fixture freshness
and the parent's complete compiler/E2E CI remain in progress at this checkpoint.

## Vendored source placement

The first new source-view location triggered two new full-template header
violations because the byte-identical upstream files correctly identify
`midnightntwrk/midnight-did`, not Compact. Preserve those original headers and
place the complete source view under
`examples/rust_backend/third_party/did_functional_lifecycle`, using the existing
third-party directory policy. No header rule or new exception is added. The
original failed header receipt and pre-move source/proof snapshot remain retained.
Regenerate origins and all twelve programs from the actual new input path;
do not rewrite or normalize path-bearing generated metadata. Recreate only the
47 unpublished declaration identities at that new path, preserving all 418
predecessor authority rows. Fresh source/API/proof receipts bind this successor
separately from the earlier valid development proof epoch.

## Accepted predecessor and current gate

Parent PR #692 is GitHub MERGED with valid GPG signature at `acf529eff`.
Reviewed source `98d6b141c`, CI checkout `57aa037bc` and the signed merge have
identical complete tree `abb79ea5fb2f72151b54380184fe034edd8a8c5a`.
All primary workflows passed, including the original Apple Silicon consumer and
Linux full native/proof/package/archive/wallet/E2E gates. The retained CI archive
keeps its literal `57aa037bc` identity; it is not relabeled as this new slice.
The current slice still needs its own clean compiler138, final source/proof
qualification, exact-PR review and complete integration CI.


## Clean local qualification — compiler 0.31.138

Signed reviewed implementation `bf221d0604490cfc3267820a90aeebbb5c10ee07` is
based on accepted canonical `acf529eff`. Its original Nix compiler realization
selects the same immutable 0.31.138 package built from clean `3ac403d8a`, with
its own Scheme frontend: all declared compiler/CLI inputs are unchanged. The
proof runner is rebuilt and API/proof checks rerun at `bf221d060`. Its build has
`doCheck = false`; the separate maintained compiler suite passes both ZKIR lanes
with 636/636 manifest and 1/1 JavaScript checks each at its recorded earlier
preparation epoch. These compiler tests remain part of complete integration CI.

Fresh qualification with the clean original compiler passes:

| Gate | Result |
| --- | --- |
| Original DID generation | Full IR/origins/capabilities and all twelve programs match |
| Live TypeScript | 24 observations, including all twelve exact DID rows; proof preparation consumes the same fixture oracle |
| Selected Rust behavior | Five test methods; ten invalid inputs on four routes (40 refusal executions) |
| Ordinary and consuming APIs | E0308 wrong version/signature, E0382 Program reuse; strict consumer Clippy |
| Selected standalone proof | Fresh direct key k11 / 1,804 rows; 3,296-byte proof; immutable verification |
| Proof controls | Changed binding and same-geometry version input refused; changed IR/program refused before keys/state |
| Independently proved local transaction | 7,413 retained bytes; separately funded default-strict apply, exact final state, IntentAlreadyExists replay with unchanged ledger |
| Applied-state refusal | Functional generated Program rejects the stale version first on actual applied state, without witness callbacks |
| Source hygiene | Clean checkout required before/after both gates; changed proof Rust formatting and proof-runner all-target strict Clippy pass; complete workspace format passed at prior source epoch |
| Backend library | 1,846 passed, seven existing explicit-run helpers ignored at clean `3ac403d8a`; owner code/lock unchanged, not a relabelled new run |
| Public Python | 353 discovered: 333 passed and 20 existing explicit skips |
| Empty-result regression | Genuine checked empty-tuple red then seven focused tests green; actual Compact `[]` canonicalizes to Unit and its complete generated consumer passes strict Clippy |

The prior clean `926e8de8c` compiler/API/proof receipts and 281-fixture census
remain retained at their literal original identities. All three exact version
checks and zero new header violations also passed at that earlier epoch. Their
full required CI gates must run on the updated PR; they are not relabeled as
new local execution. Final exact-head review and complete integration CI remain
required. This proves one selected standalone call and separately verifies and
applies its independently proved transaction; constructor execution,
timestamp-zero Rust success and all twelve exports are not proved.

## PR #693 independent review follow-up

Claude reviewed published `6d2981e9d` against canonical `acf529eff` and confirmed
the lazy, once-only Unit-call fix. Its ten advisory findings were checked against
the source before changes. The empty-tuple return case reproduced a real
`let_unit_value` regression and now shares the Unit statement emission while
retaining its checked tuple identity and typed binding semantics.

The funded strict transaction seam now accepts source-specific final checks.
The functional route checks flags and stale refusal on the actual applied state
using its own generated Program, and reads its own live-TS-qualified capture.
Both gates share the immutable source check and require a clean checkout.
Functional proof preparation consumes its own live-TS-qualified fixture oracle;
the historical ordinary selector retains its independently maintained capture. The
historical ordinary selector no longer acquires the new mutation control/file.
Proof exceptions write a failed receipt and nonzero exit; the delivery table
link and count spacing are corrected.

Artifact scope is explicit: one standalone proof carries immutable-verifier
binding/version controls; an independently proved transaction is default-strictly
verified, applied and replayed, with exact transaction bytes retained separately.
Receipt flags summarize live assertions; protocol unit tests are not proof
substitutes. Earlier clean `926e8de8c` results above remain historical evidence.
Fresh compiler/API/proof qualification now passes at clean signed `3ac403d8a`,
as recorded above. Repeat exact-head review and full CI remain required before
integration; no previous receipt is relabeled.

## Final CI setup and artifact retention corrections

The repeat Claude review of `cab5b3e01` confirmed the earlier fixes and the
workflow cleanliness conflict also found during author triage. Preserve both
existing binary smoke tests but write their output under `RUNNER_TEMP`; do not
exclude dirty source from qualification. Both actual smoke sources/options pass
locally with external outputs and leave checkout status unchanged.

Save the named applied-transaction bytes only after strict verification,
application, exact replay refusal and functional applied-state checks succeed.
The in-memory serialization still identifies the exact transaction used.
Both Python gate entry points now have a dirty-source refusal control that
checks failed receipts/nonzero exit and that no generation command executes.
Remove unnecessary equality with the historical runtime oracle now that the
functional proof consumes its own live-TS-qualified fixture capture directly.

Receipt flags are conclusions of the fallible shared proof helper, not future
configuration promises. The private ordinary selector discards the shared
internal summary, emits no new receipt/artifact, and is not claimed to acquire
new mutation controls. Defensive empty-tuple IR coverage is not a claim that
Compact source `[]` currently emits that IR shape. No public runtime/API or
broader source-language support is introduced by these corrections.

Fresh clean-source API and separate proof/application gates pass at signed
`bf221d060`. The new applied artifact is written after every verification,
application, replay and post-state check; the fresh standalone proof remains
3,296 bytes and the independently proved applied transaction remains 7,413
bytes. Both smoke commands pass with external outputs and unchanged checkout
status. Python now has 353 methods (333 passed, 20 existing skips) after replacing
weaker duplicate controls with the two real gate-entry refusal controls.
Repeat exact-head review and complete updated CI remain required.
