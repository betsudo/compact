# Current-candidate DID lifecycle handoff

Status: reviewed selected local compiler183 API and separate proof/application
qualification pass at `67cbded3e8e6b0fd830679e8eae967e63dfadfb5`.
The broad goal and full CI/Linux/integration remain open.
The later [zero timestamp witness slice](did-zero-timestamp.md) qualifies the
previously unexecuted positive edge separately at compiler183 source `4e49b59e5`.
Parent: signed `940183cbba9a04af2a530ce3462b2ed1935e72b5`.
Branch: `codex/rust-ast-current-did-handoff`.

## Gap and preserved acceptance

Selected DID deactivation is already integrated through PR #693 at compiler138,
Ledger8.1.3. Preserve its complete original source, all twelve Programs, selected
native/recorded/proof/strict-application results and limits at their literal epoch.
This is candidate reconciliation after later compiler/rendering work, not a new
DID feature or reopening of that accepted implementation.

The complete two-file original closure and maintained API/proof gates match the
accepted merge. The checked-in generated functional Rust differs after earlier
renderer work. The newer compiler and bundled witness bridge also differ from
the earlier qualification. A source comparison alone cannot declare current
behavior or proof acceptance; run the maintained gates before freezing the profile.

## Owners and unchanged public API

- `examples/rust_backend/third_party/did_functional_lifecycle` and the identical
  `did_adoption` authority: unchanged v0.7.0 two-file closure and original manifest.
- `tests-rust-backend/did-functional-lifecycle-source`: whole generated fixture,
  exact IR/origins/capabilities and twelve checked Programs, existing genuine
  Schnorr/reduction witnesses and selected four-route lifecycle tests.
- Checked typed lowering, existing functional Program/witness emission, native
  and recorded Ledger8 adapters: semantic, ownership, ordering and error owners.
- `check_did_functional_lifecycle.py` and separate
  `check_did_functional_lifecycle_proof.py`: current-generation, TS/API/refusal
  qualification and uncached selected proof/default-strict application.

Consumers keep ordinary native/recorded methods and consuming functional
`deactivate::Program::new(...)` / execution methods with typed witnesses. No
new facade, IR/ABI, crypto assumption, shortened contract or erased type is proposed.

## Bounded acceptance and order

1. Freeze clean actual source and resolve the supplied original compiler through
   the build-disabled recipe recorder. Keep exact compiler/Scheme/Node/Rust/TS
   runtime, source/runtime/dependency and parameter identities. Reuse the owned
   Cargo target with bounded local jobs; never delete stores or old artifacts.
2. Run the unchanged API gate: complete original source, all twelve Programs,
   exact current fixture IR/origins/capabilities and separate formatting-copy
   comparison. Compare twelve live DID TS observations and five selected Rust
   methods across all four routes, including 40 intentional failing-route
   executions, witness order, actual crypto, errors, checkpoint and metering.
   Require strict consumer Clippy and marked E0308/E0382 refusals. Keep whole
   source/Program binding; on a real failure preserve it and fix its actual owner.
3. Only after API acceptance, run the separate maintained proof gate. Keep all
   twelve generated Programs/complete IR bound before keys; changed IR/Program
   bytes must refuse before keys/state. Verify the retained upstream built-ins
   and actual selected parameters. Existing DID k11 material/geometry is its
   maintained explicit recipe, not an increase to the tiny/nested-product k8 cap.
4. Require fresh direct deactivation keys and an actual standalone proof with
   immutable verification, changed binding/version input and same-geometry
   controls. Separately prove and apply the funded DEFAULT-strict transaction,
   check expected state/stale-version and unchanged replay refusal. Unit has no
   output atoms, so no invented changed-result control. Constructor execution
   and all-twelve-export proofs remain unclaimed.
5. Inspect source/actual API and proof receipts independently with fresh Claude
   CLI review (actual-quota Codex fallback). Bind current acceptance to the literal
   source/tool/profile; preserve historical epochs. Retain all required native,
   Linux, package/archive and proof integration gates.

## Exclusions and continuation

No new semantic feature, broad refactor, CI optimization, Forge, key/proof/verdict
cache, source trimming, geometry normalization, cap increase, registry/release or
wallet/network claim. No current output repin just to get a pass. The source-run
fixture may be formatted only through the maintained separate-copy contract;
IR and all twelve Program bytes remain exact and cannot be normalized.

