# Primary Rust AST direction and next slice

Current bounded semantic follow-up:
[constant-valued Vector indices](constant-valued-vector-indices.md). Its local
implementation follows accepted literal projection and the standalone consumer
work; original compiler, new-source proofs and complete integration gates remain
required. This link does not reopen completed acceptance or resume Forge.

Decision: 2026-10-25, direct owner instruction. Implementation of the next
feature is not part of this separation change.

## Forge is a preserved experiment

Forge CI adoption is deliberately **deferred**, not completed or declared
technically impossible. `experiment/zkir-forge` is the Compact continuation
point if the owner explicitly revisits it. Separate Forge repository branches,
signed results, benchmarks, stores, snapshots, stopped containers and failure
evidence remain retained. Backend development does not depend on completing
the experiment. The Forge-specific scheduler was removed at the owner's
request after being paused. Its preserved historical prompt grants no authority
to restart deferred work.

Selected contract-level reuse benefits are valid; whole-CI value remains
unproven. The earlier “1% key generation” interpretation was incomplete:
46 seconds measured only Forge-managed generation. Direct bypass generation
was not separately timed. The total ZKIR/prover-key/verifier-key share remains
unknown. Parking the experiment satisfies none of its former outstanding
compiler, real-proof, native Linux or performance acceptance criteria.

## Baseline and separation

The canonical development line is `codex/rust-backend-ast`. Its accepted
published history at `33e5a74de` precedes the delivered functional Ledger8 work
and latest integrated Compact head `278f7c3dd`. The stale local ref `6e37443b4`
has exactly the same Git tree as the accepted #637 squash `14b585c27`; its
original history is preserved on
`backup/rust-ast-before-forge-deferral-20261025`. It is not a replacement
baseline. The primary ref was reconciled locally with the later accepted
history after the separation passed focused local checks. The forward separation
is signed commit `cfb28592d3e213d3dda58a3dc02f6aa66b61e777`.

The experiment retains `278f7c3dd`, alternate refs and unpublished diagnostic
patches. The primary branch removes compiler cache options, the helper and its
dependencies, test adapters, Forge qualification workflows and snapshot
publication/restore machinery. Three-lap functional cache comparison drivers
are parked with the experiment; their Rust proof cases and original source
fixtures remain. They are not required CI entrypoints.

Retain legitimate functionality and independent improvements:

- Rust AST, checked lowering, functional Ledger8, runtime, generated interfaces
  and proof/negative/application case implementations.
- Complete output-manifest traversal, pathname quoting and JSON escaping fixes,
  including their independent compiler tests.
- Authenticated compiler build-once sharing, bounded Rust checks, Cargo reuse,
  early lint and refusal checks.
- The complete ordinary compiler/native proof/wallet/archive/E2E and strict
  Clippy gates. Direct native ZKIR generation replaces adapter routing.

Focused local baseline checks passed before feature development: 1,797 backend
unit and 154 renderer tests, 27 CLI tests, the maintained 327-case Python suite
(20 expected fixture skips), strict backend Clippy, proof-runner compilation,
formatting and workflow/handoff checks. The ordinary offline Nix compiler build
at `cfb28592d` also passed; its help and binary roster have no Forge helper or
cache options, and an actual removed-option invocation fails before creating an
output directory. A genuine Counter source generated Rust with the existing
ordinary, recorded and functional APIs; its fresh generated crate passed
`cargo check --offline`. Evidence is retained in
`/Users/ysh/.codex/local-evidence/rust-ast-forge-deferral-20261025/`.

The complete required CI and real proof gates remain required for integration;
these focused local checks are not fresh proof or hosted acceptance.

## Publication review repairs — 2026-10-27

The clean Ledger8 baseline is published through existing PR #664 before the
vector, Map-size and Fold source slices. Its new-head CI and independent review
remain required; previous component reviews retain their exact scope.

The compiler handoff streams Nix diagnostics without contaminating its
machine-readable stdout. A real failed-child regression establishes the
previously hidden stderr failure and its repair. Handoff identity and consumer
ownership controls run before producing platform material. Functional consumer
qualification uses the authenticated `COMPACTC` from the same-run import.
Consumer jobs retain producer-failure refusal while `!cancelled()` avoids
starting runners after cancellation. Runtime package uploads require successful
package rehearsal and retain strict missing-file checks at that stage.

