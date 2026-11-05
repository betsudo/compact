# Constructor fixed-Vector iterable expressions

Status: bounded functional qualification accepted at frozen producer `f09900533`;
complete model sizing accepted at `775b662ed`; selected later-entrypoint proofs
and strict applications accepted at separate qualifier `a8133996b`. Constructor
cryptography/total cost, new-head CI/Linux and integration remain open.
Local continuation of the primary Rust AST goal, 2026-11-05. Earlier sections
retain their historical execution epochs; the current results are below.

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
At characterization time, PR #701 had separately passed Rust required checks,
both compiler producers and the macOS consumer while Linux remained running.
The later read-only observation confirms all 11 configured checks succeeded at
published `8d100a1b5`; the PR remains open and mergeable. That hosted result does
not qualify the subsequent constructor changes. Preserve the published proof
repair and separate reviewed local proof-CI successor `58c1e5083`.

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


## First maintained execution and identity-review corrections

At clean `5aad4d3cc`, the original maintained compiler builds and its recorder
passes. The first maintained gate executes all 120 commands successfully: 13
complete sources, 22 TS cases, 60 later Rust routes, 30 replay comparisons, five
executing API twins with marked refusals, eight IR refusals and four semantic
negative controls. This result remains diagnostic rather than final acceptance.
Independent source review found two P2 gaps: Program assets were checked by count
without proving exported constant associations, and the consumed TS oracle was
not hashed into the receipt or rechecked during consumption.

The correction admits unique expected Program roots with their review-only
policy, records each root/file/hash, and checks the actual three exported Rust
constants against their exact retained assets. An isolated duplicate-root
control refuses; a separate cloned SDK swaps only metadata assets and must fail
the real compiled public-constant check. Positive generated SDKs and generated
Rust are unchanged. This validates metadata identity, not semantic equivalence or
execution authority for the review-only sidecars.

The TS receipt now binds the actual wrapper, fixture, capture and runtime
identities and records the oracle hash. Each subsequent command checks the
selected oracle path and bytes; final success repeats both hash and identity
checks. A changed oracle path must refuse before a sentinel child starts, and
changed bytes must fail against the retained digest. Current source review and
a fresh clean-head original recipe plus full gate run remain required for this
correction. Earlier results retain their source identities; proof, constructor
cost and integration boundaries remain open.


## Separate proof qualification producer and consumer

The producer branch is frozen at `f09900533`: the original compiler, corrected
122-command functional gate and independent source/receipt reviews pass within
that bounded scope. A separate local `codex/rust-ast-constructor-vector-proofs`
branch starts at that actual head. Compiler/SDK producer identity and subsequent
proof-qualifier source identity stay distinct. No later code or platform run is
inferred from the producer receipt; full combined integration remains open.

Before generating keys, the opt-in proof-smoke feature
`constructor-vector-models` and selector `--constructor-vector-inventory ROOT`
load every intact emitted model with the existing `IrSource::load/model().k`
owner. The observer requires all 13 source directories, exact export/model
membership and matching proof-required capabilities, and reports all 39 `k`
values. It opens no keys, generates no proof and makes no selection. A model's
numeric `k <= 8` classification is not full proof eligibility. Do not replace
this observation with metadata guesses, trim sources or raise the tiny cap.
The forthcoming qualifying controller binds actual model bytes, producer receipt,
observer source/binary and upstream/tool identities independently.

The 15 positive constructor cases are candidate `advance` calls across 13
complete sources. The next proof lane uses the existing isolated default-bundled
SDK and source-testkit/ProofLab owner. Each SDK gets one physical runtime and a
separate consumer; generated source and its manifest remain unedited. After
pre-key IR/all-Program/source bindings and verified material admission, require
fresh selected standalone proofs, immutable verification and separately proved
funded DEFAULT-strict transactions with replay and relation/identity negatives.
There is no constructor/deployment cryptographic or wallet-lifecycle claim.
The observer and this proof preparation still require actual execution and review.

The first model observation built successfully but refused the producer root
before loading any model: the root contains the compiler's 13 persistent
`.<role>.compactc.lock` files alongside the 13 SDK directories. Independent
review confirmed the overly strict root-membership premise. The corrected
observer requires exactly these named regular sidecars and real SDK directories,
without opening, deleting or modifying the locks. Unknown root entries, missing
members and directory/symlink substitutions still refuse. This does not establish
lock ownership or liveness. The failed observation remains separate evidence.

## Accepted intact-model observation

