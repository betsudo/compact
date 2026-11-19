# Compact Rust backend and runtime

[Milestone 0.3.0 closure report](0.3.0-closure.md) · [Acceptance crosswalk](evidence/0.3.0/closure/crosswalk.md)

[Historical proposed 0.4.0 Ledger 8 functional scope](0.4.0-ledger8-functional-scope.md) preserves the `ab132650` Aiken-inspired generated Rust direction and proposed capability seams for later Ledger9 work. Its illustrative APIs are not delivered interfaces, an accepted milestone scope or a Ledger9 support claim.

The Rust target uses the Compact frontend, a typed private IR, a Rust AST emitter (`syn`, `quote`, `proc-macro2`, `prettyplease`) and a native runtime built on Midnight ledger/zk primitives.

## Ledger 9 migration

Use the [Ledger 9 adoption plan](plans/ledger9-adoption.md) for the local
migration status and the [versions and targets guide](guides/versions-and-targets.md)
for the matching compiler, Rust and TypeScript runtime, and ZKIR tools.
The [upstream compatibility plan](plans/ledger9-upstream-compatibility.md)
tracks mainstream Ledger 9 capabilities and test coverage across both targets.
The [dated Ledger 9 status digest](ledger9-status.md) gives a GitHub-safe
publication and CI resource summary.

