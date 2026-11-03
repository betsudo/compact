# Outer Vector product widening: baseline prerequisite and next semantic gap

Status (2026-11-03): admitted from the accepted bounded component Vector proof
stage f2fef3e7a / documentation8a358edea. This branch currently characterizes a
baseline regression; no semantic widening has been implemented. The existing
whole original source remains at its original path. Forge is deferred and the
broader Rust AST goal is incomplete.

Branch: codex/rust-ast-outer-vector-product-widening.
Evidence: /Users/ysh/.codex/local-evidence/rust-ast-outer-vector-product-widening-20261103.

## Actual gap

Preserve the complete existing original source at
`tools/compact-rust-backend/tests/nested_component_products/unsupported/top_level_nested_tuple.compact`.
It returns `Vector<1, [[Uint<8>], Uint<8>]>` as
`Vector<1, [[Uint<16>], Uint<16>]>` after a source-defined CellWrite.
The fresh compiler187 protocol in proof-v2 preserves separate TS/Rust/IR scope
controls; the earlier proof-v1 current187 controls already record TS exit0 and
Rust/independent frontend IR exit255. The diagnostic mentions Cell.write; that
text alone is not a causal explanation.

Source inspection identifies a concrete classifier boundary: element-widening?
permits scalar or component-Vector conversions inside an outer product but does
not recurse into a product child there. component-product-widening? already
recurses through exact product/Vector shapes below component Vectors. This
candidate concerns the missing outer-element composition, not already accepted
Vector leaves below component products.

## Owners and public API

- Scheme frontend `compiler/rust-ir-passes.ss::vector-value-widening?` owns
  ordered transport of frontend-supplied conversions and source diagnostics.
- `pure_semantics/coercion.rs` owns the bounded checked coercion plan. Characterize
  existing recursion first; do not assume a new operation is necessary.
- `value_lowering/coercion.rs` emits structured Rust AST from checked facts.
- `recorded/effect_plan/validate.rs` independently validates original operations,
  declarations, aliases, dead arms and static-empty child types.
- Existing generated ordinary/recorded/functional/observed facades retain ownership.

The original generated carrier should remain
`FixedVector<((BoundedUint<255>,), BoundedUint<255>), 1>` to
`FixedVector<((BoundedUint<65535>,), BoundedUint<65535>), 1>` with the existing
Result/CompactError boundary, typed witness/context and consuming Program APIs.
No dynamic carrier, public conversion trait, new compiler option or SDK rewrite.

## Baseline before implementation

From a clean preserved accepted local predecessor, qualify the original whole
source and separately declared complete pure and same-type controls. Compile TS,
Rust and independent IR with the original maintained compiler recipe. Lint each
complete SDK at its own crate root. Execute ordinary native/recorded and functional
native/recorded routes against current independent TS captures. Close any actual
same-type/ordinary baseline regression before changing widening selection.

Declare the complete application/export roster before semantic implementation.
Include direct/helper/local returns, checked-helper failure precedence, witness
before/after writes and failures, nonuniform first/last values, aliases, exact
nested tuple arity, deeper tuple/Vector alternation, Boolean/Field identity,
Uint-to-Field where already legal, empty/single tuples and empty Vector children.
Do not trim the original or select an easier replacement contract.

## Acceptance

1. Original whole source and every declared application export generate exact
   typed APIs and execute all four routes. Bind actual Rust input/result FAB to
   independent current TS, plus private state/outputs, public transcript, ledger
   state/effects/gas, VM replay and first-error/witness once-only ordering.
2. Marked ordinary API misuses reject wrong nested leaf bounds, tuple arities,
   Vector dimensions, result carriers and consumed Program reuse. Matching valid
   twins must compile and execute. Strict whole-SDK and consumer lint stays on.
3. Independently reject forged IR and recorded conversion annotations, wrong
   alias/identity/source/helper binding, narrowing, malformed dead/empty child
   declarations and resource excess. Preserve old diagnostic tables literally;
   reclassify only genuinely implemented current rows.
4. Observe all complete source/export models before selecting proofs. Keep k<=8
   and all source/geometry intact. For every applicable in-cap export, generate
   fresh direct keys and genuine proofs with independent verification and actual
   corrupt/binding/result-preimage/CellWrite/wrong-immutable-verifier controls.
   State inapplicable empty-carrier result controls explicitly. Separately prove
   funded DEFAULT-strict transactions and unchanged-state replay refusal. Retain
   over-cap exports intact and unproved; never extract or normalize a circuit.
5. Bind exact original compiler recipe, source/backend/SDK/dependency/tool/runtime/
   serialization/material/parameter/verifier identities. Obtain authentic fresh
   source and separate receipt reviews; retain the complete required CI/proof gates.
   Local arm64 evidence does not establish native Linux or full integration.

## Exclusions

No nominal Struct/new identity carriers, narrowing, changed dimensions/tuple
arities, dynamic shapes, runtime/schema/ABI change, broad refactor, framework,
new recorder/profile, CI optimization project, Forge work, remote publication,
merge, shared installation or cleanup. This plan is a bounded continuation of
semantic/API fidelity, not a claim that all remaining capabilities are known.

## Actual compiler187 baseline characterization

Four complete source roles were declared outside the checkout before execution:
the untouched original counterexample, a same-type stateful contract, pure
same-type and pure widening contracts. Checked-in control copies and sources.json
now preserve their exact bytes and roles; the first observations retain their
external physical paths and8a source epoch, not a new checked-in-path execution.
All controls keep their complete single exported run, and are not trimmed or
extracted replacements for the original.

