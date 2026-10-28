# Ledger8 Rust: delivered work and the finish line

Historical implementation snapshot · 18 October 2026, 14:59 UTC · Ledger **8.1.3**, ZKIR **v2**

## In plain words

We now have a Rust compiler path that composes supported operations instead of recognizing whole examples. On top of it, the opt-in functional surface generates an immutable request with one execution body. Native execution and recording run that body through different Ledger8 adapters.

The latest delivery adds the missing witness, value, collection, Merkle and Ledger8 effect families to that functional surface. It has real execution, proof, verification and ledger-application evidence for selected cases.

**The bounded Ledger8 implementation is integrated through reviewed PR #664. The primary Rust AST goal remains incomplete.** Published slices through #696 have passed full CI and review and are merged into the canonical branch. Current real-application requalification, distributable artifacts and remaining source-language gaps are separate work; the historical evidence below retains its original scope.

The statement-action follow-up is verified merged as PR #696 at signed canonical
`f0e4ba788`, with the same complete tree as reviewed `9667072e7` and actual CI
checkout `b7c15323b`. All eleven checks, actual Claude review and retained runtime
artifact verification pass. Local compiler141 receipts retain literal `cf7a2fa2c`
and earlier epochs; the [integration closeout](plans/516-unit-action-bindings.md#verified-integration--pr-696)
records current scope. Historical development checkpoints remain separate.


There is no honest overall completion percentage. A checked-operation variant, a source-language construct, a test scenario and a proved transaction are different units.

## What has been delivered

| Area | Delivered behavior | Boundary to keep visible |
| --- | --- | --- |
| Rust-AST foundation | Structured Rust generation with `syn`, `quote`, `proc-macro2` and `prettyplease`; checked effect composition replaces production shape admission in the delivered algebra work. | Earlier delivery, local acceptance and merged availability have different revision histories. |
| Readability | Measured, focused generated-code simplification; smaller named component owners and ordinary Rust constructs. | This is not a promise that every generated body reads like handwritten application code. |
| Immutable functional surface | A compiler-issued `Program`, one generic `run` body, native/recorded convenience methods and consuming execution. | Opt-in experimental API; ordinary generated APIs remain available. |
| Witnesses | General typed signatures; Unit, Field, Bytes and products; explicit exact opaque transport; metered witness views and typed unknown-identity errors. | Handwritten callbacks are ordinary Rust, not compiler-certified privacy/effect implementations. |
| Pure values and composition | Shared arithmetic, comparisons, checked conversions, aggregates/projections, local calls, context-free VectorMap and certified pure helper Fold. | Type combinations and resource bounds remain checked. Direct stateful Fold is integrated through reviewed PR #690 with its selected source/proof scope; arbitrary Fold/source combinations remain checked. |
| Collections | Cell, Counter, scalar Map/Set, List and structural nested-Map inspection/removal, through existing slots/frames. | Nested Map inspection does not imply scalar value codecs, arbitrary nested insertion or every frontend expression form. |
| Merkle | Plain/historic typed operations, retaining depth/history, slot identity, paths and existing validation authority. | Representative proof cases do not prove all tree operations or all depths. |
| Crypto | Sixteen admitted Ledger8 hashing, Field encoding and Jubjub operations use shared syntax and official primitives. | Keccak is refused for this pinned ZKIRv2 functional target. Curve arithmetic is not a blanket signature-security claim. |
| Ledger8 effects | Execution identity, kernel claims/minting, Zswap intents and qualified Cell/Set storage via explicit official-domain adapters. | Constructors keep their original owner. Held allocation indices do not prove offer provenance; `ownPublicKey` observation is not an ownership proof. |
| Diagnostics | Typed witness identity failures and checked type/target/path refusals, preserving original causes. | Nested source ranges and consumed-source/privacy guarantees are not complete. Unknown remains Unknown. |
| Test infrastructure | Generated consumers, ContractLab and official proof/provider integration using direct uncached key generation. Forge comparison is deferred on `experiment/zkir-forge`. | Proofs and assertions execute every lap. No proof, pass/fail decision or test result is cached. |

The exhaustive functional match currently admits **71 of 72 top-level checked Operation variants**, conditionally under existing guards. Keccak is the excluded variant. List suboperations, Merkle forms, claim kinds, type combinations and source syntax are additional dimensions. **71/72 is not 98.6% language coverage.**

### Why Keccak is excluded here

Native Rust Keccak and the ordinary typed checked operation already exist ([#581](https://github.com/betsudo/compact/issues/581)). The new functional surface refuses it under the pinned Ledger8.1.3/ZKIRv2 profile: the upstream v2 compiler explicitly requires `--feature-zkir-v3` for this primitive. The flag selects a different pinned ZKIR dependency revision; it does not qualify that revision with our Ledger8.1.3 Rust proof/application graph.

The roadmap currently places proved Keccak with later ZKIRv3 target qualification. There is no accepted commitment to backport it to the pinned v2 profile. A compatible Ledger8 extension is a feasibility question, not an established impossibility: first demonstrate one complete Keccak proof/verification/strict-application flow on the exact supported graph, then expose a target capability. The existing host implementation alone is insufficient evidence.

## How the pieces fit

```text
Compact source + retained invocation origins
                    |
          checked typed effect plan
                    |
       generated immutable Program
                    |
         one generic execution body
             /                 \
      NativeLedger8      RecordedLedger8
             \                 /
       existing frames, slots and codecs
                    |
         official ledger/ZK domains
                    |
       preparation -> proof -> verification
                    |
       application with stated test policy
```

The checked plan owns types, scope, ordering and branch validation. Adapters own Ledger8-specific physical paths, frame state and official values. Generating Rust syntax is downstream of those decisions. This separation prepares the architecture for another ledger target; it does not implement Ledger9.

Recorded execution produces the material used by preparation and proving. Native success alone does not establish that a proof can be built or that a funded transaction will apply.

## What a developer uses

This real source reads a Field, replaces it and returns the previous value:

```compact
import CompactStandardLibrary;
export ledger stored: Field;

export circuit replace(next: Field): Field {
  const previous = stored.read();
  stored.write(disclose(next));
  return previous;
}
```

Generate with the matching source runtime:

```sh
compactc --target rust --rust-functional-ledger8 --skip-zk \
  --rust-runtime-root /absolute/path/to/compact \
  examples/rust_backend/let_return_oracle.compact ./generated
```

The following consumer excerpt uses the actual generated API; the caller supplies an initialized context:

```rust
use compact_contract_let_return_oracle::{
    ledger_contract::functional::replace,
    runtime,
};

pub fn record_replace(
    context: runtime::context::CircuitContext<()>,
    next: runtime::Field,
) -> Result<
    runtime::recording::RecordedCircuitResult<(), runtime::Field>,
    runtime::CompactError,
> {
    replace::Program::new(next).execute_recorded(context)
}
```

`execute_native` selects native execution. Building the request performs no witness or ledger work; execution consumes the request and context. Witness-bearing entries additionally use the generated witness interface. A failed run does not magically recover the consumed frame or erase external witness activity.

`--skip-zk` skips key generation, not code generation. Functional mode currently needs a stateful exported circuit, retained frontend origins and the matching source runtime; registry runtime mode is refused. A pure-only library remains a pure-library consumer, not an artificial stateful/proof fixture.

## What the evidence establishes

### Keep four questions separate

| Question | Accepted evidence | What it does not establish |
| --- | --- | --- |
| Is the implementation internally checked? | Compiler/runtime tests, typed refusals, resource controls, Clippy and header gates. | Complete language/specification coverage. |
| Do selected generated programs behave the same? | Fresh whole-source generation and ordinary native/recorded versus functional native/recorded comparison; state, effects, outputs, witness order/failure, gas and replay controls. | A fresh TS execution/proof comparison for every one of these cases. |
| Can selected recordings prove and verify? | Real core/domain proof runs with original bindings, verifier and negative controls. | Every admitted operation or every application is proved. |
| Will a transaction apply? | Explicitly funded default-strict cases, separately labeled diagnostic/application lanes. | All proof-only or relaxed-balancing cases are valid production transactions. |

### Accepted delivery at `2477c3e25`

| Gate | Result |
| --- | --- |
| Backend, all features | **2,146 passed**, zero failures, seven explicit-run helper entry points ignored |
| Runtime on Rust 1.89.0 | **285 passed**, zero failures/ignores |
| Strict Clippy | Both packages, all targets/features, warnings denied: passed |
| Generated consumers | **56 tests, 160 scenarios, four routes, 23 whole source contracts**; generated Clippy passed |
| Actual proof qualification | Seven scenario groups passed cold Forge, warm Forge and direct uncached lanes |
| Artifact/guard audit | Complete artifact hashes match across the three lanes in each group; live premise guards and cleanup passed |
| Review | Independent coding-agent and Claude CLI reviews found no remaining blocker; nonblocking hardening is tracked |

These cohorts overlap; they must not be added into a language-coverage score. The ordinary backend command did not directly run those seven entry points; their external workflows need separate invocation/evidence.

### Why seven entry points are ignored by ordinary Cargo discovery

These are explicit-run exporters and measurement helpers. They need selected input corpora, output paths or experiment parameters; several deliberately require a new output location to protect existing evidence.

| Entry point | Why it is opt-in |
| --- | --- |
| Candidate admission report | Scans a selected IR corpus and writes evidence to an explicit output path. |
| Helper-planner differential export | Compares original/current helper plans using a supplied corpus and fresh evidence output. |
| Complete old Unit-rule matrix | Requires an authenticated external matrix/authority and fresh durable output. |
| Retained Map-family export | Requires authenticated external inputs, retained roots and an explicit output location. |
| Cumulative typing resource measurement | A controlled single-fixture timing/RSS experiment with mandatory report path. |
| Keccak FAB-domain consumer export | Writes raw Rust that a separate qualification step must compile. This is not a ZKIRv2 proof-support claim. |
| Checked Fold runtime-order probe | An expression-export child entry intended for its parent harness with a selected output path. |

The ignore annotations keep these special workflows out of the default suite. The suite summary alone does not establish they were separately invoked, passed recently or cover any missing feature. A blanket `--ignored` run without their required setup would not be meaningful qualification.

Core qualification contains five case executions per lap: two Merkle branches, hashing, Jubjub and exact witness transport. Core Merkle/crypto retain the original balancing-disabled `check_transaction` lane; exact transport is funded default-strict. Domain qualification contains call-proof-only cases, funded default-strict applications and explicitly relaxed-balancing diagnostics. Its 42 positive-flow markers across three laps are **not 42 accepted transactions**.

The initial consumer/core/domain pipeline took 792.95 seconds locally. This is an observation of that machine/run, not a performance guarantee or a general Forge speedup claim.

### Historical publication candidate (superseded by the 2026-10-27 integration below)

The published final head is `3739976ca`. It differs from `2477c3e25` only in three license-header blank lines and module declaration ordering. It is not byte-identical, so its current-byte qualification is tracked separately.

The current-byte compiler rebuild, strict Clippy, fresh generated consumer and both core/domain proof suites have now passed. An independent audit accepted all seven three-lap receipts: inputs and guards stayed unchanged, cleanup succeeded, and complete ZKIR/prover/verifier artifact hashes matched across cold, warm and direct uncached lanes. These are fresh successor results, not relabeled predecessor receipts. Hosted CI and PR review remain pending, so release acceptance is not yet complete.

## Current integration — 2026-10-27

Canonical `codex/rust-backend-ast` reached `c3a529cb5f4dae3a74f1e4ac0baf94878f245d91` through signed merges. The following PRs are verified merged after their actual final-head required CI, full compiler/native/package/proof gates and independent review:

| PR | Contribution | Qualified source head | Signed merge |
| --- | --- | --- | --- |
| [#664](https://github.com/betsudo/compact/pull/664) | Ledger8 baseline and Forge isolation, retaining the earlier contribution stack | `de0c9ae5f` | `c353907d8` |
| [#688](https://github.com/betsudo/compact/pull/688) | Constant fixed-vector projection | `a98543313` | `d300a02ec` |
| [#689](https://github.com/betsudo/compact/pull/689) | Map size expressions and whole-Map metadata | `0fccce1be` | `38deaf9d3` |
| [#690](https://github.com/betsudo/compact/pull/690) | Direct Fold, including captured/local Counter amounts and source refusal controls | `fa8c8cc11` | `feefa6577` |
| [#691](https://github.com/betsudo/compact/pull/691) | Scalar terminal return widenings, preserving selected effects and failure order | `e568b6064` (CI tree equals synthetic merge `a84d08158`) | `c3a529cb5` |

Each merge tree matched its qualified source tree exactly. Native Linux x86_64 was qualified in GitHub Actions; local development uses Darwin arm64. The final Fold gate covers 21 full sources, 50 live-TS cases, 200 Rust route executions, 74 successful recorded replays, nine compile-fail controls and eight source refusals. Selected proof gates remain finite; they do not prove constructors or every accepted input. Earlier local/source receipts above remain historical epochs.

Forge CI adoption is deliberately deferred and incomplete. Selected contract-level reuse benefits are valid; whole-CI value remains unproven. The earlier “1% key generation” interpretation was incomplete: 46 seconds measured only Forge-managed generation; direct bypass generation was not separately timed, so the total ZKIR/prover-key/verifier-key share remains unknown. Backend progress does not depend on Forge. Its continuation is `experiment/zkir-forge`; separate repositories, signed results, stores, snapshots and failure evidence are retained.

Issue state is not a completion counter. The older recording-algebra PR #634 remains a separate history/publication item; do not blindly merge overlapping history.

## What is left, in priority order

### 1. Preserve accepted return and statement extensions

The typed-pair extension is integrated by reviewed PR [#692](https://github.com/betsudo/compact/pull/692)
at signed canonical `acf529eff`. Ordered return-scope Unit initializers are
integrated by [#694](https://github.com/betsudo/compact/pull/694) at `1d0288846`;
the bounded functional bind receiver prerequisite is integrated by
[#695](https://github.com/betsudo/compact/pull/695) at `709a86230`. These accepted
capabilities are not new feature work. Statement-action PR #696 is now integrated
at `f0e4ba788` after full CI/review and artifact verification. None closes all of
#516.

#### Historical typed-pair preparation (superseded by PR #692 integration)

The [scalar #516 slice](plans/516-field-return-wrappers.md) is integrated by
[#691](https://github.com/betsudo/compact/pull/691) after full current Linux and
Apple Silicon CI. Its packaged arm64 epoch retains 36 cases /135 executions /
56 replays, 321 passed /20 skipped Python controls and two selected fresh
2,912-byte proofs. Canonical merge `c3a529cb5`, reviewed head `e568b6064` and CI
synthetic merge `a84d08158` have the same complete tracked source tree.
Constructor/unselected proof and whole-#516 closure are not claimed.

The [typed-pair follow-up](plans/516-tuple-return-wrappers.md) is now based on the
actual accepted canonical baseline. Its local prototype passes 42 cases /159
Rust executions /68 recorded replays, typed mixed Uint/Field results, inline
Fold and exact each-component tampering controls. Four fresh selected
proof/application cases pass; 328 Python cases pass with20 existing skips.
Those earlier receipts describe an uncommitted development epoch. Fresh clean
original compiler 0.31.137 qualification now also passes the 42/159/68 API gate,
all four selected proof/application cases, all 280 generated fixtures and
328 Python tests with 20 existing skips. The maintained ordinary compiler suite
passes both ZKIR lanes. Full published-head CI, independent Claude PR review
and verified merge remain required. Other aggregate/initializer debt,
real applications and the broader Rust AST goal remain open. Backend work does
not depend on the deferred Forge experiment.

### 2. Requalify the real applications on the current profile

**DID:** the selected current-profile deactivation lifecycle is integrated by
reviewed PR [#693](https://github.com/betsudo/compact/pull/693), signed merge
`373fe452c`. It preserves the complete original source closure and all twelve
Programs, with live TS/four-route Rust comparisons, typed refusals and selected
fresh proof/default-strict application/replay evidence. See its
[integration closeout](plans/did-functional-lifecycle-current-profile.md#verified-integration--pr-693).
Constructor execution and proofs for every export are not claimed. Broader
application acceptance must be assessed against its actual missing scenarios;
do not reopen the accepted deactivation slice. Original Ledger8.0.3 and narrower
tutorial receipts retain their own profiles and scope.

**Digital passport:** retain the `midnight-vc-passport` integration baseline and
its referenced credential source closure: **75 public pure exports and zero
proof-applicable exports**. The [current-profile standalone consumer](plans/passport-current-profile-consumer.md)
is freshly qualified at clean signed `f0c09ff56`, original compiler
0.31.142/Ledger8.1.3: 202 sampled live TS cases plus codec vectors, 16 adoption
Rust methods and the actual unchanged two-test guide manifest/source. The guide
uses the documented current source-lock recipe; its original historical lock is
retained separately. Raw SDK, guide and adoption consumer pass strict Clippy.
Correct-type twins compile and two independent intended-locus E0308 refusals
pass; changed-expectation failure/restoration controls pass. All 389 public
Python methods pass with 20 existing manual-only skip events, and all 30 focused
controls pass. Cargo compiler/lint configuration guards, exact runtime-bundle identity, bounded
POSIX process-group/pipe timeouts and attempt-specific step-failure
uploads retain explicit hard-cancellation/runtime-transitive limits. Earlier
`834a41025` formatted-SDK and `2bbf4daaf` guide-source-only epochs remain distinct.
Original source/oracle authorities are unchanged. Own repeat review, complete
current-head CI and verified merge remain required. Recording/proof/application are not applicable to this
original pure library; do not invent stateful exports.

**Finish when:** both intact pinned source closures build and their applicable tests pass on the selected profile. Each refusal becomes a minimal regression and an issue; fix its actual cause before accepting the dependent application. Do not remove exports to make an oversized/unsupported contract appear supported. “Latest” upstream after these pins is a new adoption decision.

### 3. Close the known source/composition gaps where they affect the claim

| Issue | Current gap | Required evidence for a fix |
| --- | --- | --- |
| [#674](https://github.com/betsudo/compact/issues/674) | Integrated by #688 in the supported constant fixed-vector scope. | [Scope and controls](plans/674-constant-vector-projection-decision.md); no arbitrary dynamic indexing claim. |
| [#661](https://github.com/betsudo/compact/issues/661) | Integrated by #690 for the maintained direct Fold cohort. | [Source-family plan](plans/661-direct-fold-expressions.md); 21 full sources, selected proofs, explicit checked/refusal boundaries. |
| [#610](https://github.com/betsudo/compact/issues/610) | Integrated by #689 for stateful Map size expressions. | [Scope and controls](plans/610-map-size-expressions.md); whole-Map metadata, malformed-state controls and two selected proof/application cases. |
| [#516](https://github.com/betsudo/compact/issues/516) | Scalar wrappers integrated by #691; typed scalar-pair/inline-Fold extension by #692; ordered return-scope Unit initializers by #694. Statement-action extension is integrated by #696 after exact-head review, full CI and retained-artifact verification; whole-issue closure is not claimed. | [Statement-action plan](plans/516-unit-action-bindings.md); lexical/effect order, TS/four-route parity, independent refusal controls and selected fresh proofs. This is not whole-issue closure. |

Arm64 checks are the local development gate and Linux required integration runs in GitHub Actions. A local Linux runner is not a prerequisite for bounded backend work. Preserve accepted crosswalk and historical authorities; later source heads require their own integration evidence.

These are not retroactive failures of the bounded delivery. They **are blockers for contracts using them, and for an unqualified full-language claim**. For a deliberately supported-profile release, publish explicit exclusions instead of calling them solved.

Also retain the existing type/conversion/resource refusals. Counting a top-level conversion variant does not make every conversion legal. No new full-specification research campaign is required just to record these known gaps.

### 4. Make the generated artifacts independently consumable

**Finish when:** a fresh directory can extract/build the generated package and its declared matching runtime inputs, implement witnesses, run ContractLab and prepare a supported proof without paths back into a development checkout. Record deterministic compiler/source/target/interface/artifact identities, the supported lock/profile and early incompatible-artifact refusals.

A successful `cargo package --list` or a manifest that names an unpublished registry version is insufficient. Decide whether release distribution is a paired source SDK or published packages. Keep deployment identity, wallet/network clients and provider adapters separate from generated contract semantics.

The bounded [isolated witnessed SDK slice](plans/functional-isolated-source-sdk-consumer.md)
now has a concrete paired source-testkit recipe and actual executable ContractLab/
ProofLab example. Clean local proof/strict-application qualification passes for
one complete witnessed Cell source. Final source reconciliation, review and full
Linux CI are still required; this does not complete the distribution or handoff criterion.

### 5. Freeze a finite capability contract and finish the handoff

**Finish when:** one selected revision/profile has a readable capability/exception book, per-stage evidence, examples, migration notes and independent review. Include actual application outcomes, supported hosts, residual source gaps and relevant decoder limits. Reconcile the existing conformance register rather than inventing a new percentage from fixture counts.

Run affected tests during delivery and the complete required qualification at candidate freeze. Historical checks must run when their machinery or relevant premises change; accepted evidence can be reused only while its complete relevant premises remain valid.

Refresh ADR/CoPS publication and release metadata. The current `skip-changelog` allowance covers the experimental contribution stack, not a permanent exemption for a release. Keep exact tool/ledger/ZKIR/ABI/package versions separate. A milestone name is not a runtime version.

## What should not silently block a native Ledger8 release

| Work | Current disposition |
| --- | --- |
| [#655](https://github.com/betsudo/compact/issues/655), Forge hardening | Deferred to `experiment/zkir-forge`, outside the backend dependency chain. Historical follow-up: automate cross-lap identity/counter assertions and record actually used built-in proof material. Current successful guarded execution is accepted within its scope; no full offline reproducibility claim. |
| [#653](https://github.com/betsudo/compact/issues/653), historical public evidence | Required before claiming the entire historical acceptance is publicly reproducible. A clearly labeled maintainer-only historical report can retain that limitation. |
| [#633](https://github.com/betsudo/compact/issues/633), nested source ranges | Developer experience follow-up unless a selected acceptance case needs it. Current structural-path fallback is not exact source-range coverage. |
| Wasm | Earlier bounded Node/browser prototypes exist. The current functional surface needs its own feasibility slice before claiming support. Not a prerequisite for a native-only supported profile. |
| [#495](https://github.com/betsudo/compact/issues/495), aggregate decoder limits | Separate deferred resource/security policy. Existing encoded-byte admission is not aggregate heap/CPU containment. Reopen a concrete applicable blocker; do not describe the risk as resolved. |
| [#496](https://github.com/betsudo/compact/issues/496), network/wallet client | Separate application integration product. Official provider/test composition is not a complete MidnightJS replacement. |
| Ledger9/ZKIR3, P-256, secp256k1, ACC and cross-contract/network rollout | Separate target/application goals. They are not required to finish the declared native Ledger8 profile. |
| Formal proof of the compiler/specification | Separate assurance goal. Existing typed checks and tested behavior are valuable but are not a formal equivalence theorem. |

## A practical definition of “Ledger8 complete”

Recommended release wording:

> The Rust backend supports the published Ledger8.1.3 profile at a named compiler/runtime revision. Its stated source subset and applications compile and execute; selected required recordings prove, verify and apply under documented policies. Unsupported forms and platform/security limits are explicit. The generated distribution is usable from an isolated consumer and the frozen candidate has passed its required checks and review.

Use three distinct status labels:

| Label | Status now | Meaning |
| --- | --- | --- |
| Bounded functional implementation complete | **Yes, locally** | Approved slices 1–5 plus focused diagnostics are delivered and tested at their recorded revisions. |
| Maintained native Ledger8 supported-profile release complete | **Not yet** | Requires candidate/review closure, real-application requalification, consumable artifacts and a finite release handoff. Known exceptions must be explicit. |
| Every valid Ledger8 Compact program / full TS-spec parity | **Not established** | Known source gaps must be fixed and a broader requirement-to-stage coverage claim needs its own bounded agreement and evidence. |

The current bounded work is **Passport's current-profile standalone consumer integration**, with its own review and complete required CI. Statement-action integration is accepted through #696. Accepted typed-pair, DID deactivation and return-scope Unit work retain their recorded scope. After integration, assess the remaining application, distribution and capability obligations against the actual selected candidate; select a concrete semantic or API gap only when current evidence shows it. This does not expand the target to Ledger9 or declare the broader goal complete.

## Source and decision map

- Current API and qualification scope: [functional Ledger8 prototype](functional-ledger8-prototype.md), at the reviewed source revision. Historical API sketches are not delivered interfaces.
- Completion problem: individual checked-operation admission does not establish full-contract behavior or proof acceptance. [#666](https://github.com/betsudo/compact/issues/666) tracks the bounded functional delivery; the release still needs the explicit application and packaging criteria above.
- Proof/input isolation decision: preserve immutable source and tool premises, separate current functional proof producers from ordinary comparison paths, and keep actual proofs, verification, negative controls and uncached generation executing. A cache hit is not a proof result.
- Qualification versus reproducibility: an accepted finite receipt establishes its recorded scenarios at its recorded revisions. The clean historical 974-case public reproduction is tracked by [#653](https://github.com/betsudo/compact/issues/653). Forge/lifecycle reproducibility hardening is a separate obligation tracked by [#655](https://github.com/betsudo/compact/issues/655); later source changes need a valid qualification bridge or a new run.
- Qualification records name their compiler/runtime/source identities and limits. This guide summarizes those decisions without requiring access to private planning notes; it does not claim the associated historical ADRs or evidence archives are published here.
- This page is a dated explanation. It does not close issues, merge PRs or create new release requirements by itself.

Related audit record: [Keccak and the pinned Ledger8 proof profile](audit/2026-10-18-keccak-profile-boundary.md). Recorded implementation and qualification revisions remain authoritative.

## Original-DID preparation history — before PR #693 integration

The canonical backend is clean at verified signed merge `acf529eff` of PR #692.
Its complete Linux/Darwin/compiler/native/proof/package/archive/wallet/E2E gates
and exact-head Claude review pass. The next bounded slice qualifies the original
DID deactivation lifecycle on the current Ledger8.1.3 profile and fixes six
strict-Clippy Unit locals at the existing checked PureCall AST owner. It adds no
source-shape admission rule, new IR/ABI or public builder framework.

The development source/API/proof gates pass: complete unchanged two-file original
source, all twelve generated programs, live TS, five selected Rust test methods,
ten invalid inputs across four routes, typed refusals and one fresh functional
proof/default-strict application. Constructor execution and all twelve exports
are not proved. The [DID plan](plans/did-functional-lifecycle-current-profile.md)
and [developer guide](guides/did-functional-lifecycle.md) separate these receipts
from final clean compiler138, full integration CI and review acceptance. The
broader Rust AST goal remains incomplete; Forge remains deferred.

### Retained DID clean-source qualification

Original compiler 0.31.138 realizes at clean signed review correction `bf221d060`
as the same immutable package built at `3ac403d8a` (compiler inputs unchanged).
Fresh source/API and separate real proof gates pass: twelve exact live TS DID
rows, five selected Rust methods and 40 invalid-route executions, three typed
compile-fail controls, a fresh k11 / 1,804-row / 3,296-byte standalone deactivation
proof with immutable binding/version controls, and an independently proved
7,413-byte transaction with funded default-strict application/replay refusal.
The actual applied-state stale check uses the functional fixture. Both gates
require a clean checkout and pin the same complete source-fixture oracle used
by live TS and proof preparation; the historical ordinary oracle stays separate.
The clean backend library passed 1,846 tests with seven existing explicit-run
helpers ignored at `3ac403d8a`; that unchanged owner-code result is not relabelled
as a new run. Python passes 333 with 20 existing skips. Changed proof Rust
formatting and strict Clippy pass; prior complete workspace formatting is retained. Earlier 281-fixture/version/header receipts
retain their original source identities; full updated CI is still required.
See the [qualification table](plans/did-functional-lifecycle-current-profile.md#clean-local-qualification--compiler-031138)
for stage identities and limits. Repeat exact-head review and full integration CI
remain open. This is selected local application acceptance, not broad-goal completion.

## Historical Unit initializer preparation before PR #694 integration

PR #693 is verified merged at canonical `373fe452c` after all required CI and
exact-head Claude review. Its complete original DID source, four execution
routes, compile-fail controls, selected standalone proof and separate strict
transaction are retained; the constructor and all exports are not claimed
proved. See the [integration closeout](plans/did-functional-lifecycle-current-profile.md#verified-integration--pr-693).

At this earlier checkpoint, the next bounded slice was
[ordered Unit mutation initializers](plans/516-unit-mutation-initializers.md).
Its clean compiler139, qualification, PR review and full CI were still open.
The verified PR #694 closeout below supersedes that preparation status. Recording-retirement authority and all historical evidence
remain unchanged. This is a semantic/API improvement, not a new CI optimization
or Forge adoption project.

## Unit initializer integration and renderer prerequisite — 2026-10-28

PR [#694](https://github.com/betsudo/compact/pull/694) is verified merged at
signed canonical `1d0288846` after complete required CI and exact-head Claude
review. Selected initializer source E2E, proofs, immutable verification and
separate strict ledger application/replay retain their recorded scope; they do
not prove constructors or all exports. The completed worktree was recoverably
archived after preserving generated artifacts.

The statement-action slice was separately preserved on
`codex/rust-ast-unit-action-bindings`. Its valid mixed value/Unit scope exposed
a shared generated-code formatting problem. The bounded
[functional bind receiver fix](plans/functional-bind-receivers.md) was integrated
first as PR #695 on the accepted canonical baseline. It retains typed IR,
checked-program and runtime semantics. The statement slice has resumed with its
own qualification and draft PR #696. Forge remains deferred and the primary
goal remains incomplete.


## Historical statement preparation before PR #694 integration

As of 28 October 2026, [#694](https://github.com/betsudo/compact/pull/694)
(order-preserving Unit mutation initializers and comparisons) has clean local
qualification and accepted exact-head independent review; its complete Linux
integration gate remains pending. The next bounded [statement-action slice](plans/516-unit-action-bindings.md)
addresses valid Unit initializers in statement blocks and before trailing
mutations. Its clean local implementation and original compiler qualification pass
95 live TS cases, 344 Rust executions, 126 replays and four selected proof/strict
application calls. All 284 registered sources pass their configured emission
checks. These results remain separate from final canonical/version
reconciliation, independent review and full CI acceptance. Backend development
continues independently of the deferred Forge experiment.


## Clean local statement-action qualification — compiler141

The dependent local stack is qualified at signed `83d967324` with original
compiler141: 95 live TS cases, 344 Rust route checks, 126 replays, six independent
compile-fail controls and two admission-crate compilations pass. Four selected
fresh Unit proofs and separate strict transactions verify/apply with negatives
and replay checks. Selected original-DID API/proof regression also passes.
All 284 configured source-emission comparisons pass; 617 proof-required rows
are applicability accounting, not 617 executed proofs. Public Python has no
failures/errors and retains its 20 existing manual skip events.

See the [literal qualification table](plans/516-unit-action-bindings.md#clean-original-compiler141-local-qualification)
for source/tool epochs and limits. The child PR's own review/full CI remain
required. This does not complete the primary goal or the deferred Forge experiment.

## Receiver prerequisite integrated; statement canonical history reconciled

PR #695 is verified merged at signed canonical `709a86230`. Reviewed source
`7134fb240`, actual CI checkout `9534bac96` and the canonical merge share the
same complete tracked tree. All eleven checks pass, including native Linux,
original Apple Silicon consumers, selected proof gates, package/archive consumers
and E2E. Independent retained-artifact verification checks the authenticated ZIP
digest, complete Unit/DID IR and Programs, reviewed source hashes and archived
proof protocols; it is not a new host proof execution or an all-export proof.

The completed prerequisite worktree was recoverably archived after preserving
all 51 ignored entries and confirming no open file dependencies. Shared targets,
stores, snapshots and failure evidence remain preserved.

Normal signed child merge `2da45e015` includes actual accepted canonical history
and preserves the entire tracked tree of `1a3ace5fa`. Fresh affected
statement-action API/proof qualification now passes with original compiler141
at literal `cf7a2fa2c`, after review-control fixes. The earlier all284 emission
and selected DID receipts retain `83d967324` and their unchanged compiler/runtime/
fixture inputs. No documentation successor renews a proof receipt. PR #696's
final exact-head review and full required CI remain mandatory.
