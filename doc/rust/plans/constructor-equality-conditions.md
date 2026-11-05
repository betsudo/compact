# Constructor equality conditions

Status: local follow-up to PR #705, reconciled by a signed forward merge. Current compiler source/API and proof qualification remain pending. The original accepted canonical baseline was `90cd758c`.
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

## Constructor owner reconciliation

The first independent195 run at `8ee538a8` admitted source emission and all
sixteen TypeScript observations, then failed native compilation with E0277:
a borrowed Field loop element was compared with an owned Field argument. Native
behavior and marked compile failures were not reached. Preserve that run.

PR #705 already owns the typed element normalization required at the existing
borrowed Vector loop boundary. Reuse that independently locally qualified owner
through forward merge `5ff482536`; do not introduce a second ownership framework.
Both complete test cohorts are retained. This makes equality a local follow-up
to #705; its hosted CI and human integration gates remain separate.

The independent source review also identified unbound Rust tools in the new
qualifier. Resolve actual Rust1.99 Cargo/rustc paths, verify versions and pin/recheck
their hashes before invocation. Repeat current original-recipe and whole-SDK
qualification after these changes; earlier source or proof epochs do not substitute.

The reconciled candidate passes 1,890 backend unit tests (seven existing ignored
controls), the complete render suite, formatting and library Clippy. A real
ForEachVector AST regression checks typed value normalization for borrowed Field,
Boolean, Uint and Bytes elements before comparison, complementing the complete
current generated-SDK consumer gate. Failed predecessor tests remain retained.

## Accepted source/API epoch and model continuation

At clean `71fe8d7db`, the current original compiler recipe and complete source/API
gate pass nineteen commands: eight complete SDK generations, sixteen independent
TypeScript/native cases with four later-call routes, four source refusals, three
marked E0308 refusals and a changed-reference actual assertion failure. Independent
Codex CLI source/API review accepts this bounded result after checking all 1,633
input pins, eighteen fixture hashes, complete SDK membership and 219 runtime files.
The sole fixture correction since `cac317bd` uses the existing charged-state
`get_ref()` accessor; all expectations and cases remain unchanged.

The next maintained local command observes all 24 intact models across the eight
complete sources, with the existing five membership refusals. A separate static
equality roster reuses the existing private model parser; the original constructor
14-source/42-model entry contract remains unchanged. Model k classification is not
proof eligibility or cryptographic acceptance. No equality keys, proofs, strict
applications, hosted CI or integration are accepted by this paragraph.
