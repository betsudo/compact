# Map size in stateful expressions (#610)

Decision: 2026-10-26. Local implementation slice after #674; no hosted or release
acceptance is implied. Forge remains deferred.

## Observed gap and accepted controls

At clean `c1f571a64`, the actual ordinary compiler accepts direct-return
`Map.size()` for both scalar values and whole nested-Map values. The same
compiler accepts genuine TS sources using size inside a named result, equality
or conditional, but the Rust frontend refuses each with the source diagnostic
`Rust backend does not yet support this nested ledger query: Map.size`.
The six-source matrix is retained in
`/Users/ysh/.codex/local-evidence/rust-ast-map-size-expression-20261026/source-characterization-v1.json`.
These are source executions with `--skip-zk`, not proof qualification.

The existing checked `Operation::MapSize` and its declaration/type validation,
native metadata query, recording, functional adapter and direct-return API are
accepted work. Reuse those owners. Do not replace their machinery or reopen
retired whole-body recognizers. #674's distinct projection and prior tuple
refusals stay unchanged.

## Decision and public API

Transport an explicit `Expr::MapSize { field, index }` through the existing
private IR. Its JSON spelling `map_size` already exists for direct returns;
this adds the expression position. The compiler and backend remain a paired
private-IR producer/consumer at schema20. Old frozen records are unchanged;
this does not promise that an older decoder accepts the new expression form.
No public runtime ABI, capability schema or ledger profile changes.

Use the existing `BoundedUint<18446744073709551615>` result carrier. Source
names and named result structs determine ordinary generated methods and fields.
Recorded/observed APIs and the opt-in functional consuming `Program` gain this
expression wherever their existing composition rules apply. There is no new
Map executor, container codec or public indexing/query trait.

Native execution must perform one canonical metadata query at its original
source position and add its real query cost. Checking must bind the declaration,
Map kind and physical index, without requiring scalar decoding of the Map's
value. Checked recording lowers the expression to the existing `MapSize`.
Both branches are checked statically; only the selected branch queries at run
time. Preserve query/witness/mutation order, lexical scope and first failures.

## Affected owners

| Owner | Change |
| --- | --- |
| `compiler/rust-ir-passes.ss` | Emit the existing `map_size` spelling from the stateful expression query arm, preserving origins and the current ledger-path restriction. |
| Backend `ir.rs`, shared expression classification, circuit/resource/origin visitors | Explicit expression variant, ledger-query classification and exhaustive original-occurrence/slot accounting. |
| `stateful/expression.rs` | Structured `syn` query syntax using the existing slot/context metadata authority and Uint64 conversion. |
| `recorded/effect_plan/lower/{expressions,maps}.rs` | Route the new ingress through the accepted Map metadata checker and existing typed operation. |
| Existing Map validation/rendering and functional owners | Preserve declaration/result checks and execution semantics; extend only if a genuine missing exhaustive case requires it. |
| Genuine Compact fixture, unedited generated consumer and native proof smoke | Positive behavior, wrong ordinary arguments, malformed ingress/plan controls and applicable genuine proof/verification. |

Backend paths are under `tools/compact-rust-backend/src`. No resource budget,
caller stack or source-origin policy is relaxed.

## Finite acceptance

1. Keep direct scalar and whole nested-Map size working. Generate genuine Rust
   for named results, comparisons, arithmetic/casts within existing supported
   integer semantics, lexical locals and conditional size expressions.
2. Execute empty and populated scalar Maps, replacement versus new insertion,
   repeated reads around a mutation, and whole nested-Map metadata. Metadata
   counts outer entries without pretending to decode a nested value as a Cell.
3. Compare independent expected values and actual TS outputs across ordinary
   native/recorded and functional native/recorded execution, canonical results,
   public query replay, state/effects/private outputs and actual summed cost.
   Cover inactive queries, witness ordering and the original first failure.
