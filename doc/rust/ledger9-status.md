# Ledger 9 Rust AST compatibility status

Updated 26 November 2026. This page summarizes the public branch and the
limits of separate local qualifications. The [adoption plan](plans/ledger9-adoption.md)
and [upstream compatibility plan](plans/ledger9-upstream-compatibility.md)
retain the detailed, source-bound history.

## Current Clear/Set/Map proof-data qualification

The [standalone proof-data cohort](plans/clear-set-map-proof-data.md) retains the
original eight-export Map fixture and adds typed membership capture. At signed
`827963ed`, current-SDK arm64 execution now passes three fresh checked proofs and
15 independent-process SDK verification outcomes, including real current
default-parameter acceptance and binding/corruption/valid-wrong-key refusals.
The public Map runner separately repeats three fresh proofs and all 15 SDK
outcomes at signed `03efbac5`; its executed-source and stopped-output identities
are recorded in the linked cohort. Public Tiny Clear/Set proof runs and the real
default-size refusal remain pending.
The evaluator shares SDK cryptography and does not regenerate the before state.
The selected seven original export host callbacks pass their scoped stopped
qualification, with the operator's telemetry-title RED preserved. Neither
result grants full original-suite, released SDK matrix, Linux, generated
TypeScript-runtime or funded transaction/replay qualification. See the linked
record for exact source, artifact and review identities and cost limits.

## Earlier compile and host observations, 20 November 2026

