# Clear, Set and Map standalone proof-data qualification

The optional standalone proof-data API now covers the admitted Tiny Clear/Set
families and root `Map<Field, Field>` membership/Field-valued Lookup reads. Default generated APIs
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
The exported `lookup(Field) -> Field` helper is now implemented on the local
Lookup source line. Its public-runner integration remains a separate unrun path
until fresh qualification, as recorded below.

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
