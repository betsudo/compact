# Direct fixed-vector Fold in stateful expressions (#661)

Decision: 2026-10-26. The preceding #610 slice passes its maintained local gates
at signed `e14103c5d`; independent review/Linux integration remain separate.
This is the active implementation plan. The partial checked-core checkpoint
below does not complete this full source family. Forge remains deferred and
is not a prerequisite.

## Genuine gap and accepted control

The actual ordinary compiler at `e14103c5d` was used for nine Compact sources
and 27 TS/Rust/required-recording generation invocations:

| Source form | Actual current result |
| --- | --- |
| Named pure Fold directly in a stateful lexical expression | TS and native Rust compile; required recording refuses VectorFoldCall. |
| Zero-length direct Fold | Same recording gap; zero length does not establish body admission. |
| Named guarded pure reducer | Same direct-recording gap. |
| Certified pure helper containing Fold | TS, Rust and required recording all compile. This is accepted work. |
| Inline captured reducer | TS compiles; Rust refuses named-functions-only before IR admission. |
| Witness-valued seed and iterable | TS compiles; current native Rust reports unknown circuit `seed`. |
| Named stateful reducer | TS compiles; Rust refuses the selected stateful return form. |
| Named witness reducer | TS compiles; current native Rust reports unknown circuit `step`. |
| Multiple equal-length vector iterables | TS compiles; Rust refuses one-iterable-only before IR admission. |

Exact sources, diagnostics and output dispositions are retained under
`/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-fold-20261026/` in
`source-characterization-v1.json` and `source-characterization-v2.json`.
Generation success is not runtime or proof evidence.

`ts-operand-order-v2.json` executes the genuine TS wrapper with real witnesses:
the seed witness runs first, then the iterable witness, each once. A seed
failure prevents the iterable call; an iterable failure follows the successful
seed call. The result is 123 for a noncommutative decimal-accumulating reducer.
The retained v1 probe had assumed source-first evaluation and failed; it is a
failed hypothesis, not a compiler regression. Existing effect-free Fold syntax
has a documented source-first initialization shape. Do not transfer that order
to new stateful operands without actual TS evidence.

## Public API and semantic scope

Generate the normal typed Rust method, recorded/observed method and functional
consuming `Program` for the original source declaration. For example, a source
`run(seed: Field, values: Vector<2, Field>): Field` retains Field and fixed-vector
carriers; it does not gain an untyped reducer callback in its public Rust API.
Witness reducers use the generated typed `TryWitnesses` surface. Wrong item,
accumulator or fixed-vector types must fail ordinary Rust compilation.

Address the direct fixed-vector Fold family exposed above: named pure,
stateful and witness reducers; inline captured bodies; ordered effectful seed
and iterable operands; and multiple equal-length iterables. Preserve owned
accumulator/items/captures, guards, zero iterations and lexical shadowing.
Do not silently reduce the plan to literal, noncapturing one-vector folds after
seeing the broader genuine source dependencies.

Use existing supported type, conversion, callee, recursion and resource rules.
This does not legalize every type or every source program. An actual additional
source dependency must be recorded and resolved or explicitly left open before
claiming this selected Fold plan complete. General #516 normalization outside
the selected Fold sources remains a distinct gap.

## Affected owners and intended representation

