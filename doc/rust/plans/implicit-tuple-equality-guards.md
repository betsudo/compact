<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Preserve implicit tuple widening in constructor equality guards

Latest execution epoch: `5758c0244`, compiler 0.31.210, ledger8.1.3 on macOS
arm64. Current API, model/native preflight and fresh proof/application execution
pass with a successful independent parent artifact audit. Independent actual
proof-result review accepts only this bounded local execution epoch. Earlier progress sections below preserve their
stated source epochs; they do not override the latest bounded results and limits
at the end of this plan.

## Demonstrated gap and baseline

This continuation of #720 starts at signed documentation head `91179e374`,
preserving the accepted execution head `7f690b7a6`, the canonical development
line, all unpublished slices and their source, failure and proof evidence.
The clean baseline passes 66 constructor unit tests, 14 constructor renderer
tests, 49 coercion-owner tests and strict library Clippy on macOS arm64.

Seven complete genuine sources were characterized with original compiler
0.31.209: flat equality and inequality, nested tuple equality, a mixed literal
and variable tuple, reversed equality operands, asymmetric tuple element widths,
and a captured Field Vector reused across iterations and two loops. Every source
generates raw IR and TypeScript; every Rust generation is refused at tuple
typing. No SDK execution or cryptographic result is claimed for those probes.
The original inputs stay Uint<8>; comparison context requires Uint<16> in one
or more tuple positions. Literal inference already retains its expected bound.
The historical source and raw IR must remain byte exact compatibility fixtures.

## Owners and intended API

- `compiler/rust-ir-passes.ss`, `typed-expression-ir-value`: investigate and
  retain the original analyzed component source type passed through tuple casts
  to variable references, only under the scoped constructor-condition parameter
  introduced in 209. The constructor has no callable variable-type table;
  ignoring this existing source annotation loses the needed widening.
- Existing `pure_semantics` coercion planning independently validates every
  tuple element and unsigned bound; `pure_native` and `value_lowering` render
  its checked facts through structured Rust AST. Do not relax that relation.
- Constructor condition admission and `constructor_vm` retain infallibility,
  both-arm validation, original-owner diagnostics and left-before-right order.
- Existing source/API, TypeScript, model and proof owners retain exact compiler,
  source, raw IR, generated SDK, parameter and immutable verifier identities.

Generated `initial_state(ConstructorContext, ...)` keeps the exact Compact
parameter types, including caller-owned Vectors passed by value and typed tuples.
No public API, runtime ABI, IR schema, dependency or recording operation changes.
Local branch inventory identifies 0.31.210 as unused among the inspected local
heads; this is a source identifier, not a release or global availability claim.

## Bounded implementation and acceptance

1. Preserve complete 209 source/raw IR refusals and add a compatibility test
   that still rejects their mismatched tuple types. Current compiler output is
   tested independently; never patch generated IR or accept either outcome.
2. Add the minimal constructor-condition tuple context transport through the
   existing Coerce node. Nested tuple/Vector elements reuse the recursive checked
   coercion owner. Preserve scalar, literal, ordinary circuit and initializer
   behavior, fallible operand refusals and narrowing rejection.
3. Require all seven complete sources to generate current raw IR, TypeScript,
   ordinary and functional Rust SDKs, with original source-origin ownership.
   Extend the maintained scope gate to the new exact tuple shape; keep the old
   209 refusal explicitly historical, not as a current success criterion.
4. Compare actual generated initial states against an independent live TS
   oracle. Cover both operators, both operand orders, zero/255, wider right
   values 256/65535, differences in each nested/component position, literal
   inference, asymmetric widths and owned Vector capture/reuse across loops.
   Include marked wrong-width/shape ordinary compile failures and valid twins.
5. Exercise malformed private IR: component narrowing, tuple shape mismatch,
   invalid inactive arms and the first original failing coercion owner. Run
   focused Rust/Python checks while iterating; retain full integration gates.
6. Before keys, observe every export of every intact contract under automatic
   k<=8. Then run applicable fresh selected later-entrypoint proofs with actual
   verification, exact binding negatives, and separately proved DEFAULT-strict
   transaction application/replay. Preserve all failures; no trimming or forced
   k. These proofs do not establish constructor/deployment cryptography.
