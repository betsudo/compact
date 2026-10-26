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
