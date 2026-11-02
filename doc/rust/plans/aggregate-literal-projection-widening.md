# Next bounded proposal: numeric projection elements in typed aggregate literals

Candidate Q is admitted on `codex/rust-ast-aggregate-literal-projection-widening`
from the exact clean P docs head `3d94b719635df5424e065d8593d38395eecb8180`.
No implementation is accepted yet. Primary Rust AST
goal ACTIVE / INCOMPLETE, LOCAL ONLY; Forge deferred/incomplete.

## Actual gap and accepted behavior

`next-indexed-scalar-characterization-v1/receipt.json` has6 complete sources /
36 original144+current165 TS/Rust/independentIR commands. Scalar Uint, Field
and wide projection returns, explicit casts and equal-bound controls already
succeed in all lanes. Do NOT reopen scalar projection support. Only the direct
indexed values placed into a wider aggregate literal fail Rust after frontend
success. The original changed-producer failure is retained as the discovery,
not converted into a passing negative control.

`next-literal-elements-characterization-v1/receipt.json` refines this with8
complete sources /48 original144+current165 commands. TS and independentIR
accept four cases that Rust rejects under BOTH compilers:

- Vector<2,Uint<8>> indexed values in Vector<2,Uint<16>> literal;
- the same narrow values in Vector<2,Field> literal;
- Vector<2,Uint<129>> indexed values in Vector<2,Uint<248>> literal;
- tuple [Uint<8>,Uint<16>] projections in [Uint<16>,Uint<32>] literal.

Equal-bound literals, explicit per-element casts, and narrow variable elements
in a wider Vector literal already succeed in all lanes. A wrong result length
refuses at the same frontend boundary in all lanes. Literal IR currently
labels elements with the target type while leaving each tuple_index at its
original narrower actual type. This is a missing expected-context conversion,
not a new numeric conversion relation or an unsupported scalar result.

## Baseline and owners

First read CURRENT checkpoint/refs and finish P receipt/docs closeout. Canonical
remains codex/rust-backend-ast at1bdf185ea; P3cd and O03a114 are separate local
candidates, not integrated canonical. Preserve all earlier refs/qualification.
Admit one separate local branch from an exact clean recorded baseline. Prefer
an independent canonical baseline if the actual owner needs no candidate code;
otherwise record an explicit shallow-stack dependency rather than inventing
an integrated baseline. Current observations hold on original144 and165.

Existing roadmap/crosswalk and checked-recording/capability ownership keep
frontend typing, typed IR validation, checked pure-call certification and Rust
syntax emission distinct. Proposed changed owner: pure typed aggregate literal
element lowering in compiler/rust-ir-passes.ss. Reuse existing native Coerce
planning/rendering and signature guards, including ADR0330 bounded/fallible
aggregate behavior. Do not infer an actual projection type from the desired
target or erase its original bounds. No runtime ABI, IR variant, framework,
operation family, executor or dependency is proposed.

## Intended API, scope and exclusions

Retain original typed narrow Vector/tuple inputs and expose the declared wider
FixedVector<BoundedUint<65535>,2>, FixedVector<Field,2>, appropriate WideUint,
or Rust tuple output through ordinary existing pure methods. Where stateful
exports call the pure helper, retain the ordinary/recorded/functional APIs.

Scope: valid numeric projection elements of typed Vector/tuple literals and
cohesive nested/alias contexts needed by those actual cases. Preserve already
accepted standalone scalar projection, variable-element, explicit-element and
identity literal behavior. Characterize more cases before expanding scope.
Exclusions: arbitrary expression-context normalization, nominal redesign,
struct/member projection expansion without a concrete admitted case, new
Map/Fold semantics, stateful whole-body casting, broad refactoring, CI
optimization, Forge and publication.

## Acceptance

1. Focused clean actual baseline and accepted-owner reconciliation before
   source edits. Keep complete independent original/current source evidence.
2. Current independent TS expected values/alignment and unedited generated Rust
   must execute each admitted narrow/wide/Field/product/nested case, with
   source-once and first-error/left-to-right controls where source assertions
   make those properties observable. Retain exact existing scalar/control
   outputs and historical fixtures; do not normalize producers or trim source.
3. Positive ordinary typed twins compile; wrong input/output length, element,
   numeric maximum and consuming Program reuse refuse at the marked Rust
   expression where applicable. Source dimension, kind, implicit narrowing,
   nominal and out-of-bounds/zero-length projection refusals retain their owner.
4. Malformed untrusted IR must reject wrong element conversions and dimensions
   under the existing native validators. Complete declaration/callee/static
   obligations cannot be skipped by a constant or empty route. Changed full
   producers and same-geometry expected results must fail actual consumers.
5. Bind exact complete source/imports, independent/goldenIR, all Programs,
   current original compiler/recipe, serialization, Rust tools and full SDK;
   strict root Clippy and compiler/wrapper substitution guards stay required.
6. Inspect all models for full admitted stateful helper sources; verify locked
   material before fresh direct keys. Under unchanged automatic k<=8, require
   selected genuine proof/immutable verification plus separately independently
   proved funded default-strict applications, with corrupted/binding/result/
   different-verifier/replay controls. Disclose selected coverage and any
   over-cap original obligation; never trim, raise the cap or substitute proofs.