7. Obtain fresh Claude CLI source/caller and actual-result reviews, retain their
   limits, and record meaningful progress in the existing midnight knowledge
   base and CAS checkpoint. Commit locally with GPG, DCO and Assisted-by trailers.

## Exclusions and current state

No top-level constructor if, general arithmetic/fallible guard admission, new
coercion framework, accepted-capability reopening, scalar-triple model workaround,
CI optimization, publication, hosted trial or Forge work. Linux/full required CI
remains an integration gate. Do not infer proof or API acceptance from generation.

The initial210 candidate built correctly, but all seven Rust generations still
refused: preserving an outer safe-cast wrapper did not address the actual lost
variable source-type annotation. That ineffective change is removed in a
forward repair, retaining the failed epoch and complete outputs. The repair uses
the existing per-component source annotation under the scoped condition flag.
Its current original compiler, generated API, model, proof and application
acceptance remain open. The initial focused test run also caught a borrow/move
error in the new narrowing test; its expected diagnostic now clones the type
already borrowed by the parameter fixture. Stop new slices at
2026-11-08 09:05 UTC and preserve an exact continuation handoff.

## Current characterization and review follow-up

Original210 at `458cc6231` generates raw IR, TypeScript and Rust for all seven
complete sources (21 commands). This is generation evidence only. All 68
constructor unit tests pass. The historical renderer test initially overlooked
the original source-location wrapper; it is strengthened to require the exact
location and complete expected/actual tuple types. A separate Python control
authenticates all seven full source/raw hashes and intact three-export rosters.

Fresh Claude source review found no traced unsafe acceptance, but identified
two P2 coverage gaps: the existing analyzed-source fallback also repairs direct
aggregate variables and other parents, and the seven original probes lack a
maintained exact-condition/component-origin generation gate. Both findings
remain acceptance work. The external 50-case API draft is not an executed gate.
Do not restrict legitimate affected shapes merely to avoid their validation.

Eleven supplemental complete sources now have actual original209 and
candidate458cc210 comparisons, preserved under `supplemental/`. Direct tuple
and Vector variables, both tuple/Vector conversion directions, Uint-to-Field
tuple components, widened Vector capture and aliased Vector variables gain Rust
generation. A tuple hash-argument source already generates with209 and remains
a regression control. Numeric-alias tuple components still stop at checked tuple
typing; aliased tuple variables stop earlier at the Scheme cast owner. These
are remaining valid-Compact transport limitations, not invalid-language rules.

The struct-literal guard source exposes a real existing renderer panic in BOTH
209 and458cc210, after successful frontend/raw generation. Repair this concrete
baseline defect before dependent feature qualification: construct the existing
constructor `if` from a fixed AST skeleton and insert the checked condition AST,
following the existing Fold owner. This lets the AST printer retain required
grouping around struct-literal comparisons without parsing ambiguous condition
tokens. Preserve condition/arm validation and counter/error order. Unit controls
must round-trip both operators and with/without else; generation and native
compilation of the complete original source must confirm the fix.

The next maintained gate must include exact current conditions and component
source origins for all affected supported shapes, actual ordinary/functional
API behavior against live TS, static misuses, ownership and existing refusals.
Qualify every intact export before selected proofs; no source/model trimming.
The two compiler epochs and all failed attempts remain historical, not current
native/proof or integration receipts. Linux/full CI and the broader goal remain
open; Forge remains deferred.

At signed repair head `64ec7c326`, the focused checks pass 69 constructor,
15 renderer and 51 coercion tests plus strict library Clippy. Its rebuilt
original210 compiler independently runs all 18 complete repository sources
(54 generation commands): 16 Rust successes and the same two explicit alias
transport refusals, with TS success for all 18. The original struct source now
generates without the previously observed panic. This still does not prove
generated Rust compilation, actual constructor behavior or cryptography.

The maintained tuple scope gate now has explicit expected parameter/loop,
condition and original source-origin records for all 18 sources, and separate
wrapper/value ranges for each conversion. Aggregate cast wrappers can cover
the whole original comparison while their values keep the variable use-site;
do not force those original ranges to be equal. Source/HOST preparation checks
pass on the completed64ec outputs: 17 raw-success validators and eight mutated
copies refused without compiler/runtime execution. The new current-head CLI,
original recipe, expanded API/native cohort and fresh review are separate gates.

