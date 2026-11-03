# Fixed Vector leaves below component products: next bounded source gap

Status (2026-11-03): the compiler187 native/model stage is locally accepted at
`005a10603` after its original recipe, complete 88-command gate, all-export model
observation and separate independent source/receipt reviews. Fresh proofs and
strict applications remain open; this is not full slice or integration acceptance.
The broader Rust AST goal remains incomplete.

## Historical admission baseline

Source characterization began from the accepted compiler186 metadata baseline
at `3ebb20905`. Both intact original sources were TS-positive and Rust/independent
IR refused. The complete fourteen-export application was declared before the
classifier change; its current186 acceptance/refusal subsequently executed.
The characterization head `08ac3bb51` changed no semantic classifier, checked
plan, AST renderer, runtime or ABI.

## Gap and affected owners

The current frontend selector permits scalar/tuple subtrees below a component
Vector but stops at another fixed Vector inside those products. Both complete
existing counterexamples must be preserved at their original declared paths:

- tests/nested_component_products/unsupported/vector_inside_nested_product.compact
- tests/stateful_vector_product_elements/unsupported/vector_inside_component_product.compact

Paths above are below tools/compact-rust-backend. These existing one-by-one and
two-by-two sources predate this proposed feature; neither is a newly trimmed
replacement for the other.

- compiler/rust-ir-passes.ss::vector-value-widening? owns ordered transport of
  frontend-supplied conversions. Extend only the existing component-product
  relation to compose fixed Vector children with exact dimensions/tuple arities.
  Preserve the outer element relation: top-level nested tuple widening remains
  outside this slice.
- pure_semantics/coercion.rs already has bounded recursive Vector/Aggregate
  checked plans; characterize its actual acceptance before changing any owner.
- value_lowering/coercion.rs already emits structured AST from checked facts.
- recorded/effect_plan independent validation remains authoritative for every
  original child, including dead arms and zero-length element types.
- recorded/effect_plan/functional_items.rs::request_field retains exact private
  typed carriers. Close any actual same-type baseline lint defect first.
- Maintained source/API/static/refusal/proof gates own whole-source qualification,
  literal historical authority, current intended rows, consumer and CI wiring.

No IR/schema/ABI/runtime/carrier change is intended; do not assume one is needed.

## Public API

Keep nested FixedVector<(...) ,N>, tuple and BoundedUint carriers, ordinary native,
recorded and observed APIs, Program::new with consuming native/recorded execution.
No public conversion trait, dynamic value, alias erasure, handwritten SDK fix,
broad evaluation framework or source-name admission.

## Finite scope and characterization

Only legal monotone Uint widening, Uint-to-Field and existing Boolean/Field
identity under fixed Vector/product alternation below an existing component
Vector. Preserve every Vector length and tuple arity, aliases under existing
type rules, lexical order, once-only original helper/witness evaluation and
first-error behavior. Do not admit nominal Struct/new identity carriers,
narrowing or arbitrary top-level nested products. Keep resource limits unchanged.

Before dependent feature work, use exact current compiler185 and both complete
original sources plus complete separately declared pure and same-type controls.
Characterize TS success/current Rust-IR refusal and actual supplied casts; lint
whole same-type SDKs at their own roots. Preserve all failed epochs.

## Acceptance

1. Build original maintained next compiler recipe from clean signed source with
   owner-generated docs/version alignment, not an alternate compiler pipeline.
2. Both original whole sources must generate exact original carrier APIs and
   execute across ordinary native/recorded and functional native/recorded routes.
   Independently capture current TS; bind actual Rust arguments to TS typed
   values. Check exact results, state/effects/gas/private outputs and VM replay.
3. Exercise nonuniform first/last values, bounds, helper and witness once-only
   ordering, before/after writes, failure precedence, inactive arms, nested
   Vector emptiness and empty/single tuples using complete declared sources.
   Inventory every export; never extract a circuit for easier qualification.
4. Wrong inner leaf bounds, inner/outer dimensions, tuple arities/results and
   consumed Program usage must fail at separately marked ordinary-use sites
   with matching valid controls. Keep strict whole-SDK/test-code lint.