The first controller passed the current original compiler187 recipe, observed
original TS0/Rust255/independentIR255, and same-type TS/Rust/IR0 with identical
Rust/independent IR. Strict whole same-type SDK all-target/all-feature Clippy then
failed at eight distinct generated execution signatures. The failed20.77s epoch
is retained. A separate remaining-controls run (5.00s) passed both complete pure
SDKs through TS/Rust/IR and strict own-root lint. Pure widening is already admitted
and is not a new capability to implement. No runtime/proof claim follows from
these compile observations.

## Prerequisite: signature metadata, before semantic widening

CompactSignatureLint in tools/compact-rust-backend/src/lib.rs owns generated
signature-local metadata. Its current predicates recognize wide products,
nested Vectors and Vector components; they miss direct nonempty tuple children
in the outer Vector product. The observed carrier fails only when nested inside
execution result wrappers. Preserve the already-clean pure SDK bytes by adding
only this case for Compact CircuitResult, RecordedCircuitResult, RecordedCall
and the generated interpreter E::Outcome return wrappers. Keep every previous
signature test body, exact public types, bodies, ownership and attributes.
Do not add crate-wide allowances, erase carriers or change coercion admission.

Before proceeding, execute focused AST controls for free/impl/trait methods,
first/last nested tuple components, pure/direct/foreign exclusions and exact
post-attribute-removal identity. Build the next maintained compiler with its
owner-generated version documentation. On the same physical baseline sources,
require strict complete SDK lint and preserve all raw IR/Program/ZKIR and carrier
geometry; only declared method attributes and compiler metadata may differ.
Execute current independent TS/typed four-route/VM/static consumer checks and
fresh source/receipt review. Original widening must still refuse before any
separate semantic classifier change. Retain both failed and passing epochs.

Required CI/Linux/integration remain open even after a bounded local metadata
repair. No new proof is inferred from unchanged proof inputs or a metadata fix.

## Focused metadata development checkpoint

Four signature-owner AST tests now pass, including both complete unchanged prior
test bodies and two new methods covering free/impl/trait execution signatures,
first/last tuple children, exact post-attribute-removal AST identity and pure,
direct, empty-child and foreign exclusions. Strict backend lib/test-code Clippy
passes. The repair requires a Compact execution result wrapper (or the generated
E::Outcome interpreter) before marking this new carrier shape. It changes no
frontend selector, checked operation, AST body, public carrier or runtime API.

The next maintained compiler label is188. The generated ledger document is still
187 until its owner runs after the clean signed original build; this is a
transient, unqualified version mismatch. Generated SDK lint, exact raw-input/
AST comparison, runtime/static consumers, owner documentation/version gate and
independent reviews remain required. No repaired SDK or semantic admission is
claimed from focused unit/lint evidence alone.

## Original compiler188 and documentation owner

The clean signed e05ab source builds through the original maintained .#compactc
recipe from a frozen checkout in188.58s. Both executable identities and full
source pins are retained. The actual independent Codex medium source review
(59.38s, after retained authentic Claude weekly quota) found no actionable issues;
it does not qualify generated SDK behavior.

The maintained ledger.ss owner then regenerated the complete raw ledger document
in50.71s from an isolated source/object copy. Its only output change is the
compiler187-to188 version sentence. The literal existing workflow version step
passed in12.09s. The earlier transient version mismatch is resolved by this
owner output, without manual generated-document editing. Complete generated SDK
lint, same-physical-source raw/attribute continuity, typed/runtime/static checks,
maintained regression wiring and independent receipt review remain open.

## Compiler188 complete baseline continuity and maintained gate

The exact same physical four-source controls pass under original compiler188
at clean3098177ae. The original widening remains TS-positive/Rust-and-IR-refused.
All three admitted complete SDKs pass strict own-root Clippy. Both pure SDK Rust
outputs are byte-identical to187; the same-type stateful Rust delta consists only
of eight inserted reasoned method attributes. Whole raw IR, checked Programs,
capability geometry, Cargo and ZKIR bytes/membership remain identical. Only
declared compiler-version metadata and consequent manifest hashes differ. The
25.76s receipt retains all actual inputs, output comparisons and diagnostics.
These observations do not establish runtime or proof acceptance.

Extend the existing required nested-product request-field gate with this complete
same-type stateful source. Retain every previous fixture byte and all prior checks.
The fourth typed consumer uses nonuniform/boundary values, exercises all four
ordinary/recorded/functional routes, compares result/private state/public writes/
effects/gas and executes both VM replays. Five marked refusals cover nested leaf
bounds, tuple arity, outer dimension, tuple-versus-Vector kind and consumed Program
reuse, with matching valid twins. The planned complete gate has64 commands, four
consumer methods,16 routes and8 VM replays. This metadata prerequisite gate makes
no new TS-runtime, proof, semantic widening or full integration claim.

## Maintained execution and source-review correction

At signed0764e2264 the complete gate passed64 commands in61.58s: four SDK/consumer
roots,16 typed execution routes,8 actual VM replays and18 marked refusal/positive
pairs. Current original188 recipe qualification took13.91s. All15 workflow-owner
and31 source-registry tests passed. No source/runtime/proof boundary was widened.
The fresh independent Codex medium source review (78.93s) confirmed preserved
fixtures, test bodies, carrier fidelity and gate accounting. It found one P2:
the top CHANGELOG entry still named187 despite compiler/flake/document188. Add
the missing188 source-checkpoint entry and execute the literal existing workflow
version checks. This review finding is retained; it is not a clean final review.
Separate receipt review and corrected source review remain required.
