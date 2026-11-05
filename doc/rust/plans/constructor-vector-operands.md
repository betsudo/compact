# Constructor fixed-Vector iterable expressions

Status: source-characterized proposal; implementation and execution qualification
are not accepted. Local continuation of the primary Rust AST goal, 2026-11-05.

## Proven gap and baseline

At exact source `8d100a1b584bbcef1c9de0a0eaa555eb4fd839c9`, the original
compiler 0.31.191 admits all nine complete characterization sources for TS.
Independent Scheme IR and the Rust/functional target admit the parameter control
and refuse the other eight sources: a local alias, local literal, local helper,
direct helper, local witness, direct witness, conditional and empty local Vector.
The first five local forms receive “Vector iterable must be a constructor
parameter”; direct expression forms receive “needs a literal or parameter Vector
iterable”. These are source refusals, not yet observed runtime differences.

The existing `ConstructorStep::ForEachVector` has an expression source, but its
frontend accepts only constructor parameters, and its native VM owner checks
`Expr::Parameter` explicitly. A local alias can use the existing VM parameter
scope once frontend admission is fixed; direct expressions need checked native
operand preparation as well. Merely changing the frontend guard is insufficient.

Focused clean baseline: 60 constructor-related unit methods and nine render
methods pass with Rust 1.99, unchanged limits and the existing Cargo target.
PR #701 has separately passed Rust required checks, both compiler producers and
the macOS consumer; its full Linux consumer remains running. Do not interpret
these local checks as full CI. Preserve the published proof repair and the
separate reviewed local proof-CI successor `58c1e5083`.

A possible Counter operand gap was independently eliminated: all 27 source
admission commands for nine Counter cases pass. Existing normalization hoists
those expressions into checked bindings. Do not change CounterAmount or reopen
that accepted capability. #674 is already integrated by #688 and is not reopened.

## Public API and affected owners

Keep the generated `ledger_contract::initial_state(ConstructorContext<Private>,
...) -> Result<ConstructorResult<Private>, CompactError>` signature and existing
`FixedVector<T,N>` inputs, typed witnesses and consuming context ownership.
Ordinary/recorded and functional native/recorded calls after initialization retain
their current APIs. This does not introduce a functional Constructor Program or
a new public indexing/iteration trait.

| Owner | Bounded responsibility |
| --- | --- |
| `compiler/rust-ir-passes.ss`, constructor Fold transport | Admit one fixed-Vector expression source in lexical scope; preserve the inline Unit reducer, original operand and source order. Keep the existing literal transport and serialized forms. |
| Backend `ir.rs` | Reuse `ForEachVector { binding, source, length, steps }`. No new IR schema or runtime ABI is planned. Any discovered decoder obligation needs an explicit decision first. |
| Backend `lib.rs::render_constructor_vm_steps_observed`, `constructor_vm` | Check exact Vector element/length; prepare a nonparameter operand once through the existing stateful expression owner; emit structured Rust AST. Retain the existing borrowed parameter loop and its output unchanged. |
| Backend `stateful::render_state_expression_observed` | Reuse checked query/witness/call/conversion preparation, lexical scope and context transfer. Do not duplicate semantics or add a whole-contract recognizer. |
| Constructor witness/VM prepasses, origins and resource visitors | Preserve original-child identity, first error, static branch/body checks, expanded work and unchanged limits. Evaluate a zero-length iterable once even though its body never executes. |
| Constructor unit/render controls and whole-source consumer gate | Add independent ingress mutations, real generated API compile-fail twins, TS comparisons and selected proof/application coverage. Historical authorities stay unchanged. |

Backend paths above are under `tools/compact-rust-backend/src`.

## Scope and design constraints

1. Cover the whole demonstrated single-Vector operand boundary: local aliases
   and values, helper and witness returns, conditional selection and empty values.
   Characterize ledger-read and owned-value operands before claiming their
   support. Do not fix only the easiest local-alias case and label the gap closed.
2. Reuse the existing typed source expression and constructor scope. The renderer
   must obtain checked operand/type/ownership facts before producing the loop.
   A fallible operand executes once at its source position and updates context
   before iteration. No per-iteration re-evaluation or branch speculation.