5. Malformed untrusted IR/recording controls must independently reject shape,
   bounds, wrong identity/source/helper binding, invalid static/dead/empty
   children and resource excess. Preserve historical refusal tables byte-for-byte;
   change only the genuinely implemented current boundary rows. Keep unrelated
   nominal/new-identity/narrowing/top-level nested-product source refusals.
6. Observe every intact model before selecting proofs. The two existing intact
   original sources keep separate qualification. Use actual applicable intact
   k<=8 source/export(s) for fresh direct keys, standalone proofs/independent
   verification and identity-bound corrupt/binding/result-equal-geometry/
   preimage/wrong-verifier negatives; separately prove/apply funded DEFAULT-strict
   transactions and unchanged-state replay refusal. Keep any over-cap source
   unchanged and unproved; no trimming, geometry normalization or cap increase.
7. Actual independent source and receipt reviews: Claude first, authorized
   Codex CLI fallback only after actual quota. Focused local iteration; retain
   complete required CI/proof/integration gates. No hosted execution is inferred.

## Exclusions / prerequisites

The current combined rebase qualification and separate receipt review must settle;
correct real baseline/regression findings before features. No broad refactor,
CI optimization, Forge work, push, merge, release, hosted trial, shared install
or cleanup. The broader Rust AST goal remains incomplete.

## Concrete baseline defect before implementation

External complete controls in
`/Users/ysh/.codex/local-evidence/rust-ast-component-product-vector-leaves-20261103`
have actual current185 TS/Rust/independent-IR success (12 commands). Both pure
whole SDK roots pass strict all-target/all-feature Clippy. Both same-type
stateful whole roots fail exactly `clippy::type_complexity` on the private
functional Program input field. The original failed first gate and remaining
three first-time lint runs are preserved separately; this is an actual existing
baseline defect, not a new widening failure.

Next prerequisite: extend only the existing reasoned field-local metadata to
these characterized nested Vector/product carriers, preserving exact type and
private visibility, constructor/method/context/Program ownership, all previous
AST cases and unrelated negative carriers. Add exact AST controls for direct
Vector and singleton-tuple-wrapped Vector children in either product position;
keep all prior request-field test bodies. Generate both complete same-type SDKs
from a clean signed maintained compiler, lint whole roots/test consumers, execute
four routes/replays and marked bound/arity/dimension misuses with positive twins.
Compare raw whole IR/Program/ZKIR and SDK AST to compiler185, allowing only the
declared field attribute and compiler metadata differences. No normalization,
new proof substitution or wider language admission. Preserve both original
widening refusals until a separate semantic change qualifies them.

## Baseline prerequisite accepted — 2026-11-03

The [private request-field baseline repair](vector-leaf-request-field.md) now has
bounded local acceptance at `9f630221a` with original compiler186, exact raw
input comparisons, maintained three-source API/static execution and independent
source/receipt review. Its two actual same-type lint failures, zero-selected
signature attempt and documentation precursor timeout remain preserved. No
widening classifier has changed yet.

The original complete direct Vector and singleton-wrapped Vector counterexamples
still pass TS and refuse Rust/independent IR under compiler186, with exact
previous185 diagnostic bytes on the same physical inputs. Begin the semantic
slice from this repaired baseline, retain both whole original sources and all
unrelated refusals, and use the existing bounded checked coercion/render/validator
owners. Do not trim a source or reinterpret baseline lint acceptance as proof
or new language support. Full CI/Linux/integration and the broad goal remain open.


## Semantic slice admission

Branch: codex/rust-ast-component-product-vector-widening.
Worktree: /Users/ysh/.codex/worktrees/rust-ast-component-product-vector-widening/compact.
Evidence: /Users/ysh/.codex/local-evidence/rust-ast-component-product-vector-widening-20261103.
The accepted metadata baseline is preserved in its own branch and inert backup
ref. The source roster has two complete original one-export counterexamples and
one new complete fourteen-export application (two private helpers, one witness).
The latter includes direct/helper/local conversions, checked-helper first-error
order, both witness positions/failure, both branches, deeper alternation,
Boolean-to-Boolean and Uint-to-Field, Field identity, empty outer/leaf Vectors,
and empty/single tuples. It is not a smaller replacement for either original.
Source/model/API/proof acceptance remains open pending actual execution.

