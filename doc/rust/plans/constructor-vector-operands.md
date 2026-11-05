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


## Paired source candidate — 2026-11-05

The candidate admits lexical Vector operands in the frontend and emits direct
expressions through the existing stateful expression owner. A private
`constructor_vm/vector_source.rs` prepares and checks the source before the
borrowed loop. The old parameter path remains borrowed and unchanged; the new
expression result is bound once, including for length zero. No IR variant,
serialized member, runtime ABI or public executor is added.

An independent IR render regression fails on the original parameter-only guard
and passes with the candidate. The first test filter selected zero methods and
is not qualification. The first unit run caught changed priority for a malformed
Field literal. The known non-Vector literal/constructor guard is restored before
payload parsing; all 60 existing constructor unit methods pass, with an added
bad-Uint guard case. Eleven constructor render methods, strict all-target and
all-feature backend Clippy, and the maintained readability corpus pass locally.
These are focused source checks, not execution/proof acceptance of the new APIs.

Compiler version 0.31.192 is an unreleased source candidate. Its maintained
ledger documentation owner generates the one-line version change. The first
Nix setup attempt excluded an unstaged new module; the second caught an unbound
frontend source-location variable. Both failures remain retained. After staging
the module and fixing the source location, the third compiler-devShell setup and
documentation invocation pass. This dirty-candidate dependency build does not
replace clean-head original-recipe recording or current source qualification.

The maintained fixture roster is
`tools/compact-rust-backend/tests/constructor_vector_operands/sources.json`.
It keeps nine full header-bearing sources and adds `advance` plus a distinct
`advance_other` relation for later post-initialization proof controls. Their new
hashes/three-export membership are explicit; the original one-export sources and
failed receipts remain untouched. Source admission, generated runtime behavior,
compile-fail/refusal controls, model inventory, fresh selected proof/application
and independent review remain open. All new work stays local.


## Clean-source admission and first independent review

At candidate `d3e91d0e7`, the clean original recipe realizes compiler192 package
`55gcpn0wdrvnsy2rsmx3q24mip2mw64f` and the maintained recorder accepts it.
The 8.76-second realization reuses the package built during the preceding
compiler-devShell setup; it is not another compiler rebuild. All 27 source
commands pass for the nine complete three-export fixtures. All nine Rust IR
and raw-origin outputs equal the independent Scheme outputs. The parameter
constructor function remains byte-identical to the original191 control.
This establishes admission, not generated consumer compilation or runtime/proof
acceptance.

Independent source review found one P2: a known non-Vector `Default` could parse
an invalid Uint maximum before the preserved constructor-kind refusal. A new
independent guard case reproduces it. The correction rejects non-Vector defaults
before lowering and retains the valid Vector-default path, with a separate
positive/shape control. Focused constructor checks now pass 60 unit and twelve
render methods; old controls remain present. Fresh corrected-head recipe,
source admission and review are required; `d3e91d0e7` is not silently relabelled.
Generated API behavior/compile-fail, owned and fallible operand ordering, broader
ingress controls and fresh proof/application acceptance remain open.


## Generated-SDK ownership correction

The corrected guard source `8fc0100d3` passes its clean original compiler build,
recipe recording, all 27 complete-source admission commands and independent
source review. The next gate compiles entire unedited generated SDKs with strict
all-target/all-feature Clippy. It found a real `E0277` in the conditional fixture:
the borrowed loop item `&Field` reached a Cell write requiring an owned Field.
Admission and rendering tests alone did not establish generated Rust validity.

Keep borrowed iteration of the whole Vector, but introduce a typed per-element
value before rendering the body: copy Copy elements and clone owned elements
through the existing ownership owner. This also corrects the same latent problem
for the old parameter route. Therefore its earlier byte-identical constructor
observation stays specific to the earlier epoch; byte identity is superseded by
this explicit correctness repair. The whole input Vector is not cloned or moved.
The caller can reuse it; owned element support gains no Copy requirement.
A focused AST regression covers Copy and owned bindings, while complete untouched
SDK compilation remains the controlling compiler check. SDK runtime, misuse,
model and genuine proof/application qualification remain open.


## Maintained whole-source gate candidate

At clean `1320b9855`, the original compiler, 27 complete-source admissions,
nine unedited SDK strict Clippy checks, external Rust and TS behavior, typed
misuse and independent source/receipt reviews pass within their stated scopes.
Receipt review found that early replay consumers compared effects only. The
corrected consumers compare full replayed state and independently expected typed
Cell values; a genuine changed-expectation assertion fails. All old failures and
reviews remain retained. None of these external checks is relabelled as a
maintained source gate or a proof of constructor initialization.

Four additional complete sources admit and compile as untouched SDKs at that
same compiler epoch: a non-Copy String Vector temporary with later input reuse,
a conditional witness iterable, a zero-length witness iterable and a fallible
helper followed by a witness. They retain all three post-initialization exports.
The maintained fixture now has 13 complete sources and 22 cases (15 successful
constructors and seven ordered errors). These are new source identities; the
original nine files and original one-export counterexamples remain intact.

The new `check_constructor_vector_operands.py` candidate checks current original
recipe and compiler identity, full manifests, all three models/Programs per
source, independent IR and raw origins, unchanged bundled runtime sources and
SDK Clippy, actual TS and four later Rust APIs. It compares complete replay state,
effects, typed results, private state and constructor witness prefixes; it also
compares native later-call gas across the four Rust routes. Five ordinary misuse
controls require executing correct twins and exact marked primary diagnostics.
Eight independent IR mutations retain scope/type/static branch/body and resource
refusals. Three complete source mutations change owned projection, helper failure
and conditional witness selection without editing generated Rust. A separate
wrong replay expectation must fail the actual consumer assertion.

A first resource probe used Vector length 131073 and was admitted: native borrowed
iteration does not statically unroll the loop. This is not a resource defect and
no new length restriction is introduced. The maintained control instead tests
actual nested source depth against the unchanged syntax-depth limit of 48.

The gate candidate has not yet executed from its own clean source epoch. Its
current source and receipt review, original compiler recording and complete gate
run are required next. It creates no keys/proofs and makes no Linux or integration
claim. `ConstructorResult` exposes state/private/Zswap results without cumulative
constructor gas. Constructor total-cost acceptance therefore remains open; later
Rust route-cost equality cannot substitute for it. Fresh applicable entrypoint
proofs, independent verification, separate funded strict applications, immutable
identity negatives and full integration remain required. Forge stays deferred.
