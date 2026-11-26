# Portable recorded proof-data checks

This package exercises the public recorded helper API through actual generated
Map member/Lookup/emptiness, Tiny clear and Tiny set contracts, then runs a checked SDK producer
and a separately implemented verifier in separate processes. It uses the pinned
Ledger 9.1.0.2 / transient-crypto 3.1.0 SDK source in this workspace. It does not
implement a second cryptographic verifier.

The backend also admits a zero-argument Boolean `is_empty_proof_data` facade.
Its [separate native/proof and staged TypeScript qualification](../../doc/rust/plans/clear-set-map-proof-data.md#map-emptiness-checked-proofs-and-staged-typescript-parity-26-november-2026)
now passes with four fresh proofs and twenty independent SDK verification returns.
The public runner now accepts `--case map-is-empty` and reuses that capture adapter.
A fresh default-lane public CLI run now passes all 18 phases; its source-bound
qualification is recorded below, separately from the staged TypeScript run.

Use Rust 1.99.0, Python 3.11+, the pinned SDK sources already available to offline
Cargo, and a self-contained installed normal Compact compiler with the helper flags. A raw
Cargo source facade requiring an external Scheme shim is not supported by this
runner; use its normal installed compiler package. The
runner requires 50 GiB free, two build jobs and default thread stacks. It creates
an exclusive target under the selected fresh output directory; retain run.json
and outputs, then remove that recorded rebuildable target when no longer used.
On macOS it selects the installed Apple clang and SDK using xcrun. Other hosts
need their normal native linker; Linux execution has not been qualified here.

```sh
python3 tools/compact-rust-proof-data/run.py --compiler /absolute/current/compactc \
  --case map-member --out /absolute/fresh-map-run
python3 tools/compact-rust-proof-data/run.py --compiler /absolute/current/compactc \
  --case map-lookup --out /absolute/fresh-lookup-run
python3 tools/compact-rust-proof-data/run.py --compiler /absolute/current/compactc \
  --case map-is-empty --out /absolute/fresh-emptiness-run
python3 tools/compact-rust-proof-data/run.py --compiler /absolute/current/compactc \
  --case tiny-clear --max-k 13 --allow-large-k --out /absolute/fresh-clear-run
```

The regular lane defaults to measured k <= 8. Tiny clear/set currently require
an explicit larger lane; the maximum allowed opt-in is 13. Both producer and
verifier independently remeasure the actual IR and apply the selected limit
before parameter/key/proof readers. The runner first writes an immutable
`check-input.json` with `parent: null`, runs the actual SDK check/model and lane,
then copies parent DATA and writes a distinct immutable `prove-input.json`. It
never overwrites check evidence or copies the parent for an over-budget case.
Earlier SDK source-integrity hashing of static DATA performs no parameter decoding. An over-budget model is a refusal, never
an automatic larger retry. Counts and sizes are measured at execution.

The normal CLI invocation preserves the intact pinned originals and includes
`--target rust --target ts --feature-zkir-v3 --skip-zk
--ts-runtime-profile released-0.21.0 --rust-require-recording
--rust-standalone-proof-data`. Generated Rust and Cargo files are copied without
editing. Separate native capture crates preserve each generated crate's
`crate::ledger_slots` layout. No TypeScript execution is implied by emission.

Map membership's three supplemental cases are present key 1, absent key 1, and present
2^128+1. They compare native, ordinary recorded, and helper results, complete
state, gas, effects and VM replay. Tiny preserves constructor value 64, the
original zero Bytes32 witness, original refusal messages and a separately
labelled nonzero witness. Set uses Field 5 after a proper clear. Every actual
helper/witness is called once per fresh route. The adapters serialize official
canonical tagged preimages and expose the real public-read stream separately.

Map emptiness uses the same intact eight-export Map fixture. Its four supplemental
states are empty -> true, default-zero singleton -> false, nonzero singleton ->
false, and actual reset -> true. The `map-is-empty` input profile has no declared
input scalars and exactly one canonical Boolean output scalar (LE32 zero or one).
It uses the regular default8 lane; all model dimensions and proof sizes must be
measured by a fresh run. The existing capture body is reused without duplication.

`produce check INPUT OUT` checks every case and emits measured model dimensions.
`produce prove INPUT OUT` additionally derives the measured own-k SRS from the
pinned official parent14 bytes, creates one fresh canonical initialized keypair
and checked proof per case, and compares all SDK-returned PI fields/skips with
its complete projection. One fixed supplemental Impact constant mutation must
produce a canonical initialized byte-distinct same-k control key; ineligibility
stops immediately, with no search or control proof.

`verify INPUT PRODUCED REPORT` never reads private preimages, returned PI vectors,
model check vectors or prover keys. It independently projects each statement
from the exact public IR and admitted actual public-read stream. It authenticates
the parent14 hash, derives the entire canonical own-k RAW object and requires
exact own-byte equality before unchecked decoding. Each proof gets two actual
SDK positive calls (own and official default14) and three actual negative calls:
changed caller binding, canonical-outer corrupted proof, and the current valid
initialized distinct same-k control VK. Admission or decode failures do not
count as SDK refusals. The corrupt proof's internal rejection layer is unknown.

All bundle paths are relative, contained, nonsymlink paths with explicit size and
SHA256. Distinct roles require distinct paths; identical public bytes may be
snapshotted at separate paths. In particular the runner copies raw_ir separately
from generation.files. Bundle loading and complete input/produced role preflight
are metadata-only. Generation payload hashes belong to the actual capture/build
runner receipt; the verifier does not traverse arbitrary generation payloads.
The public stream is admitted data: this tool does not regenerate the before-state
or evaluate private witness hashes. Caller fields are explicit canonical nonzero
distinct inputs, with public deterministic test defaults; use fresh values when
separate caller bindings are needed. Output/preimage/key/proof files are private.

The SDK gate selects the proof package's reachable resolve graph and separately
binds actual compiler artifacts, targets and enabled features. Workspace metadata
may include unrelated owners; workspace membership is not selected-graph or
compiled-graph credit. Current SDK source revisions, registry archive checksums and
physical source bytes are checked before capture/proofs and after completion.
Registry source bytes and full membership are authenticated against the cached
`.crate` whose SHA256 equals Cargo.lock; `.cargo-checksum.json` is optional and
secondary. A missing or unauthenticated archive is a refusal.
The compiler image/hash is recorded separately from the harness Git revision.
Its source revision remains UNKNOWN; a version string does not establish it.
Bounded harness/runtime sources, original fixture, generated originals, capture
overlay and executed producer/capture/verifier images are checked before and
after use. An Owner replacement prevents all closeout writes under that path.
The temporary native capture manifest and resolved lock are retained; its
external lock tuples must be a subset of the unchanged workspace lock.

## Executed public Map runner, 26 November 2026

A fresh local macOS arm64 run at signed source
`03efbac5e24f5e2975e0afafc81bd6aaac463c58` completed all 18 phases.
The three supplemental Map cases produced three fresh 3,845-byte checked
proofs at measured k7/62 rows excluding custom gates and lookups. The separate
verifier reconstructed 17 fields and five skips per case; all 15 actual SDK
calls returned as expected (six acceptances, nine refusals). Captured before
state and shared SDK cryptography retain the limitations above.

The stopped runner SHA256 is
`29beba9f59c2a348dfadb82e8d0841aa33bd04c9c7c3218f161b819420a1fbd5`;
the verification report is
`44c9d9738e8e2971346a6d251944c86e744690f7b9f7c0f4ce7e6cb2048bbee8`.
These bind the executed source and outputs; documentation successors do not
transfer execution credit to changed code. The first capture-gate failure is
retained: native runtime dependencies require five exact legacy SDK tuples
alongside the current SDK. Only capture mode admits those tuples; SDK-only
pin, artifact, feature, lock and physical-source guards remain strict.

## Executed public Tiny Clear lanes, 26 November 2026

At signed source `c247a972610c0be0b0b5c139889cc5ae9f76f599`, using the
fresh normal installed compiler 0.31.221, the default max8 lane completed real
native capture and both SDK checks, measured k13/3,904 rows, then refused with
`measured k exceeds explicit budget`. No parent copy, prove input, key/proof or
verification output was created. This is an expected size refusal.

The explicit max13 lane completed all 18 phases and created two fresh
5,413-byte checked proofs: original zero witness and a labelled nonzero
supplemental witness. Each statement has 55 fields and 15 None skips. All ten
separate-process SDK calls returned as expected: four acceptances and six
refusals. Independent stopped-result review SHA256 is
`020d483440bbc40bfee3f7c60a397e3294ad3c86b8d44effdb6657beac8e9744`;
large-run verifier report is
`f155195ac7f19474848e1fc148f10fce1d39df3984b1b417b7ffcca1ed461d66`.
The initial interrupted attempt and unavailable-constructor compile failure
remain recorded; the corrected adapter uses the generated `From<W>` API.

Map, Clear and Set retain their separate executed source/artifact identities.
Shared SDK cryptography, admitted before-state and the other limitations above
still apply. Native Linux remains unrun.

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
qualification, separate from the portable CLI's 18 phases. It uses Node24.14.0,
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

Focused tests (run only after source review and dependency admission):

```sh
cargo +1.99.0 test -p compact-rust-proof-data --locked --offline --tests --jobs 2
python3 -B tools/compact-rust-proof-data/tests/sdk_gate_test.py
python3 -B tools/compact-rust-proof-data/tests/runner_test.py
```

Execution observations cover public Map member, Map Lookup, Clear and Set above;
Lookup's independent stopped audit passed the scoped checks above. Released Compact 0.36.0
with Ledger 9.1.0.1, Linux, funded transactions, chain inclusion and a broad
contract matrix remain separate qualifications. Private local receipts and
signed Root authorizations are not required public dependencies.

## Executed public Map emptiness runner, 26 November 2026

Signed harness `234d16bfb3cc00db243dd0582829d10c334a5808` completes all 18
public phases in 361.455 seconds on macOS arm64. The default k<=8 lane measures
k6/41 rows and produces four new, distinct 3,845-byte checked proofs, disjoint
from the earlier bounded run. Complete returned PI/skips compare at 18 fields
and six None skips per case. The separate verifier makes 20 actual SDK calls:
eight accepts and twelve expected refusals.

The installed normal compiler uses retained Rust819f and Schemec247 components,
with the existing Nix Chez dependency; neither is relabelled as harness234d.
All 773 package files match before/after. A supplemental capture-SDK source
postguard also matches its initial gate; this is outside the 18 runner phases.
Inherited non-root patch warnings remain in three capture Cargo phases.

Signed qualification SHA256:
`a597cc1eeac72bdb7bba218821386a156276a99a9dfc3ded61082fb4c0c100a6`.
Independent stopped-run review passes. Exact executed SDK/capture images and all
source, capture, model, proof/key/parameter and raw evidence are retained; the
completed Cargo target is removed. This fresh public run emits TypeScript but
does not rerun Node. Original-case counts, SDK1 matrix, off-Nix, native Linux,
EntryPoint/funded/transaction/two-contract gates remain separate.
