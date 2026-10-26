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


## Paired source-ingress representation decision — 2026-10-26

Retain private schema20 and add a strict `vector_fold` expression tag carrying
`accumulator: Parameter`, `items: Vec<Parameter>`, `initial: Expr`,
`sources: Vec<Expr>`, `body: Expr` and `length`. Both producers and consumers are
updated together. Existing `vector_fold_call` decoding and the accepted pure
named one-source producer remain valid. An old decoder rejects the new tag;
this is not a forward-decoding guarantee or a new public interchange format.
No runtime ABI/API addition is required.

Named reducers become declaration-directed body calls, including existing
stateful and witness calls. Inline bodies retain their own typed formals and
lexical captures. The seed and each iterable are evaluated once, before any
reducer iteration, in the actual observed order. Independent checks validate
all annotations, including empty/inactive bodies. Any further actual source
dependency (including inline action-body normalization beyond existing Expr
forms) must be recorded and resolved or left explicitly open before claiming
full acceptance. The nine-source plan is unchanged.

The before-edit decision is retained in `ingress-schema-decision-v1.json` under
the existing evidence root. Its existence is not implementation qualification.


### Source-driven body correction

`source-characterization-v3.json` adds an actual inline reducer that captures a
factor and writes/reads a Cell. TS accepts it; the first development ingress
refused its nested Cell.write. Therefore the v1 expression-only body decision
is superseded before acceptance: `vector_fold.body` is `Box<ReturnPlan>`.
`ReturnPlan::Value` owns expression-only bodies; its existing typed actions,
lexical bindings and conditional branches own inline effects. This keeps one
result/body owner and requires the native loop to carry the successful owned
context through every branch and iteration. The checked owner reuses its
existing return-plan lowering. No general Expr-action framework or runtime ABI
is added. Original v1 development artifacts/failed proposal remain retained;
there is no compatibility promise for that uncommitted tag shape. Legacy
vector_fold_call and original records remain valid. The full plan stays open.


## Source-family implementation checkpoint — 2026-10-26

The paired producer and consumer now support the corrected lexical Fold tag.
Named pure/stateful/witness calls and inline captures reach one checked loop;
inline mutation/Let/conditional/action bodies use the existing ReturnPlan.
Native emission carries `(context, accumulator)` through each successful body
and branch, retaining local witness outputs and query-summed cost. Captured
owned values retain the existing clone rules; no Copy or new runtime API is
required. Pure Value bodies use the shared typed aggregate owner. The original
pure named-helper producer and legacy VectorFoldCall decoding remain intact.

The current whole-expression/call-site walkers, named-type collection,
source-origin schema and resource traversal visit all operands and the complete
body. Unknown or forged formals/types/captures, missing fields and cycles remain
refusals even for empty loops. The retired typed_plan owner is byte-identical
to the preceding baseline; new admission belongs to effect_plan. Four retired
exhaustive matchers receive only an explicit denial of the new enum tag. The
new narrow compatibility manifest reconstructs their exact predecessors before
all unchanged historical authorities. No historical manifest was repinned.

Local evidence under the existing direct-Fold evidence root:

- `ingress-package-v4-result.json`: successful paired development compiler
  build with exact dirty-source pins. This precedes the source-family commit;
  do not relabel it as a new exact signed-head compiler qualification.
- `ingress-generation-v2.json`, `source-characterization-v5.json` and
  `ingress-source-programs-v1.json`: fourteen actual complete source outputs,
  IR, capability reports and checked programs, including inline mutations,
  owned and Unit accumulators, multiple witness operands and nested branches.
  The earlier v4 branch probe correctly failed missing source disclosures; its
  source and diagnostics remain. Explicitly disclosed v5 passed both targets.
- `ingress-behavior-v2-result.json`: **31 external consumer tests**, each using
  four Rust routes against live TS: **20 success cases and 11 first-failure
  cases**. All 14 complete generated crates are unedited. Results, the source's
  stored field, private state/outputs, witness events and real summed gas match
  TS; whole Rust query state/effects agree across routes. Both recorded routes
  replay their actual public operations. Held prior-state snapshots stay
  unchanged. Error cases do not claim that callback effects roll back or that
  a failed call exposes a returned partial ledger/cost result.
- `ingress-public-api-v1.json`: seven isolated ordinary API controls reject
  wrong captures/items/vector lengths, a bare String, incompatible witness
  arguments, reused owned inputs and a consumed Program at intended sites.
- `ingress-laws-v5-result.json`: six focused tests, including twelve finite
  shape/depth worker cases. Five preflight-admitted cases render on ordinary
  two-MiB workers; seven larger cases return explicit resource refusals. This
  does not establish aggregate heap/CPU containment or a universal stack proof.