## Current source characterization and existing checked owners

At signed characterization head `08ac3bb51`, the unchanged original compiler186
recipe was rebuilt because its declared inputs include the new full example.
The existing-only check correctly refused the older package; that failed epoch
is retained. The resulting original package built successfully in 56.21 seconds.
Its separate ten-command gate passed: every complete source succeeds in TS with
all 1/1/14 declared exports and refuses both Rust and independent IR before the
selector change. No refused Rust/IR artifact was published.

Before changing the selector, all five conversion-domain tests passed, including
new direct and singleton-wrapped Vector children, native checked-plan positives,
independent recorded-domain positives/refusals, narrowing, mismatched dimensions,
wrong arity/result, identity leaves and static-empty validation. The original
four test bodies are unchanged. Strict backend library/test Clippy passed.

The next compiler187 change only composes the existing component-product
relation with the existing component-Vector relation. The root element rule,
checked coercion, AST renderer, recording validator, runtime and ABI are intact.
Whole generated SDK execution, malformed ingress/source boundaries, model/proof
qualification and independent reviews remain open; this is not slice acceptance.

## Original compiler187 and native-stage harness

The unchanged original recipe built from signed `dbfe7eb5d` in 175.17 seconds.
The maintained ledger documentation owner regenerated its full output with only
the compiler version line changed; the literal workflow version check passed.
Scheme/controller hashes and compiler input pins were retained before execution.

The separate maintained native-stage entrypoint is
`tools/compact-rust-backend/check_component_product_vector_widening.py`.
It keeps both whole original sources and the full fourteen-export family,
explicit independent TS input/output type descriptors, twenty-six declared
cases and four Rust execution routes with exact typed argument FAB binding.
It requires unedited whole SDK strict lint, exact independent IR, all sixteen
Program/capability rows, actual helper/assert/cast/witness errors, once-only
witness observation and successful VM replay/state/effects/gas comparisons.
This describes the gate's intended scope, not an executed result. Marked static,
malformed IR/recording/source/producer controls, all-model inventory, applicable
fresh proof/verification/strict-transaction gates and independent reviews remain
required before local slice acceptance. Full CI/Linux/integration remain open.

## First native-stage attempt retained

The first clean `03dd17807` native-stage run accepts all three whole generated
SDK roots under strict lint and exact independent IR, then stops in fresh TS
argument encoding at the deeper-alternation case. Its JSON declaration had one
surplus list below the component product; the unchanged Compact source and
Rust typed constructor agree. Preserve that failed epoch and repair only the
explicit test value. The TS descriptor now independently asserts every Vector
dimension and tuple arity before encoding, retaining full raw FAB/source binding.
No successful TS/native/proof verdict is inferred from this failed run.

Actual independent Codex CLI source review of production `dbfe7eb5d`, after the
retained authentic Claude session limit, reports no actionable correctness
findings. It covers classifier source and existing-owner characterization only;
it does not accept the new native-stage harness or execution/proof receipts.

## Native stage passed; negative controls now required

Fresh `a6a48fe3a` original compiler187 recipe recording passed (7.25s), followed
by the complete native stage (18.92s): three whole SDKs under strict Clippy,
exact independent IR, sixteen Programs/capability rows, twenty-six fresh TS
cases and 104 Rust routes. Seven genuine error cases produce 28 expected errors;
nineteen success cases produce 38 public-transcript/state/effect VM replays.
Rust arguments are independently compared with source-typed TS FAB values.
The failed earlier deep-case declaration remains retained, not relabeled.