Omitting a structural Map's scalar-only witness view now still validates its
key carrier. A genuine red regression accepted an invalid Map key before the
repair; both scalar/structural-key refusals and the valid nested Map witness
view control pass afterward. This changes no valid generated API or snapshot.

## Selected gap: #674, constant fixed-vector projection

Compact's frontend accepts a statically in-range constant projection from
`Vector<N,T>` and transports it as `tuple_index`. Current Rust pure semantics,
stateful native lowering and checked recording admit tuples only. Existing
negative tests explicitly characterize the vector boundary. This prevents a
valid existing source operation from obtaining useful generated Rust APIs.

The 0.3.0 acceptance crosswalk, checked-recording ownership guide, capability
book and retirement ledger remain authoritative for their accepted scopes.
Typed witnesses, supported collection/Merkle/Ledger8 effects, #612 source
diagnostics, #622 shared checking and #491 Map/Fold/readability work already
have owners and acceptance. Do not reimplement or reopen them. Historical
“next” recommendations are not evidence of a current missing capability.

## Intended public API

Keep the generated facade and existing runtime carriers:

- Inputs remain `FixedVector<T,N>`; outputs use the existing element type `T`.
- Existing ordinary native, recorded and observed-call methods gain support
  for this source expression where their existing capability rules apply.
- Functional consumers retain
  `ledger_contract::functional::<entry>::Program::new(...)`, consuming `run`,
  `execute_native` and `execute_recorded`. No replacement executor API.
- Wrong ordinary element/length arguments fail Rust compilation. Executing an
  already consumed program also remains a compile error. Untrusted private IR
  and recording plans retain explicit validation and diagnostics.

Retain the existing private IR ingress spelling with type-directed checking;
do not change the public runtime ABI or add a public indexing trait merely to
emit constant access. Add a distinct checked vector-projection operation/fact
so `TupleProject` continues to require a tuple. If the implementation discovers
a serialized compatibility obligation, resolve it explicitly before changing
that boundary; do not silently reinterpret a frozen plan.

## Affected owners

| Owner | Bounded change |
| --- | --- |
| `compiler/analysis-passes.ss`, `compiler/rust-ir-passes.ss`, backend `ir.rs` | Characterize genuine source transport and existing frontend bounds diagnostics. |
| Backend `pure_semantics/aggregate/projection.rs` and shared body-check/sink owners | Check operand once, vector length, constant index and exact result; produce syntax-free checked facts. |
| `pure_native/aggregate/projection.rs`, `stateful/expression.rs` | Render structured `syn` element access from checked facts; preserve effects, error ordering and existing ownership/retention policy. |
| `recorded/effect_plan/{model,lower/aggregates,validate/aggregates,render/pure}.rs` | Add explicit checked vector projection and independent scope/type/bounds validation. |
| Functional projection/rendering and `functional_items.rs` | Reuse the same checked operation and existing interpreter/public signatures. |
| Exhaustive resource, circuit-analysis and origin visitors | Preserve traversal/accounting and operand identity; no resource-budget increase. |
| `runtime-rs/src/primitives.rs` and ownership tests | Reuse `FixedVector`; test valid non-Copy values without broad Copy/Default requirements or arbitrary whole-vector cloning. |

Backend paths above are under `tools/compact-rust-backend/src`.

## Acceptance criteria

### Positive behavior

1. Genuine Compact sources select first/last elements of `Vector<2,Field>` and
   a singleton. Include a witness/local helper operand evaluated exactly once.
2. One supported owned element domain exercises repeated projection, later
   vector reuse and a temporary returned vector. Preserve valid ownership.
3. Independently expected values agree across ordinary native/recorded and
   functional native/recorded execution; compare pinned TS where applicable.
   Check state, effects, private output, witness ordering/failure, cost and replay.
4. A failing operand prevents later effects. Inactive branches do not execute;
   malformed static children still receive checking. Tuple behavior and error
   precedence remain unchanged.

### Compile-fail and refusal controls

1. Real-source index `N` and indexing `Vector<0,T>` fail at the source boundary;
   a generated Rust bounds panic is not the admission policy.
