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
one complete witnessed Cell source. Published-head review and full Linux CI
are still required; this does not complete the distribution or handoff criterion.

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


## Local source-semantics qualification — 2026-11-02

Canonical `codex/rust-backend-ast` remains at
`1bdf185ea35929292a79d35946d4a98fb344d11c`. The local candidate stack is not
integrated canonical.

The [aggregate literal projection slice](plans/aggregate-literal-projection-widening.md)
has reviewed bounded compiler 166/source/API and two selected genuine
proof/default-strict application receipts at execution `e8d9293ba`. Existing
native coercion, typed validation and AST ownership are reused. No IR/runtime
ABI change is made. Earlier branch and evidence authorities remain distinct.

The [struct-member literal slice](plans/struct-member-literal-widening.md) adds
reviewed bounded compiler 167/source/API and two genuine helper proof/application
receipts at execution `886a69bc5`. Original narrow struct fields and accepted
output controls are preserved.

The [direct stateful Vector literal slice](plans/direct-stateful-vector-literal-returns.md)
now has reviewed bounded compiler170 qualification at execution `7ab60b620`:
31complete sources,50cases/200Rust routes,155source/API commands,31strict unedited
SDK roots and independent refusal/mutation controls. The ordered return owner
preserves original bounds and action order; the native aggregate owner evaluates
effectful Vector children once while preserving successful pure Vector ASTs.
No validator, recording certification, runtime or ABI change is introduced.
Three genuine proofs cover both direct Uint branch states and the Field export;
three separately proved funded default-strict applications and immutable negative
controls pass. All1833native tests pass/7existingignored;283retained compiler
fixtures are freshly identical. Two unselected wide-input models are k9 and remain
unproved;31required exports beyond the selected two and all unenumerated inputs
remain outside proof acceptance. Constructor execution is unproved.

The [stateful Vector value-widening slice](plans/stateful-vector-value-widening.md)
now has reviewed bounded compiler 172 qualification at execution `eaefb3752`:
23 complete sources, 37 cases / 144 stateful Rust routes plus one pure route,
136 API commands and 23 strict unedited SDK roots. It preserves original source
values before the existing checked Vector conversion, including helpers,
parameters, aliases, queries and once-only witnesses. Eight misuse/twin pairs,
seven frontend refusals, seven malformed-IR refusals and three real producer
controls pass their intended outcomes. Three fresh proofs and separate funded
default-strict applications cover both helper branch states and the Field export.
All 283 retained compiler fixtures and the predecessor S API gate (155 commands,
50 cases / 200 Rust routes) freshly pass. The other 21 required T exports remain
unproved, including the intact k=9 wide input; constructor execution is unproved.

The [stateful local Vector slice](plans/stateful-local-vector-widening.md) has
reviewed bounded compiler 173 qualification at execution `408dd00ee`:24 complete
sources,34 cases/132 stateful Rust routes plus one pure route,144 API commands and
24 strict unedited SDK roots. Local initializers retain their supplied source
before conversion and execute once with surrounding actions. Witnesses observe
both before/after-write positions; checked/discarded initializers preserve first
errors. Eight misuse/twin pairs, seven frontend and eight native IR refusals,
four complete producer controls and the result oracle cover the boundary.
Three fresh selected proofs and separate funded default-strict applications pass
with immutable negatives/replay. All860worktree proof-runner dependencies are
prebuild-pinned. Current T API qualification passes unchanged IR/Programs; the
283 fixture and 35 baseline results retain their explicit source/tool bridges.
The other 22 required U exports remain unproved, including intact `wide_local` k=9.

The proposal-time [stateful product-Vector baseline](plans/stateful-product-vector-widening.md)
retains 13 complete sources / 39 current 173 commands. Eleven pass TS: seven valid
stateful product/empty/nested/helper/Field/Boolean-component/full-proof-source
forms fail before IR at nested Cell.write; a local product conversion emits IR
then fails native type checking. Three controls pass all lanes and two frontend
refusals remain. This is source-only evidence; no new implementation, model or
proof acceptance is inferred. Existing pure coercion and prior return/local
binding work stay accepted within their recorded scopes.