7. Signed Conventional DCO/Assisted-by commits; authentic independent CLI source
   and receipt reviews (Claude first, actual-limit Codex different-model
   fallback); documentation, verified Obsidian process notes and checkpoint CAS.
   Focused local iteration; complete required arm64/Linux CI and reviewed
   integration remain required for later integration. No cleanup before verified
   GitHub merge of completed owned work.


## Admission decision

Use an explicit local shallow-stack on P/O. The literal slice must preserve and
exercise the already accepted Vector alias expected carriers and source-alias
conversions in cohesive nested/alias contexts. Those existing owners are in the
held candidates, not canonical. Reuse them without duplicating their fixes or
claiming they are integrated. Canonical remains clean1bdf185ea and all earlier
refs/evidence are retained. A bounded literal fix and current local tests do
not replace full CI or reviewed integration.

First establish a focused clean baseline and reconcile actual literal-cast
owner metadata before implementation. In particular, retain the actual source
type supplied by frontend cast analysis rather than guessing projection types
from their desired target. Standalone scalar projections are accepted controls.


## Baseline and authoritative source-type decision

At signed plan44d972b46 the focused20 coercion/Fold/historical Vector checks
pass. Actual --trace-passes output under original165 proves infer-types retains
`safe-cast(target Vector2Uint16, source Tuple<Uint8,Uint8>, literal projections)`
through emit-rust-ir. The source element types already exist; no inferred-type
registry, target-derived bounds or new conversion relation is needed.

Expanded25-source/75-command original165 characterization distinguishes21
TS-valid sources (16Rust mismatches and5acceptedcontrols) from4frontend refusals.
The first lexical local declaration used `>>=input`, parsed as >=; its failed
receipt is retained and v2 corrects only whitespace. Empty mixed literals are
included to preserve existing Unit/empty-Vector construction without inventing
a Unit-to-Vector Coerce relation. Standalone scalar projections remain controls.

The implementation carries an optional actual source type from existing pure
Vector/Tuple safe casts through the existing typed literal/lexical/conditional
owners. Each actual literal argument receives its exact inferred source type;
only actual source syntax is expanded, never a declared Vector length product.
Projected values and complete explicit/intermediate checked casts use existing
Coerce when source and target differ. Identity contexts return original IR;
other operation families retain their existing owners. Conditional outer Coerce,
original explicit narrowing guards, complete source calls and empty/nested
literal construction stay intact. Compiler166 metadata identifies this source;
no ABI/IR/dependency/runtime/executor change is added. Actual current compiler,
source/API, proof and review acceptance remain open.


## First current compiler and maintained corpus

Default original166 at clean signed2ffa957f8 builds without substitutions.
All25-source/75-command characterization outcomes now match intent:21TS-valid
sources emit Rust and independentIR;4frontend refusals remain. All20original
IR/raw/Rust/capability control assets across5sources are byte-identical to165,
including standalone scalar, explicit, variable-element and nested identity
controls. Empty mixed literals and intermediate/checked narrowing casts emit
without adding a Unit-to-Vector conversion relation.

The maintained gate is
`tools/compact-rust-backend/check_aggregate_literal_projection_widening.py`.
Its21complete repository sources and31independent cases exercise29pure calls
and8stateful route executions. Distinct first/second source helpers expose
first-error order; narrowing controls distinguish the original checked Uint8
bound and actual projection order. Source/entry-specific TS descriptors cover
Uint16/Uint32/Field/wide/nested/product/scalar and empty-mixed output geometry.
Two helper exports return different genuine carriers: Vector<2,Uint16> and
Vector<2,Field>; qualification must not normalize or conflate them.

Five ordinary Rust refusals include a Field-result assignment mismatch beside
length/element/maximum/Program-reuse controls. Four frontend refusals retain
length/kind/implicit narrowing/zero-index causes. Five malformed nativeIR
controls include wrong geometry/conversion/index, zero-sized input projection,
and an invalid unselected branch under a true condition. Complete producer
mutations and same-geometry expected changes must fail actual consumers. Full
SDK strict roots, exact source/independentIR/allPrograms and tool bindings are
required. Development generation alone is not current clean-head API or proof
acceptance. All earlier matrices, lexical diagnostics and predecessors remain
retained. Full CI/integration and the broader goal remain open.


## Initial gate and independent review repairs

At signed2f6 the full historical fixture freshness check,1830backend tests
(7pre-existing ignored) and strict backend Clippy pass. The current source/API
gate executes all31consumer cases,21strict unedited SDK roots,5Rust refusals,
4source refusals and complete producer/result mutations, then stops at an
incorrect new diagnostic assertion. Vector operands correctly report vector
index2outside length2 and index1outside length0; expecting tuple-index wording
was a test error. Retain the mutations and require those exact native diagnostics.
The invalid unselected conditional arm also independently rejects the actual
Unsigned255-to-Boolean conversion even when the condition is true.

Actual independent Codex CLI review of2f6 (gpt-6-astra/high, after actual
Claude quota) identifies that diagnostic issue and unrecorded Cargo lint/compiler
overrides. Reuse the established cargo_configuration admission owner; capture
and recheck config membership/bytes, reject build/target lint flags, wrappers
and unchecked profiles. Focused controls exercise suppression, wrappers and a
new config file. No source/proof binding or native validator is relaxed. The
failed original run remains evidence; clean successor qualification is required.