The extended gate adds twelve separately marked ordinary-use compile-fail
controls with valid twins, fourteen private whole-IR shape/bound/static/dead/
source controls with valid twins, all current source lanes and complete changed
helper/witness producers plus a corrupt-result control. These new negative
checks have not executed yet. Whole IR rendering refusal does not substitute
for independent recorded-plan forgery/resource or raw proof identity controls.

The two specifically implemented current refusal rows now point to their whole
native consumers. Both prior compiler186 scope tables are preserved byte-for-byte
under distinct historical filenames; every unrelated row remains unchanged.
The predecessor boundary gate handles these successful current lanes while
retaining exact diagnostics and no-output checks for actual refusals. No nominal,
narrowing or top-level nested tuple capability is admitted by metadata.

## Independent recorded-plan ingress controls

Three focused tests now pass against the entire raw fourteen-export source IR:
all fourteen roots keep independent checked-plan validation; fifteen forged
conversion annotations across ordinary, empty outer and empty leaf Vector roots
are rejected at their original conversion operation; noncanonical numeric text
and resource-excess annotations cannot hide behind a zero-length leaf. The
initial test asserted a lowercase diagnostic fragment while the actual owner
reported `Compact Uint`; its failed epoch is retained and the corrected test
requires the exact canonical-bound diagnostic. Strict backend library/test lint
also passes. No production recorded validator or resource limit changed.

The complete fourteen Program inventories and raw IR are retained as an exact
fixture, explicitly review-only metadata rather than execution authority. The
maintained gate compares all raw bytes and membership before using the fixture
and now executes these three independent validator tests. The previous clean
native/boundary gate passed 87 commands at `5e0c4694f`; current-source compiler
recipe and gate requalification are still required after the new test inputs.

A separate opt-in `component-product-vector-models` mode in the existing proof
runner observes every complete source/export and checks emitted capability and
ZKIR membership. It generates no keys/proofs and selects no exports. Observe
all sixteen models before deciding the applicable whole-source proof cohort;
retain any over-limit source intact. This mode is pending actual compilation
and execution. Raw proof identity, fresh proofs/verification/strict applications,
final source/receipt reviews and full CI/Linux/integration remain open.

## Bounded native/model closeout — 2026-11-03

Qualified source: `005a10603587e2e751f62be65c7ef72de415c6ae`.
Current original compiler187 build passed (227.38s), exact current recipe recording
passed (19.57s), and the whole native/negative/independent-recorded gate passed
88 commands (103.18s). These are local arm64 results, not hosted/Linux results.
The sixteen Program rows span two original whole contracts and the full
fourteen-export family; the raw fourteen family Program fixtures and source IR
are independently byte-bound. Program metadata remains review-only.

All sixteen intact source/export models were observed separately in 2.27s with
unchanged producer hashes and no keys, proofs or selection. Exact observed models
are retained in the fixture's `model-expectations.json`. The direct original and
family branch both require k9 and remain intact and unproved under the cap8.
The remaining fourteen exports are in cap: original_nested.run and all thirteen
other family exports. Observation does not establish proof verification.

Actual independent Codex CLI high source review (204.61s) and separate receipt
review (238.78s), after the retained authentic Claude session limit, found no
P1/P2/P3 contradictions. The receipt review recomputed source/artifact/tool hashes
and accepted only this finite native/model stage. Earlier failed and successful
epochs retain their original scope; no prior result is relabeled current.

The [next proof phase](component-product-vector-leaves-proofs.md) proposes all
fourteen in-cap exports, fresh direct keys/proofs/independent verification and
separately proved funded DEFAULT-strict applications with replay refusal. The
empty export's output has a unique empty carrier: another same-geometry result
is inapplicable, not a fabricated passing negative. All its other applicable
proof controls remain required. No new keys/proofs/transactions have run yet.
Constructor/all-input/wallet, full required CI/Linux/integration and broad-goal
acceptance remain open. Forge, canonical/protected refs and evidence stay intact;
no publication, branch merge, shared installation or cleanup is performed.

External execution/review evidence:
`/Users/ysh/.codex/local-evidence/rust-ast-component-product-vector-widening-20261103/native-model-stage-closeout-v1.json`.