- `ingress-backend-v2-result.json`: **1,822 passed, 7 inherited ignores**.
  Strict backend all-target/all-feature Clippy and header/format gates pass.
- `ingress-consumer-lock-v1.json`: all336 nonlocal consumer package identities
  and checksums match the repository lock. This is not full Node closure or
  Linux qualification. The TS capture remains a local host development cohort.

Still OPEN before #661 acceptance: publish the full source/consumer cases in a
maintained gate, add its source/refusal and changed-reference controls, freeze
and qualify the exact compiler/runtime/source cohort, execute fresh applicable
proofs with immutable verifier and binding/result negatives plus default-strict
funded application/replay, and retain all required compiler/native/wallet/
workspace/package integration gates. Independent review and intended Linux
qualification remain separate. No hosted run, push, merge, release or shared
installation is authorized. This checkpoint advances the full selected family
and does not complete #661 or the broad Rust AST goal. Forge stays deferred;
cleanup is still not due before completion and verified GitHub merge.


## Maintained source-family gate checkpoint — 2026-10-26

`tools/compact-rust-backend/check_direct_fold.py` now owns reproducible fresh
Rust/TS generation and separate external consumers for all fourteen actual
source cases. The sources in `examples/rust_backend/direct_fold/` retain their
original complete bytes. Golden crates retain full IR, origins, capabilities
and every checked program; only their Rust golden is rustfmt formatted.
The maintained consumer shares its route/replay checks within this one test
family; generated contracts are never patched to make a consumer pass.

The selected TS runtime is required, version checked and resolved identically
from every generated wrapper. The gate hashes its complete production package
closure, Node executable and external host libraries. Known OS/kernel paths
are reported explicitly with their narrower scope. Injected kernel names are
portability controls, not Linux execution; an unknown missing library must fail
before publishing a capture. A changed live TS aligned result must fail the
actual Rust consumer.

Local `maintained-gate-v3-result.json` passes 31 cases (20 success, 11 first
failure) across four Rust routes, successful recorded VM replay, private/witness/
state/cost comparisons, seven intended Rust compile failures and four source
refusals on both targets. The complete external consumer also passes strict
Clippy. This does not return failed partial state/cost or promise callback
rollback. The finite static/ordinary-worker laws remain required and passing.

Publishing the whole new corpus exposed two actual generated-Clippy failures:
a redundant iterator conversion in multi-input zip and an immediate frame move
back to itself for an effect-free Unit body. Their emitters are corrected; the
Unit body remains statically validated and operand/iteration evaluation is
retained. Fresh generation changes only three Rust golden libraries. All full
IR/program/origin/capability bytes stay exact. Failed initial lock resolution,
original Clippy outputs and the consumer-macro witness-argument mistake are
retained separately; none is hidden by patching generated code or suppressing
lints. The fourteen fixtures add only local workspace packages; registry
versions/checksums do not change. The 43 new source declarations extend only
the additive parity baseline, preserving every historical declaration.

The existing native CI workflow invokes this source-family gate and retains
its full generated outputs/reference/consumer lock. These are local workflow
edits; no hosted run, publication or activation was performed. Evidence remains
under `/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-fold-20261026/`.
`maintained-package-v1/build-result.json` records the paired development package
and exact dirty-source pins. Later consumer/fixture/inventory/docs publication
is not a new exact signed-head compiler qualification.

The maintained functional publication subgate is now qualified locally. Still
OPEN before full #661 acceptance: freeze and qualify the exact signed compiler/
source/runtime/SDK/parameter/verifier cohort; fresh selected captured/multiple-
input and witnessed/mutating/inline Field proofs, independent verification and
binding/result negatives; funded default-strict apply/replay; the complete
required original compiler/native/proof/wallet/workspace/package gates;
independent review and intended native Linux qualification. Do not substitute
old Map/vector/Forge proof receipts. Do not select another feature before
completing this plan. Forge stays deferred; cleanup remains conditional on
completion and verified GitHub merge, with no push/merge authority inferred.


## Selected genuine-proof cohort design — 2026-10-26

The original ordinary `.#compactc` recipe now resolves successfully at clean
signed `3f09c0b8f`, with exact tree/signature and 3,750 pinned source/recipe files
unchanged during the invocation. `frozen-compiler-v1/build-result.json` records
the same `himx58...` package with current clean-source evidence; prior dirty
build receipts retain their original scope. Compiler reuse is explicit, not a
renamed historical proof or a new binary build claim.

