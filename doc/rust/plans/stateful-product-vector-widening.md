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
No Forge work, publication, shared install or cleanup. This remains a proposal;
implementation has not started.


## Admission

V is separately admitted on `codex/rust-ast-stateful-product-vector-widening`
from reviewed U docs `f10393353065c8d9af142594181ed5b1b0802368`. Exact current
checkpoint, refs/worktrees and clean canonical/U states were inspected first.
All predecessor work and evidence are preserved. Focused baseline and actual
product-cast traces precede implementation; no V compiler/API/model/proof
acceptance is claimed. Existing pure coercion, typed native/recording validators,
AST owners and original retirement/crosswalk authorities stay intact.
