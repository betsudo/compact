# Stateful Vector value widening

## Status and baseline

T is admitted on `codex/rust-ast-stateful-vector-value-widening` from reviewed S docs `cf02d5693b1680823a25cb719c23c836db1ce5bd`, after inspected current refs/worktrees and clean canonical/predecessor states. Bounded local compiler/source/API and selected proof/default-strict application qualification is complete at execution `eaefb375285b2f7b2185ac63111887bd45a75e41`, with actual independent CLI review. Full CI and integration remain open. Prior reviewed local S execution
`7ab60b62061a728b3e81f34ae31e9693b1063d41` (compiler170). Canonical
`codex/rust-backend-ast` remains clean at1bdf185ea. The explicit local candidate
stack is not integrated canonical. Inspect the exact checkpoint/refs/worktrees
before admitting a clean successor separately. Preserve every
predecessor and all evidence; no Forge, publication or cleanup work.

## Concrete missing capability

`next-stateful-vector-value-characterization-v1/receipt.json` under S evidence
retains eight complete sources and24current default170 commands. Every source
passes TS. Five stateful sources fail both Rust and independent IR on a nested
`__compact_Cell.write`:

- A narrow Vector-returning pure helper widened to Uint16 or Field after a write.
- A narrow Vector parameter widened to Uint16 after a write.
- Transparent input/output Vector aliases in the same stateful flow.
- A Struct field holding the narrow Vector widened after a write.

The identity-helper, pure-parameter widening and direct-literal controls pass in
all three lanes. This is an actual source ingress gap. S's mixed helper branch
already works because its sibling literal selects one ordered return plan. A
helper-only or parameter-only root has no literal to select that owner.
Accepted literal, pure alias/member, Map/Fold and projection work is not reopened.

## Owners and intended public API

Reconcile `compiler/rust-ir-passes.ss` aggregate cast classification, storage
mutation/return selection and `stateful-return-plan-ir-value`. Reuse S's ordered
source-value conversion, typed contextual expression owner and existing native
Vector coercion. Preserve the frontend's actual source type and intermediate
checks before converting the reached value. Native and recording validators
remain independent, with no source-shape admission shortcut.

Keep the original complete exports and helper calls. Ordinary/recorded APIs
accept their existing `FixedVector<BoundedUint<255>,2>` or declared generated
Struct input and return `FixedVector<BoundedUint<65535>,2>` or
`FixedVector<Field,2>`. Functional Programs retain consuming context/program
ownership. No new flag, dependency, IR/runtime ABI or public helper API is planned.

## Scope exclusions

No source extraction/substitution, generated-code editing, arbitrary aggregate
conversion, narrowing relaxation, dynamic index, nominal redesign, effect lifting
framework, backend refactor, CI optimization or new primitive. Existing checked
bound/layout/resource/private/profile rules and all full integration gates stay.
Automatic proof k<=8 stays; preserve full sources and record every over-cap model.

## Acceptance criteria

1. Preserve refs/worktrees and reconcile current roadmap/crosswalk/checked-recording
   and retirement owners. Admit a separate clean baseline; focused existing
   source/API/coercion/return checks must pass before implementation.
2. Fix all five complete source forms at their existing owner. Characterize local,
   conditional, guarded and alias tails before extending beyond the observed
   Vector-to-Vector conversions. Preserve original Cell writes, helper calls,
   supplied bounds, source order and single condition evaluation. Keep complete
   identity/pure/literal controls unchanged where their route is unaffected.
3. Execute actual TS and unedited generated ordinary/recorded/functional APIs at
   original narrow input and widened result types. Check alignment, state/effects,
   private outputs, witness/call order, query gas and recorded replay. Include
   observable first-error controls and both meaningful state-dependent branches.
4. Correct-type twins accompany marked compile refusals for wrong input/result
   element/bound/length and reused Program/context. Keep frontend narrowing,
   length/kind/nominal refusals and independent malformed-IR/dead-branch checks.
   Changed real producer and result-oracle data must fail actual consumers.
5. Bind full source/import/IR/Program/SDK/compiler/default-recipe/tool/config/
   runtime/serialization identities and strict unedited SDK roots. Inventory all
   exports/models with explicit nonproof applicability before selecting proofs.
   Verify locked material before fresh keys. Require selected genuine Uint/Field
   proof/verification plus separately proved funded default-strict application,
   immutable wrong verifier, corrupted/binding/same-geometry typed-result controls
   and replay refusal. Cover both branches where the selected source has them.
   Do not trim/raise caps or imply constructor/all-input/unselected proofs.
