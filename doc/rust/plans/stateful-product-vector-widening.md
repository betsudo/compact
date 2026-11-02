# Stateful widening of Vectors of scalar products

## Concrete gap and source authority

Current original compiler 173, U execution `408dd00ee8c35b050ed6f307be0bf89edd839984`,
accepts the pure product-Vector coercion, same-type stateful product return and
newly accepted scalar-Vector local conversion controls. Stateful widening of
Vector elements that are scalar products is still a real source gap.

U evidence `next-stateful-product-vector-characterization-v1/receipt.json`
retains 13 complete sources / 39 commands. Eleven sources pass TS. Seven fail both
Rust and independent IR on a nested Cell.write: parameter/helper results,
Uint-to-Field products, nested Vectors, an unchanged Boolean component, empty
product Vectors and the complete two-export helper/branch proof candidate.
The local product initializer emits IR but fails independent native type checking
because it lacks its declared conversion. Three controls pass all three lanes;
implicit narrowing and wrong product arity remain frontend refusals. No model,
key/proof feasibility, new implementation or API acceptance follows from these
source-only results.

## Owners and intended public API

Reconcile `compiler/rust-ir-passes.ss` supplied aggregate-cast classification,
existing ordered return selection and U's typed local conversion. Trace actual
source/target product carriers first. Reuse native/recording Coerce and existing
typed product/vector AST owners; do not duplicate renderer or validator rules.
Keep matching dimensions and product arity, original component bounds, intermediate
casts, lexical/effect/callback order and single evaluation. The classifier chooses
an evaluation owner; independent native/recording shape/type/layout/resource
validation remains authoritative. An unchanged Boolean component is identity,
not a new Boolean-to-number conversion.

Public APIs retain actual complete types, for example
`FixedVector<(BoundedUint<255>,BoundedUint<65535>),2>` input and
`FixedVector<(BoundedUint<65535>,BoundedUint<4294967295>),2>` or
`FixedVector<(Field,Field),2>` output. Preserve ordinary, recorded and consuming
functional Program/context APIs. No new public flag, helper, ABI or IR primitive.

## Acceptance

1. Inspect exact checkpoint/refs/worktrees, preserve U and all predecessors, admit
   a separate clean branch from the reviewed U documentation head, and run focused
   baseline checks. Reconcile accepted roadmap/crosswalk/recording/retirement
   ownership; pure product coercion and prior scalar/Vector work stay accepted.
2. Fix every observed valid complete form at the existing owners, retaining source
   bounds before target conversion. Characterize transparent aliases, checked
   helper/guard first errors and before/after-write witness variants before adding
   them to the corpus. Do not infer support from type shape or emission alone.
3. Execute independent TS and complete unedited generated Rust ordinary/recorded/
   functional SDKs at their real nested tuple/Vector types. Check element/component
   order with asymmetric values, both selected branches, state/effects/private
   outputs, once-only calls/witnesses, Rust-route metering and independent replay.
   Preserve accepted controls byte-for-byte where the route is unaffected.
4. Include marked wrong input/result component type/bound, product arity and Vector
   length compile refusals with independent correct-type twins, plus Program/context
   reuse. Keep frontend narrowing/arity/kind/nominal refusals and independent
   malformed Coerce/binding/dead-branch/static-empty-case validation. Changed real
   producer/component order and result-oracle data must fail actual consumers.
5. Bind exact complete source/IR/Programs/SDK/default compiler recipe/backend/tool/
   config/serialization/parameter/verifier identities and physical generated/golden
   includes. Inventory every export/model/applicability before proof selection.
   Verify material before fresh keys. Require genuine selected Uint/Field product
   proof/verification and separate funded default-strict application with immutable
   wrong-verifier/corruption/binding/real same-geometry result/replay controls.
   Keep complete over-cap sources under unchanged automatic k<=8; no trimming or
   assumed feasibility. Constructors/unselected/all-input coverage stays explicit.
6. Use focused local arm64 iteration and actual Claude-first source/receipt review,
   actual-limit Codex fallback, signed Conventional DCO/Assisted-by commits, docs
   and verified Obsidian process/CAS. Retain complete required CI/proof/integration
   gates; one local slice cannot complete the broad goal.

## Exclusions