The [cross-contract foundation](https://github.com/betsudo/compact/pull/774)
merged into `codex/rust-backend-ast` at `eb3b2adbb`. The [20-circuit
original-token follow-up](https://github.com/betsudo/compact/pull/775) was
merged into the same branch at `b6e7d156e` on 19 November 2026, from
rebased head `297ea0b17`. All three hosted workflows passed at that source
head, including existing Rust proof qualification. The earlier green
`867b25d88` run retains its separate source identity. Signed local macOS arm64 proof
evidence covers all 20 unique original-token Rust circuits at its recorded
source revision. Native Linux proof and funded application remain open in
[#773](https://github.com/betsudo/compact/issues/773).
The separate [released SDK1 artifact consumer](plans/original-token-sdk1-consumer.md)
now verifies all 31 retained standalone original-token proofs with 93 refusal
controls and decodes 23 tagged transactions, with independent process replay.
This adds no fresh proofs or SDK1 ledger application/replay.
Ordinary generated public `pay` is not qualified by the merged foundation.
An independently reviewed local follow-up now qualifies public generated
`pay` with fresh A/B proofs and funded offline replay at its signed head;
the [upstream compatibility plan](plans/ledger9-upstream-compatibility.md)
records its exact scope and remaining gates.
The exact original Events pair now has reviewed local trusted CLI admission,
fresh proofs, separate verification, and two sequential funded default-strict
applications with replay refusal on source SDK 9.1.0.2 and released SDK 9.1.0.1.
Actual applied Outer-before-Inner logs and persisted Inner values 1 then 2
match. The qualified frontend uses a source interpreter wrapper; packaged
compiler and native Linux execution remain open.

The unchanged original Basic, Storage, Witness, Callee Witness and Caller
TypeScript suites have separate local qualifications. Callee Witness passes
3/3 and Caller passes 6/6 on both released and source-generated paths. Their
`--skip-zk` results do not establish generated Rust caller proofs or funded
caller transactions. Key Agreement also passes its unchanged 7/7 cases on each
TypeScript path after independent review. A source-reviewed native Events
handoff is prepared to execute admitted output; full Linux mode is unrun.
A reviewed consolidation additionally passes all 40 paired original Basic,
Storage and Witness TypeScript cases with unchanged neighboring Events bytes.
Original recursion passes six active cases per path with five original skips
retained; its metadata-cycle fix preserves public and legacy refusals. Purity's
original runtime refusal passes 1/1 on each path with stage-correct legacy
compiler coverage. A separate reviewed [Rust caller API slice](plans/kernel-caller-rust-api.md)
qualifies original Inner's VM/native/recorded query and constructor; its changed
runtime requires a signed package; the v5 admission continuation is linked
below. A private generated Caller continuation now executes `viaInner` with the
observed Outer identity; its reviewed six-key physical candidate now has a
typechecked identity gate and reviewed private token execution. Caller
A/M/I `viaMiddle` now has reviewed local source-SDK2 qualification with six
contract proofs, one final Dust proof, strict funded application and unchanged
replay refusal. Independent fresh artifact verification passes. A separate
released-SDK1 registry-only Caller consumer verifies the same six retained
proofs and exact strict funded/replay ledger bytes, with independent process
replay; no additional proofs are counted. Public admission, `Some(User)` and
native Linux remain open. The
[status digest](ledger9-status.md) and both plans record exact source heads,
receipt hashes and remaining gates; historical slice limitations remain bound
to their original revisions. General cross-contract parity, post-child event
ordering, packaged compiler, native Linux and live-node qualification remain open.
The [v5 runtime/CLI continuation](plans/original-events-v5-status.md) now
passes independent physical and actual admission review, preserving archived
v4 admission. Reviewed local `bffd8d788` now qualifies seven fresh source-SDK v5
proofs and two sequential strict funded applications/replay refusals. Reviewed
`4715d0648` verifies all seven proofs and both exact funded state/replay sequences
on released SDK1. Packaged compiler and native Linux qualification remain open.
The [original block-time case](plans/original-block-time-testchain.md) also
passes all twelve TypeScript assertions per path. A reviewed separate Rust
slice also passes twelve native and twelve recorded observations; its proof,
funded and Linux gates remain open.
The [original host metadata gate](plans/original-host-metadata-status.md)
passes five unchanged expectedVk cases and 31 guards at a separate reviewed
source profile. Caller physical identity checking now passes Cargo checking;
private token execution is reviewed, while full Outer/public multi-hop admission remains open. These results
do not qualify public defaults, fresh new-runtime proofs or native Linux.
The reviewed combined host branch now passes 21/21 unchanged original cases
on released and private source profiles. [ECDSA generated Rust parity](plans/original-ecdsa-parity.md)
also runs the original suite through a private adapter, with nine Rust calls
and one host-only case. Caller token execution and a bounded unfunded VM tree
are reviewed locally; full Outer/public multi-hop admission and proof/funded gates remain
open. These source/runtime identities remain distinct from archived packages.
The released-matrix pure ECDSA Rust consumer is independently reviewed with
the same nine Rust calls and one host case; optional ledger 1.0.1 is not
compiled. [Original curve runtime tests](plans/original-runtime-curves.md)
also pass 41 P-256 and 44 Curve25519 cases against published runtime 0.21.
The accepted source runtime also passes those 85, and original secp256k1 adds
30 on each profile: 115 unique original curve cases per profile, across
separately bound slices. These are pure local arm64 JavaScript results.
Private generated Outer/Middle/Inner now record the actual nested caller
chain with persisted receiver checks. A joint signed package, authenticated
assembly, proofs/funding and Linux remain open.
The new A/M/I physical candidate is reviewed, with 1,652 files/77 directories
and 16 refusal controls; checked execution is next. [Runtime resolution](plans/original-runtime-resolution.md)
passes the original 14 cases on published runtime, while source keeps its
closed-export incompatibility explicit. The source's existing suite also passes
465/465 across 24 modules; that overlapping adapted/internal count is separate
from intact original coverage.
Checked A/M/I token execution is now independently reviewed: five private VM
tests, 26 dynamic refusals, 23 physical refusals and six compiler refusals pass.
Closed unfunded prepartition preparation is next; proof/transaction authority,
funded replay, released SDK1 Caller execution and Linux remain open.
The [CI test graph](plans/ci-test-graph.md) and [runner resource budget](plans/ci-runner-resource-budget.md)
record measured hosted runs and local transfer observations. The grouped #774
and earlier #775 runs consumed 468.67 and 439.60 summed assigned runner
minutes respectively, with different scopes and source heads. This migration
checkpoint does not claim a measured circuit-size, prover-speed or CI-cost gain.

## Retained Ledger 8 delivery

The [native Ledger8 candidate profile](guides/native-ledger8-profile.md) records exact versions, usable source SDK, stage-specific evidence, type/runtime boundaries, exclusions and remaining integration gates. It is a local handoff contract, not release or whole-goal completion.

- [Delivered features, architecture, examples, qualification and completion checklist](ledger8-delivery-and-completion.md): dated 18 October 2026; separates bounded local implementation from supported-profile release readiness and full language parity.
- [Keccak capability and proof-profile audit record](audit/2026-10-18-keccak-profile-boundary.md): native/ordinary checked support, the ZKIRv2 exclusion, observed target selection and the qualification needed for an extension.
- [Stateful product Vector widening](plans/stateful-product-vector-widening.md): reviewed bounded local compiler177 source/API and selected genuine proof/default-strict application qualification; full coverage and integration remain open.

- [Stateful Vector products with Vector components](plans/stateful-vector-product-elements.md): reviewed bounded compiler180 API/static/ingress/error qualification and two selected fresh proofs with separate funded default-strict applications. Seventeen required exports, including four intact k9 models, remain unproved; full integration is open.

- [Products below Vector components](plans/vector-component-products.md): reviewed bounded compiler183 source/API/static/ingress/producer qualification, 24 complete SDKs and 35 cases. Three existing complete sources now have reviewed fresh proofs and separately proved funded default-strict applications. Twenty-one required exports remain unproved, including all eighteen over-limit models. The complete cohort is wired into the required Linux compiler workflow and locally requalified in the arm64 Nix shell; hosted CI and full integration remain open.

- [Current-candidate isolated witnessed SDK handoff](plans/current-profile-sdk-handoff.md): reviewed repaired compiler 0.31.183 local qualification of the existing documented consumer, actual proof-feature library Clippy, typed refusals, fresh proof and separate funded strict transaction. Accepted compiler143 integration remains separate; full CI and profile handoff stay open.

- [Exact-package default bundled SDK](plans/current-package-bundled-sdk.md): current compiler183 package qualification passes with explicit installed-prefix admission, refusal of runtime overrides (including empty), runtime-byte controls and fresh selected proof/strict application. Prior SDK epochs and full integration remain separate.

- [Current-candidate Passport consumer handoff](plans/current-passport-handoff.md): reviewed local compiler183 qualification passes for the intact pure application, raw SDK and documented consumer, with independent provenance controls. Historical application/fixture epochs remain separate; full integration is open.

- [Current-candidate DID lifecycle handoff](plans/current-did-handoff.md): reviewed local compiler183 API and fresh selected deactivation proof/default-strict application pass with all twelve Programs preserved. Its original selected scope remains distinct from the later zero timestamp extension; full integration stays open.

- [Zero timestamp DID witness](plans/did-zero-timestamp.md): reviewed local compiler183 coverage of both original success cases across four routes, case-bound fresh proofs and separate funded strict applications. The zero proof now has explicit local workflow wiring; hosted CI and full integration remain open.

- [Finite native supported-profile handoff](plans/current-supported-profile-handoff.md): reconcile current stage-specific capabilities, exceptions, examples and remaining integration gates without reopening accepted work.

These records retain their exact source/evidence revisions. They do not claim PR merge, completed hosted validation or a released Ledger9 target.

- [Nested product request-field baseline](plans/nested-product-request-field.md): local compiler184 metadata repair passes strict complete-SDK lint, four typed routes and marked static controls. Nested widening, automatic consumer-driver wiring and full integration remain separate; the compiler183 application/proof epochs are not relabelled.

- [Outer Vector product semantics](plans/outer-vector-product-semantics.md): reviewed local compiler191 qualification of the complete91-command native/API/ingress protocol, all16 models and all15 eligible fresh direct proofs with separate funded DEFAULT-strict applications/replay refusals. The intact family branch remains k9 and unproved. Earlier application/proof epochs are not relabelled; full CI/Linux and integration remain open.

- [Outer Vector proof CI wiring](plans/outer-vector-product-proof-ci.md): merged through #704 at canonical `9dc8d22b6`, with all nine required ordinary checks passed. Retained tree-equivalent Linux execution includes all 91 native checks, 15 fresh verified proofs and 15 separate strict applications. Hosted macOS arm64 remains optional and unqualified by skipped jobs. The cron runs only on default `main`; #706 records the post-merge coverage/cost-policy follow-up.

- [Constructor Vector iterable operands](plans/constructor-vector-operands.md): reviewed local compiler194 qualification at `847758e3` covers 14 complete SDKs, nested lexical captures, constructor state/witness/owned-value behavior, all 42 intact models and 17 selected fresh later-entrypoint proof cases with separate funded DEFAULT-strict applications and replay controls. Earlier compiler192 results retain their own identities. The maintained combined CI command is being qualified separately; constructor cryptography/total cost, new-head CI/Linux and integration remain open.

- [Constructor equality conditions](plans/constructor-equality-conditions.md) (#707): local compiler195 qualification covers eight complete SDKs, all 24 intact k6 models and sixteen selected later-entrypoint proofs with separate funded DEFAULT-strict applications/replays. The current maintained command passes at `041b74e47`; independent actual-result review accepts its bounded local results. Required Linux wiring is implemented locally, with hosted qualification still required. Top-level constructor `if`, constructor cryptography and full integration remain separate.

## Checked recording checkpoint

The current recording-effect branch is documented in [Checked recording and source diagnostics](guides/checked-recording.md) and the [capability and retirement book](recording-effect-algebra/capability-and-retirement-book.md). The compiler/runtime ABI is **53**, private Rust IR is **20**, finalized capability reports use **3**, and private source-origin transport uses **3**. Package versions remain separate.

The previous milestone reports below retain their own revisions and evidence. Current closure requires the final local CLI and authenticated historical qualification, followed by independent review; their success is not asserted by this documentation draft.

## Guides

- [Start with a task: compile, implement witnesses, test and integrate](guides/index.md)
- [ContractLab testkit](../../testkit-rs/README.md)
- [Unit-valued mutation initializers and ordered comparisons](guides/unit-initializers.md)

- [Compiler backend and generated crate guide](../../tools/compact-rust-backend/README.md)
- [Runtime APIs, ledger ownership and ABI](../../runtime-rs/README.md)
- [Mechanical representation derives](../../runtime-rs-macros/README.md)
- [Rust architecture decision register](adr/README.md): 360 published records, topic index, original aliases and dated engineering history
- [ADR publication provenance](adr/publication-manifest.json) and [evidence conventions](adr/references.md)

## Accepted baseline and governance

Current 0.3.0 engineering acceptance uses implementation `f9a49666`, private IR20 and runtime ABI50; see the [closure report](0.3.0-closure.md) for its composite qualification, application scope and known limits. Package versions are unchanged.

Milestone 2 acceptance belongs to implementation revision [`03c03a39`](https://github.com/MediaNoxLabs/compact/commit/03c03a39a2d39c43c91143b9526ca1c2ffae706c): compiler 0.31.133, private IR schema 20 and runtime ABI 49. See the [public closeout](https://github.com/MediaNoxLabs/compact/issues/103#issuecomment-6017383781) for source/behavior/proof/live evidence scopes and distribution limits. Publishing these documents does not rerun implementation validation.

The Rust ADR series records branch implementation decisions. [CoIPs](../../coips/README.md) remain the separate upstream proposal and adoption process, with independent numbering and review. No new milestone is opened by this publication.


The reviewed [six-case Rust Curve serialization bridge](plans/original-runtime-curves.md#six-original-serialization-callbacks-through-rust)
at `df89e5c82` passes six unchanged original callbacks with 37 independently
replayed calls including separate diagnostics; 38 Curve callbacks remain
filtered on this Rust path. This is a pure runtime API result, not generated
contract, proof, full SDK ABI or Linux qualification. A separately reviewed
[zero-width Rust encoding counterexample](plans/zero-width-ledger9-encoding.md),
tracked in [#784](https://github.com/betsudo/compact/issues/784), records the
historical difference from released v3 attestation hashing. The subsequent
private profile/API and proof qualification below reproduces the oracle while
preserving legacy Ledger8/v2 behavior.


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
refusals and strict macro Clippy. This direct API gate predates the private
generated/profile55 results below; public ABI55 admission remains open.


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
runtime test cases or proofs. The separate [private SDK1 proof gate](plans/zero-width-ledger9-encoding.md#private-sdk1-fresh-proofs-and-funded-replay)
now qualifies four fresh V3 proofs, four default-strict funded applications,
eight targeted mutations and exact replay refusal, with independent fresh
four-verifier replay. Public schema21 admission, global SDK1 ABI, full parity
and native Linux remain open. The separately qualified [original CamelCase
host gate](plans/original-host-metadata-status.md#qualified-original-camelcase-and-artifact-finalization)
passes 2/2 on released and source profiles. Together with the earlier gates,
the host union is eight modules/24 cases per profile across three separate
qualifications, without a combined 24-case or full 45-module result.
The [hardened v5 Linux source handoff](plans/original-events-native-linux-handoff.md#git-object-binding-hardening)
at `b3e677de6` passes independent source review and an independent rerun of
12 tests/77 refusals. Actual Linux compilation/proofs remain unrun. Owned
completed build copies and worktrees are removed; source stays local.
