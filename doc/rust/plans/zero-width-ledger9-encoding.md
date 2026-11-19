# Rust zero-width encoding on Ledger 9

[Issue #784](https://github.com/betsudo/compact/issues/784) tracks a confirmed
local compatibility gap against pinned mainstream Compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`
and released Compact 0.36.0. Preserve the separate Ledger 8/ZKIRv2 boundary
qualification in [#702](https://github.com/betsudo/compact/issues/702).

Accepted local Rust source `3eafaf3eb92d6c7a39ba93ff6d41df11eb352643` assigns
zero-byte alignment to `BoundedUint<0>` and a single-variant `CompactEnum`.
The pinned issue-588 attestation places its one-variant `Kind` before Uint64;
the actual released v3 `attest.zkir` uses persistent-hash alignment [1,8].

For amount 7, a direct native tuple/enum probe produces
`aae89fc0f03e2959ae4d701a80cc3915918c950b159f6abb6c92c1433b1a8534`.
The actual released runtime 0.21.0 with the original ZKIR alignment produces
`7dd1657a36197d82bfae690c3212ef9b3c0d7bbaef12ddb8ef8f9f30b2373185`.
The audit mirrors the original struct field order through direct Rust APIs;
it does not execute a generated Rust original contract.

Signed audit v2 SHA-256 is
`b2fa9eaa8b3fc1f886ffbb4d4a90bc1c1615add0aeb000525b875a467b02c2ac`;
independent review SHA-256 is
`22c9bf90d2de28304231c4831df7a536cecaf1e18c599d6774aa88121b014dc0`.
It authenticates 770 unchanged runtime/macro source files, the private
manifest-only released SDK1 pure dependency overlay and 393-package lock.
A fresh Node oracle checks the exact 416 published SDK files and its
script/ZKIR/native-binary inputs before and after execution. Both processes
exit successfully and reproduce the mismatch. The earlier v1 post-run-only
graph attestation remains historical. Optional ledger is not compiled; no full
SDK ABI, generated-contract, proof, funded transaction or Linux result is claimed.
Owned Cargo and Node copies are removed; source, binary, lock, original ZKIR,
oracle and logs remain retained.

## Runtime and derive API qualification

The reviewed first runtime slice is signed local
`3e5625f148e269ce067ae84591803a991cbc8735`. It introduces explicit
`BoundedUint<MAX, ONE_BYTE_MIN = false>` encoding selection and the scoped
`#[compact_encoding(ledger9_v3)]` enum derive attribute. Legacy/default types
retain zero-byte behavior; selected types use at least one byte. Both profiles
coexist in one binary without a Cargo feature. The global ABI54 marker remains
unchanged; an additive ABI55 marker reserves a future generated-profile handshake.

Actual final-head execution passes 7 existing bounded-unsigned tests, 3 existing
enum tests, 3 new mixed-profile/hash tests and 33 macro unit tests. Macro strict
Clippy passes. A downstream valid control and four compiler refusal controls pass.
Independent review replays the retained test binaries and checks those controls.
The explicit v3 attestation hash now equals the published oracle `7dd1657a...`;
the legacy hash remains `aae89fc0...` in the same consumer.

Signed final qualification SHA-256 is
`f7f0c41e317e4726a0334d101f5ca978b0a62cfbd4e01e992587cbde033c7bea`;
independent review SHA-256 is
`132ac73e54caa591e006019145ba6eef8d667177f05e4ff7f5e7a71e0513e354`.
The stage contains 776 files/20 directories. Its manifest-only pure registry SDK1
overlay resolves 24 Midnight packages matching the signed baseline; optional ledger
is absent from the compiled tree. Source compatibility metadata retains its SDK2
labels. This is not a full SDK1 ledger ABI result. Earlier compile failures at 4df,
6ee and 470 remain historical; each fix was signed and reviewed before rerunning.
Owned 845 MiB Cargo target and clean source worktree are removed; source, locks,
binaries, stage and evidence are retained.

The next slice must carry the encoding profile explicitly through compiler IR,
carrier/literal/coercion/arithmetic/stateful lowering and runtime compatibility.
Generated issue-285/588 contracts, the actual ABI55 handshake, new physical proof
packages, fresh proofs/funded replay and Linux remain unqualified. Existing ABI54
packages and Ledger8/ABI53/Forge evidence keep their exact identities.

Owners are runtime primitive alignment/binary hash representations, enum
and cell derives, IR extraction/schema, generated carrier/literal/coercion/
arithmetic/stateful lowering, compatibility metadata and original-host gates.
Retain the original issue-285 and issue-588 fixtures and assertions. Complete
generated Rust/TypeScript behavior, source-bound fresh proofs with independent
verification, funded strict application/replay and native Linux remain open.


## Optimized backend regression checkpoint

Signed local `8b5e9619e90fffa7d5dc6554c576136c8c0052c1` passes the unfiltered,
locked offline optimized library suite under Rust1.99/default worker stack:
1,970 passed, zero failed, seven ignored. Independent receipt review verifies
44 changed source blobs, four authority manifests, lock, retained test binary
and log. Qualification SHA-256 is
`ca93d39d0347af6a82f1c67df79cf115cecf150f6dacd873076e1c31b9e08e42`;
review SHA-256 is
`04ca0838f1a441c2eecd4e15d2a701d82831a51d95994976a340072551ecef19`.

The run exposed inherited authority drift from accepted cross-contract edits.
[Issue786](https://github.com/betsudo/compact/issues/786) adds digest-pinned
current-to-predecessor projections, preserving old manifests and owner bodies.
Earlier red runs and the test-only borrow compile failure remain historical.
Unoptimized default-stack baseline aborts are not relabeled as passing runs.

A separate narrow lint cleanup at signed
`beeb2199597a088c1b0ae4b459425b4119f3155e` passes strict library-only Clippy.
Signed lint receipt SHA-256 is
`be01e8676da0f4c6347f1b8a23499a3c074dfc034ad26b6d0ecd8fdeaa3359cb`;
independent review SHA-256 is
`f6078126d2cdb1ea131bfcb4c7fe69249178b735e5a67578da1e2066c10fec49`.
This is not a fresh full library-test run at beeb, nor passing test-target lint:
frozen contract_calls.rs test diagnostics remain unchanged. The owned 775MiB
Cargo target and clean comparison worktree are removed; signed sources, locks,
binary and logs remain. Recorded/profile propagation continues as source-only
work. Public schema21 still refuses; compiler emission remains20. Generated
original285/588, physical ABI55 compatibility, proofs and Linux remain open.


## Recorded checkpoint and Cargo graph correction

Signed `167de8b701178c3343bd2bd848cde0b704fedb93` passes a fresh full optimized
library run (1,972 passed, zero failed, seven ignored) and strict library-only
Clippy. Receipt SHA-256 is
`867e9aa96e0d3496c547c7344e78668750d5145c965061fb8a1ae6382aed1b6f`;
independent review is
`b4cefcb637f04157ae4a4171fcacb9c153a1d84d77e95c877c5f467d8101d225`.
This qualifies the bounded recorded renderer preparation; schema21 public
admission and generated original contracts remain closed/unqualified. The
owned build target is removed. Functional preparation is a separate source-only
slice, explicitly refusing v3 through the archived Ledger8 adapter.

The earlier 8b5 test and beeb lint receipts labeled the inactive member lock as
the Cargo lock. Actual workspace builds use root Cargo.lock, SHA-256
`1871651dc10800a3bc94c81f9427293abcbcb73309c21093ec4a255f0626513e`.
New immutable signed addenda bind that lock, both manifests and the inactive
child lock. 8b5 addendum-v2 SHA-256 is
`4a71d4b383b2a5940fcd87a481d73aade43462d90c2de69da867b752765d4556`;
beeb addendum-v1 is
`9e25d99c9891b9602e00e05f211d14f206b8a80d874b9bcd47d0497934a96048`.
The original signed receipts/logs are unchanged. This corrects graph labels,
not execution results; no rerun is claimed for the addenda. Fresh a383/167
receipts identify both lock roles directly. Full test-target lint remains a
separate historical red scope in frozen contract_calls.rs.


## Functional checkpoint and original compiler material

Signed `9ca0e9c8791914d8508578edd212823e62058ce7` passes the full optimized
library suite: 1,977 passed, zero failed, seven ignored, zero filtered. Strict
library-only Clippy also passes. Functional schema21 is explicitly refused by
the archived Ledger8 adapter. Signed qualification SHA-256 is
`847fcd3034db611f4f0fff980b1ee7ed566d6e57bdbec145957fbc287c2efcfc`;
independent reviews are
`17e062971717fb57381aa087bb2aa5257ab3add68bb8393cdd9be6f116d76cb5`
and `2fa0a412773e0780df5dc55e81c947066e9d62873da6a8b22651e459d812b294`.
The owned 781MiB build target is removed. This still does not qualify public
schema21/ABI55 generated contracts, proofs, funded application or Linux.

The first explicit `--rust-ir-encoding-profile ledger9-zkir-v3` material
preserved schema20/ABI54 for prior invocations. That historical run exposed
original285's nested Cell.write refusal and original588's source `[0,8]`
alignment. Selector-only equality did not establish mainstream parity. Those
failed Scheme imports/pass loads and partial captures remain immutable history;
the separately qualified integration below supersedes their pending fix status.

## Reviewed original compiler material at 258

Signed `258d29c3bee52b48b570e7b16608fac655ed4f5e` integrates only the reviewed
v3 minimum-byte frontend hunks and ordered write/read/widen return lowering.
Twelve actual compiler invocations pass the source-owned gate: six positives
and six atomic refusals, plus seven direct selector checks. Original285 retains
all three circuits and original588 retains its attestation. Their v3 generated
TS/declarations/info and raw ZKIR match the signed host414 source artifacts;
zero descriptors use `(0n,1)` and attestation alignment is `[1,8]`. Default-v2
zero descriptors and all three raw ZKIR bodies remain byte-identical to the
historical baseline. Ten archived Events v4 artifacts and 26 private composable
artifacts remain exact. Callee14/Caller17 are frontend controls, not original
runtime-case counts.

Signed material receipt SHA-256 is
`7d068eecfc7836c3f720c1b175e897412e5eed97df46004d7476301d7c6ede76`;
independent root review is
`98e2ae98abae2dbe67a2f75b97d90331f4b9f681f34142a390ee09f2874793ef`.
It authenticates all 9,848 source blobs, 176 retained outputs, commands/logs and
signed host/v4 oracle bindings, without an independent Scheme rerun. The owned
143MiB compiler snapshot is removed; all material artifacts remain.

This qualifies frontend material on Darwin arm64. The pinned zero-bit original
module has **two compile-only cases**, with no runtime callbacks. Fresh state
and persistent-hash execution checks are supplemental controls, still pending.
Ordinary schema20 with v3 now has one-byte TS/ZKIR widths while legacy ABI54
Rust carriers can be zero-byte; a generic typed refusal is a separate reviewed
source-only follow-up awaiting execution. Public schema21 rendering remains
closed. Same-root ABI55 handshake/physical identity, generated native/recorded
Rust parity, fresh proofs, funded strict replay and Linux remain unqualified.
Archived Events/CCC packages and proofs keep their original identities.


## Private profile55 source preparation

Reviewed signed `ac129a465858928e233b7999901ca11989ab6535` adds a test-only
private renderer for the bounded original 285/588 surface. It requires a sealed
IR21 document, a typed structural whitelist and a validated same-root runtime
token. The token checks the compiler-owned exact 120-file/15-directory runtime
and macro closure; selected self-reported compatibility hashes alone are
insufficient. Runtime identity is checked again after rendering. Ordinary CLI
and public renderer admission still refuse IR21.

The ignored material driver is source-prepared to invoke the source-built
frontend afresh and retain TS/info/IR/origins/ZKIR, Rust/capability output and
runtime bytes. At that source-only revision, backend/CLI execution, actual driver output,
generated crate typechecking, native/recorded/Verify and proofs were pending.
The later EDF checkpoint below qualifies backend/CLI/lint only. The earlier
1,977 library pass belongs to `9ca0e9c87`, before these source changes.

Separately, four new TypeScript behavior controls and three cast refusals pass
on retained compiler258 output. `foo1/2/3(0)` return and store zero; `attest(7)`
returns/stores the original struct and hash `7dd1657a...`. Root independently
reran a sanitized Node process with the complete 416-file SDK graph checked
before and after. These are supplemental controls, not additional original
cases: the pinned original zero-bit module contains two compile-only cases.
Signed behavior receipt SHA-256 is
`bf6e5e91208aceabe8e7d0ab16635ffb7c4b2fe126d57fb77b2e9fd2c9aee2c5`;
independent replay review SHA-256 is
`358c8289e08b68a7b77fe3a9cd5dd88d7b8250255671e7ac137b44f53922d09f`.


## Reviewed backend and CLI checkpoint

Signed local producer `edf0b88d14c65dfd92894e23a994d537641ad47f` now passes
all 1,987 backend library tests, with zero failures, eight ignored and zero
filtered. All 48 CLI tests pass; strict library-only Clippy exits successfully.
The optimized profile uses Rust1.99, opt2/debug0/incremental0/jobs2/Rayon2 and
the default test-thread stack. The active workspace lock is `1871651d...`;
the inactive member lock is distinct. Cargo JSON records 251 actual compiled
package IDs (250 external); 524 external metadata candidates are authenticated
separately. These candidate and compiled counts are not interchangeable.

Signed partial qualification SHA-256 is
`7ce48c3a0407f030a40802effb8477b329f2aa20df072cc13c326fffea9863d9`;
independent review SHA-256 is
`e6addc575f6b119578de45b339baf23533ae6f06cb32af4eb6a507788707b94d`.
The reviewer verifies raw counts, both retained binaries and all source/support
and dependency bytes. A contemporaneous post-map was not retained after the
failed material phase; the author and reviewer separately rehash the current
9,853-file source, six support roots and full cache against the pre-map.
Full test-target Clippy remains a separate historical limitation.

The observed frontend call needed one explicit `PublicLegacy` admission
argument; adding it fixes the actual compilation diagnostics without opening
schema21. The first subsequent private 285 material attempt fails because the audited
PATH lacks `sha256sum`; 588 is unrun in that attempt. That failed phase remains
historical. It does not establish SDK compilation, native/recorded/Verify or
proof success.

A separately signed material-only launcher uses the exact retained EDF binary,
retained source/compiler wrapper and a pinned `sha256sum` added last in PATH.
It ran only the two ignored material drivers, with fresh disjoint output,
50 GiB continuous free-space floor and 256 MiB output cap. The launcher and
producer identities remain distinct. Both drivers pass: one test each, zero
failures and 1,994 filtered tests each. #285 retains 133 files/21 directories
and three raw ZKIR files; #588 retains 131 files/21 directories and one raw
ZKIR file. Each includes the copied 120-file runtime closure; those duplicated
runtime files are not additional original cases. Private IR21 uses
`ledger9_zkir_v3`, and generated Rust asserts ABI55. Execution receipt SHA-256 is
`2ff8a23668fdbf3c0b924233f6a352b02a2e21789577c0c4877d1dd3a5da7d10`;
independent review SHA-256 is
`203a961ce612622cc5ff5b12d1778ce2d53629cb0d79b13cdf4619b1ec55b005`.
The original upstream tests are compile-only; this adds no original runtime
callbacks. Owned completed Cargo targets are removed; source fixtures,
binaries, fresh material, logs and receipts remain.

## Generated SDK1 execution preparation

Signed source `69dcb88dc2684d8530d8ef944f63f093ae96d1f4` adds a finite
stage plan, manifest-only released SDK1 overlay and uncompiled generated API
test template. Independent static review SHA-256 is
`d73337eac46084b557b5956af5c075d2197176ca502fa266f651a8091ad7c4ab`;
source PREP SHA-256 is
`05409b11b58fce98a3fb55435d8b8162ec235934ccd0c5e8c6943f4a001aae38`.
All nonmanifest runtime bytes and fresh generated contracts remain unchanged;
source-SDK2 compatibility metadata is not relabelled as SDK1. The template
targets native, recorded, observed and Verify paths, wrong-read/input refusals,
the v3 `7dd1657a...` hash and the legacy `aae89fc0...` hash in the same binary.
That source-only checkpoint was followed by the failed fc74 typecheck and the
corrected d065 qualification below. Public schema21 admission, fresh profile55
proofs, funded replay and native Linux remain separate gates.


## Four-circuit size inventory before proving

The exact four raw ZKIR inputs from the signed fresh-material closure were
copied into disjoint local analysis directories and measured with the pinned
released ZKIR 3.2.0 `mock-compile` tool on Darwin arm64:

| Original fixture | Circuit | k | Rows |
| --- | --- | ---: | ---: |
| #285 | `foo1` | 7 | 75 |
| #285 | `foo2` | 7 | 75 |
| #285 | `foo3` | 7 | 75 |
| #588 | `attest` | 13 | 1,972 |

Signed size receipt SHA-256 is
`87eceee790401ca41e8bf35d06a40ba487dbb36242187de933c7fe8bfaaba9ec`.
The retained script, exact raw copies, resulting binary IR and raw logs bind
these measurements. No keys, proofs, Cargo build or original runtime callbacks
ran in this size analysis. No size reduction, proof latency or CI saving is
inferred; the numbers bound the next proof work after generated Rust execution.


## First actual generated SDK1 typecheck

The sealed SDK1 stage at `fc74c7a91807eee9bbc471b88d488d20df6d92e2`
reaches real Rust 1.99 compilation and fails on generated #588. The emitter
places `#[compact_encoding(ledger9_v3)]` before the derive that declares it,
which Rust rejects as `legacy_derive_helpers` under `future_incompatible`.
The run is **failed and unqualified**: 367 partial Cargo artifact records and
322 package IDs establish build activity, not a completed matrix or test pass.
No selected test binary or three-test native/recorded/Verify result exists.
The original material and the complete failed command/outer/raw logs remain.

Signed failure receipt SHA-256 is
`a0412495dbb1db4c9827e67777b2875b5c97aeb9d34e254290a5c1a2ca5b961b`;
independent failure audit SHA-256 is
`8f57f4112210beea30c6db5e270c286cebbd8997ef29f5ace8f980da8dac2e29`.
One structured diagnostic is retained; Cargo's summary of two prior errors is
not expanded into an invented second diagnosis.

Signed local fix `25a0c14649576d678e326ccf3ea6a90946b23e29` appends the
helper attribute after the derive. Its conditional Legacy20 path is unchanged.
A focused emitted-enum AST check protects that ordering; independent static
review SHA-256 is
`91f47dedfd5ee0ec74dc9784a4f56a36c0dceb60b42bfa371fedd2186e2675fc`.
The next reviewed producer recipe at `09dac33d7` reanchors the established
backend/CLI/library-lint/material pipeline to this source. That producer and
the subsequent SDK1 retry now pass in the separate qualification below. No
generated output is hand-edited and no Rust lint or contract assertion is
weakened.


## Corrected producer and generated SDK1 execution

Signed producer `09dac33d7d6cb5cb14f35d547615f584bf2c623e` passes all
1,987 backend library tests (zero failed, eight ignored, zero filtered), 48 CLI
tests and strict library-only Clippy. Both ignored original-fixture material
drivers pass one test each with 1,994 filtered. Fresh maps contain #285
133 files/21 directories and #588 131 files/21 directories. The #285 Rust
bytes remain identical; #588 changes only the helper attribute order. Full
9,859-source/six-support and dependency pre/post checks pass; Cargo records
251 actual compiled IDs; 524 external candidates are authenticated separately. Signed
producer receipt SHA-256 is
`1422204bdeaa3a1ca6231761d30fd8e039767d3d56bd1eaf0e95609500bc8a59`;
independent execution review SHA-256 is
`0f9a4d12cd9bd95c73f03381cde3ee103e22c3eefc261de201f8e2dc740a1625`.

The separate private registry-SDK1 consumer at
`d0655a12f36ddb02a797cc02eb89130b133cfa65` compiles the unchanged freshly
generated contracts and passes all three supplemental Rust API tests. These
cover #285's three circuits and #588's attestation: native/recorded result,
state, effects and gas parity; Verify replay and wrong reads; input-1 cast
refusals; observed handles; stored attestation/hash; and the legacy carrier and
`aae89fc0...` hash in the same binary as the v3 `7dd1657a...` hash. Independent
execution of the retained binary repeats the same three named passes, with
zero failures, ignored or filtered tests and unchanged input/binary bytes.

Cargo JSON has 370 artifact records and 325 actual compiled package IDs,
including 26 Midnight registry records and all ten required released core
libraries. The new lock has 404 registry packages plus five local packages;
405 authenticated cache candidates comprise the older Caller 400 plus five
reviewed extensions. These counts are distinct. Each source runtime closure
has 120 files; the SDK1 stage preserves 119 files byte for byte and substitutes only
the reviewed runtime Cargo manifest. Source-SDK2 compatibility metadata is not
renamed to SDK1. The historical fc74 failure remains failed.

Signed generated execution receipt SHA-256 is
`4b85bdec27376d1ebc2435c57c506e92ad5537d867174c54dcbd7c1c7b3b4e57`;
independent review SHA-256 is
`9ad823f96afd0c0e59bad49f706c763bd7b5a72fcba4969059c96de0f144a693`;
fresh replay SHA-256 is
`2f3d8bd8a3608bf012487da1b9d8d0d743acbaafb6f00734ae13f6eb0fd6e03f`.
The completed owned producer and SDK1 build targets are removed after review;
the exact source fixtures, binaries, stages, lock, material and raw logs remain.
This qualifies private local arm64 generated APIs against SDK 9.1.0.1. It
creates no proof or funded application and qualifies no public schema21
admission, global SDK1 ABI, live deployment or native Linux result. The original
upstream two-case suite remains compile-only; the new Rust tests are additional
controls. A separate four-circuit proof/funded source slice is next.

## Private proof harness: SDK1 compile correction

The first compilation attempt for the separate four-circuit proof fixture at
signed `f007efc56` fails before any test or proof execution. SDK1's field type implements
addition but not `AddAssign`, so both supplemental wrong-input mutations using
`+=` fail with E0368. Cargo retains 369 partial artifact records and 324
package IDs; these are not a completed typecheck. The separate verifier build
is unrun. Signed failure receipt SHA-256 is
`3da93a0fedd771e5e970d0cdc2ef8184a16d05a42112d2449fef7b9afa994300`.

Signed correction `5a17e13ea` uses equivalent `x = x + Fr::from(1u64)` mutations
and removes one unused import, preserving the negative assertions. The `a9b0993f2`
stage binds the corrected harness and verifier, full prebalance transaction
snapshot and unchanged d065 generated crates. Its offline lock still has
404 registry and five local packages. The reviewed cached compile-only retry
now passes both `cargo test --no-run` and the separate verifier binary build.
Cargo reports 741 artifact records across the two commands and 325 distinct
package IDs, including 26 Midnight registry records and all ten required core
libraries. Both executables and the exact lock are retained. Signed compile
receipt SHA-256 is
`a52d156225056e34e92782d60c41a1c219b97806aea2965a9a9ec087d3488cbc`.
Neither executable has run a proof or application in this gate.

The retained 28-file parameter set matches released SDK1's embedded hashes
and is reused without copying or mutation. The separate four-circuit proof
and funded gate below now passes independent qualification. Public schema21, global SDK1 ABI and Linux remain open.


## Private SDK1 fresh proofs and funded replay

The separate run at signed source `a9b0993f2` now qualifies all four unchanged
#285/#588 circuits against released SDK 9.1.0.1. Four fresh key-generation
commands produce distinct verifier keys. One supplemental ignored Rust test
executes the four cases and passes: `foo1`, `foo2` and `foo3` prove the zero
argument/result writes; `attest(7)` applies the expected v3 `7dd1657a...` hash
while preserving the legacy `aae89fc0...` result in the same binary.

Each funded transaction passes default Real proof checking and balancing,
applies successfully, and refuses replay with `IntentAlreadyExists` and exact
unchanged ledger bytes. Eight targeted strict mutations refuse: the contract
proof and a positive-fee Dust proof for each case. Corrupt proof, changed
public input and distinct wrong-key controls also refuse. Full sealed
prebalance transactions are retained, and their complete call, binding,
proof and public-input bytes match the funded transactions.

Four separate retained-verifier processes pass. Independent review then
freshly repeats all four verifier calls with unchanged run, binary, parameter
and static-input bytes; this generates no additional proof. All nine producer
commands exit zero. The 68-file/11-directory run retains hashes, sizes and
modes, raw logs, keys, proofs, preimages, transactions and before/after/replay
states. The 28 parameter files are reused read-only. The seven test-static
files also match the authenticated released SDK1 crate archive byte for byte.

Signed qualification SHA-256 is
`59d95b5572cac9c2abdbdbd0d63e5649a74eb0023edfcb5d66958e9792206db6`;
independent review SHA-256 is
`8a7f7de4ad7c04695612fc6dea16959c3b2d65149a5786bfae50ce9c605f6f89`;
fresh verifier replay SHA-256 is
`7d7c92d1ae71387d7906a6aafb6862ffac1877d4048c543e40133a9940c1b76b`.
The earlier missing-environment and compile failures remain failed. Completed
owned Cargo target and worktree are removed after review; retained inputs and
all qualification evidence remain.

This adds four actual contract proofs, separate from auxiliary Dust/balancing
work. It adds no original runtime cases: upstream's two tests remain unchanged
compile-only cases, while this new proof test has four supplemental cases.
This private Darwin arm64 gate establishes neither public schema21 admission,
global SDK1 ABI, live deployment nor native Linux qualification.