3. Preserve the old parameter/literal paths, borrowing and required public
   representation constraints. Retain a temporary result for the loop lifetime;
   avoid introducing whole-vector clones into the existing parameter route.
4. Wrong source kind, length, element, scope, declaration identity, layout and
   resource premises remain explicit refusals at untrusted IR boundaries.
   Static body obligations still apply to empty iterables and inactive branches.
5. Ordinary invalid generated API calls must fail Rust compilation. Runtime
   validation remains explicit for fallible witnesses, host state and raw IR.

## Acceptance

### Source and generated behavior

- Preserve all nine original complete counterexamples/control sources in retained
  evidence. Committed header-bearing or expanded proof fixtures have their own
  hashes and execution epoch; do not relabel the original failures.
- All intended source forms admit through current TS, independent IR and Rust,
  with exact complete SDK/Program membership, source maps and original geometry.
  Include sequential local bindings, shadowing/name hygiene and repeated reuse.
- Run unedited generated constructors and compare independently expected state,
  witness count/order/private state and output, errors and query cost with TS.
  Check both conditional choices, inactive branches, empty vectors, witness and
  helper failure, and failure before later writes. A witness must observe writes
  that precede its source call. Replay relevant later recorded entrypoints.
- Include one valid owned-element case with later value reuse and a temporary
  iterable; confirm it compiles without added Copy/Default requirements.
- Keep existing accepted constructor and ordinary circuit behavior unchanged.

### Static and ingress controls

- External generated consumers refuse wrong Vector length/element and an
  incompatible witness signature at the intended call/implementation. Correct
  type twins compile and execute. Keep consuming functional Program controls for
  the post-initialization entrypoint when that surface is exercised.
- Independent IR mutations refuse unbound/use-before-definition/leaked loop
  bindings, wrong source kind, false length/element annotations, invalid static
  children and unchanged resource-limit violations. Preserve meaningful original
  refusals; change only sentinels whose intended support boundary is extended.
- Altered complete IR/Program/source bindings fail before selected proof keys or
  ledger state. A type-correct plan is not a semantic-equivalence certificate;
  use expected state/order controls for same-type wrong operands and dropped work.

### Genuine proof and integration boundaries

Inventory complete generated models and proof applicability first. Extend an
explicit complete source with a post-initialization stateful entrypoint whose
statement depends on the constructor-produced state; declare this new source
identity. For the selected applicable cases, generate fresh direct keys and
proofs, independently verify them, and separately prove/apply a funded
DEFAULT-strict transaction with unchanged-ledger replay refusal. Bind exact
source, full IR/Programs, compiler/runner/runtime, parameters and immutable
verifier identities; keep changed binding/result/preimage, corruption and
wrong-verifier controls. No proof/verdict cache, source trimming or cap increase.

A proof of a later entrypoint does not prove constructor initialization itself.
Report constructor proof coverage and nonproof/over-limit models separately;
retain all original exports and complete sources. Full constructor cryptographic
admission, arbitrary multi-iterable/non-Unit constructor Fold, dynamic indexing,
new platforms/ledger versions and whole-language parity are not claimed here.

During iteration use focused arm64 checks. Complete required compiler, Rust,
proof, wallet/application, archive and host CI gates remain required for eventual
integration. The active goal keeps this work LOCAL: no push, merge, hosted trial,
release, shared installation or CI optimization project. Forge remains deferred
with all branches, stores and evidence retained. Clean up only after actual
verified GitHub merge under the owner's later applicable instructions.

## Execution order and evidence

Characterization -> focused clean baseline -> independent malformed/positive
controls -> paired frontend/checked operand lowering -> original compiler build
and current-source source/API/TS qualification -> model inventory -> selected
fresh proof/strict-application controls -> independent CLI source/receipt review.
A real baseline regression stops dependent implementation until repaired.

Retained evidence: `rust-ast-constructor-vector-operands-20261105`, including
`characterization-v1/receipt.json` and `characterization-assessment-v1.json`.
Original package/recipe identity is separately retained by
`rust-ast-counter-amount-expressions-20261105/original-recipe-v1/receipt.json`.
These local locators are navigation, not portable provenance or new acceptance.
