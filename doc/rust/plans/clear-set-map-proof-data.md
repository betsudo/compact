# Clear, Set and Map standalone proof-data qualification

The optional standalone proof-data API now covers the admitted Tiny Clear/Set
families and root `Map<Field, Field>` membership, Field-valued Lookup and
zero-argument Boolean emptiness and Uint64 cardinality reads. Default generated APIs
and ABI54 remain unchanged. A helper captures the actual recorded call once;
explicit caller binding and communication randomness turn that capture into a
preimage. Capturing data does not itself generate or verify a proof.

The intact pinned upstream eight-export Map fixture remains unchanged. The
frontend repair preserves the terminal Unit return of `resetToDefault`,
`insert`, `insertDefault` and `remove`; it does not replace those operations
with a manufactured result. This returned-Unit repair is tracked in
[#795](https://github.com/betsudo/compact/issues/795). The membership helper
accepts only its declared
Field key and Boolean result. Unsupported shapes continue to have no helper.
The exported `lookup(Field) -> Field` helper and its public runner are qualified
on their separately recorded source line below. The newer emptiness facade now has
separate native, checked-proof, independent-verification and staged TypeScript
qualifications below. Its public `--case map-is-empty` path now has a separate
fresh 18-phase regular-lane qualification.

## Map emptiness host and emission gate, 26 November 2026

Signed source `819f4129422183ea12bea14434b90d35d7709b80` adds
`is_empty_proof_data(context) -> RecordedProofData<Private, bool>`. Admission is
limited to public zero-argument, action-free direct or exact expression-return
reads of one root `Map<Field, Field>`. The shared facade, runtime, ABI54 and API1
are unchanged. When a borrowed witness handle is generated, a witness-free read
retains the helper on both handles without Clone or witness bounds.

The complete locked/offline backend run passes 2,471 tests across 56 harnesses,
with seven existing ignored tests and no failures. Strict all-target/all-feature
Clippy passes with warnings denied. The first failed run at `9149caf79` is retained:
a new renamed-slot positive fixture moved its index to 15 but retained path 0,
which the classifier correctly refused. The correction changes only that test's
physical path and adds the equivalent empty-root control; no production guard or
original fixture was weakened. Independent source and stopped-run reviews pass.

Fresh normal Rust/TypeScript generation produces 19 artifacts, all eight original
native/recorded/observed exports and exactly three optional Map read helpers:
`is_empty`, `member` and `lookup`. All 9,888 committed source files and the retained
773-file compiler package match before/after. The new signed Rust facade is built
from `819f41294`; the separately authenticated Scheme component remains `c247a972`.
This one-off generation explicitly selects that component. It does not establish
a new self-contained portable compiler package or TypeScript runtime execution.

The intact 1,364/1,500/876-byte original fixtures and ten original assertions are
unchanged. This adds no globally new original-case execution. The later
native/model/proof/TypeScript gate below records the measured completion; the
initial host snapshot alone does not transfer that evidence. Portable-runner,
SDK1 matrix, funded transaction and Linux qualification remain separate.

## Map emptiness checked proofs and staged TypeScript parity, 26 November 2026

The newer source-bound qualification separates generation at `819f41294`, native
capture at `f793412eb`, and producer/independent verifier at
`586dddb8815d34b5545f524ce00a2ef1164594e5`. All original Compact fixtures and
assertions remain intact. These are supplemental API cases, adding zero unique
original-test execution credit.

| Gate | Actual local arm64 result |
| --- | --- |
| Native capture | Empty, default-zero singleton, nonzero singleton and actual reset return true/false/false/true through native, recorded and helper routes. Whole state/context/gas/private91 and all six public operations agree; four official VM Verify replays pass. |
| SDK model | Four genuine checks measure k6 and 41 rows excluding custom gates/lookups. The statement has 18 projected fields and six actual None skips. This check API has no returned-PI accessor. |
| Checked producer | Four fresh, distinct 3,845-byte proofs compare complete SDK-returned public inputs/skips with the projection. Own k6 parameters derive from the authenticated official parent14; fresh subject/control keys use the same SRS. |
| Independent verification | Twenty actual SDK returns: eight accepts (own/default14) and twelve refusals (changed binding, canonical-OUTER corrupt proof, valid initialized distinct same-k control VK). No producer PI/preimage/PK body or private-hash evaluation is used. |
| Staged generated TypeScript | Strict TSC and four Vitest tests pass, with eight delegated genuine SDK checks and eight awaited queue flushes. Unit/Boolean/LE32 results, all six ordered operations/values and four query-only gas dimensions match native observations; seven data-mutation controls refuse. |

SDK/refusal checks pass 38 tests across seven harnesses without ignores; strict
all-target/all-feature Clippy passes. The first SDK compile attempt's new-test
`Fr +=` error is retained; a one-line test-only supported-addition correction
preserves the mutation and assertions. Cargo reports the local `proof-checks`
tools as unoptimized; their SDK dependency artifacts retain their actual profiles.

Signed joint qualification SHA256:
`e39c3b0d158c7260d8ee191184fd100771df481a0f377d1b00785bb4f32d280d`.
Separate reviews cover native/SDK model, actual TypeScript public parity and fresh
proof/verification results. C authored verifier math from public IR before the
producer and never reads private producer bodies. B later reviewed C's source;
no blind-producer claim is made. C's producer-source guard checks Git identity,
mode and size; Root separately attests all 9,892 physical source files before and
after verification. The public sandbox contains no private PI, preimage or PK
bodies. Retained signed tools, proofs/keys/parameters and all receipts survive
cleanup of the completed Cargo caches and archived Node stage.

This Rust SDK3.1/Ledger9.1.0.2 gate and staged TS runtime0.21/onchain4.0.1/ZKIR3.2.0
gate do not establish the full released Rust SDK1 matrix or native Linux.
Worker/network/independent heap facts remain unknown. No whole-ledger, complete
cross-language PI/preimage/proof-byte equality, typed observed-call wrapper,
new funded application/replay, EntryPoint or real two-contract transaction is
qualified here. The bounded operators above retain their original source scope.
The portable `run.py --case map-is-empty` path subsequently passes a fresh run at
`234d16bfb3cc00db243dd0582829d10c334a5808`: all 18 phases, four new 3,845-byte
checked proofs, k6/41 rows, 18 fields/six skips and twenty independent SDK calls
(eight accepts/twelve refusals). The installed compiler retains Rust819f/Schemec247
component identities and Nix dependency. All 773 compiler files match before/after;
a separate capture-source postguard passes outside the 18 phases. Actual review
passes and the completed Cargo cache is removed with all evidence retained.
Public qualification SHA256:
`a597cc1eeac72bdb7bba218821386a156276a99a9dfc3ded61082fb4c0c100a6`.
No TypeScript runtime, released SDK1 matrix, Linux or chain qualification is
transferred from this public run. The next bounded API is Map.size Unit→Uint64,
whose circuit dimensions remain unmeasured.

## Executed Map member qualification, 26 November 2026

The source-bound local qualification is at
`827963edc15b8d58c2e0b752c37b5eafad321c7d`, using the current SDK3.1 / Ledger
9.1.0.2 graph on macOS arm64. It is separate from the released Compact0.36.0 /
Ledger9.1.0.1 interoperability matrix.

| Gate | Actual result |
| --- | --- |
| Native, recorded and optional helper | Three supplemental cases: present key1, absent key1, and present key2^128+1 return true, false and true. Gas and five-operation streams agree; non-Clone private state, no-witness execution and complete state/context preservation pass. |
| Codec and runtime refusal | Canonical EOF/re-encode, wrong-tag, appended/truncated-input refusals and official VM replay pass. Missing lookup retains `Execution(ExpectedCell(null))`. |
| SDK model | All three fresh checks measure k7 and 62 rows excluding custom gates and lookups. |
| Checked producer | Three fresh proofs return all 17 expected public-input fields and five None skips, exactly equal to the producer's independent projection. |
| Separate verifier | All 15 actual SDK calls return as expected: own7 and real current default14 accept each proof; changed binding, canonical-outer corrupt proof and a fresh valid initialized distinct same-k verifier key refuse each proof. |

The verifier has a separately authored statement evaluator and runs in a
separate process. It reads admitted public transcript observations and typed
Field/Boolean inputs, without reading producer PI vectors, check reports,
private preimages or prover-key bodies. It shares the SDK cryptographic
implementation. The captured before state is not independently regenerated.
Corruption has a canonical tagged outer Proof encoding; its internal rejection
layer is unknown. Default-parameter negative controls were not run.

Signed final qualification SHA256:
`0f2c2c80c0fb18597cd9d8da71b81516566ac9a0ee02efe2077189d20d7dc29d`.
The separate stopped-result review is
`bdefd6d7cde62f877f80905be2646f0e0ab1a786a72180bac4549af82a8aed05`.
These identities describe that executed source and its retained artifacts;
they do not transfer qualification to a later source revision.

The selected seven unchanged original export compiler/metadata callbacks also
have scoped local qualification: seven compiler successes, empty raw streams,
real Acorn parsing and 28 observed original assertions. Their adapter uses the
default TypeScript0.18 environment. A final telemetry-title mismatch remains
preserved as the operator's first RED; the exact stopped test identities were
independently joined without changing assertions or rerunning tests. This is
not full45-module coverage, execution of the generated TypeScript runtime, or
new globally unique original-case credit.

## Fresh public Map runner, 26 November 2026

The portable runner separately completed all 18 real phases at signed source
`03efbac5e24f5e2975e0afafc81bd6aaac463c58` on local macOS arm64. It generated
three fresh checked proofs of 3,845 bytes each, measured k7/62 rows, and passed
all 15 separate-process SDK verification outcomes: six acceptances and nine
refusals, with 17 reconstructed fields and five skips per case. This is new
public-tool execution, separate from the historical private qualification.

Stopped run SHA256 is
`29beba9f59c2a348dfadb82e8d0841aa33bd04c9c7c3218f161b819420a1fbd5`;
verifier report SHA256 is
`44c9d9738e8e2971346a6d251944c86e744690f7b9f7c0f4ce7e6cb2048bbee8`.
The runner preserves its first capture-gate failure and the subsequent finite
capture-only legacy SDK tuple repair. The first interrupted Tiny attempt supplies no refusal credit. A later
source-bound public Clear qualification is recorded below. The shared-SDK,
captured-before-state, SDK1, Linux and chain limitations still apply.

## Fresh public Clear lanes, 26 November 2026

At signed source `c247a972610c0be0b0b5c139889cc5ae9f76f599`, fresh installed
compiler0.31.221 generated the intact Tiny fixture. The default8 lane completed
native capture and both real SDK checks, measured k13/3,904 rows, and refused
before copying the parent or creating prove input, keys, proofs or verification
output. Its exact failure is `measured k exceeds explicit budget`.

The explicit13 lane completed all 18 phases and produced two fresh 5,413-byte
checked proofs. Original zero and labelled supplemental nonzero witness cases
each reconstruct 55 fields and 15 None skips. All ten actual SDK calls return
as expected: four acceptances and six refusals. Independent actual review is
`020d483440bbc40bfee3f7c60a397e3294ad3c86b8d44effdb6657beac8e9744`;
verifier report is
`f155195ac7f19474848e1fc148f10fce1d39df3984b1b417b7ffcca1ed461d66`.
The unavailable-constructor compile failure and earlier interrupted attempt
remain preserved; the native adapter now uses the generated `From<W>` API.
Public Set explicit-lane qualification is recorded below. Map, Clear and Set
keep their own executed source/artifact identities; these supplemental checks
add no globally new original-case credit.

## Executed public Tiny Set explicit lane, 26 November 2026

At signed source `fba76460ae0d62115fa8038cf4d79415348079f5`, the installed
compiler0.31.221 explicit max13 lane completed all 18 phases. Both actual Set
cases measured k13 / 3,887 rows and produced two fresh 5,413-byte checked proofs.
The separate verifier reconstructed 45 fields and 12 None skips per case; all
ten SDK calls returned as expected (four acceptances, six refusals).
Independent stopped-result review is
`8a2a0b8b6a6c13ed8114e30c5988d6447d8e4dd338d9fa08620e3ad4a6e8555c`;
verifier report is
`92cadbcc2db52d020bac3475d8efe066a0617c9e6e7117b4b70c44fa62b8e441`.
This explicit Set run supplies no Set default-lane refusal credit. It retains
its own source/artifact identity and the same shared-SDK/admitted-before-state,
Linux/released-matrix/funded/chain limitations.

## Executed public Map Lookup runner, 26 November 2026

`--case map-lookup` selects the unchanged eight-export Map fixture and the
actual generated `lookup_proof_data(Field) -> RecordedProofData<Private,Field>`
method. Use an installed compiler containing that helper; a compiler version
string alone does not attest its Rust component. The public adapter compares
native/ordinary-recorded/helper routes on key 1 -> 7, insertDefault key 1 -> 0 and
key 2^128+1 -> Field 2^129+7. It preserves non-Clone private state 91, exact state/context/gas
and complete recorded operations, official VM replay, canonical tagged codec
and real public streams. Missing lookup keeps exact
`Execution(ExpectedCell(null))`, with no successful helper carrier. It includes
no original-test generated crate, observed-call/ledger-transaction feature, or
rewritten original assertions. All three export cases are supplemental.

A fresh public runner execution at signed source
`8d339b5029a13922d240fa47eca18041a7f17f89` completed all 18 phases in
355.155 seconds on local macOS arm64 with the current transient-crypto3.1.0 / Ledger9.1.0.2
source graph. All three supplemental cases freshly measured k7 / 60 rows
excluding custom gates and lookups and produced three new 3,845-byte checked
proofs. Complete returned SDK public inputs/skips matched the producer's
projection. The separately implemented verifier reconstructed 15 fields and
four None skips per case; all 15 actual SDK calls returned as expected: six
acceptances and nine refusals.

This public run is distinct from the earlier staged native capture at2bf2432
and SDK producer/proof work at8399fd69. It uses its own fresh generation,
captures, keys and proofs; earlier private receipts supply no public execution
credit. The three export cases add zero globally new original-case credit.
Default measured k <= 8 and no automatic larger retry remain unchanged.

Stopped public runner SHA256:
`ae2419750f090110f9a598b29685ce45b1dd3df528fc48e515857dd7c0e599d2`.
Separate-process verification report SHA256:
`424d19064e1ac7bd568fccdb60d4a9003092429e88d0f0978200d7ad22ac61e9`.
The independent stopped actual audit passed 399 checks with no producer
PI/preimage/check/model/PK body access or SDK rerun (report SHA256
`c37cee9782e45a2f43092730e2b6941b3f26f01fad072ed9159f5cd3334ee8eb`).
A separate capture-graph postguard matched its initial gate; it is outside
the 18 public runner phases.
Shared SDK cryptography, admitted rather than regenerated before-state,
unknown internal corruption rejection layer and unrun default-parameter
negative controls retain their existing limits. This run does not execute
TypeScript, observed/EntryPoint calls, funded/chain actions, native Linux or
the released Compact0.36.0 / Ledger9.1.0.1 matrix. Public Map member, Clear and
Set keep their separate executed source/artifact identities.


## Staged generated TypeScript Lookup parity, 26 November 2026

A separate local macOS arm64 qualification at documentation head
`33c5f481809a79d409cdfade857af2705be4a0c1` executes the exact TypeScript
artifacts generated by the public `8d339b50` run. This is a staged Node
qualification, separate from the portable CLI's 18 phases. It uses Node 24.14.0,
strict TypeScript6.0.2, Vitest4.1.11, released runtime0.21.0, onchain-runtime4.0.1
and ZKIR3.2.0, with a freshly authenticated read-only dependency closure.

All four tests pass: three supplemental Field cases plus a missing-key refusal.
Six actual delegated SDK checks and seven awaited queue flushes complete. For
all three positive cases, canonical Field keys/results, all four ordered public
operations including every operand/value, and all four lookup-only gas deltas
match the native capture. Cumulative setup cost is recorded separately; it is
not compared with Rust's per-invocation cost. Checker results contain five JavaScript
`undefined` entries for setup and four for lookup; these are separate from the
complete Rust proof public-input vectors.

The failed lookup returns `CompactError: Error: expected a cell, received null`.
An explicit observed-message classification relates it to native
`Execution(ExpectedCell(null))`; no error-byte or cryptographic-internals
equality is inferred. It adds no checker or successful trace/result and preserves
the public empty Map and original context gas.

The first failed test run remains preserved. Its authored setup-result `[]`
expectation was corrected only after a direct reference executed the unchanged
fixture's APIs under official Compact0.36.0 and the current compiler: both return
`undefined` for insert/default setup, then the expected Field7/0. Original
Compact fixtures, assertions and refusal annotations were not rewritten.

Signed qualification SHA256:
`d762ba2f94f349fc78cd86b6e27e318cd6d399b903a6c47c64e0fdcabcc71150`.
Independent public-data review78 checks and seven mutation controls supplement
the disclosed Source-author stopped-data audit424 checks. The three cases add
zero globally new original cases. No whole-ledger encoding, preimage, complete
PI or proof-byte equality, new TypeScript proof, funded/replay/EntryPoint,
native Linux or full released Rust SDK1 matrix qualification follows.

## Reproducible public harness and cost policy

The [portable proof-data tool](../../../tools/compact-rust-proof-data/README.md)
provides the cohort's relative input contract and commands. Its own current
source and fresh outputs must be qualified independently of the historical
private receipts above. Producer and verifier statement construction remain
separate; only input and canonical codec plumbing are shared.

Keep the existing automatic k<=8 limit. Current Map membership measures k7;
current Clear/Set qualification measures k13 and belongs in an explicit slow
lane. Measure the selected circuit before reading parameters or generating
keys. A size refusal is not a proof pass or a skipped original test. Use the
authenticated SDK static parent and clone/downsize; this does not create a new
trusted setup. Report actual metadata-reachable dependencies separately from
actual compiler artifacts when workspace metadata overapproximates a tool's
selected graph.

Normal generation uses `--rust-standalone-proof-data` together with
`--rust-require-recording` and `--feature-zkir-v3`. Released TypeScript output
also requires `--target rust --target ts --ts-runtime-profile released-0.21.0`.
Generation and native proof qualification do not establish execution of that
TypeScript output.

The broad migration remains incomplete: released SDK matrix compatibility,
remaining original suites, native Linux and applicable funded default-strict
application/replay gates remain open. Generated EntryPoint invocation and its
real two-contract qualification remain held. Ledger8, archived Forge evidence
and earlier failed attempts retain their original identities and scope.

## Map size native, proof and TypeScript qualification, 26 November 2026

The bounded `size_proof_data(context)` helper retains the checked
`BoundedUint<18446744073709551615>` result and Unit input. Admission requires a
public zero-argument/action-free direct or exact expression-return read of one
root `Map<Field, Field>`. One shared physical-slot predicate replaces the three
existing duplicated tails; operation, parameter and result guards remain in
their domain modules. Renderer/runtime, ordinary APIs, ABI54 and API1 are
unchanged. Both witness-free handles retain the helper without Clone bounds.

| Gate | Actual scoped local arm64 result |
| --- | --- |
| Host/emission `3f45a947` | 2,480 backend tests/57 harnesses, seven existing ignores, strict Clippy; fresh dual-target 19 artifacts/all 8 original exports/exactly 4 helpers. All 9,894 source files and 773 retained compiler files match before/after. |
| Native `82d57fc3` | Empty 0/default-zero singleton 1/wide-key two entries 2/real reset 0 through 12 native/recorded/helper calls; whole context/state/gas/private91 and 4 ordered operations agree; 4 official VM Verify replays. Pure Uint64 MAX carrier bounds and 12 codec refusal assertions pass; no maximum-cardinality Map execution. |
| SDK/public `e18ba099` | 49 SDK/refusal tests/eight harnesses, strict Clippy and 24 Python controls. All 18 public phases pass in 356.833s at default max 8. Measured k6 / 35 rows,12 fields/four None; 4 fresh distinct3,845-byte checked proofs compare full SDK-returned PI/skips. |
| Independent verifier |20 actual SDK returns:8 accepts own/default14 and 12 expected binding/canonical-OUTER-corruption/current valid initialized distinct same-k VK refusals. No producer PI/preimage/check/model/PK body or private-hash evaluation; shared SDK crypto. Captured before-state is admitted. |
| Staged TypeScript | Strict TSC/four Vitest cases,10 genuine zkir-v3 3.2.0 JS/WASM checks/10 awaited flushes on Node 24.14.0. All 4 Unit/Uint64 canonical LE32 results, all 4 ordered operation values/alignment and 4 query gas dimensions match native captures. Six metadata mutations refuse; these are not SDK crypto refusals. |

The public runner selects `--case map-size`, operation `size` and manifest
profile `unit-uint64`. Cardinality two is valid; canonical 32-byte unsigned
output must fit u64. The exact current 597-byte raw IR is retained separately
from the different released 0.36 template. No qualification transfers from
another Map family. All original Compact fixtures and ten assertions remain
unchanged; these supplemental API cases add zero unique original-case credit.

Signed joint qualification SHA256:
`1204ad5b71557dac5bc2f5fa04f73e1713e1b6b566ce460c83c6187f994b53fa`.
Independent actual host/native/SDK/public/TS audits and Root's full physical
source witnesses bind each epoch. C authored verifier math from public IR; B's
producer froze before B reviewed it. No later blind-producer claim is made.
Original 1,364/1,500/876-byte fixtures, signed executed tools and all
source/artifact/proof/key/parameter/raw evidence remain. Completed owned Cargo
caches are removed, and the Node stage is archived then removed without
touching borrowed dependencies. Worker/network/independent heap facts remain
unknown. Supplemental capture-SDK/full-source/773package postguards ran outside
the 18 public phases. Compiler release 0.31.222 does not relabel executed 0.31.221
Rust3f45/Schemec247 components. Full SDK1/native Linux/original-suite and
applicable funded/EntryPoint/real two-contract gates remain open.