The product-Vector successor has a bounded compiler177 source/API stage at
`2da9c2dbc`:153 commands,25 strict complete unedited SDK roots,32 TS/Rust cases
with124 stateful executions plus one pure, ten marked misuse/twin pairs, seven
exact frontend diagnostics, nine malformed-IR controls and complete producer/
result mutations. It preserves numeric/product bounds and ordered actions with
both combined nested witness positions. Independent review found a witness bridge
metadata regression; its fix and all failures are retained. Current U API
regression (144commands/34cases/24roots) and all283current fixtures pass.
The sole refreshed snapshot adds seven scoped signature style attributes;
historical complete bytes and compatibility tests remain preserved. Source
reviews cover their exact epochs. At that source/API checkpoint, no V model inventory, key generation, genuine
proof or application gate had run; emitted ZKIR did not establish model sizing
or proof acceptance. That qualification and independent receipt review remained
open until the later438bf result below. Existing predecessor proofs retain their own source epochs.

The V successor at438bf now has reviewed bounded local compiler177 qualification:
the current153-command/25-root/32-case protocol passes, all26export models are
observed, both selected exports are k8 (run212rows, run_field161), and locked
material precedes two fresh keys. Three2,912-byte standalone proofs and three
separately proved funded default-strict6,604/6,605/6,561-byte applications pass
with immutable verifier/binding/result/corruption and unchanged replay controls.
Independent source and separate receipt/hash/premise reviews report no P1/P2
within their scopes. All860physical worktree inputs are prebuild-pinned in the
actual4,439-source map; the raw depfile is retained. The other23required exports,
including three intact k9 models, remain unproved; constructor/all-input and CI/
integration coverage stays open. U/fixture/old full-test reuse keeps exact epochs;
no fresh compiler rebuild or cryptographic reexecution by reviewers is claimed.

The initial [Vector-containing product proposal](plans/stateful-vector-product-elements.md)
retains 15 complete probes/45 original177 commands and its strict identity-SDK
signature baseline failure. At that admission no implementation or proof
feasibility was inferred. The current bounded result follows below; the original
probes and failures retain their historical scope.

Full required arm64/Linux CI, reviewed integration and the broader goal remain
open. Forge remains deliberately deferred. No candidate cleanup is due before
verified GitHub merge.

## Current Vector-component qualification — 2026-11-03