`check_direct_fold_proof.py` and the proof-smoke `--direct-fold CASE ROOT`
consumer define the next finite genuine-proof gate. It links complete maintained
generated crates for captured, multiple-input, named stateful, named witness,
inline effectful and inline branch sources. Seven fresh call proofs cover these
six sources, with both inline branches selected. Every source's full IR,
origins, capability report, Rust output and checked program must match the
linked fixture. Changed full IR refuses before absent keys/state. Generation
uses original relative source labels; there is no metadata normalization.

Each case checks four Rust routes and the separate ordinary observed-call
preparation, including result/stored state, private outputs, witness journal,
effects, gas and recorded program equality. Actual direct ZKIR key generation
records each model. The locked SDK verifies built-ins/SRS; only explicit SDK
preparation may fetch missing verified material. All tool, source, generated,
material, key, verifier and proof/result identities are retained and rechecked.

The selected prover must independently verify each new standalone proof, reject
changed public binding and a genuinely changed Field result through the exact
upstream communication commitment, and refuse the changed preimage. Aligned
shape, atom count and each atom's encoded length must remain equal for the
result negative. Each selected call is freshly proved again in a funded default-
strict local transaction, applied to its expected state, then rejected on replay
without ledger mutation. Constructor execution, other source/type families and
network submission are not claimed. No proofs or verdicts are reused.

This design is locally compiled and strict-Clippy/dispatch/format checked; its
execution receipt must establish actual proof acceptance. The proof runner and
its source commit are a distinct epoch from the reused frozen compiler. Full
required compiler/native/proof/wallet/workspace/package integration, independent
review and intended native Linux qualification remain required afterward.


## Selected proof execution and integration-control successors — 2026-10-26

At signed proof-source `1b37bbb173`, `proof-gate-v1/receipt.json` passes all six
linked source families and seven selected calls, including both inline branches.
Every standalone proof is fresh, 2,912 bytes, independently verified, and rejects
changed public binding/result plus the changed preimage. Each call is freshly
proved again in a funded default-strict transaction, applied to the expected
state, and rejected on replay without ledger mutation. Six changed-full-IR
controls reject before absent keys/state. Witness reducers preserve two callback
calls and private state 9. Constructor execution and unselected type/source
families remain outside this finite proof claim.

Actual models: captured 75 rows/k7; multiple inputs 96/k7; named stateful 75/k7;
named witness 72/k7; inline effectful 76/k7; inline branch 344/k9. The branch
source is intact; it is not trimmed to a smaller circuit or represented as a
Forge tiny-contract lane. No compiler or global resource limit was raised.
The original ordinary compiler is the independently recorded clean `3f09c0b8f`
cohort; compiler/backend/runtime production bytes and all linked goldens are
unchanged by the separate proof runner epoch. Only local proof dependency
metadata changes. Verified upstream parameters, fresh keys and immutable tool/
verifier/artifact identities are retained in the proof receipt.

The original compiler `doCheck` build, including `compiler/go` and
`srcMaps/test.sh`, passes at `1b37bbb173`; all 46 reported compiler stages pass.
The broad workspace/source audit exposed eleven external integration tests and
one source-capability family that still used valid Fold as a refusal sentinel.
Their original files, actual failing broad/focused runs and complete old-source
outputs are retained. These successors deliberately admit the unchanged Fold
and its real pure callee at every original position. A copy with only length
1 changed to 2 for one item is refused, including inactive helper branches.
This is explicitly a native-invalid annotation check, not a native-valid
unsupported operation or a fabricated missing-callee negative. Existing type,
slot, arity, cycle, failed-publication/recovery and other controls stay intact.

`integration-control-successors-v2.json` records all 166 focused integration
tests passing (5 point, 154 render, 7 Unit) and the source refusal/publication
suite passing. A draft edit that dropped nine callee registrations failed and
was corrected before accepted validation; its source/output remain retained.
No producer/runtime/generated artifact or historical manifest was changed by
this test migration. The full workspace must now be rerun to completion;
full native/proof/wallet and package/archive gates, independent review and
intended Linux qualification remain open. #661 and the broader goal are not
complete. The existing Forge deferral and conditional post-merge cleanup remain.

## Finite acceptance-edge completion (2026-10-26)

The maintained source family now also includes the one-item captured reducer
and zero-length witness-valued seed/iterable case. This completes explicit plan
edges that the original fourteen sources did not independently exercise. The
expanded functional gate contains sixteen complete sources and 38 cases (25
success, 13 first failure), each through four public Rust routes against live
TS. The empty case checks seed then iterable exactly once, preserved zero-width
private output, and no reducer invocation; separate seed/iterable failures and
an armed reducer refusal remain controls. Eight ordinary compile-fail cases
include Unit refused in place of `FixedVector<Field, 0>` in a witness signature.

