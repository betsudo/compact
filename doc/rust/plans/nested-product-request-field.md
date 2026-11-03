# Nested product request field: baseline repair before widening

Status: current183 baseline defect reproduced; narrow source repair and tests
in development. No current184 compiler/SDK qualification or independent review
is claimed yet. The primary Rust AST goal remains incomplete; Forge is deferred.

## Baseline and concrete gap

At clean signed `0810a3175`, the complete historical
`nested_product_below_component.compact` source (SHA256
`33c091f6cc2907b05c08156a129a067be8ae15ad062c2c61473538e51a8177c7`)
passes TypeScript emission and refuses Rust/independent IR at Cell.write. Its
nested aggregate widening remains a real semantic gap, not accepted work.

A separately declared same-type control changes only its four return Uint16
leaves to Uint8; its source body and input shape are unchanged. All three
emission lanes pass. Actual all-target/all-feature generated-crate strict Clippy
then rejects the immutable Program's private `input` field for type complexity.
This is an existing baseline defect before widening, so it is repaired first.
The maintained complete control is `examples/rust_backend/nested_product_request_field.compact`.

The first external harness attempted locked Cargo before creating its lock;
that failed before compilation. An offline lock-generation successor exposes
the real field diagnostic. Both failed attempts and original outputs are retained;
neither is relabelled as a passing complete baseline.

## Owner, API and bounded repair

`recorded/effect_plan/functional_items.rs::request_field` owns field-local
metadata; the existing signature AST owner already handles method signatures.
Extend the existing reasoned `clippy::type_complexity` attribute only to the
characterized tuple child below a component Vector. Keep the complete nested
`FixedVector<(..., ...), N>` / `BoundedUint<MAX>` carrier and private visibility.
Do not introduce public aliases, erase types, suppress crate-wide lints or change
constructor, Program, context, native/recorded execution or checked conversion.

The public API is unchanged: typed `Program::new(input)` and consuming
`execute_native` / `execute_recorded`. Ordinary wrong bounds, tuple arities and
Vector dimensions must still fail Rust type checking. Existing request/context
ownership and untrusted IR validators remain their current owners.

## Acceptance for this prerequisite

1. Preserve the complete historical widening refusal and its literal authority;
   do not reclassify it until a separate semantic slice actually qualifies it.
2. Add exact AST regression cases for nested product children in either tuple
   position. Removing the one style attribute must recover the identical field
   AST, including type and visibility. Keep prior unmarked/foreign carrier tests.
3. Run focused request-field and existing signature-owner tests and formatting.
   Build the original maintained compiler recipe from a clean signed source,
   then generate the complete control SDK without handwritten output edits.
4. Run locked all-target/all-feature strict Clippy on that SDK; exercise the
   existing four typed routes with asymmetric values, state and recorded replay.
   Correct-type twins must compile; wrong leaf bound, arity and dimension must
   independently fail at marked ordinary-use sites.
5. Source change is metadata/version only: compare generated output against183
   after removing only the declared field attribute and compiler-version metadata;
   do not normalize IR/Programs/proof material. Applicable proof receipts retain
   their literal epochs; no new proof acceptance is inferred from this repair.
6. Obtain fresh authorized Claude CLI review (Codex fallback only on actual
   Claude quota), record local evidence and concurrent-safe knowledge/checkpoint
   updates. Complete required CI/Linux and integration remain open and local-only
   authority remains controlling.

## Following semantic slice, after this baseline passes

Gap: frontend-supplied widening through nested scalar tuple subtrees below a
component Vector. The current scalar-product classifier stops at one product.
Trace actual supplied casts at `compiler/rust-ir-passes.ss`; the existing checked
Coerce, independent validator and structured Rust AST must remain authorities.

Proposed API keeps all tuple arities and Vector dimensions and widens only
already legal Uint leaves / Uint-to-Field, preserving Boolean identity. Inspect
alias, helper, local and witness evaluation boundaries before choosing the exact
structural relation. No Vector newly nested inside the product, nominal Struct,
narrowing, new identity carrier, broad refactor or source-specific admission.

Acceptance must include whole-source live TS and four-route Rust parity,
asymmetric/boundary/empty cases, exact action/witness/error order, independent
compile-fail twins and malformed-IR/dead-branch controls. Observe intact export
models before selecting fresh proofs. Require genuine proof/verify and a separate
funded DEFAULT-strict application/replay for an applicable intact k<=8 source;
retain any over-cap result intact without trimming or raising the automatic cap.
A metadata-only fix does not satisfy this semantic acceptance.

## Restart and evidence

Branch: `codex/rust-ast-nested-product-request-field`, based on SDK documentation
head `0810a3175`. Canonical, protected checkout and Forge refs remain untouched.
Evidence: `/Users/ysh/.codex/local-evidence/rust-ast-nested-component-product-20261103`.
`source-inventory-v1.json`, `baseline-v1.json` and `baseline-lock-repair-v1.json`
retain original183 characterization and both the harness and real lint failure.
No push, hosted trial, merge, release, shared installation or cleanup is part of
this local slice. Prior accepted SDK/application receipts are not reopened merely
because this prerequisite has a new compiler identity.
