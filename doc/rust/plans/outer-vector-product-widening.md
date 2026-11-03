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