At clean qualifier `775b662ed`, the corrected observer builds and passes strict
Clippy locally. It reads all 39 retained models across all 13 complete SDKs;
every model reports `k = 6`, within the unchanged automatic cap of 8. All 1,573
declared producer assets remain bound to the accepted functional receipt and
unchanged. The controller records qualifier source/tree, Cargo metadata and the
actual build artifact, binary and tool identities. These are observation records,
not complete proof toolchain or parameter admission.

An isolated minimal HOST fixture passes, then refuses five relevant mutations:
an extra SDK directory, a lock replaced by a directory, an SDK replaced by a
symlink, a missing model and an extra model. The original generated SDKs and
lock files stay untouched; all failed predecessor and HOST evidence is retained.
Independent Codex CLI review, used after a genuine Claude quota failure, confirms
the P2 correction and recomputes input/log/artifact bindings with no new finding.

The next selection remains the 15 successful constructor cases across 13 full
sources, calling `advance` with delta 4 and checking the independently expected
stored value plus 4. Both conditional choices stay selected. Preserve `read` and
`advance_other` in every SDK; `advance_other` is a distinct relation available
for wrong-verifier controls. Numeric sizing alone authorizes no key generation:
first bind the exact source, IR, all exported Programs, isolated dependency graph,
tools and verified parameters. Then generate fresh direct proofs and independently
verify them, followed by separately proved funded DEFAULT-strict application and
unchanged-ledger replay refusal. Constructor initialization itself is not proved
by a later entrypoint proof. No new keys or proofs have run in this phase.

Retained observation: `model-observation-v2/receipt.json`; independent review:
`model-source-review-v2.json`, under the evidence directory named above.

## Isolated proof-consumer preflight

The new maintained `check_constructor_vector_proof_preflight.py` copies each
complete frozen SDK into its own external consumer closure. It reuses the
existing copied-testkit dependency recipe and admits exactly five local packages
and one bundled runtime per SDK. Its native mode checks the original generated
constructor, private state and witness trace, all exported Program associations,
and the selected later `advance` result. This phase generates no keys or proofs.

The first run at `a391316bb` passed nine SDK lanes and then found a test-consumer
alias error in the owned-value constructor fixture. The fixture uses `generated`
for its additional typed ledger slots; the proof consumer now supplies that alias
without editing the fixture or generated SDK. Independent source review also
found two P2 identity gaps: undeclared SDK files could enter Cargo, and inherited
PATH could select tools other than the hashed rustup executables. That run remains
failed evidence and supplies no proof admission.

The correction requires exact full SDK membership, including the compiler
manifest's retained hash, before copying. Complete source and SDK inventories are
rechecked before children and at completion. HOST controls refuse an extra build
script, changed manifest, changed IR, swapped Programs and changed source before
staging. Cargo commands use resolved executable paths and its child toolchain is
selected explicitly. A fresh complete preflight and independent review remain
required before the later proof consumer may generate keys or prove.

The corrected preflight at `ecd227b8b` subsequently passes all 13 builds, strict
Clippy runs and 15 native cases, including the owned-value fixture. Acceptance
for key generation is still withheld: fresh review confirms those fixes but
requires physical dependency-source authentication and binding the reviewed
external executor's exact bytes. No cryptography has run.

The next correction authenticates each Git dependency's actual checkout commit,
tracked blob contents, symlink targets and unexpected inputs. Registry source
contents are compared to the cached crate archive whose SHA256 matches the
locked checksum; an optional local checksum table cannot rebind altered source.
Admitted source bytes and membership are rechecked before builds and completion.
Preparation observes one Git root and 337 registry roots, totaling 24,230 files.
Isolated HOST controls reject modified Git source, modified registry source and
modified archives. Six maintained unit methods also cover untracked build
scripts, wrong revisions, post-admission drift and rewritten checksum tables.
An initial test caught macOS `/var` path alias handling; normalization fixes it.

Source inspection also identified a staged-file bypass: a new build script
added to the dependency index is absent from both HEAD and untracked-file lists.
A new control reproduces that refusal gap before correction. Admission and
post-admission checks now require the dependency index to match HEAD. The v3
preflight was safely stopped between child commands before this source fix;
its interrupted receipt remains diagnostic, with no keys or proofs generated.

The external executor must be reviewed as a read-only content-addressed copy.
Parent triage records that exact digest and path, and the trusted launch checks
them before execution. Repository head and launch-time self-hashing alone do not
bind external source to its earlier review. All failed/withheld predecessor
receipts remain retained. Full preflight, fresh review and actual proof/application
execution remain required; these preparation checks do not imply completion.

