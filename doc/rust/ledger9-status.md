# Ledger 9 Rust AST compatibility status

Verified 19 November 2026. This page summarizes the public branch and the
limits of separate local qualifications. The [adoption plan](plans/ledger9-adoption.md)
and [upstream compatibility plan](plans/ledger9-upstream-compatibility.md)
retain the detailed, source-bound history.

## Baseline and publication

The language and test baseline is
[LFDT-Minokawa/compact main at `672bd0f`](https://github.com/LFDT-Minokawa/compact/commit/672bd0f226f47a7d036c5c074ce71fbe1af1925e).
Interoperability uses [released Compact 0.36.0](https://github.com/LFDT-Minokawa/compact/releases/tag/compactc-v0.36.0)
and the [Ledger 9.1.0.1 SDK matrix](https://github.com/midnightntwrk/midnight-ledger/releases/tag/ledger-9.1.0.1).
The later Ledger 9.1.0.2 client dependency update is tracked separately in
[#771](https://github.com/betsudo/compact/pull/771); it does not change the
contract-semantic baseline.

| Slice | Public state | Qualification boundary |
| --- | --- | --- |
| [Cross-contract foundation #774](https://github.com/betsudo/compact/pull/774) | Merged into `codex/rust-backend-ast` at `eb3b2adbb` | Checked call context and refusal foundation; ordinary generated public `pay` execution remains separate. |
| [Original-token Rust circuits #775](https://github.com/betsudo/compact/pull/775) | Open, rebased on #774 at `297ea0b17`; Rust proof qualification was still running at the last check | The complete green `867b25d88` run belongs to the earlier source head. Local proof/funded evidence for 20 unique circuits remains bound to its recorded source, not the rebased head. |
| Public generated A `pay` / B `transfer` | Signed local source `49ff23d9d`; unpublished | A source-bound offline gate produced fresh A/B V3 proofs, separately verified them, applied a funded default-strict two-contract transaction with positive Dust, and refused replay with unchanged ledger bytes on Darwin arm64. Native Linux, live-node and original decoy-transcript parity remain open. |
| [Native Linux gate #773](https://github.com/betsudo/compact/issues/773) | Open | Local Darwin arm64 proof and funded results do not qualify the required x86_64 Linux gate. |

Local bounded slices also include released-compiler host E2E cases and the
byte-exact original composable Events sources. The signed Events Rust source
gate emits Inner Rust and TypeScript and confirms that ordinary Outer Rust
refuses at the live-coordinator boundary without publishing output. A later
signed private check binds the generated Outer IR's pre-child emit and direct
call to the separately generated Inner IR, with changed event, receiver,
argument, method and child effects refused. The exact-head local gate passed
with 12 Inner and five private Outer manifest files; 13 focused contract-call
tests and library Clippy with warnings denied passed on Darwin arm64. This
private check does not publish an executable Outer. The
separately qualified public `pay` route does not execute this original Events
pair. Public original-suite parity, generated Events A/B Rust execution,
applied event ordering, the host-tool census, native Linux integration and live-node
qualification remain open. [Compatibility tracking #726](https://github.com/betsudo/compact/issues/726),
[coverage #727](https://github.com/betsudo/compact/issues/727),
[cross-contract calls #730](https://github.com/betsudo/compact/issues/730),
[event ordering #776](https://github.com/betsudo/compact/issues/776), and
[integration #732](https://github.com/betsudo/compact/issues/732) retain their
owners and follow-ups.

The pinned PM-15797 host E2E case also passed its original 1/1 assertion on
released Compact 0.36.0, with six physical manifest artifacts and drift
refusals, at a signed local head. Its required header check passed after a
review fix. This is Darwin arm64 host compilation evidence, not generated
JavaScript execution or a complete host-suite result.

## Later local original-suite and generated API checks

The byte-exact original composable Basic suite passed all 12 cases with
released Compact 0.36.0 and all 12 with the private source-generated TypeScript
path at signed local `4a574d8e2`. The source gate passed 12 checks, and test and
Outer source changes were refused. The ordinary public Outer remains refused;
this gate does not establish a public Rust API, proof or funded application.
Its detached-GPG receipt SHA-256 is
`18ab5f495c6b2c8c532261f68e8224b4aadc230114aad17aba953435fbd71113`.

The byte-exact original composable Storage suite passed all six cases with
released Compact 0.36.0 and all six with public source-generated TypeScript at
signed local `13dc44a12`. The public generated Rust crate passed seven tests:
six analogues of the original assertions and one recorded VM replay. The
10-check source gate, five physical manifest audits, two source-drift refusals
and zero-new-header check passed. Its detached-GPG receipt SHA-256 is
`0d44d26677f2ef3ee6a4b4968a32839280083ba39cf92229276d96875db21f3a`.
This `--skip-zk` suite stores contract values in collections; it does not
invoke another contract or establish proof/funded transaction parity.

The byte-exact PM-19252 host E2E case passed 18/18 original assertions on
released Compact 0.36.0 at signed local `62cf97f91`. Five drift controls
refused and the zero-new-header check passed. Its detached-GPG receipt SHA-256
is `02412fbae9fc1e171d54e63ba9f797c432dedf7abb03a444b8ce62c12ca3484c`.
This is host compilation and output-shape evidence on Darwin arm64; it does
not execute generated JavaScript, prove a circuit or cover the full host suite.

At signed local `0ddc77fac`, a private generated Events Outer continuation
consumed the sealed child call result and opening, emitted A's
`ShieldedSpend(n)` before B's Cell read, and passed two bounded in-memory
runtime scenarios plus 14 focused tests and library Clippy. The retained
v2 detached-GPG receipt SHA-256 is
`8db15ccf9eba951129605274dd837e2d363a00851529756dcf3fb5baeeacdcda`.
Ordinary public Outer still refuses. Trusted physical A/B identity, fresh
keys/proofs, funded default-strict application and replay, applied event order,
live-node execution and native Linux remain open.

The next signed local Events material package at `1ccb63150` binds the three
byte-exact original files, physical Inner Rust/TypeScript and private Outer
IR/ZKIR manifests, and freshly generated k=8 `innerEmit` (160 rows) and
`outerEmit` (208 rows) prover/verifier keys. Its 40-file identity verifies
with an external public keyring and exact digest; 17 physical and semantic
refusals pass. The package includes its compiler recipe and source-gate
attestations. An independent review found no actionable issue after the trust
anchor and membership fixes. Detached-GPG qualification receipt SHA-256:
`59a18777e7a513af0a491063d48088642510820e33df56158aabd1f95dfbd6a0`.
The ZKIR tool binary hash is a signer attestation; its bytes are not bundled.
The material package does not publish Outer or establish a proof, funded
transaction, applied event order, live-node or native Linux result.

The signed local Events stage-only companion at `2c7e4eaa3`, stacked on the
checked-entry head `b465dcce6`, binds 838 physical files: the exact prior
package, generated Inner and private Outer material, fresh keys, and the
tracked runtime source closure. Its stage identity SHA-256 is
`859d8cabae8a1391a82555a78ceb29a662ad089bb77e4d8f35064603656a4413`;
17 semantic and membership refusals pass. Independent review confirmed the
final exact-membership repair. Its detached-GPG qualification receipt SHA-256
is `0a813df62474c3527c7f7107a1930501ffa2325fb9670cf540710e211a24ea90`.
This package has no public Outer TypeScript or compiler manifest, Events
identity crate, or public `outerEmit` admission. The prerequisite checked-entry
source at `b465dcce6` passed 15 focused contract-call tests and library
Clippy with warnings denied; its signed receipt SHA-256 is
`e2d62179b2aa37eaf3cdf08b4f5db39d6b0dd9844eed901a209f84c595ec9f46`.
The generated entry is a source-order check, not a Cargo type-checked crate.

Separately, source-built private TypeScript generation passed 13 profile
checks, then the unchanged original composable `events.ts` body passed 2/2
TestChain cases using generated Inner and Outer TypeScript on Darwin arm64.
The superseding signed receipt, issued after independent audit, has SHA-256
`21892f815688f2b84817e21485dbe6857769c1d286e0c4d3d25694fb50383e3a`.
It pins all 35 retained artifacts, including the upstream verifier template
consumed by the test harness. The original run did not separately capture
exact command arguments, environment, and exit codes; the receipt discloses
reconstructed reproduction commands. This private result does not qualify
public generated Rust/TypeScript Outer, genuine proof, independent proof
verification, funded strict application/replay, applied event delivery,
live-node execution, or native Linux.

The signed v3 private Events companion at local `e6f94deda` adds exact
generated Outer TypeScript and its physical compiler manifest to the prior
material and binds 1,649 files across 84 directories. Its identity SHA-256
is `dd6c75875e40461713da289aae49f1f3c6f780433bb6de2f74b1030db5c90072`.
The exact-head verifier checks a separate GPG trust anchor and all local
checker dependencies before accepting the package; 18 negative controls
pass. Independent review found no remaining actionable issue. The
detached-signed receipt (`qualification-receipt-e6f94deda.json`, retained locally)
has SHA-256 `984f9cc7b6c645c4e618491e2fb7b29ff3adf5415873e1f2be346f0ea00e8d33`.
This remains private and has no public Outer admission.

At signed local `8302e704f`, a source-owned checked Events identity crate
binds the original A/B sources, physical generated artifacts, runtime closure,
verifier keys and resolver key bytes. An explicit-keyring gate verifies the
v3 package before and after copying it, then Cargo checks the identity crate
and generated Outer, runs identity Clippy with warnings denied, and refuses
three physical tamper probes. The independently reviewed
detached-signed receipt (`qualification-receipt-830.json`, retained locally)
has SHA-256 `27d7efd26f33395655c61e443e78b7d9d60839e7bd240a420cdb1e3bac1c5717`.
The generated Outer Cargo check passes; its nine strict Clippy diagnostics
are retained separately in that receipt's evidence.

A follow-up source fix at signed local `17bf098f8` exposes the generated
checked Events error types and removes unnecessary Field clones. Focused
source tests, fresh scratch generated Outer Cargo check and Clippy with
warnings denied pass on Darwin arm64. The local qualification (`qualification.json`, retained locally)
has SHA-256 `97a9831a396465d0ea6b8bf9e6d8a4b743c2d25c3f5ce454f1e4e29a50f114cd`;
its scratch identity pin substitution was unsigned. The reviewed source fix,
checked identity and v3 checker chain are integrated locally at
`c9ab32346`; the signed v3 package verifies from that head, but its combined
generated crate and complete candidate have not been qualified there.
Public generated Rust/TypeScript parity, genuine proof, independent proof
verification, funded default-strict application and replay, applied event
order, live-node execution and native Linux remain open.

The next signed local v4 physical candidate at `8ea671f79` combines the
reviewed checked Outer Rust, identity crate, full Outer manifest and Cargo
dependency with the exact prior TypeScript, IR, ZKIR, keys and runtime bytes.
Its identity SHA-256 is
`605312a3bc2dfbe82212921e402f2cc1705a015e0dc12e59ca6432af46facf8b`;
2,460 files and 119 directories are bound, including the unchanged signed v3
parent and its receipt. Eighteen refusal controls pass. Independent review
ran the explicit-keyring exact-digest verifier and found no actionable issue.
Its detached-signed receipt (`qualification-receipt-8ea671f79.json`, retained locally)
has SHA-256 `84b2e7532177e2adeb7cb2b8f345e31a02f095b45dac65c9b0958007ba7874e4`.
Status remains `candidate-no-cli-admission`. Cargo qualification of the
unchanged complete candidate, public admission, original generated Rust
execution, fresh proof and independent verification, funded strict application
and replay, applied event order and native Linux remain open.

At signed local source `7c86fc29f`, the unchanged signed v4 candidate now
passes identity and generated Outer Cargo checks and Clippy with warnings
denied on Darwin arm64. Four forced-rebuild controls refuse changed original
source, Inner verifier, Outer Rust and Outer TypeScript; a clean positive
preflight followed by dependency redirection refuses the changed Cargo file.
The copied package re-verifies before and after, allowing only two captured
Cargo lockfiles. Independent review checked the source and all 26 command
logs; the signed qualification (`evidence-7c/qualification.json`, retained locally)
SHA-256 is `4558cfb52d55623ca1ec099be59f4c790dd4979dac6f5b306dec8e010b9ff7a4`.
This qualifies local compilation of the physical candidate; original Rust
execution, public CLI admission, proof, funded application and Linux remain open.

The checked generated Rust Events consumer at signed local `60de2b12d`
executes `outerEmit` from a verified private copy of the unchanged v4 package.
The first call returns Field 1 and records two version-1 ShieldedSpend events
with exact payload and Outer-before-Inner emitting addresses. Two prepared
call preimages include the expected Outer child-result/randomness transcript;
four installed/resolved key mismatches refuse. The independently reviewed
signed qualification (`qualification-60d.json`, retained locally)
SHA-256 is `17cf3cace4f7c71b469bb970a43f9ec42dfee1cc9d22193576221149a7ea8cbd`.
It binds Cargo arguments, working directory, Rust/Cargo versions, selected
environment, exact source/template, logs and lockfile. This Darwin arm64
execution prepares preimages without proving or applying them. The original
second result after persisted state, applied event order, public admission,
fresh proofs, funded application/replay and Linux remain open.

At signed local proof source `6e56e0af1`, generated standalone Inner and
joint Outer/Inner calls produced one and two fresh V3 proofs. A separate
process at signed `0c4ecc1a6` authenticated the producer receipt and retained
artifacts, deserialized both transactions, and checked all three proofs
against physical verifier keys and transaction public inputs. Three changed
proofs, three changed public inputs and three wrong keys refused. Both
transactions pass Ledger 9 Real verification with balancing disabled.
Independent source and evidence review found no actionable issue. Producer
receipt SHA-256 is `38cece7c57352654547c5529513e02641c779206194f046ae9d98030da353a1d`;
the signed separate-verifier receipt (`retained-verification-0c4.json`, retained locally)
SHA-256 is `e59fd3eb18296f4e103cef3811114e1f0a7863147787f35b4cd6e5b4d582787d`.
The original signed package remains unchanged. These local arm64 transactions
are unfunded; default-strict funded application/replay, second persisted
result, applied event order, public CLI admission and native Linux remain open.

The Events-only CLI trust preflight at signed `053e85eb6` passes 45 unit
tests and 11 signed-candidate controls with prior-output preservation. Its
signed receipt SHA-256 is
`64700f73ba52497f47cc24ad8dd463c4b49a4aeaaf8d94e94514322c28ddb3f0`.
Independent review passed. This preflight always refuses publication;
ordinary Outer refusal here is a focused Rust unit test. Clippy excludes
the existing pay helper's argument-count lint and denies other warnings.

The private source-pinned Scheme route at signed `217242332` freshly
reproduces both original Inner/Outer TS, IR, compiler metadata and raw ZKIR.
Source maps differ only in `sourceRoot` and resolve to the copied originals.
Two positive compilations and eight actual process refusals pass, including
ordinary Outer and changed original bytes. Signed qualification SHA-256:
`2404e0cdb877171baa3b3820d4034fd5fd68dc3088d0306a211b3f05ade80923`.
The runner compares a supplied reference; independent review authenticated
the frozen v4 reference separately. The signed reference attestation (`authenticated-reference.json`, retained locally)
SHA-256 `0cc6ea48cc5e8f02f81df0084c9c07f6ad4cf546b49dd996618cdfd7a2b68a32`
binds that package identity/signature/keyring and the original qualification.
This is a controlled interpreter run with captured Scheme 10.3 executable,
library environment and exact commands, without packaged public admission.

## CI resource and optimization evidence

The complete [#774 run](https://github.com/betsudo/compact/actions/runs/44328366189)
used 468.67 summed assigned runner minutes and took 119.67 minutes from first
assigned job to final completion. The complete
[#775 pre-rebase run](https://github.com/betsudo/compact/actions/runs/44357332678)
used 439.60 summed assigned runner minutes and took 131.27 minutes over the
same interval. These are job start/end sums, not billed minutes. The runs
cover different scopes and heads, so they do not measure a CI saving.

The [CI test graph](plans/ci-test-graph.md) documents admission of expensive
jobs after cheap source and fixture checks, exact same-run compiler material,
and separate proof/native owners. The [resource budget](plans/ci-runner-resource-budget.md)
records a local compiler-shell export/import feasibility check. Its Linux
cold-run transfer cost and full hosted runner-minute effect remain unmeasured.
No circuit-size, prover-latency or CI-cost improvement is claimed here.
The newer original-suite gates are functional coverage checks; they add no
controlled performance comparison. A CI optimization claim still needs a
same-scope hosted comparison, including cold compiler-shell transfer and every
required proof and native job.

This page is a dated status digest. A green check or historical receipt applies
only to its stated source revision and test scope.
