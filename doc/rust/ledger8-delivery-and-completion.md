# Ledger8 Rust: delivered work and the finish line

Engineering snapshot · 18 October 2026, 14:59 UTC · Ledger **8.1.3**, ZKIR **v2**

## In plain words

We now have a Rust compiler path that composes supported operations instead of recognizing whole examples. On top of it, the opt-in functional surface generates an immutable request with one execution body. Native execution and recording run that body through different Ledger8 adapters.

The latest delivery adds the missing witness, value, collection, Merkle and Ledger8 effect families to that functional surface. It has real execution, proof, verification and ledger-application evidence for selected cases.

**The bounded implementation goal is complete locally. A general claim that “Ledger8 is complete” is still premature.** Publication/review is in progress; real-application requalification, distributable artifacts and known source-language gaps remain separate work.

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

### 1. Complete the typed-pair return extension

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
Those receipts explicitly describe an uncommitted development epoch. Compiler
0.31.137 source qualification, final fresh proofs, full CI, published-head
independent review and merge remain required. Other aggregate/initializer debt,
real applications and the broader Rust AST goal remain open. Backend work does
not depend on the deferred Forge experiment.

### 2. Requalify the real applications on the current profile

**DID:** retain the exact `midnight-did v0.7.0` source baseline and real witnesses. Run current Ledger8.1.3 ordinary and functional paths, state/effects/private outputs/gas/replay and selected applicable proof/application cases. Older original application acceptance used Ledger8.0.3; later tutorial receipts are narrower and do not establish fresh full functional acceptance.

**Digital passport:** retain the `midnight-vc-passport` integration baseline and its referenced credential source closure. The accepted passport source is a pure library: **75 pure exports and zero proof-applicable exports**. Re-run its actual pure behavior/TS oracle controls on the current toolchain; report recording/proof as not applicable where appropriate. Do not invent stateful exports to claim functional/proof adoption.

**Finish when:** both intact pinned source closures build and their applicable tests pass on the selected profile. Each refusal becomes a minimal regression and an issue; fix its actual cause before accepting the dependent application. Do not remove exports to make an oversized/unsupported contract appear supported. “Latest” upstream after these pins is a new adoption decision.

### 3. Close the known source/composition gaps where they affect the claim

| Issue | Current gap | Required evidence for a fix |
| --- | --- | --- |
| [#674](https://github.com/betsudo/compact/issues/674) | Integrated by #688 in the supported constant fixed-vector scope. | [Scope and controls](plans/674-constant-vector-projection-decision.md); no arbitrary dynamic indexing claim. |
| [#661](https://github.com/betsudo/compact/issues/661) | Integrated by #690 for the maintained direct Fold cohort. | [Source-family plan](plans/661-direct-fold-expressions.md); 21 full sources, selected proofs, explicit checked/refusal boundaries. |
| [#610](https://github.com/betsudo/compact/issues/610) | Integrated by #689 for stateful Map size expressions. | [Scope and controls](plans/610-map-size-expressions.md); whole-Map metadata, malformed-state controls and two selected proof/application cases. |
| [#516](https://github.com/betsudo/compact/issues/516) | Scalar wrappers integrated by #691; typed scalar-pair/inline-Fold extension in progress. Other lexical/initializer gaps remain. | [Plan](plans/516-field-return-wrappers.md); conversion/effect order, TS/four-route parity, refusal controls and selected fresh proofs. |

Arm64 checks are the local development gate and Linux required integration runs in GitHub Actions. A local Linux runner is not a prerequisite for bounded backend work. Preserve accepted crosswalk and historical authorities; later source heads require their own integration evidence.

These are not retroactive failures of the bounded delivery. They **are blockers for contracts using them, and for an unqualified full-language claim**. For a deliberately supported-profile release, publish explicit exclusions instead of calling them solved.

Also retain the existing type/conversion/resource refusals. Counting a top-level conversion variant does not make every conversion legal. No new full-specification research campaign is required just to record these known gaps.

### 4. Make the generated artifacts independently consumable

**Finish when:** a fresh directory can extract/build the generated package and its declared matching runtime inputs, implement witnesses, run ContractLab and prepare a supported proof without paths back into a development checkout. Record deterministic compiler/source/target/interface/artifact identities, the supported lock/profile and early incompatible-artifact refusals.

A successful `cargo package --list` or a manifest that names an unpublished registry version is insufficient. Decide whether release distribution is a paired source SDK or published packages. Keep deployment identity, wallet/network clients and provider adapters separate from generated contract semantics.

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

The next useful engineering work is **real-application functional qualification, led by DID**, while the PRs are reviewed. It will select which remaining source gaps actually block adoption. Package/extraction acceptance can proceed independently. This is a recommendation, not a newly activated goal or an expansion into Ledger9.

## Source and decision map

- Current API and qualification scope: [functional Ledger8 prototype](functional-ledger8-prototype.md), at the reviewed source revision. Historical API sketches are not delivered interfaces.
- Completion problem: individual checked-operation admission does not establish full-contract behavior or proof acceptance. [#666](https://github.com/betsudo/compact/issues/666) tracks the bounded functional delivery; the release still needs the explicit application and packaging criteria above.
- Proof/input isolation decision: preserve immutable source and tool premises, separate current functional proof producers from ordinary comparison paths, and keep actual proofs, verification, negative controls and uncached generation executing. A cache hit is not a proof result.
- Qualification versus reproducibility: an accepted finite receipt establishes its recorded scenarios at its recorded revisions. The clean historical 974-case public reproduction is tracked by [#653](https://github.com/betsudo/compact/issues/653). Forge/lifecycle reproducibility hardening is a separate obligation tracked by [#655](https://github.com/betsudo/compact/issues/655); later source changes need a valid qualification bridge or a new run.
- Qualification records name their compiler/runtime/source identities and limits. This guide summarizes those decisions without requiring access to private planning notes; it does not claim the associated historical ADRs or evidence archives are published here.
- This page is a dated explanation. It does not close issues, merge PRs or create new release requirements by itself.

Related audit record: [Keccak and the pinned Ledger8 proof profile](audit/2026-10-18-keccak-profile-boundary.md). Recorded implementation and qualification revisions remain authoritative.