The selected proof gate adds one-item and empty-witness calls while retaining
all seven prior selected calls. Fresh proof/application acceptance must be
recorded separately for this expanded nine-call/eight-source cohort; earlier
seven-call receipts cannot establish it. Constructor and other unselected
sources remain outside the selected proof claim. No compiler/runtime/IR change
was required for these edges.

Local `--full` now schedules the same Vector/Map/Fold proof gates already in
required CI, in addition to every prior specialized proof gate. Each child must
return its successful typed receipt, retained and rehashed after all children;
command/missing/malformed/wrong-format/failed-status controls apply to all eight.
The frozen Scheme frontend reaches expression gates through their supported
environment interface. Existing five gate command arguments are preserved.
Independent review, intended native Linux and complete integration remain open;
this addition does not waive any required gate or complete the broad goal.

## Expanded local integration checkpoint — 2026-10-26

Execution source: clean signed `fb229b9cde34b9ba34f7774f82cff04072113889`.
All sixteen full sources and 38 live-TS cases pass four public Rust routes:
152 executions and 50 successful recorded VM replays, with eight ordinary Rust
compile-fail controls and eight source refusals. The one-item capture and empty
witness-valued operands are included without dropping the earlier family.

The expanded selected gate generates nine fresh 2,912-byte Field-call proofs
across eight complete sources, independently verifies them with immutable
verifiers, rejects changed binding and same-geometry result/preimage, and runs
nine freshly proved funded default-strict applications with unchanged replay
refusal. Eight changed complete IRs refuse before key or state actions. The
empty source has 27 rows/k6, the one-item source 72 rows/k7, and intact branch
source 344 rows/k9; actual models remain in the receipt. Zero-width private
output and seed/items callback order remain present in empty-source execution,
preparation and proof. Constructor/unselected/type-family proofs are not claimed.

The frozen current workspace passes 3,486 tests in 713 harnesses, with the same
seven inherited ignored identities; all-targets/all-features strict Clippy also
passes. All 273 full registered golden fixtures pass freshness with zero stale
or failed outputs. Earlier eight doctests, original compiler checks, full native
consumer/proof and wallet handoff, and runtime package/manifest/archive consumer
receipts retain their separate source epochs. Compiler/backend/runtime source
bytes are unchanged from `3f09c0b8f`; no old whole-source receipt is relabeled.

Maintainer evidence root:
`/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-fold-20261026/`.
Read `proof-gate-v2/receipt.json`, `edges-integration-v1/receipt.json` and
`acceptance-edges-maintained-v2/gate-result.json` for their exact bindings.
`independent-review-packet-v1/` prepares the complete implementation diff and
pinned source/receipt inputs; it contains no independent reviewer verdict.

This is selected local qualification. Independent external review, intended
native Linux and current-head hosted integration remain open without authority;
#661 and the broad Rust AST goal are not declared complete. No new feature is
selected ahead of those acceptance boundaries. Forge stays deferred and all
work/evidence remains local. Cleanup is not due until completion AND verified
GitHub merge, with unpublished work and qualification artifacts preserved.

## Ownership and lexical source audit — 2026-10-26

A source review of the plan's named obligations identified two combinations not
independently represented by the maintained sixteen-source corpus: owned vector
items plus an owned capture reused across iterations, and reducer-local shadowing
of outer formals/captures in both pure-valued and mutating bodies. Three complete
source probes and eight live-TS/four-route Rust cases passed without a compiler,
backend or runtime implementation change. Their complete source/outputs and
standalone consumer remain in `source-scope-audit-v1/` under the evidence root.

These are now maintained regressions in the same Fold gate: nineteen full
sources, 46 cases (33 success, 13 first failure), 184 Rust route executions and
66 successful public VM replays. All earlier sources/cases/refusal controls are
retained. Eight Rust API compile-fail controls and eight source refusals remain
required. The new generated crates stay complete, with full IR/origins/capability
reports and checked programs; source arguments use actual relative repository
labels. No generated output is patched or normalized. Only six additive source
declarations and three local workspace fixture packages are added; historical
rows and all nonlocal dependency versions/checksums are unchanged.

The expanded functional gate, its strict external-consumer Clippy, all three
fresh golden checks, strict new-fixture all-targets/all-features Clippy, public
CI Python controls and workspace formatting pass locally. These runs are not a
fresh execution of the whole previous workspace/compiler/native/package suite.
The selected nine-call proof receipt keeps its exact earlier source/runner
identity and scope. None of the three new sources is claimed proved, and no
unchanged cryptographic proof/timing run is repeated by this source audit.
Independent review and intended native Linux/current-head CI remain open.