The local candidate at `e293f2fe1500783b84d456f679767dfa5315b621` closes the
observed signature baseline and bounded component-Vector coercion gap through
existing checked lowering and structured AST owners. It passes 141 commands:
19 complete SDK roots, 27 TS/Rust cases, 84 successful stateful executions,
20 intended error attempts and one pure case, plus 42 exact public transcript
comparisons. Actual source and separate receipt reviews report no P1/P2. A
separate current compiler180 check passes all 283 retained fixtures without
updating them. See the [scope and next acceptance gates](plans/stateful-vector-product-elements.md#reviewed-ordered-error-result).

At that source/API checkpoint, model inventory, keys, genuine proofs and funded
default-strict applications remained open. The selected successor result below
now qualifies two complete existing source/export pairs. Earlier compiler177
proofs are not credited to the component-Vector candidate. Keep intact sources
and the unchanged k<=8 policy.

Broader completion still requires reviewed logical integration slices and full
required host CI, remaining real-application scenarios, independently consumable
generated packages and the finite capability/evidence handoff. Preserve the
accepted crosswalk, checked-recording owners and retirement ledger; do not reopen
accepted capabilities or infer universal language parity. Current work remains
local. Forge is deferred and does not block backend development. No cleanup is
due before verified merge.

## Component-Vector model boundary — 2026-11-03

The local source `6ca41ba55` passes the current141-command API protocol and
actual default/opt-in build/strict checks. After fixing two fail-closed
preparation defects, all919physical runner inputs match prebuild maps. Actual
Claude source and separate receipt reviews find no P1/P2 within the finite
model/input observation scope. All20exports are observed before selection:
19required/onepure;15required fitk<=8 andfourintactk9 models remainunproved.
The conditional Uint export is k9 and correctly refuses the automatic limit.
Preparation remains failed at that guard; material, keys, genuine proofs and
funded transactions were not reached. This is not proof or full integration
acceptance, and earlier compiler177 proofs are not credited here.

Next qualify the existing complete Uint helper and original complete Field
export (bothk8), preserving exact sources/SDKs and source-scoped identities.
Keep the conditional branches and other over-cap/unselected scenarios open.
See the [model/input boundary and next cohort](plans/stateful-vector-product-elements.md#reviewed-modelinput-observation-and-retained-cap-refusal).
FullCI/Linux/integration, remaining application/distribution/capability handoff
and the broader goal remain open. Current work stays local; no Forge or cleanup.

## Selected component-Vector proof qualification — 2026-11-03

Clean signed execution `81b7bbb43` passes the current 141-command API protocol
and complete paired proof gate. The existing whole Uint helper and original
Field export each produce a fresh 2,912-byte standalone proof and a separately
proved funded default-strict transaction (6,555 / 6,561 bytes). Matching immutable
verifiers accept; wrong-source verifier, corruption, changed binding, genuinely
typed changed output/IR-preimage and unchanged-state replay controls reject.
Actual Claude source review and separate receipt audit find no P1/P2.

All20exports are observed before selection. Seventeen required exports remain
unproved, including the four intact k9 models and both original conditional Uint
branch outcomes. Constructor proofs and full-source deployment are not claimed.
The helper retains its own crate root and generated bytes; 926 physical build
inputs and locked proof material precede keys. This is bounded local proof/
application acceptance, with [explicit audit limits](plans/stateful-vector-product-elements.md#reviewed-selected-uintfield-proofs--2026-11-03),
not full integration or a universal language claim.

The remaining goal work is reviewed integration/full required host CI, concrete
unaccepted semantic/API gaps, required real-application scenarios, isolated
generated-package distribution and a finite capability/evidence handoff. Accepted
foundation, checked-recording and historical crosswalk work is preserved. Forge
remains deferred and independent of backend progress. No cleanup is due before
verified merge.

## Products below Vector components — 2026-11-03

Local execution `cb38c00ff2` extends the existing ordered lowering selection to
one scalar product below a component Vector, retaining arity, original bounds,
single evaluation, action/witness order and checked Coerce/AST owners. Existing
runtime, validator, IR and ABI authorities are unchanged. The private request
field style metadata preserves exact carriers and visibility. Empty/single/mixed
product arities, aliases, upper second-child bounds and large Field values have
whole-source controls; deeper products and Vectors inside that product remain
explicitly refused.

The [bounded qualification](plans/vector-component-products.md#reviewed-bounded-sourceapi-result--2026-11-03)
passes 170 commands, 24 full unedited strict SDK roots, 35 TS/Rust cases,
116 successful stateful routes, 20 expected errors, one pure route and 58 exact
public transcript comparisons. Marked misuse/twin, native ingress/twin,
whole-producer, corrupt-oracle and exact source-refusal controls pass. Separate
current183 fixture comparison passes all283 without updates. Actual Claude quota
causes separate Codex CLI source and receipt fallbacks; both find no P1/P2 within
the recorded finite scope and disclosed provenance limits.

All 24 proof-required exports remain unproved in this family. A separate author
model observation finds six within k<=8 and eighteen over the unchanged limit;
it grants no proof/input/material admission. The next unchanged-source proposal
qualifies both complete witnessed pair sources and the single-product source.
Original helper/Field/pair sources remain intact and unproved. No previous
compiler180 proof is relabeled as this result. Full required CI/Linux, reviewed
integration, remaining application scenarios, isolated distribution and finite
capability/evidence handoff remain open. The broader goal is incomplete and
independent of deferred Forge work.


## Bounded products below Vector components — local proof checkpoint

The [nested-product slice](plans/vector-component-products.md#reviewed-selected-proofapplication-result--2026-11-03)
retains reviewed compiler183 source/API/static/ingress/producer acceptance. At
signed source `6b45a4051`, three existing complete sources (witness-before,
witness-after and single-element inner product) each produce a fresh verified
2,912-byte proof and a separately proved funded default-strict transaction, with
changed-result/binding/preimage, corruption, wrong-verifier and unchanged replay
refusals. Actual independent source, preparation and crypto-receipt reviews report
no current P1/P2. All source identities, historical failures and receipt limits are
retained in the slice plan.

This proves three selected exports, not the whole corpus: 21 other required
exports remain unproved, including 18 over-limit models; the pure export is N/A.
Callback implementation, constructor execution and full-export deployment are not
proved. Full CI/Linux/integration, final application/standalone-SDK acceptance and
capability handoff remain open. Accepted historical crosswalk and algebra work is
not reopened, and the broad goal stays incomplete. Forge stays deferred.


### Nested-product required CI wiring — local readiness

At signed source `5ef204e8e`, the existing-only original-recipe recorder and full
nested-product gate pass in the arm64 Nix environment: 170 API commands, three
fresh verified proofs and three separately proved funded default-strict
applications. Current source and separate actual receipt reviews report no
P1/P2. All prior failed attempts and literal source epochs remain retained.

The required Linux compiler workflow now calls this complete gate using the
existing authenticated compiler handoff and devShell-resolved ZKIR. It retains
k6/k8 locked material, all old required checks, failure/success artifacts and
required header-policy controls. This is checked local wiring, **not a hosted
Linux/CI result**. See the [slice plan](plans/vector-component-products.md#reviewed-local-ci-readiness-result--2026-11-03)
for exact source/header/proof identity, remaining 21 unproved exports and limits.
Logical integration and final profile/application/SDK/capability handoff remain
open; the broad goal and Forge deferral are unchanged.


### Current-candidate isolated SDK — repaired local handoff

The [current SDK slice](plans/current-profile-sdk-handoff.md) passes at signed
compiler 0.31.183 execution `cd3f432c6`: all 29 commands reach intended outcomes, with
three documented ContractLab methods, correct-type twins and E0053/E0382 misuses,
actual proof-feature Clippy for all three selected libraries, fresh k6 / 49-row keys,
a verified 2,912-byte proof and a separately proved funded DEFAULT-strict
6,607-byte transaction with expected state and unchanged replay refusal.
Fresh Claude source/receipt review reports no current P1/P2 in this finite scope.

The inherited external-package lint gap is repaired without changing qualified
SDK/consumer/testkit manifests or public APIs. A private lint root preserves every
original Cargo package/feature node, and 14 focused controls include actual warm-
check lint refusal and its corrected positive. The initial false-lint-success and
failed feature-flag epochs remain retained. Older physical-input/transitive/log
limits and verifier-loader versus proof controls are explicit in the plan.

Accepted compiler 0.31.143 SDK integration is not relabeled or reopened. This Cell case
proves no constructor/deployment, wallet lifecycle or unselected nested-product
export. Full required CI/Linux, logical integration, current application
reconciliation and finite supported-profile capability handoff remain open;
the broad goal stays incomplete and Forge remains deferred.


### Current-candidate Passport handoff — local qualification

The [current Passport handoff](plans/current-passport-handoff.md) passes at signed
compiler183 execution `9286fe749`. It retains the intact 17-file source and all
75 pure exports, with 202 live TS samples plus codec comparison, 16 adoption Rust
methods, two actual guide methods, strict SDK/consumer Clippy, intended E0308
misuses and changed-oracle assertion failure/restoration. Three existing
provenance methods pass separately; fresh Claude source/receipt review reports
no P1/P2 within the recorded finite local scope.

Only a separate formatting copy matches the retained compiler144 Rust fixture;
the executed SDK stays raw and historical identities/labels remain unchanged.
This is current-candidate application qualification, not a new semantic feature.
The pure library has no recording/proof/application stage. Current Linux/full
required CI, logical integration and the finite supported-profile handoff remain
open. The broader goal is incomplete; Forge remains deferred.


### Current-candidate DID lifecycle — selected local qualification

The [current DID handoff](plans/current-did-handoff.md) passes at signed compiler183
execution `67cbded3e`: whole original source and all twelve Programs, twelve live
DID TS rows, five selected four-route Rust methods, strict consumer Clippy and
three typed refusals. A separate fresh k11 / 1,804-row deactivation proof verifies
(3,296 bytes), and an independently proved funded DEFAULT-strict transaction
applies (7,413 bytes), with binding/version and unchanged replay controls.
Fresh Claude source/receipt review reports no P1/P2 in the finite selected scope.

At this `67cbded3e` epoch, the positive TS `did/timestamp-zero` row was not
executed by the selected Rust suite. The later
[zero timestamp witness slice](plans/did-zero-timestamp.md) closes that bounded
coverage gap at actual `4e49b59e5`: seven Rust methods retain all five old methods,
eight success-route executions and 40 error-route executions match the twelve
original DID TS rows. Two separately selected case-bound proof gates retain
k11 / 1,804 rows, each with a fresh verified 3,296-byte standalone proof and an
independently proved funded DEFAULT-strict transaction (7,413 bytes valid,
7,412 zero). Source and actual receipt Claude reviews find no actionable P1/P2.
The original sources, fixture, IR and twelve Programs remain exact. The zero
proof CI invocation is still pending; local success is not hosted acceptance. Constructor/all-export proofs,
full current protocol/network acceptance, complete CI/Linux and logical
integration remain open. Existing compiler138 deactivation acceptance stays
intact; current source/receipt epochs and capture/host limits are explicit in
the handoff. The broad Rust AST goal is incomplete and Forge stays deferred.


The next bounded delivery task is the [finite supported-profile handoff](plans/current-supported-profile-handoff.md), joining current application/SDK and product evidence with explicit stage, host and proof limits. It does not reopen accepted historical capabilities or complete the broader goal.