## Accepted selected proof/application qualification — 2026-11-05

At clean qualifier `a8133996b625615540cb24d7f46013068fa19884`, the final maintained
preflight passes all 57 commands: 13 complete SDK builds and strict Clippy runs,
15 native cases, actual exported Program bindings and the full isolated dependency
graph. Physical dependency admission covers one Git and 337 registry roots.
Six maintained source-authority controls and the five source/SDK staging refusal
controls pass. Earlier failed and withheld runs keep their original identities.

Independent source review reports no remaining finding and explicitly identifies
the read-only executor digest. Parent triage binds that digest/path to the actual
review. The trusted parent rechecks the literal reviewed bytes, clean source head,
packet and successful preflight immediately before invoking that copy. Mutable
prototype executors are not execution authorities.

The actual fresh run passes all 40 commands. Every complete source receives new
direct `advance` and distinct `advance_other` key generation: k6 with 50 and 51
rows respectively, without changing the cap of 8. All 15 selected success cases,
including both conditional choices, generate a fresh 2,912-byte standalone proof
and verify it directly against the immutable selected verifier. Each case also
separately proves, funds and applies a DEFAULT-strict transaction. Complete final
contract data matches the generated execution; replay returns
`IntentAlreadyExists` with the whole ledger unchanged.

Controls reject changed binding, input, output, input/output proof preimages,
same-length corrupted proofs and the distinct relation's verifier. Positive proof
verification is repeated after mutation controls; key bytes remain unchanged.
Constructor state/private state/witness trace/owned extras and ordinary-versus-
functional prepared calls are checked. No skipped public inputs are admitted.
All 13 SDKs and their 39 models remain intact; material outputs are separate.

Independent receipt review rehashes 29,529 distinct files, checks physical source
authorities, source/tool/parameter/key/binary/log/artifact bindings and finds no
actionable defect or unsupported success claim. This review is source/hash
inspection, not another cryptographic execution. The actual run retains 15 unique
proofs and 15 unique sealed transactions among 162 lane artifacts. No proof or
verdict cache is used.

Evidence under the retained directory: `proof-preflight-v4/receipt.json`,
`fresh-proofs-v4/receipt.json`, `trusted-proof-launch-v4.json`,
`proof-source-review-v3.json`, `proof-receipt-review-v1.json`,
`proof-results-parent-audit-v1.json` and `proof-qualification-closeout-v1.json`.
These are local evidence locators, not a portable public evidence bundle.

The accepted scope is later-entrypoint qualification from constructor-derived
test state. Constructor insertion and Night/Dust fee funding are privileged test
setup. Constructor cryptography/total cost, wallet lifecycle, native Linux,
new-head required CI, full supported-profile handoff and canonical integration
remain open. The broad Rust AST goal is incomplete; Forge remains deferred.


## Canonical reconciliation review repair — 2026-11-05

Forward merge `88878b068` preserves both the constructor192 history and canonical
`cb39def70`, with unreleased compiler194 and unchanged 473 external Cargo records.
Its original 13-source gate passes 122 commands, but independent source review
finds a P2 gap outside that cohort: nested loops lose an outer local Vector binding
when fold-body lowering discards the binding environment. The original194 compiler
accepts a complete nested source for TS but refuses independent Rust IR and Rust
with the intended scope diagnostic. This is a reproduced frontend gap, not a
successful proof or a claim that nested lexical support was already accepted.

Thread the existing lexical bindings through fold-body recursion, including
sequences and both conditional arms. Keep constructor parameters/item bindings,
lexical exit, public APIs, IR schema and runtime unchanged. Retain all thirteen
original complete source files and their hashes; add a separate fourteenth
`nested_capture` source exercising both conditional arms inside an outer loop.
The intended current cohort is 23 constructor cases, 16 successes and seven
errors, with 42 intact models and 16 selected later-entrypoint proofs. Earlier
13/22/39/15 receipts remain literal and are not relabeled as this extension.
Current clean original194 recipe, complete API/TS/refusal execution, preflight,
fresh proofs/strict applications and renewed independent reviews remain required.

The initial194 model preparation also refuses a shared registry source containing
an extra Cargo.lock not present in its locked archive. All archive source bytes
match; no source repair, deletion or exclusion is authorized by that observation.
Preserve the global cache and failed receipt. Use an isolated Cargo home populated
by Cargo from the original locked archives before full dependency authentication;
retain the same source/checksum identities and exact source-inventory guards.