2. Independent plan mutations reject wrong kind, out-of-range index, wrong
   element/result type, forged length, unbound/use-before-definition operands
   and a same-type but different operand identity.
3. External, unedited generated consumers reject wrong vector length/type and
   double execution of a consumed `Program`; include a valid owned consumer.
4. Keep forged vector-as-`TupleProject` refusals. Replace only the old source
   sentinel whose boundary intentionally changes. Preserve resource/overflow
   controls for affected visitors under the existing limits.

### Genuine proof coverage

Use one complete proof-applicable stateful source that selects a private Field
and discloses/writes it through an existing Cell operation. Bind exact source,
full IR, ZKIR, key, runtime and parameter identities; actually record, prepare,
prove and independently verify it, with changed public binding/result refusal.
Require funded default-strict application and replay rejection if application
support is claimed. Use the existing official direct provider path. Report
owned-opaque execution-only cases according to actual proof applicability.

During iteration use focused semantic/renderer/validator tests and the small
generated consumer. At integration retain the complete required compiler,
Rust, negative, proof, wallet/application and packaging gates. Neither a
capability row nor key generation alone establishes proof coverage.

## Exclusions and implementation order

No dynamic indexing, slicing/update, direct stateful Fold #661, nested Map size
#610, wrapped effectful returns #516, new ledger/ZKIR/crypto platform, SDK/Wasm,
visibility framework, broad AST refactor, CI optimization or Forge work.

First close concrete separation/baseline regressions. Then write the narrow ADR
and a genuine-source regression for the existing failure, extend the shared
checked operation, implement ordinary and functional rendering, and run the
finite acceptance above. This plan does not implement #674 or claim full
language parity, fresh proof qualification or release readiness.

## Autonomous implementation checkpoint — 2026-10-25

After the owner activated the primary Rust AST goal, #674 implementation began.
See [the narrow decision and current evidence](674-constant-vector-projection-decision.md)
and the maintained `check_constant_vector_projection.py` gate. Source transport,
checked projection, structured native/functional emission, owned reuse and
focused generated compile-fail controls are implemented locally. Real proof,
verification and full integration acceptance remain open; this amendment does
not reopen or complete the deferred Forge experiment.


## Qualification checkpoint — 2026-10-26

The bounded #674 implementation now has executed pinned TS comparisons and
genuine direct native proofs for both private Field selections, including
independent verification, changed-binding/result refusals, default-strict funded
application and replay rejection. Full Rust workspace tests and strict Clippy
pass. See the narrow decision for exact scope and retained failures. Complete
integration and remaining review stay open; Forge remains deferred.

After completion and verified GitHub merge, clean up task-owned worktrees and
reproducible targets while preserving unpublished work and required evidence or
inputs. Do not remove shared stores/caches, containers or qualification artifacts.

## Local integration checkpoint — 2026-10-26