The [native All20 SDK1 library](plans/original-token-sdk1-consumer.md#native-all20-sdk1-compile-qualification-20-november-2026)
now passes default and `ledger-transaction` Cargo checks at signed `d14b07869`,
with independent actual review. This ABI54 native graph uses ZKIR 2.2.0;
the retained-proof consumers use a separate 3.2.1 graph. Compilation does
not establish generated method execution or original TypeScript parity.

The latest [original ADT/exports attempt](plans/original-host-metadata-status.md#original-adt-and-exports-attempt-20-november-2026)
is red: released ADT 27/27 and exports 7/7 pass, but source ADT passes
3/27 and source exports is unrun. Failed captures remain for diagnosis;
this attempt adds no accepted host coverage. Local arm64 results do not
qualify native Linux, and the broad migration remains incomplete.

## Latest private Caller qualification

The private original Caller A/M/I `viaMiddle` route now has reviewed local
qualification at signed `2fd02c6fd93a6546fa51a25cee766a0dd1919522`:
six genuine V3 contract proofs across unfunded and fresh funded runs, one final
auxiliary Dust proof, 18 direct proof/input/wrong-key refusals, and four strict
funded mutation refusals. Default Real balancing and contract-proof checks pass;
funded application succeeds and replay refuses with `IntentAlreadyExists` and
exact unchanged ledger bytes. An independent fresh retained-verifier process
repeats both artifact checks and strict funded application/replay without new
proving. This is a private synthetic no-owner fixture on Darwin arm64/source
SDK 9.1.0.2. A separate released SDK 9.1.0.1 registry-only consumer at
`8b75577b8` independently verifies the six retained proofs, 18 direct refusals,
four strict funded mutations, successful default-strict application and exact
source-SDK2 after/replay ledger bytes. Its fresh independent process replay
also passes. This consumer creates no additional proofs. Public admission,
`Some(User)`, other Caller routes and native Linux remain open. The earlier failed FAA attempt is
retained as failed and is excluded from these six qualified proofs.

See the [Caller qualification record](plans/kernel-caller-rust-api.md#latest-private-three-contract-proof-qualification)
for full receipt hashes, setup boundaries and retained artifacts.

## Original-token artifacts on released SDK1

The independently reviewed [retained-artifact consumers](plans/original-token-sdk1-consumer.md)
verify 31 distinct standalone proofs across 20 circuits with 93 refusal
controls and decode 23 tagged transactions using released SDK 9.1.0.1.
A separate consumer applies three retained shielded-mint transactions with
default Real strictness, including the two-contract mint. Four embedded
contract-proof and three positive-fee Dust-proof mutations refuse; successful
application and replay refusal match the original after/replay bytes exactly.
Both consumers pass fresh independent retained-process replay. They generate
no new keys or proofs. Exact before states for the other 19 successful
transactions, native generated-API compatibility and Linux remain open.

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
| [Original-token Rust circuits #775](https://github.com/betsudo/compact/pull/775) | Merged into `codex/rust-backend-ast` at `b6e7d156e` on 19 November 2026, from rebased `297ea0b17`; all three source-head hosted workflows passed, including existing Rust proof qualification | The green `867b25d88` run retains its earlier source identity. Local proof/funded evidence for 20 unique circuits remains bound to its recorded source; the separate full native Linux gate is still required. |
| Public generated A `pay` / B `transfer` | Signed local source `49ff23d9d`; unpublished | A source-bound offline gate produced fresh A/B V3 proofs, separately verified them, applied a funded default-strict two-contract transaction with positive Dust, and refused replay with unchanged ledger bytes on Darwin arm64. Native Linux, live-node and original decoy-transcript parity remain open. |
| [Native Linux gate #773](https://github.com/betsudo/compact/issues/773) | Open | Local Darwin arm64 proof and funded results do not qualify the required x86_64 Linux gate. |

The reviewed local Events continuation admits the exact original Inner/Outer
pair through a trusted CLI path at `ecf3b0338`. Signed receipt SHA-256:
`17926e1a576c3d1ff21aee29f39944ccb6d15c7773931cb2e57762fad4aca2d6`.
It authenticates the source/artifact/key closure and retains ordinary Outer's
live-coordinator refusal. Its frontend uses a source interpreter wrapper;
a packaged Scheme sibling is a separate required qualification.

Both original calls have fresh two-contract proofs, separate verification,
positive-Dust default-strict application and identical-byte replay refusal.
Actual logs carry Outer-before-Inner payloads `0x42` then `0x07`, and Inner
persists 1 then 2. The corrected source SDK second-call receipt at `9f42c8552`
has SHA-256 `8122ae04a23cc6f8ba2ae09bc9c32e27eed1faf2377db5a76c5ef499b6ad858d`.
The released Ledger 9.1.0.1 consumer at `656fceed5` applies both retained
transactions, matches exact prestates/final bytes, verifies proofs, and checks
mutations and replay. Its signed receipt SHA-256 is
`9b0d591e6b15f07b4b828bae4a1d2f70a4f50ac34363cd1b44391d677cc88b85`.
An earlier refusal omitted the post-block transition; signed supersession
`dbf9c387268a124b1d9ee9cc60cb43da245d041bf73715c048450448700d8842`
records the correction. This is not evidence of an SDK compatibility defect.

Original Callee Witness at `238b6f168` passes 3/3 on each TypeScript path;
original Caller at `d2ba7669d` passes 6/6 on each. Independent review also
verified profile refusals and neighboring exact Events artifact regression.
Signed delivery index SHA-256 values are respectively
`62ddba5a0121c2bdd1bbd8942d2020a1b0e8ff8ba6a42e8a4015f62a63a2cf73`
and `3e0fd4636ccdcd9c1be049c44040cfee8fbf36af79fa700668b5b66c8a563adc`.
These original `--skip-zk` suites do not qualify generated Rust caller proofs.
[#777](https://github.com/betsudo/compact/issues/777) and
[#778](https://github.com/betsudo/compact/issues/778) track integration and
remaining caller work. Key Agreement at `1d335e4e2` passes 7/7 on each
TypeScript path after independent review, with 12 frontend controls and all
58 artifact hashes checked. Signed delivery-index SHA-256:
`30d364d96236d1c1ffc4b35629c945871c9c556f97a21070a01425503f02c71d`.
[#779](https://github.com/betsudo/compact/issues/779) retains its mock-verifier,
skip-ZK and remaining Rust/proof scope. Recursion [#780](https://github.com/betsudo/compact/issues/780)
passes six active cases and retains five original skips on each path. The
reviewed metadata-cycle fix `5f124a6d6` passes 22 source controls and preserves
public/legacy/invalid-metadata refusals; signed receipt SHA-256 is
`3216b990e8520047f890929a5172708d284b27c5b4c2c2895e9db431fe971142`.
Stage-correct Purity `d5bafa5cd` passes the original 1/1 runtime refusal on
each path plus eleven frontend controls. The synthetic compile-time refusal
now explicitly targets the legacy source profile; released 0.21 checks actual
runtime Provability. Signed receipt SHA-256:
`4a5851e42ca424ec50b56176331156d64f7e19c2907cc6cc5957e6a209c25505`.
[#781](https://github.com/betsudo/compact/issues/781) and
[#782](https://github.com/betsudo/compact/issues/782) retain grouped publication.
All eleven original composable bodies are byte-identical in local integration;
that is a source inventory, not completed execution/proof coverage. EntryPoint
invocation remains held.

Separately, [generated Rust caller support](plans/kernel-caller-rust-api.md)
at `23af5a8e7` passes independent review for original Inner's native/recorded
VM query, constructor/root and sealed child replay. Its signed delivery index
SHA-256 is `77c4209a33e29d76feafd821e846abb6597eb7d3f35886eede739a9f8fa1d9a8`.
The separate v5 Events package below binds that runtime for current-runtime
admission. A later private Caller continuation at `34494e61e` passes generated
`Some(Contract(Outer))` execution and method/receiver/frame refusals, using
another coordinator extension. Its reviewed physical candidate `ac767d25`
binds six genuine key pairs and the full runtime/macros closure (841 files,
43 directories; ten controls). Signed package receipt SHA-256:
`391102c51c800623260c35d7cf0cbbde17b410dab6c82a9e27d7c6b3ade53f3a`.
Its later checked installed-key/method/ABI candidate is recorded below.
Public Caller admission, fresh Caller proofs, funded replay, nested/multiple
paths and Linux remain open; later private token execution is recorded below.

The checked Caller identity layer at reviewed `1cff17c1b` now passes exact
850-file/47-directory verification, eight physical checks and offline
all-features Cargo checking. This earlier head predates private token wiring
and executed binding refusals; its signed receipt SHA-256 is
`3caa6958d65b4e9f746b2a70cdaf24216305ab7c36c9e99826997405d5486242`.

The [original host metadata record](plans/original-host-metadata-status.md)
qualifies the unchanged expectedVk five-case body and 31 source guards at
reviewed `94b97c691`, with three retained real key sets. It preserves safe
`__proto__`/null-prototype behavior and unchanged public default omission.
This is a private arm64 source profile, separate from archived package
identities, Rust descriptors, proofs and Linux. The original compiler-version
commit assertion subsequently passes at `72b54c227`; full host packages remain open.

The later [combined host gate](plans/original-host-metadata-status.md) at
`99d08ec89` passes 21/21 unchanged original cases on both released and private
source profiles, with actual compiler outputs captured before consumption.
Signed receipt SHA-256:
`35f14e82ead10ef4ea01926157d63b5e44b28fb31ec508293c866b1bf15188cb`.
[Original ECDSA Rust parity](plans/original-ecdsa-parity.md) at `50df7ede8`
passes the ten-case body through a private adapter: nine generated Rust
calls and one host-only case, independently replayed. Signed receipt SHA-256:
`51b09911a6a96be08a090f04629e424d31b26cfd59b2232bb2bf4ba580a23608`.
The fixture is pure; no ZKIR/key/proof/funded obligation is implied. These
results retain their distinct source heads and local arm64 scope.

The separately reviewed registry-only pure Rust ECDSA consumer at `180e68be5`
passes the same ten original cases and nine cross-SDK raw replays. Only the
pure resolved SDK subset is qualified; optional ledger 1.0.1 is pinned but
not compiled. [Original curve runtime interoperability](plans/original-runtime-curves.md)
at `3f1069833` also passes all 41 P-256 and 44 Curve25519 original cases on
published runtime 0.21/onchain 4.0.1. Independent re-execution passes. A review
fix makes failed or incomplete Vitest runs fail the gate while retaining logs.
This adds pure published JavaScript coverage, with source-runtime and generated
Rust qualification kept separate.

The accepted source runtime now passes the same 85 cases at reviewed `38a554df3`,
using actual emitted runtime 0.18 and its own lock. A signed build attestation
authenticates all 84 emitted files and 275 runtime dependency files before use.
Reviewed `2ccfcb86c` adds the intact original secp256k1 30-case module on each
profile. This gives 115 unique original curve runtime cases per profile in
separately bound slices. Independent replays pass; generated Rust curve coverage,
malformed host-atom compatibility and Linux remain separate.

The private Caller token is now wired at reviewed `d2464cc84`, with actual
physical-key execution and binding refusals. Reviewed `a41abb36d` passes ten
private unfunded VM tree tests; it does not execute generated multi-hop
adapters or authorize a transaction. The Caller API record gives exact
receipts and restrictions. Full Linux and new-runtime proof gates stay open.

Reviewed private `Middle.forward` at `b39adeec1` now executes the actual Inner
adapter with its installed physical key and observes `Some(Contract(Middle))`.
Wrong key and persisted receiver refuse. Independent retained-binary replay
passes backend 2/2, generated Middle 1/1 and all 11 runtime tree tests. The
wrapper's root Outer recording remains manual; generated `Outer.viaMiddle`,
a new signed three-call package, proofs/funding and Linux remain open.

The subsequent reviewed `89bf52445` executes generated private Outer, Middle
and Inner, replacing that manual root. Caller observations are root `None`,
Middle seeing Outer, and Inner seeing Middle; both persisted receivers are
checked before child execution. It returns typed records and sealed handles,
without an authenticated A/M/I package or external preimage assembly. A new
physical package and source-specific checked token are next; proof/funded,
released SDK1 execution and public/Linux gates remain open.

The new physical A/M/I candidate at reviewed `c1c61e77b` now binds 1,652
files/77 directories under identity
`999d62b1c764f13b6aa9de60c09cb7f4b4ae089ac2b51d1c9f9d39e024e795ed`.
Exact verification and 16 refusal controls pass. It adds physical packaging,
with checked token/execution, package Cargo, proof/funded and public/Linux
qualification still open.

The subsequent checked A/M/I token at reviewed `2c5f11330` now passes package
Cargo qualification and private VM execution under a new 1,664-file/82-directory
identity. It consumes the actual root, sealed observations and owned resolved
material. Five VM tests, 26 dynamic refusals, 23 physical refusals and six
downstream compiler refusals pass, with independent replay. This source SDK2
no-owner route still has no external authenticated assembly, proof/transaction,
funded, SDK1, public or Linux claim. Its target and worktree are cleaned.

[Original runtime resolution coverage](plans/original-runtime-resolution.md)
at reviewed `127f75a12` passes all 14 original cases on published runtime 0.21.
Source runtime 0.18 retains its closed export: zero passes and 14 missing-function
failures, recorded as incompatibility rather than success. The accepted source's
existing 24-module suite separately passes 465/465 with no skips. These are
source TypeScript tests, including adapted/internal cases; their counts overlap
the original curve suites. Finished owned Node scratch and worktree are cleaned.

The original Basic, Storage and Witness slices are also consolidated at
reviewed local `108ff4473`: unchanged original bodies pass 12+12, 6+6 and
2+2 released/source TypeScript cases respectively (40 total). Source guards
pass 12/10/12, Key Agreement remains 7+7 and the exact Events artifacts and
eight refusals remain unchanged. Signed delivery-index SHA-256:
`ee5b2028531bc79c161fbfd0fd4a840e922d557e587749bc6fbe0ae63393d8f6`.
This mock-verifier/skip-ZK consolidation adds no Rust proof or funded claim;
Storage's earlier seven Rust tests retain their original source scope.

The [native Events handoff](plans/original-events-native-linux-handoff.md)
is independently source-reviewed at local `6f838e075`; signed source-only
receipt SHA-256 is
`131ac626321f98439a3bac747da398188a9efefa6b0f23dad0ec7088ae7de0d0`.
Seven provenance/physical-tree refusals, four retained parser controls and
zero-new-header checks pass on Darwin arm64. Full mode prepares packaged
compiler admission and executes the actual admitted Rust crate for fresh
proofs, independent verification, sequential funded strict application and
released SDK replay. Full native Linux mode has not run.

A later [v5 runtime/CLI continuation](plans/original-events-v5-status.md) is
independently reviewed. The signed physical identity
`635a6303044c59a1b38370fe5211b6117c24f03b4f1c12c0a7e81fec49eccb9d`
binds the reviewed caller runtime; source `73fb2ab80` admits v5 with 18
controls and strict Clippy and preserves actual v4 admission with 13 controls.
Signed CLI receipt SHA-256:
`dec02cb4c96ec6677749a7ffd6ac49ca6d69586a4e7306ed1246a7fd1c9af2fd`.
Fresh v5 source-SDK native proofs and sequential strict funded replay now pass
at reviewed `bffd8d788`; released SDK1 at reviewed `4715d0648` verifies all seven
proofs and both strict funded/replay state boundaries byte-for-byte. Packaged
compiler and native Linux remain open. V4 keeps its original identity.

The [original block-time runtime case](plans/original-block-time-testchain.md) at
`2d2132f0b` passes its unchanged twelve assertions on each TypeScript path
while preserving the older synchronous Ledger 8 adapter. Rust IR refuses
the first time operation at that source; [#783](https://github.com/betsudo/compact/issues/783)
tracks the separate implementation. Its VM comparisons use slot 2 `tblock`.
The subsequent reviewed Rust slice `b577d8d84` passes the original twelve
values in native and recorded generated Rust (24 observations), source
profile controls and focused VM/backend tests. Signed receipt SHA-256:
`ba0629197aa9b5cab6dda1d6bfe5ceaa62258fc766e2c4882b1e3581fa516bea`.
This is local API/VM qualification; its fresh proof/funded/Linux gates remain
open and its runtime closure is separate from v4/v5.

All results above are local Darwin arm64, bound to their signed source heads.
The broad milestone remains incomplete: generic generated Rust cross-contract
calls, nested/multiple caller paths, post-child event ordering
[#776](https://github.com/betsudo/compact/issues/776), packaged compiler,
native Linux [#773](https://github.com/betsudo/compact/issues/773), live-node
qualification and the complete original-suite census remain open. The later
sections retain historical slice scopes; they do not override this current
snapshot. [Compatibility #726](https://github.com/betsudo/compact/issues/726),
[coverage #727](https://github.com/betsudo/compact/issues/727),
[cross-contract calls #730](https://github.com/betsudo/compact/issues/730), and
[integration #732](https://github.com/betsudo/compact/issues/732) retain owners.

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

At signed local `b465dcce6`, the source-owned Events generator gains a staged
checked `outerEmit` entry. It places identity binding before the generated
Inner child effect and compares native replay against the sealed pair. All
15 focused contract-call tests and library Clippy with warnings denied pass
on Darwin arm64. Independent read-only review found no confirmed defect.
Detached-GPG receipt SHA-256:
`e2d62179b2aa37eaf3cdf08b4f5db39d6b0dd9844eed901a209f84c595ec9f46`.
The test checks generated source order only: the emitted crate has no Events
identity dependency, Cargo type-check or public compiler admission. The
public Events companion, original generated A/B execution, proofs, funded
application, applied event order and native Linux remain open.

The signed local Events stage-only companion at `2c7e4eaa3`, stacked on the
checked-entry head, binds 838 physical files: the exact prior package,
generated Inner and private Outer material, fresh keys, and the tracked
runtime source closure. Its stage identity SHA-256 is
`859d8cabae8a1391a82555a78ceb29a662ad089bb77e4d8f35064603656a4413`.
Seventeen semantic and membership refusals pass; independent review
confirmed the final exact-membership repair. Detached-GPG receipt SHA-256:
`0a813df62474c3527c7f7107a1930501ffa2325fb9670cf540710e211a24ea90`.
The package has no public Outer TypeScript or compiler manifest, Events
identity crate, or public `outerEmit` admission.

Separately, source-built private TypeScript generation passed 13 profile
checks, then the unchanged original composable `events.ts` body passed 2/2
TestChain cases using generated Inner and Outer TypeScript on Darwin arm64.
The superseding signed receipt, issued after independent audit, has SHA-256
`21892f815688f2b84817e21485dbe6857769c1d286e0c4d3d25694fb50383e3a`.
It pins all 35 retained artifacts, including the upstream verifier template
consumed by the harness. The original run did not separately capture exact
argv, environment, and exit codes; the receipt discloses reconstructed
reproduction commands. This private result does not qualify public generated
Rust/TypeScript Outer, genuine proof, independent verification, funded strict
application/replay, applied event delivery, live-node execution, or native Linux.

The signed v3 private Events companion at local `e6f94deda` adds exact
generated Outer TypeScript and its physical compiler manifest to the prior
material and binds 1,649 files across 84 directories. Its identity SHA-256
is `dd6c75875e40461713da289aae49f1f3c6f780433bb6de2f74b1030db5c90072`.
The exact-head verifier checks a separate GPG trust anchor and all local
checker dependencies before accepting the package; 18 negative controls
pass. Independent review found no remaining actionable issue. The
detached-signed receipt (local receipt `qualification-receipt-e6f94deda.json`)
has SHA-256 `984f9cc7b6c645c4e618491e2fb7b29ff3adf5415873e1f2be346f0ea00e8d33`.
This remains private and has no public Outer admission.

At signed local `8302e704f`, a source-owned checked Events identity crate
binds the original A/B sources, physical generated artifacts, runtime closure,
verifier keys and resolver key bytes. An explicit-keyring gate verifies the
v3 package before and after copying it, then Cargo checks the identity crate
and generated Outer, runs identity Clippy with warnings denied, and refuses
three physical tamper probes. The independently reviewed
detached-signed receipt (local receipt `qualification-receipt-830.json`)
has SHA-256 `27d7efd26f33395655c61e443e78b7d9d60839e7bd240a420cdb1e3bac1c5717`.
The generated Outer Cargo check passes; its nine strict Clippy diagnostics
are retained separately in that receipt's evidence.

A follow-up source fix at signed local `17bf098f8` exposes the generated
checked Events error types and removes unnecessary Field clones. Focused
source tests, fresh scratch generated Outer Cargo check and Clippy with
warnings denied pass on Darwin arm64. The local qualification (local receipt `qualification.json`)
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
Its detached-signed receipt (local receipt `qualification-receipt-8ea671f79.json`)
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
logs; the signed qualification (local receipt `qualification.json`)
SHA-256 is `4558cfb52d55623ca1ec099be59f4c790dd4979dac6f5b306dec8e010b9ff7a4`.
This qualifies local compilation of the physical candidate; original Rust
execution, public CLI admission, proof, funded application and Linux remain open.

The checked generated Rust Events consumer at signed local `60de2b12d`
executes `outerEmit` from a verified private copy of the unchanged v4 package.
The first call returns Field 1 and records two version-1 ShieldedSpend events
with exact payload and Outer-before-Inner emitting addresses. Two prepared
call preimages include the expected Outer child-result/randomness transcript;
four installed/resolved key mismatches refuse. The independently reviewed
signed qualification (local receipt `qualification-60d.json`)
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
the signed separate-verifier receipt (local receipt `retained-verification-0c4.json`)
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
the frozen v4 reference separately. The signed reference attestation (local receipt `authenticated-reference.json`)
SHA-256 `0cc6ea48cc5e8f02f81df0084c9c07f6ad4cf546b49dd996618cdfd7a2b68a32`
binds that package identity/signature/keyring and the original qualification.
This is a controlled interpreter run with captured Scheme 10.3 executable,
library environment and exact commands, without packaged public admission.

The funded Events gate at signed local `0148c881e` proves a fresh two-call
transaction, balances a positive Dust fee, and accepts default-strict Real
verification and application on the Ledger 9.1.0.2 source graph. Replay refuses
with `IntentAlreadyExists` and identical ledger bytes. Actual applied
ContractLogs contain exactly Outer `outerEmit` then Inner `innerEmit`, with
version, ShieldedSpend type, address and payload checked. Inner v becomes 1;
a second checked call returns 2 from that persisted state. Its second proof
and application are not claimed. Independent source and receipt review passed.
Signed receipt SHA-256:
`b737d65f4efb4584f3b452a3ff58b4f1a63ef371f6ce281c1ad1d3cf1eda4d8f`.
Released Ledger 9.1.0.1 cross-matrix acceptance remains required, together
with final public CLI admission and native Linux. This qualifies the original
pre-child event case; [#776](https://github.com/betsudo/compact/issues/776)
remains open for broader post-child applied event ordering.

Local trusted CLI admission now passes at signed `ecf3b0338`. The exact
signed original Events pair is freshly compiled through the private Scheme
route, compared with authenticated TS/IR/ZKIR and the source-owned Rust
renderer, Cargo checked offline, rechecked after Cargo, and atomically
published. The final output binds 2,475 files, preserves the signed v4 subtree,
and has a confined five-package local Cargo graph. Independent review passed.
Signed receipt SHA-256:
`17926e1a576c3d1ff21aee29f39944ccb6d15c7773931cb2e57762fad4aca2d6`.
Its direct positive compilation returned zero; a separate 13-control run
validated retained output and refusals without claiming another compile.
Ordinary Rust Outer still refuses at the live coordinator boundary and keeps
the admitted output intact. Qualification uses a controlled source interpreter
wrapper on Darwin arm64. A packaged source-built Scheme sibling, broader
generic calls, released-baseline completion and native Linux remain separate.

The released Ledger 9.1.0.1 consumer at signed `d68883f82` independently
verifies and applies the same funded transaction using ten crates.io SDK
pins and no Git dependency substitutions. It checks physical V3 keys and
transaction public inputs, refuses changed proof/input/key and strict
Outer/Inner/Dust proof mutations, and uses default Real strictness. Applied
ledger bytes match the source SDK exactly; actual ordered logs include both
original payloads with bytes32 FAB alignment. Replay refuses with unchanged
ledger bytes. Independent source and evidence review passed. Signed receipt
SHA-256:
`33899b39366efd79bc4220704f65856205ae90cd19a44053028b8dc4c2c1786e`.
This establishes the released SDK baseline for the first retained transaction.
The second checked result still has no second proof/application qualification;
packaged compiler and native Linux gates remain open.

The next original suite is tracked in [#777](https://github.com/betsudo/compact/issues/777).
Released Compact passes all three original Callee Witness assertions. The
local frontend initially failed to resolve the separately named `Peeker`
interface; its bounded Ledger 9 policy fix now reaches runtime witness and
provability conformance. Source qualification, neighboring Events artifact
regression and independent review are being finalized. This TypeScript work
has no generated Rust or proof claim.

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


The reviewed [six-case Rust Curve serialization bridge](plans/original-runtime-curves.md#six-original-serialization-callbacks-through-rust)
at `df89e5c82` passes six unchanged original callbacks with 37 independently
replayed calls including separate diagnostics; 38 Curve callbacks remain
filtered on this Rust path. This is a pure runtime API result, not generated
contract, proof, full SDK ABI or Linux qualification. A separately reviewed
[zero-width Rust encoding counterexample](plans/zero-width-ledger9-encoding.md),
tracked in [#784](https://github.com/betsudo/compact/issues/784), differs from
released v3 attestation hashing. Its next runtime/derive fix must preserve
legacy Ledger8/v2 behavior; generated profile/API qualification follows.


The [four original runtime modules](plans/original-runtime-descriptors.md) now have
faithful reviewed red evidence: 924 parameterized cases per profile, published
918 passed/6 failed and accepted source3eaf 894 passed/30 failed. A separate
reviewed local bigint-helper repair at `d79f397fe`/runner `b1041130f` fixes three
unchanged original cases, giving 897 passed/27 failed while existing source
24-module/465-test regression stays green. Preserve the remaining diagnostics
and accepted additional APIs; [#785](https://github.com/betsudo/compact/issues/785)
tracks the gaps. These are JavaScript/API results, with no proof or Linux claim.

Closed Caller preparation at reviewed `8707e3a82` now produces only three private
unfunded SDK prepartition calls. Thirteen test groups cover 80 refusal cases;
17 downstream compiler refusals and an ordinary SDK-clone positive pass, with
independent replay. Product provenance stays opaque; SDK clones remain mutable
ordinary data. No actual proof preimage, transaction, funded, SDK1 or Linux claim.
Finished owned build targets and clean worktrees are removed; exact replay
fixtures and qualification evidence are retained.


The reviewed [zero-width runtime/derive slice](plans/zero-width-ledger9-encoding.md)
at `3e5625f14` now executes both encoding profiles together: legacy alignment
and hash remain unchanged, while the explicit v3 attestation matches the released
oracle. Thirteen runtime tests and 33 macro tests pass, with four downstream
refusals and strict macro Clippy. This is direct API qualification; IR/profile
threading, generated original contracts, the ABI55 handshake and proofs remain open.


The [owned Ledger 9 coin amount boundary](plans/original-runtime-descriptors.md#owned-ledger-9-coin-amount-refusal)
now passes the unchanged original stdlib suite 31/31 at reviewed `5a7fc67db`.
The four-module original cohort is 898 passed/26 failed, and source regression
is 469/469 including four new error/state-identity cases. Generic descriptor
encoding and legacy context behavior are retained. The remaining 26 failures
are recorded; no Rust/proof/Linux claim follows from this JavaScript API repair.


The later [opaque atom and owned byte decoder slice](plans/original-runtime-descriptors.md#opaque-atom-refusal-and-owned-byte-decoding)
at reviewed local `8da87375b` fixes exactly seven further unchanged originals.
The same four-module cohort is now 905 passed/19 failed; source tests remain
469/469, and independent original replay passes 7/7 with 917 filtered. Both
source builds bind all 6,042 files and 16 links; only compact-types.js/map change.
Signed final delivery SHA-256 is
`805cbd11f9bd98892b77ab6960e4f1842a38cb2eb6560cbe09083803e30afc28`.
Owned build copies and worktree are removed. The 19 original failures remain
explicit, with accepted extra exports, strict scalar/point policies and unchecked
encoder APIs preserved. No generated Rust/proof/funded/Linux result is claimed.


The follow-up [Enum/Bytes refusal slice](plans/original-runtime-descriptors.md#enum-and-bytes-encoding-refusals)
at independently reviewed `035e85a11` fixes exactly three more original cases.
The four-module cohort is now 908 passed/16 failed, while source regression
remains 469/469; independent original replay passes 3/3 with 921 filtered.
Signed final delivery SHA-256 is
`661bd8f8a4b8c6b8de8e7da63e7aab2e49305228181f38422aabc2138e17cd65`.
Owned build copies and clean worktree are removed. The optimized backend
checkpoint at `8b5e9619e` separately passes 1,970 library tests with seven
ignored. `beeb21995` passes library-only strict lint; schema21 admission,
generated profiles, full test-target lint and Linux remain separate/open.


The [byte-cast API overloads](plans/original-runtime-descriptors.md#byte-cast-helper-overloads)
at reviewed `96b4ce803` preserve legacy calls and qualify mainstream explicit
forms: source 474/474, emitted API 5/5 and declaration shapes 4 accepted/4
rejected. Original 924 remains 908/16; new controls overlap these other gates.
The [complete twelve-module comparison](plans/original-runtime-resolution.md#all-twelve-original-modules-on-one-current-source-graph)
now collects all 1,091 original cases on both profiles: current source runtime
has 1,061 passed/30 failed; released runtime has 1,085 passed/six failed. Both
remain red, with original assertions retained. Reviewed local backend
`09dac33d7` passes 1,987 library tests, 48 CLI tests and library-only strict lint.
The separately reviewed material-only retry now passes both original-fixture
private drivers and retains fresh TS/info/IR21/ZKIR/private Rust ABI55 outputs.
The corrected producer `09dac33d7` passes 1,987 backend tests, 48 CLI tests,
library-only strict lint and both fresh-material drivers. Its private generated
SDK1 consumer at `d0655a12f` now passes all three API tests, including native,
recorded, observed and Verify paths, wrong-read/cast refusals and both the v3
and legacy hashes. An independent retained-binary replay repeats the result.
These supplemental tests cover four original circuits; they add no original
runtime test cases or proofs. A separate private proof harness first fails
SDK1 compilation on field `+=`; signed correction `5a17e13ea` retains the
same wrong-input assertions. The reviewed a9b cached retry compiles both the
ignored proof harness and retained verifier, with 741 Cargo artifact records
and 325 distinct package IDs; neither executes a proof or application here.
The separate [private SDK1 fresh-proof gate](plans/zero-width-ledger9-encoding.md#private-sdk1-fresh-proofs-and-funded-replay)
now qualifies four fresh V3 proofs, four default-strict funded applications,
eight targeted mutations and exact replay refusal, followed by independent
fresh four-verifier replay (`59d95b55`/`8a7f7de4`/`7d7c92d1`). The
[CamelCase original-host retries](plans/original-host-metadata-status.md#efficient-retries-and-original-output-transport)
retain the prior setup failures. Executed 27d runs pass 2/2 on each profile;
independently accepted artifact-only finalization preserves their original
source identity (`c9dfcfc1`/`faa15e5f`). The union is eight modules/24 cases per
profile across three separate gates, without a combined 24-case qualification.
Public schema21 admission, global SDK1 ABI, full parity and native Linux remain
open. The [hardened v5 Linux source handoff](plans/original-events-native-linux-handoff.md#git-object-binding-hardening)
at `b3e677de6` passes independent source review and an independent rerun of
12 tests/77 refusals. Actual Linux compilation/proofs remain unrun. Owned
completed build copies and worktrees are removed; source stays local.