No arbitrary Struct/nominal conversion redesign, narrowing relaxation, unchecked
identity extension to unprobed types, new primitive, cap increase, effect framework,
backend refactor or CI optimization. Preserve original full source and output
geometry; no generated-code editing, source extraction or proof/verdict cache.
No Forge work, publication, shared install or cleanup. Qualification remains bounded to the actual completed gates below.


## Admission

V is separately admitted on `codex/rust-ast-stateful-product-vector-widening`
from reviewed U docs `f10393353065c8d9af142594181ed5b1b0802368`. Exact current
checkpoint, refs/worktrees and clean canonical/U states were inspected first.
All predecessor work and evidence are preserved. Focused baseline and actual
product-cast traces precede implementation; no V compiler/API/model/proof
acceptance is claimed. Existing pure coercion, typed native/recording validators,
AST owners and original retirement/crosswalk authorities stay intact.

## Source implementation and qualification boundary

Restart reconciliation found clean admission `1b04a74ce`, all 38 focused baseline
checks passed and the unchanged original compiler 173 recipe passed. The retained
trace shows supplied Vector-of-product casts enclosing ordered ledger writes and
local bindings. Existing native Coerce already owns independent tuple/Vector
shape, scalar bounds, Boolean identity, conversion work and AST emission.

The frontend classifier now recognizes equal-arity scalar products within
matching Vector dimensions, including nested Vectors and unchanged Boolean
components. It retains numeric monotone conversions and alias carriers at this
owner, and selects the existing ordered evaluation/Coerce path. No native,
recording, runtime, IR schema, ABI or public option changes are introduced.
Compiler version 174 names this source change; original recipes are retained.

The complete source corpus contains 23 TS-valid contracts and two original
frontend refusals. Before implementation, nine valid contracts fail on nested
ledger evaluation and eleven emit IR but fail native checking; the three existing
controls pass. Additional forms include transparent aliases, checked helper first
errors, an observable stateful helper, both witness positions, read-only storage,
multiple conversions, scalar/Unit results, a three-component product and an intact
wide local. An earlier development source roster had ten `>=` tokenization errors;
its files/logs remain retained and the corrected source-only characterization is
`characterization-original-v2/receipt.json`. Source characterization alone is not
API execution, model feasibility, proof or application acceptance.

Complete current compiler/API, marked compile refusals with correct-type twins,
malformed IR and producer controls, selected genuine proofs/default-strict
applications, independent source/receipt review and integration gates remain open.
Keep all historical evidence at its actual source head. Do not label emission or
this source commit as completed slice qualification.

## Maintained API gate

`tools/compact-rust-backend/check_stateful_product_vector_widening.py` runs the
complete 23-source corpus and 30 declared TS/Rust cases: 116 stateful executions
across ordinary, recorded and consuming functional routes, plus one pure route.
The actual development consumer passed all cases with complete unedited compiler
174 SDKs. The maintained gate adds full independent IR/Program goldens, physical
local-conversion origins, strict complete SDK roots, ten marked compile refusals
with independent correct-type twins, seven frontend refusals, nine malformed-IR
controls, a same-geometry changed result and four regenerated complete producer
controls. Its acceptance still requires an actual clean-head passing receipt.

The checked helper's declared errors distinguish its guard and each of four
source component narrowings before outward widening. Asymmetric row/component
values expose reversals; the stateful helper observes the caller write and
performs its own write exactly once. Both witness positions read the specified
ledger value and preserve narrow private transcript geometry. Replay, effects,
private outputs and gas agreement are checked separately from result equality.

The first unaffected-control byte comparison used different absolute source
paths and correctly failed on preserved Program origin filenames. No origin or
JSON normalization was applied. Fresh current generation using the exact
original source paths produces byte-identical complete IR and Program membership
for all three controls (five files), retained in
`accepted-control-comparison-v2/receipt.json`. The failed comparison/diffs remain.
No proof/application or integration result follows from these API controls.

## Nested public signature lint boundary

The maintained 174 gate at `30f8f1c4a` passed all 30 consumers and the first
11 strict SDK roots, then failed nested_product's strict root on eight
`clippy::type_complexity` diagnostics. The generated complete nested product
signatures are required carriers, not a runtime/type error. The failed receipt
`source-api-v2/receipt.json` is retained and does not qualify the full gate.