The #674 implementation now passes the remaining maintained local compiler,
native consumer/proof, wallet handoff, complete golden/source-refusal and pinned
oracle gates, alongside runtime archives and their generated consumers. The
[narrow decision](674-constant-vector-projection-decision.md#local-integration-checkpoint--2026-10-26)
retains exact source cohorts, the capture portability correction and proof
scope. Independent external review and intended Linux/hosted integration remain
open; no publication or merge occurred. The broad Rust AST goal remains active.
Do not reopen already accepted backend capabilities or the deferred Forge work.

## Next bounded semantic slice — 2026-10-26

The genuine six-source matrix confirms #610 remains missing specifically at
the stateful expression ingress: direct `Map.size()` returns already work for
scalar and whole nested-Map values, while named-result, comparison and
conditional expressions are accepted by TS and refused by the Rust frontend.
Focused baseline Map checks pass 117 tests with one inherited ignore unchanged.
Proceed with [Map size in stateful expressions](610-map-size-expressions.md),
reusing the existing checked metadata operation and public runtime carriers.
That plan fixes its affected owners, public API and finite behavior/refusal/proof
acceptance before implementation. It opens no Forge, broad refactor or CI project.
This future cleanup condition does not authorize publishing or merging now.

## Map size local integration and next slice — 2026-10-26

The [#610 plan](610-map-size-expressions.md#local-integration-checkpoint--2026-10-26)
now records passing maintained local compiler, API/TS, selected real proof,
full native/wallet, source/golden/refusal, workspace/Clippy and runtime archive
checks at signed `e14103c5d`. Independent review and intended Linux/hosted
integration stay open. The new malformed-Array host-state refusal is explicitly
stronger than the permissive TS VM; valid Map semantics and canonical query
cost remain qualified separately.

Proceed next with [direct fixed-vector Fold](661-direct-fold-expressions.md).
Its plan uses actual current source counterexamples and TS operand-order
observations; it preserves the already accepted certified pure-helper route.
The broad backend goal remains active. Forge stays deferred at
`experiment/zkir-forge`; no publication/merge or cleanup is authorized by this
local checkpoint.


## Current accepted direction — 2026-10-27

The owner subsequently authorized each logical slice to be pushed, pass full
required CI and independent review, and merge into `codex/rust-backend-ast`.
The older local-only publication statements above retain their historical
epoch and do not revoke that later authority. The Ledger8 baseline (#664),
vector projection (#688), Map size (#689), direct Fold (#690), and scalar
terminal return wrappers (#691) are verified merged through canonical
`c3a529cb5`. Their exact source, CI and review epochs are summarized in the
[delivery report](../ledger8-delivery-and-completion.md#current-integration--2026-10-27).

The active bounded slice is [typed scalar-pair return wrappers](516-tuple-return-wrappers.md).
Clean compiler 0.31.137 generation, live TS/four-route behavior, compile-fail
controls, all generated fixtures and four selected fresh proof/application
cases pass locally. Full current CI and published-head Claude review remain
required before merge. Then requalify the actual DID and pure passport sources
on the current profile to select the next concrete adoption gap. Do not reopen
accepted architecture work or start another CI optimization project. Forge
remains deliberately deferred and preserved; the broad Rust AST goal is open.


## Independent Unit initializer follow-up — 2026-10-27

Typed pair wrappers (#692) are now verified merged at canonical `acf529eff`.
Original DID functional lifecycle (#693, exact `18d79363`) has independent
Claude review acceptance; its full required CI is still pending at this local
checkpoint. Its clean compiler 0.31.138 and finite API/proof receipts retain their
own source epoch. Neither acceptance completes the broad backend goal.

The next independently based local slice is
[ordered Unit mutation initializers](516-unit-mutation-initializers.md), with
[the user-facing API/evaluation guide](../guides/unit-initializers.md). A genuine
TypeScript-accepted initializer refused by baseline Rust establishes the gap;
already accepted local-call/scalar/pair forms stay closed. The maintained local
API gate passes 58 live TS cases, 196 Rust executions and 62 recording replays,
including source/typed/oracle refusals and strict Clippy. Separate fresh selected
k7/76-row proofs and independently proved default-strict funded transactions
pass insertion/reinsertion, verifier negatives, exact state and unchanged replay
refusal. Constructor/prestate setup and other exports are not proved.

The full registered fixture comparison freshly checks 281 generated libraries
with zero stale outputs or failures and no updates requested. Original 282
registry rows, 418 additive declaration rows and historical ledger/retirement
authorities remain unchanged; this independent branch adds one source and 17
new declaration rows. These literal counts describe the pre-#693 local branch,
not the later combined canonical census. Final canonical reconciliation,
original compiler qualification, full required CI, exact-head Claude review and
signed reviewed merge remain required. Keep two active engineering branches at
most, preserve unique work before reconciliation and clean only unused own
checkouts/targets after verified merge. Forge remains deliberately deferred.

## DID integrated; Unit initializer reconciliation — 2026-10-28

PR #693 is verified merged at signed canonical `373fe452c`, with full required
Linux/Apple Silicon, native/proof/package/archive/wallet/E2E CI and exact-head
Claude review accepted. The independent Unit initializer implementation d7 is
preserved and merged with that actual canonical history at `e4d7f9be9`.
Both fixtures and all existing gates are retained. The corrected combined census
is 284 source registrations, 482 declarations and 196 flat libraries; historical
authority rows are unchanged. Compiler 0.31.139, fresh affected qualification,
its own PR, review and complete CI remain required. Forge stays deferred and
does not block this backend slice; the broad backend goal remains incomplete.