| Owner | Intended change and preserved authority |
| --- | --- |
| `compiler/rust-ir-passes.ss` | Typed stateful Fold expression/return ingress; correctly classify witness/stateful operands and reducers; retain original lexical capture types and source occurrences. Keep existing accepted helper output stable where its premises are unchanged. |
| Private `ir.rs` and origin/resource/child visitors | Represent ordered seed, iterable inputs, typed accumulator/item formals and reducer body. Keep old VectorFoldCall decoding and frozen records; the new paired producer/consumer extension does not promise old decoders understand it. Record any schema/compatibility decision before implementation. |
| Shared aggregate/vector semantics and `stateful/expression.rs` | Reuse canonical signature/coercion rules with stateful-aware child traversal. Do not route witness operands through a pure-only resolver. Preserve actual operand and first-failure order in structured native syntax. |
| `recorded/effect_plan/{model,lower,validate}` | One typed accumulator-carrying Fold block, ordered operands and lexical SSA captures. Independently validate types, vector lengths, callee signatures, fresh formals/IDs, body result and capture visibility before constructing CheckedPlan. |
| Ordinary and functional plan rendering | Emit a linear structured `syn` loop over checked blocks. Carry the owned context/frame/circuit and accumulator through each successful iteration; do not reparse a large token body or unroll arbitrary lengths into duplicated source. |
| Existing runtime `functional::Circuit` ownership | Prefer its current `bind`, `returning` and `try_returning` contract for the generated loop. It already passes the successful value and owned circuit to an immediate continuation. No new public runtime Fold framework or ABI is planned; record a concrete necessity before adding an API. |
| Pure certification, call graph, context-free and capability owners | Preserve accepted helper certification, static body checks and pure proof-not-applicable rules. Classify a new Fold from its actual operands/body. Do not relax VectorMap's existing context-free policy globally. |
| Current refusal/migration tests and frozen historical authorities | Inventory every direct-Fold refusal sentinel. Qualify intentional acceptance widening explicitly; preserve original cases/receipts and hard-pinned authorities. Do not manufacture missing-callee refusals or rewrite original manifests just to keep tests green. |
| Genuine source, external consumer, TS capture and proof smoke | Retain full unedited generated Rust, complete IR and all checked programs; independently exercise public APIs, replay/cost and applicable real proofs with identity-bound negatives. |

Backend paths are under `tools/compact-rust-backend/src`; runtime paths are under
`runtime-rs/src`. This is one semantic operation family, not a new general
control-flow framework, ownership model or CI optimization project.

The intended checked block has an initial accumulator, ordered vector values,
fresh accumulator/item bindings and one result-typed body. Named reducers can
use existing checked call/witness owners; inline reducers use their typed body
and lexical scope. Both original VectorFoldCall and new source forms must reach
the same checked obligations. Producer annotations alone are not authority.

## Ordering, ownership and refusal obligations

1. Resolve and validate reducer arity (accumulator plus every item), parameter
   types and accumulator/result identity. Check all vectors and coercions under
   existing bounds. Reject forged declarations rather than trusting IR labels.
2. Evaluate seed and iterable operands once in the actual source order. Keep
   failures before later operands, reducers, witnesses and mutations. Test
   multiple effectful iterables as well as the observed seed/one-vector case.
3. Execute items in order, carrying only the prior iteration's accumulator and
   successful owned state. A reducer failure prevents later reducer/effect
   execution. Retain real query-summed cost and transcript ordering.
4. Zero iterations still evaluate the operands but do not execute a reducer.
   Statically check the whole reducer, declarations and captures even for zero
   length or an inactive branch. Keep source/type diagnostic precedence.
5. Keep accumulator/item formals distinct and fresh in the body. Capture outer
   values under lexical IDs; shadowing must not leak into the outer scope or
   rebind an unrelated plan value. Independent plan validation must catch
   forged IDs/types and unavailable captures.
6. Owned values must not acquire new Copy requirements. Retain original values
   when the source uses them again, including owned accumulator/capture cases.
   Follow existing generated value retention rules rather than adding broad
   public traits or suppressing legitimate ownership errors.