The actual current36a maintained run passes both source cohorts: 72 generation
commands, 24 complete sources, and 18 never-executed HOST refusals. Fresh Claude
review accepts the baseline AST repair and actual result integrity only; P2-1
API execution remains open. Its two additional scope findings are addressed in
the next source preparation: all109 expression-origin records are pinned
structurally (including identity/literal/tuple/other operand nodes), a ninth HOST
control clears an identity child, a compiler-free test loads the expectations,
and the existing locally configured CI step invokes the tuple gate and retains
its diagnostics before the historical proof owner. Default historical pipeline
stages and sixteen-case proof behavior stay independently tested. No hosted run
or actionlint pass is claimed; actionlint was not available in the checked PATHs.

The expanded finite API cohort now has 16 complete supported sources and 105
literal cases, with five marked ordinary misuses/valid twins, live TS capture,
four native/recorded/functional routes and public VM replay checks. Its intended
counts are 1,260 calls and 630 replays, not yet execution results. Hash-argument
fixture inputs come from the pinned official runtime's typed homogeneous pair
encoding; actual live TS constructor execution must confirm their literal branch
outcomes. Source/proof-case metadata matches exactly. All new tuple sources and
the two unsupported alias cases remain in the separate complete scope roster.
Model/native-proof owners are not yet extended to this105case cohort, and no
current generated consumer, model, key, proof or strict application has run.

The first105case API run at `cc97a0ecd` independently passes live TS outcomes
for all105 cases and compiles all16 complete Rust SDKs. The native batch then
aborts before reporting results: its monolithic fixture test reserves
2,731,536 bytes at entry (corroborated from the retained failed binary's selected
prologue), above the required 2 MiB test worker. No native case or whole API pass
is claimed. Preserve the failed binary, streams and first two missing-symbol
inspection attempts. The fixture is outlined into one non-inlined typed helper
per contract, preserving all105cases, caller ownership and twelve calls/six
replays per case. The new tuple gate explicitly uses a 2 MiB worker; it does not
raise a stack limit or remove a source/case. Re-execute the complete current API
and all negative/reference controls before model or proof qualification.


## Accepted source/API milestone and proof continuation

Signed `54e428633` is accepted locally for source/API/scope after fresh Claude
review and an independent parent audit. The maintained gate passed all 16 SDKs,
105 live TypeScript/native cases, 1,260 API calls and 630 public VM replays under
the required 2 MiB Rust test-worker stack. Five marked E0308 misuse cases each
have a passing valid twin; a changed oracle fails and the restored oracle passes.
The initial monolithic fixture's stack-overflow failure remains retained. The
repair outlines typed per-contract helpers without trimming cases or raising the
stack. Linux debug stack fit remains unproven.

The outer reporting caller failed *after* this maintained gate passed because
it used a source-only directory inventory on an intentional Node runtime link.
That failure remains failed. The separate result audit authenticates the typed
link, exact runtime target, 219 runtime files and whole result membership. Source
closure checks still reject symlinks. The maintained API receipt itself does not
snapshot its binary or compiler lock sidecars; the parent audit does. No result
is promoted to constructor/deployment cryptography or full CI acceptance.

The isolated proof continuation preserves that source head and all results. It
adds an explicit tuple-guard cohort to the existing model, native-preflight and
proof owners: 16 complete sources, all 48 intact exported models and all 105
cases. Historical default 16-case and boundary 30-case modes remain independent.
Both the ordinary six-source scope gate and the complete 18-source tuple scope
(109 expression origins, nine HOST refusals) precede model admission. Every
intact model must have automatic integer k in 1..8 before keys. The new observer
and typed constructor adapters are qualification infrastructure, not a new
semantic feature or a generated-code rewrite.

Entry points are `check_constructor_tuple_equality_models.py`,
`check_constructor_tuple_equality_proof_preflight.py` and
`check_constructor_tuple_equality_proof.py`; the complete owner supports
`--preflight-only` to avoid key/proof effects. HOST orchestration tests prove only
stage ordering and early refusal. They are not compiler or cryptographic results.
Actual models, immutable native proof consumers, fresh key generation, selected
proofs/verification and separate DEFAULT strict applications remain open until
current execution receipts and independent review establish them. Seven binding,
input/output/preimage, corruption and wrong-verifier negatives remain required
per positive case. The common proof/application consumer stays unchanged.