6. Use focused arm64 checks during iteration, signed Conventional DCO/Assisted-by
   commits, actual Claude-first source/receipt reviews (actual-quota Codex fallback),
   documentation and verified Obsidian process/CAS. Retain complete required Linux
   CI/proof/integration gates. Bounded local acceptance cannot finish the broad goal.

No implementation, API execution, model feasibility or proof acceptance follows
from the source-only characterization above.


## Clean baseline, expanded evidence and owner decision

Signed clean652691a6 baseline passes35 focused coercion/projection/Fold/history/
stateful-return/native-Vector checks. Original170 characterization retains29full
sources/87commands:22TS-valid sources and7frontend refusals. Sixteen valid forms
fail before IR on nested storage/witness expression lowering; query-Vector and
stateful-helper values emit narrow IR and then fail Rust type equality. The four
passing controls include read-only conditional helper widening. Empty/nested/wide,
transparent scalar/Vector aliases, checked helper casts, guard order, stateful
callee and witnessed Vector source values are explicitly retained.

The existing literal predicate selects S's ordered return plan, while nonliteral
Vector casts use the legacy split or generic expression lowering. Add a bounded
Vector-to-Vector monotone conversion classifier (matching dimensions, recursive
Vector shape and existing Uint/Field widening leaves) along returned value flow.
Select the same ordered owner for helper/parameter/query/witness conversions.
Reach the original source type and actions first, then apply the existing checked
Coerce continuation. No native validator, runtime, schema/ABI, dependency or AST
owner change is proposed. Narrowing and nominal refusals remain authoritative.
Compiler metadata171 marks this source change; current build/API/proof/review
acceptance remains pending.


Original171 recipe passed. Current29-source characterization retains the first
attempt: all previously failing non-aliased forms emit, but scalar Uint/Field
aliases inside Vectors still miss the classifier. The pre-existing Vector carrier
helper intentionally returns scalar aliases unchanged. Unwrap scalar aliases
locally in the new Vector conversion leaf classifier, matching existing TypeIR
erasure without changing other alias owners. Compiler172 identifies this fix.
The pure control's first current Rust command accidentally requested functional
stateful emission; retain that refusal and use the correct pure flags in the
next current characterization. No API/proof acceptance is inferred.


## Current172 characterization and maintained consumer

Original default172 recipe passes at c010572. The complete current30-source/
90-command characterization passes all23TS-valid sources in Rust and independent
IR, with all7frontend refusals retained. Three original unaffected controls keep
all14IR/raw/Rust/capability/Program assets unchanged170/172. The full accepted S
helper-only control retains all6assets as well.

The maintained corpus copies complete23positive sources and7frontend negatives,
with37explicit cases. Actual TS and generated Rust development execution pass:
36stateful cases across144ordinary/recorded/functional routes plus1pure case.
Checks include both constructor-controlled helper and read-only branch outcomes,
read-only public state preservation, guarded empty helper evaluation, checked
helper first-error order, stateful helper effects, nested/wide/aliased carriers,
and a Vector-valued witness with exactly one narrow private transcript output
and private-state evolution. Current maintained strict/refusal/producer and
proof qualification is still pending, not inferred from development runs.

The opt-in proof adapter includes the whole unedited selected `proofs` SDK and
both Programs, with genuine Vector<2,Uint8> input. It must prove both Uint branch
states (keep vs flip helper) and Field separately, using actual narrowed helper
values before conversion. Default/all-features builds need no generated SDK;
explicit gate builds bind the complete current source/IR/Programs plus every
physical generated and embedded-golden include. All models/applicability,
verified material, immutable keys/verifiers, strict funded application and replay
controls remain required. Compiler/native/runtime ownership and ABI stay as in
the reviewed S baseline except the stated frontend conversion/context change.


## Original compiler 172 local qualification

Execution commit `eaefb375285b2f7b2185ac63111887bd45a75e41` is the clean
source authority for the maintained T qualification. The original Nix recipe
produced `/nix/store/q4bg0cfmlp1bcxnikxjbcc9xg5b0h0iv-compactc`; the default
packaged compiler reports 0.31.172. No alternate Scheme/backend or generated-code
editing is used. Documentation successors preserve this execution identity.

