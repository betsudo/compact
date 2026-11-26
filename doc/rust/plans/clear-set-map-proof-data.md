# Clear, Set and Map standalone proof-data qualification

The optional standalone proof-data API now covers the admitted Tiny Clear/Set
families and a root `Map<Field, Field>` membership read. Default generated APIs
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
Original exported `lookup(Field) -> Field` is a separate remaining helper gap.

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
capture-only legacy SDK tuple repair. Public Clear/Set k13 proof execution and
a real default-k8 refusal remain pending; an interrupted Tiny attempt supplies
no refusal credit. The shared-SDK, captured-before-state, SDK1, Linux and chain
limitations above still apply.

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