The later `advance` circuit may be identical across source roles and constructor
states; count actual key identities instead of claiming 105 distinct circuits.
Final-state TS comparison observes the last iteration and overwrite order, not
every intermediate guard result. Cloned-caller equality assertions alone are
weak ownership evidence; by-value API signatures, generated capture cloning,
compilation and executed reuse provide the relevant evidence. Current new-cohort
API/proof CI wiring remains integration work; the historical CI path is retained.
No hosted run, actionlint pass, Linux qualification or broader-goal completion is
claimed here.


## Local proof execution and closeout boundary

Frozen signed execution head `5758c0244` passes the current 105-case API gate
(1,260 calls and 630 public VM replays), both complete source scopes, all 48
intact exported models at automatic k6, and 16 immutable native proof consumers
covering all 105 preflight cases. The separate proving stage executes 32 keygen
invocations, 105 standalone proofs with immutable verifier calls and 105
independently proved funded DEFAULT-strict transaction applications. Every case
passes all seven binding/cryptographic negatives and an unchanged-state
`IntentAlreadyExists` replay refusal: 735 negatives and 105 replays in total.

The read-only parent audit checks all 49 proving commands, closed output-root
and material memberships, all 105 proof/transaction/result triplets, frozen
inputs, parameters, SDKs and binaries. Both 105-element proof and transaction
artifact hash sets are distinct. This is an artifact/result audit, not a parent
cryptographic rerun. Independent actual-result review accepts this bounded epoch. Retained
receipts and reviews follow the maintainer evidence availability policy.

All positive lanes have one later `advance` ZKIR circuit shape, and the 32
keygen runs yield only two distinct verifier keys (`advance` and its wrong-key
control). Initial stored values are 7 or 11. These counts do not establish 105
distinct circuits, constructor/deployment proofs, a wallet lifecycle or network
qualification. Verification occurs inside each immutable consumer; the saved
proofs were not independently reverified in a separate process.

The closeout isolates the existing preflight geometry and membership assertions
into small private Python check functions so direct HOST tests exercise the
actual checker. Geometry still refuses before fixture-spec loading; membership
still refuses before material/consumer runners. Predicate bodies, error texts,
cohort counts and order are preserved. Tests cover both the 24-model boundary
and 48-model tuple cohorts, accepted automatic k1..8, malformed/noninteger/
over-cap geometry, missing/extra/duplicate models, wrong roles/entries, producer
membership and observed-k reconciliation. They are admission controls, not new
model or proof executions. Production compiler, runtime, shared proof consumer,
source/case/proof fixtures and public API stay unchanged.

This documentation/test successor has its own source identity. It does not
repin or relabel the frozen 5758 execution receipts. New-cohort CI wiring and
native Linux/full required integration gates remain open; local arm64 results
cannot satisfy them. Both valid-Compact alias transport gaps remain explicitly
refused. Forge remains deferred, and the broader Rust AST goal remains incomplete.


The independent actual-result review accepted the frozen 5758 local execution
with no P1/P2 findings. It recomputed the proof audit's 20,845 pins, all 49
streams, complete memberships and relevant upstream strict/mock features.
Negative assertions establish refusal, not a separately discriminated cause for
every rejection. The reviewer did not deserialize persisted transactions or
re-execute cryptography. Required parameter material was verified and stayed
hash-stable, but `FetchMode::Synchronous` permits fetching and the network was
not blocked: there is no OS-level zero-network or parameter-load trace. Fifteen
superseded Cargo output files are historical; the immutable lane snapshots and
recorded build streams preserve their actual executable identities. These limits
remain part of acceptance rather than reasons to relabel or rerun the result.


The closeout review caught an eager `producer['sources']` helper argument that
changed malformed-receipt error precedence. The correction passes the whole
producer and keeps the lookup inside the original short-circuit predicate.
A direct regression test retains the count/spec ValueError before a missing
producer key, and the original KeyError only after those counts are valid.
A source-AST test also guards the order in `main`: geometry, spec loading,
membership, then the first maintained runner. Earlier read-only Git/hash probes
remain outside that assertion. Constructor logic runs natively; constructor
state insertion and fee funding are privileged fixture setup, not proved setup.