4. Refuse unknown/non-Map declarations, forged physical indices and incorrect
   result types. Check malformed static arms even when inactive. Keep pure
   ledger-query refusal and existing declaration/resource/overflow limits.
5. Compile external consumers of unedited generated crates. Wrong ordinary
   argument types fail at Rust compilation; executing a consuming `Program`
   twice fails compilation. Invalid untrusted state fails explicitly at runtime.
6. Bind exact genuine source, full IR/programs, ZKIR, verifier/key, official
   runtime and verified parameter identities. Generate fresh selected keys,
   record/prepare/prove/independently verify a stateful scalar-Map count after
   mutation, and reject changed public binding/result. If claiming application,
   require fresh funded default-strict application and unchanged replay refusal.
   Constructor execution and nested-ADT proofs are separate claims.
7. Iterate with focused owner/generated-consumer checks; retain the complete
   existing compiler, Rust, refusal, proof, wallet and package gates for
   integration. Preserve preceding receipts and separate execution cohorts.

## Exclusions and order

No direct stateful Fold #661, wrapped-return normalization #516, dynamic child
Map paths, nested-value insertion/decoding, new ledger/ZKIR/SDK platform, broad
refactor, CI optimization or Forge work. Existing path/codec refusals are not
silently relaxed to make this case pass.

First preserve the baseline/source matrix. Then add expression ingress and
exhaustive ownership, native and checked lowering, focused refusal laws,
genuine generated behavior and applicable proof coverage. Record exact local
results and remaining independent review/Linux integration without claiming
the broader backend goal complete. Keep commits local, signed Conventional,
DCO and Assisted-by. Cleanup remains conditional on completion and verified
GitHub merge, preserving unpublished work, needed inputs and evidence.

## Discovered source dependency — 2026-10-26

After expression ingress was implemented, the complete genuine fixture exposed
`Map.insert` being sent through query-only expression lowering. A lexical count
read followed by insertion and a composite return was outside the old
action/return split's selected scope. Preserve this required read/mutate/return
behavior through the existing typed return-plan `Let` and `Sequence` owners.
Select that route from the current metadata-based storage-mutation classifier
when the legacy lexical split was not selected; no ADT allowlist or source-name
recognizer is added. Keep original successful split forms and terminal
conditional routing intact. The failed source receipt remains retained.

This adds the one lexical dependency needed for the planned acceptance; general
value-wrapper normalization and the rest of #516 remain outside this slice.
Genuine generated execution, first-failure/scope controls, source-origin
checks and applicable proofs must still validate the dependent source.

## Discovered untrusted state dependency — 2026-10-26

An actual generated-consumer counterexample placed an Array in the declared Map
slot. Both the existing Rust metadata path and the TS VM returned its length,
because the VM Size opcode also accepts arrays and trees. This is invalid
declared state, not valid-source semantic parity. Keep the failed Rust test and
actual TS probe in the #610 evidence directory. The first Rust probe selected
zero tests due to an exact-name filter; only the second probe is a counterexample.

Require a Map at the original size-query position in the shared `ledger::size_map`
owner. Native context and recording must both call that owner; functional routes
reuse them. Use the existing `LedgerQueryRejected` boundary category, preserve
valid query program/cost and allow whole nested-Map metadata without scalar value
decoding. Reject before a later witness; an inactive query still performs no
validation or VM operation. This deliberately strengthens the host boundary for
malformed state relative to the permissive TS VM. It does not alter the runtime
ABI or extend this slice to other collection queries.

## Local integration checkpoint — 2026-10-26

The signed implementation is `06d5af5a`; the complete generated consumer,
Map-kind boundary guard and separate proof gates are signed at
`e14103c5d3ba6d6af2cff1c9e3526cfe1263b743`. At that unchanged candidate:

- The ordinary Nix compiler package builds. The original `compiler/go` and
  `srcMaps/test.sh` pass in a separate test-enabled derivation (132 source-map
  assertions). Rust CLI bytes match; Scheme tool bytes differ, so those binary
  cohorts remain distinct.