Keep everything local: no push, merge, hosted trial or shared install. Post-merge
cleanup requires verified GitHub integration and preserved unique evidence.
A successful selected lifecycle does not prove all exports or complete the broad
Rust AST goal, full CI/Linux/integration or the finite supported-profile handoff.

Worktree: `/Users/ysh/.codex/worktrees/rust-ast-current-did-handoff/compact`.
Evidence: `/Users/ysh/.codex/local-evidence/rust-ast-current-did-handoff-20261103`.
After this candidate application obligation, reconcile the capability/exception
book against current and retained evidence without reopening accepted work or
inventing coverage percentages.


## Reviewed current-candidate local result — 2026-11-03

At clean signed `67cbded3e8e6b0fd830679e8eae967e63dfadfb5`, the original
build-disabled compiler recipe passes, followed by the unchanged API gate and
then the separate unchanged proof gate. The compiler is the same immutable
0.31.183 package; no rebuild, compiler change or fixture repair was needed.

The API gate's ten commands pass. The complete original two-file closure matches
its authority. Fresh full IR, origins, capability metadata and all twelve checked
Programs exactly match the current fixture. Only a separate formatting copy is
compared with the checked-in generated Rust. Twelve live DID observations (plus
twelve auxiliary crypto observations) match the retained TS cases. Five selected
Rust methods cover the ordinary and functional native/recorded routes, including
ten failing scenarios across four routes (40 executions), genuine witnesses,
ordering, checkpoint and metering. Strict consumer Clippy and three intended
E0308/E0382 controls pass. These are selected deactivation cases, not every export.

The proof gate's nine commands and two manual source-binding refusals pass.
Changed full IR and changed Program bytes refuse before keys/state. All twelve
Programs and complete IR remain bound. Direct fresh keys retain the existing
k11 / 1,804-row geometry and verified upstream built-ins/SRS. A 3,296-byte
standalone mutation-control proof verifies; a **separately proved** funded
DEFAULT-strict 7,413-byte transaction applies. Changed binding and version input,
same-geometry checks, post-application stale-version refusal and unchanged-state
IntentAlreadyExists replay pass. Unit has zero output atoms, so a changed-result
control is not applicable. No constructor or all-twelve-export proof is claimed.

Fresh authentic Claude CLI source/receipt inspection reports no actionable P1/P2.
It checks the 1,507 API and 2,943 proof source pins, generated assets, components
at source and snapshot, material, dust, result and command records. This is receipt
inspection, not an independent cryptographic rerun. The Python gate enforces full
IR byte equality; the runner's separate IR validator compares parsed JSON. Program
byte binding remains exact; neither is described as a formal semantic proof.

The TS capture records Node25.6.1 and runtime0.16.102 identity, 219 declared
runtime/dependency file hashes before import with after-run equality, capture and
Node continuity, and 23 reported non-system shared-library hashes. A separate
post-run comparison matches the preceding accepted Passport cohort's 44 overlapping
pins. These are trusted capture/inspection observations, not independent external
before-use admission or exhaustive consumed-source, Rust-dependency, system-library,
kernel, continuous/atomic or hostile-host attestation. The v1 supplied-cohort prose
is clarified by its v2 record without rewriting the original evidence.

Evidence: `controller-v1.json`, `original-existing-recipe-v1/receipt.json`,
`did-api-v1/receipt.json`, `did-proof-v1/receipt.json`,
`local-claude-did-receipt-review-v1.json`, `supplied-ts-tool-cohort-v2.json`
and `did-final-adjudication-v1.json` under the slice evidence root. Old accepted
compiler138/PR#693 and earlier renderer/fixture/proof epochs remain separate.
The existing explicit DID k11 recipe does not increase the tiny/nested-product
k8 limit. Full CI/Linux, logical integration and finite profile handoff remain open.

### Concrete remaining positive edge

`did/timestamp-zero` is a successful original TS capture that the selected Rust
suite does not execute: its success helpers select `did/valid`, while the shared
four-route loop covers the ten error rows. Preserve the existing five methods and
all original captures. The next bounded slice should reproduce this positive zero-
timestamp case across the four Rust routes, then fix an actual semantic owner
only if it fails. Do not claim this unexecuted Rust edge or all twelve export
behavior from the present selected qualification. The broad goal stays incomplete.
