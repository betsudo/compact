# Current-candidate DID lifecycle handoff

Status: planned local qualification at compiler 0.31.183; no new DID result yet.
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