- Eight external consumer tests pass across ordinary and functional native and
  recorded routes. The live TS comparison covers 19 successes, four refusals
  and two inactive malformed-state cases; the changed-TS-result control fails
  as required. Wrong argument types and double Program consumption fail with
  E0308/E0382. Source purity and return-type refusals remain intact.
- Fresh direct `put` keys use `k=7`, 73 rows. Replacement and new insertion each
  produce a fresh 2,912-byte proof, independently verified. Changed public
  binding and changed same-geometry result are rejected. Both funded local
  transactions pass default-strict application; replay returns
  IntentAlreadyExists without mutation. Constructor execution and nested-ADT
  proofs are not claimed.
- The complete maintained native generated-consumer/proof gate and its wallet
  JS handoff pass. Wider cases retain their individual application policies;
  the two new Map cases do not relabel that whole suite default-strict.
- All 257 fresh generated fixtures, 37 pinned oracle sources and the maintained
  source-refusal suite pass. Workspace tests pass 3,340 cases with seven
  inherited ignores; strict workspace Clippy (all targets/features), formatting
  and workflow lint pass. The header gate reports zero new violations and
  1,308 inherited unchanged files; it is not whole-tree license compliance.
- Runtime archive rehearsal, manifest re-verification and fresh generated
  consumers of those archives pass in an isolated target. Earlier archives,
  manifests, failures and source/proof epochs remain preserved.

Exact commands, source/material/artifact identities and retained failures are
under `/Users/ysh/.codex/local-evidence/rust-ast-map-size-expression-20261026/`.
See `selected-v1/proof/receipt.json`, `native-v1/complete.json`,
`source-gates-v1/complete.json`, `packages-v1/complete.json` and
`workspace-v1-accounting.json`. The original failed Array probe is distinct
from the successful boundary fix; its first zero-test probe is not evidence.

These are maintained **local** execution results. Independent external review,
intended Linux/hosted integration, publication and merge remain open. Forge is
still deliberately deferred. This closes the selected local implementation
and execution slice, not the broader Rust AST goal or former Forge acceptance.
Cleanup remains conditional on completion and verified GitHub merge, preserving
unpublished work and all needed inputs, binaries, archives and evidence.

## Publication review repairs — 2026-10-27

The ordinary Map-size expression now uses the existing physical-path context
query when a declaration has no typed slot carrier, matching the established
Map-emptiness fallback. A red renderer regression emitted a nonexistent slot;
its repaired output uses the checked declaration's physical path.

Map emptiness now reaches the same declared-Map shape guard as size in the
ledger, native context and recording owners. A real Array counterexample was
accepted before the repair; native and recorded emptiness refuse it afterward.
Valid empty/nonempty Maps retain their original VM emptiness program, result
and native cost. This does not replace emptiness with a size comparison or
claim new proof geometry. Functional adapters continue to reuse these owners.

The review's suspected dropped insertion was checked with genuine unit-return
and non-tail lexical Map-size sources using the retained original compiler.
Both generate the insertion; no broader lexical normalizer change is made.
Focused red/green logs are retained in the 2026-10-27 publication evidence.
Earlier source/proof/compiler receipts keep their original epochs; current
complete CI and proof gates remain required before integration.

## Current PR delivery authority — 2026-10-27

The owner explicitly superseded the earlier local-only publication restriction.
Each meaningful Rust AST slice receives focused local validation, a documented
PR, complete required hosted CI/proof gates, independent CLI review and then
merge into `codex/rust-backend-ast`. No repeated push/merge approval is needed.
Local development uses arm64; Linux qualification remains in the full PR
Compiler Build lane. PR triggers provide that coverage without repeating the
same expensive full job for the working-branch merge push. Manual and scheduled
qualification remain available. No required test/proof gate is removed.
Forge is still deliberately deferred; no release or shared installation is
authorized. Cleanup remains after completion and verified GitHub merge.