The complete 23-source corpus passes 136 maintained commands and 37 actual consumer
cases: 144 ordinary/recorded/functional stateful executions and 1 pure execution,
alongside independent TS. All 23 unedited generated SDK roots pass strict checks;
eight marked Rust misuse cases fail and their separately compiled correct-type
twins pass. Seven frontend refusals, seven malformed-IR refusals, three changed
producer consumers (including witness-before-write) and the changed result oracle
fail as intended.
Read-only state, both conditional helper outcomes, checked helper first-error
order and the once-only Vector-valued witness/private-state transcript remain
explicitly checked. The 35 focused native/history regressions pass at `7a9234bef`,
with an exact unchanged owner-source bridge to this head. All 283 retained
fixtures are freshly identical without update. Native/recording validators,
runtime, ABI and historical fixture owners are unchanged from S. The S
predecessor source/API gate also freshly passes 155 commands, 50 cases / 200 Rust
routes and 31 strict SDK roots at this head. S's full 1833 native / 7 ignored and
strict native Clippy receipts retain their S head.
They are not relabeled as fresh T full-CI results.

All 24 exports are inventoried: 23 proof-required and 1 pure/nonproof. Selected `run`
uses k=8 / 130 rows and `run_field` k=7 / 85 rows. Locked upstream material is verified
before two fresh native key generations. Three fresh 2912-byte standalone proofs
cover keep/flip Uint branches and the Field export. Three independently proved
funded transactions of 6605 / 6605 / 6561 bytes apply with default strict validation.
Corrupt proof, changed binding, changed real same-geometry typed result, immutable
wrong verifier and replay refusal controls pass; replay leaves the applied state
unchanged. Full source/IR/Programs, complete unedited SDK, raw Cargo dependency
file and physical generated/embedded-golden includes remain bound to the run.

The other 21 required exports are not proved; `wide_parameter` is k=9 and remains
unproved under the unchanged automatic k<=8 cap. Constructor execution, unenumerated
inputs, prior N/S wide model proof obligations, complete required arm64/Linux CI,
reviewed integration and the broad Rust AST goal remain open. Local qualification
does not authorize publication or cleanup. Forge remains deliberately deferred.

Evidence root: `/Users/ysh/.codex/local-evidence/rust-ast-stateful-vector-value-widening-20261102`.
Original compiler `original-native-v5/receipt.json`, API `proof-v2/protocol/receipt.json`,
proof/application `proof-v2/receipt.json`, retained fixture check
`retained-fixture-freshness-v2/receipt.json`, and `regressions-v1/receipt.json`
have distinct scopes. Independent review and receipt audit authorities are added
only after their actual terminal results are inspected.


## Independent review and continuation

Actual Claude source review found two P2 acceptance gaps. The source now checks
that the witness observes the preceding write and verifies a real reordered
producer failure; input-bound/input-length refusals and separately compiled
correct-type twins complete the eight-case misuse roster. The second source
review and exact execution-head receipt/current-delta inspections found no
remaining P1/P2 issues. Receipt inspection rehashed all 6679 original pin records
across 4704 distinct paths, then independently checked embedded golden depfile
membership, all eight twin logs, all 31 predecessor IRs / 34 Programs and the
35-test unchanged owner bridge. This is inspection, not crypto reexecution,
full CI or a human security audit.

`review-adjudication-v1.json` preserves the review limits and corrects a reviewer
summary count: the current S gate has 23 expected nonzero exits, not five.
`physical-dependency-reconciliation-v1.json` records all 860 physical worktree
inputs against clean Git HEAD; 68 historical embedded IR fixtures previously
had only the immutable-head binding outside the combined hash maps. The final
`receipt-audit-v3.json` adds explicit post-run closure hashes and review evidence:
7548 pin records / 4781 distinct paths. These added hashes are not retrospective
execution-time attestations. The disclosed reviewer scratch diff is retained
separately; source and qualification evidence were not modified by the review.

The next bounded [stateful local Vector proposal](stateful-local-vector-widening.md)
is source-only. It addresses explicit widened local bindings whose narrow
initializer currently fails independent native type checking. No implementation,
model, proof or acceptance is inferred for it. This T closeout does not complete
the primary goal or the deliberately deferred Forge experiment.