Compiler 175 extends the existing `CompactSignatureLint` AST owner, which already
scopes the positional-argument style exception to affected functions. It marks
only individual signatures containing a runtime FixedVector of FixedVector of a
nontrivial product, including wrapped outcomes, with a reasoned type-complexity
style exception. It adds no crate/module-wide suppression, type erasure, runtime
rule, new API alias, dependency, generated-code edit or admission fallback.
All other strict lint checks and ordinary Rust type/ownership errors remain
active. Broad alias/API redesign is outside this repair. Semantic validators,
conversion AST bodies, original source/IR/Programs and ABI remain unchanged;
the generated signature attributes are an explicit metadata change. Current
175 clean-head validation and independent review must cover this successor.

## Independent witness interaction finding

Actual read-only Codex source review v1 found no P1/P2 at `0821e88fa`.
The separate successor review v2 found a real P2 at `1c0623fd4`: nested-product
Witnesses methods acquire the scoped style attribute, but compact_witness_bridge
previously accepted only doc attributes. A complete real compiler175 nested
witness SDK reproduces ordinary compilation failure in
`witness-style-regression-v1/receipt.json`. Emission and 1,833 backend unit passes
(seven existing ignored) plus strict backend Clippy did not cover this interaction.
Those successful receipts retain their original head; the failing SDK is retained.

Compiler176 coordinates the existing witness bridge with the signature owner.
The bridge permits only the two generated signature style lint names
(type_complexity/too_many_arguments), with an optional string reason. It still
rejects cfg/expect/semantic attributes, other lint names/groups, empty allowances,
malformed or duplicate reasons. The same allowed style attributes propagate to
the ordinary trait, fallible trait and adapter implementation; docs retain their
original two public-trait surfaces. Signature/body/parameter validation is intact.

Two complete combined witness/nested-product contracts now exercise before- and
after-write callbacks, original narrow nested private transcript geometry and
all four Rust routes against independent TS execution. The maintained roster is
25 sources / 32 cases (124 stateful route executions plus one pure). Counts are
expectations until its actual current gate passes. The gate retains its complete
SDK-root, marked refusal/twin, source/malformed/producer/result controls. Current
176 build/API, macro parser controls/strict Clippy, fresh independent review and
selected proof/application qualification remain open; no trimming or disabling
of the failing combined shape is permitted.

## Three-component signature successor

The 176 maintained gate executes all 32 consumer cases successfully (including
both combined nested witnesses), then stops at command92 on eight strict
triple_product SDK signature type-complexity diagnostics. This is retained as
`source-api-v3/receipt.json`; it is not full gate acceptance. Fresh independent
source review v3 reports no P1/P2 in the bridge repair at `2508fec89`, with SDK/
golden/current gate acceptance explicitly pending at review time.

Compiler177 extends the same function-only style marker to the other actually
observed complex carrier: a Vector of a product of three or more scalar
components. It retains the nested Vector-of-product case and all independent
macro/type/semantic/refusal checks. No module/crate-level exception or signature,
body, source/IR/Program normalization is introduced. The complete triple source
is retained and must pass strict root Clippy; it is not replaced by a simpler
source. Current177 clean-head source/API and fresh source-delta review remain
required. Earlier compiler/native/SDK/review outcomes keep their own epochs.

## Current exact diagnostic and snapshot repair

The 177 source/API gate v4 passes all 25 strict complete SDK roots, all 32
consumers and all ten marked compile refusals with independent positive twins.
It then stops at command125 because the new wrong-length oracle expected
"actual type" whereas the frontend correctly reports "actual return type".
The compiler/source refusal is correct; that failed gate remains retained.
All seven complete original177 refusal sources were independently characterized
before updating this new oracle. `frontend-refusals.json` now binds each exact
source SHA, nonzero exit and complete stdout/stderr bytes, including both private
condition disclosure paths. No substring relaxation or diagnostic normalization.
All nine independent malformed controls also reproduce the intended native
refusals, including an unreachable bad branch and zero-length Vector element.

Current retained-fixture check actually checked283, with one stale snapshot and
zero generation failures. The sole current effect_plan_public_products snapshot
requires seven function-scoped type-complexity attributes; its historical full
bytes are preserved in Git and an external evidence copy before a targeted
official --only/--update. Removing exactly those seven full attribute blocks
restores the complete old bytes. No source, body, signature, IR, Program or
historical compatibility test changes. Current no-update283fixture verification
and the complete current V gate still require passing successor receipts.
