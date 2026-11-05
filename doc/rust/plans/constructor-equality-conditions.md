# Constructor equality conditions

Status: local implementation against accepted canonical `90cd758c`; current compiler source/API and proof qualification remain pending.
Unreleased toolchain `0.31.195` is reserved for this independent slice.

## Gap and evidence

Inside an existing parameter-Vector constructor loop, disclosed equality and
inequality conditions pass the current TypeScript compiler and independent Rust
IR producer, but native emission rejects the `if` initializer. A Boolean parameter
condition passes all three routes. Twelve complete three-export contracts were
characterized in 36 local commands before implementation. The separate frontend
refusal of top-level constructor `if` is excluded.

## Owners and public API

Restrict admission at `lib.rs` constructor `If`, then reuse `constructor_vm`'s
existing pure expression adapter, typed arithmetic fold and structured native AST
sink. Preserve checked recording and original refusal owners. Do not change the
shared initializer expression whitelist. Generated `initial_state` retains its
existing typed arguments, ownership and fallible result; no new public methods,
IR variants, runtime types or serialization formats are introduced.

## Scope

Admit trees of `==` and `!=` with operands already admitted as infallible
constructor expressions. Existing typed equality checks determine comparable
operand types. Cover Field, Boolean, Uint, Bytes and nested Boolean equality.
Keep helpers, witnesses, ledger reads and arithmetic operands refused. Do not
expand Cell/Map/Set initializer operands, top-level frontend actions, recursive
helpers, general operators, compiler optimizations or CI caching.

## Acceptance

- Typed unit and render checks cover equality/inequality, nested grouping,
  operand type mismatch, left-before-right child refusal, original diagnostic
  identity, and validation of both conditional arms.
- Existing constructor/resource limits remain effective, including deep raw IR;
  no optional/default-off bypass of required recording checks.
- Complete current compiler TS/native output is tested independently of retained
  compatibility fixtures. Both branch outcomes, final ledger values, Bytes
  ownership/reuse and relevant wrong-type/shape compile failures are exercised.
- Fresh selected later circuit proofs and independent verification, plus separate
  funded strict application/replay tests, qualify affected complete contracts
  where applicable. Preserve all sources, export geometry, assertions, refusal
  controls and the existing model cap. Constructor initialization itself is not
  cryptographically proved by a later circuit proof.
- Focused local checks guide iteration. Full required Linux CI, proof gates,
  independent source/receipt review and human review remain integration gates.

This slice remains local while the separate constructor proof CI owner is
qualified. No acceptance is inferred from the previous constructor cohort.

## Local owner checks

The implementation passes Rust 1.99 formatting, 1,889 backend unit tests
(seven existing ignored tests), all 155 render tests and library Clippy with
warnings denied. Focused coverage includes 63 constructor unit tests and nine
constructor render tests. These are owner checks, not current compiler SDK or
cryptographic qualification. The generated ledger documentation was regenerated
through its maintained Scheme owner for compiler `0.31.195`.
