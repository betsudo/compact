# Primary Rust AST direction and next slice

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
This future cleanup condition does not authorize publishing or merging now.