7. Preserve encoded-byte/type/expansion limits and default worker stack checks.
   Do not raise limits or claim aggregate heap/CPU containment (#495) from this
   operation's bounded checks.

## Finite acceptance

- Generate all nine genuine boundary forms above and the already accepted
  helper control. Preserve source publication and precise refusal behavior for
  unsupported/malformed forms. Compare complete generated interfaces and
  capability/checked-program membership without post-generation normalization.
- Run empty, one-item and multiple-item folds, noncommutative order, named and
  captured bodies, multiple iterables, owned values/reuse, nested pure
  composition, guards, witness/query operands and stateful/witness reducers.
- Compare independent expected values and actual TS results across ordinary
  native/recorded and functional native/recorded execution. Include canonical
  aligned outputs, full relevant state/effects, private state/outputs, witness
  journals, real summed gas and public VM replay.
- Test inactive branches, seed/source/first/later reducer failures and static
  malformed inactive/empty bodies. Distinguish committed local state from
  external witness callback effects; do not promise rollback of those effects.
- Compile external consumers of the complete unedited generated crate. Wrong
  fixed length, item/accumulator type and double Program consumption must fail
  at compilation. Reject forged callee signatures, result types, lengths,
  formals/SSA IDs and capture leakage at the untrusted IR/plan boundary.
- Bind exact source, full IR/programs, compiler/runtime/profile, verified
  parameters, ZKIR, fresh keys and immutable verifier. Generate fresh selected
  Field Fold proofs for pure captured/multi-input and witnessed/mutating cases
  as applicable, independently verify, and reject changed public binding and
  same-geometry public result. Derive actual circuit sizes; do not invent them.
- Require fresh funded default-strict validation/application and unchanged
  replay refusal for application claims. Constructor execution, unselected
  circuits/type families and network submission remain separate claims.
- Preserve and explicitly reconcile old refusal authorities when admission is
  intentionally widened. No blanket repinning, legacy production fallback or
  substitution of invalid-callee controls for the formerly genuine Fold gap.
- Iterate with focused local owner and generated-consumer checks. At source
  freeze retain the complete required compiler/native/proof/wallet/source/
  workspace/Clippy/package gates. Independent external review and intended
  Linux/hosted qualification remain separately open without authority.

## Order and exclusions

The #610 local baseline and its exact source/proof epochs are retained. First
record this source characterization and plan, then implement typed ingress and
checked composition, followed by genuine consumers/refusal controls and fresh
applicable proofs. Stop and fix any concrete baseline regression before adding
more features. The first implementation step must inventory the existing Fold
checker/rendering/refusal consumers; a signature-only happy path is inadequate.

No Forge work, broad refactor, standalone CI optimization, new ledger/ZKIR/SDK
platform, decoder-limit campaign, proof/verdict caching, hosted trial, push,
merge, release or shared installation. Keep source commits local, Conventional,
GPG signed, DCO and Assisted-by. The broader Rust AST goal stays active until
its actual requirements are met. Cleanup is only for task-owned inactive,
unneeded worktrees/targets after completion and verified GitHub merge, with
unpublished work and needed inputs, binaries, archives and evidence preserved.


## Partial checked-core checkpoint — 2026-10-26

The internal checked model now represents one accumulator-carrying loop with
multiple fixed-vector operands and a result-typed body. Independent validation
checks source lengths/item types, accumulator/result identity, fresh lexical
formals, capture visibility and body obligations even for empty loops. Ordinary
and functional rendering use structured loops and the existing owned Circuit
continuations. Unit remains distinct from an empty tuple. No runtime API or
private Expr JSON schema has changed at this checkpoint.

The existing named pure `VectorFoldCall` is the first connected ingress. The
unchanged Scheme frontend plus development Rust backend generated the genuine
`direct.compact` source with ordinary, recorded/observed and functional APIs.
An external consumer of the complete unedited output compared three live TS
input cases across four Rust routes, including state/effects, cost and replay.
An external positive API consumer compiled; wrong vector length, seed/item type
and reuse of a consumed Program failed at their intended call sites.

The full backend library test run passed 1,815 tests with 7 inherited ignores.
All 39 original valid-Fold refusal controls have explicit current successors;
complete predecessor sources and both failed intermediate runs are retained.
Frozen selector/helper collectors that call current semantic/rendering owners
now observe new admission; this does not recertify their historical receipts.
Native-invalid, recursive, signature, layout and rendering-invariant controls
remain. Once an earlier Fold is accepted, the actual later layout refusal is
the correct resource diagnostic; no payload is attached to the earlier child.

Evidence root:
`/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-fold-20261026/`.
Key receipts are `core-laws-v2-result.json`, `core-consumer-v2-result.json`,
`core-public-api-v1.json`, `backend-widening-audit-v3-result.json` and
`control-successors-v1.json`. The general model's multiple-input/effectful-body
laws are synthetic checked-plan qualification, not frontend/runtime proof of
those broader source forms.

Still OPEN: the paired typed source/native ingress for inline captures,
effectful operands, stateful/witness reducers and multiple iterables; full
owned-value/empty/guard/first-failure execution coverage; maintained source
qualification and genuine proofs with binding negatives; complete integration,
independent review and intended Linux gates. Do not silently narrow the signed
plan or mark #661/the broader Rust AST goal complete at this intermediate core.
Cleanup remains conditional on completion and verified GitHub merge; nothing
was merged or deleted by this local checkpoint.
