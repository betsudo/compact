---
id: RUST-ADR-LEDGER9-FOR-PROPOSAL
title: "Certify finite for-loop index domains before Rust AST projection"
date: 2026-11-15
decision_status: proposed
delivery_status: not-integrated
topics: [ledger9, compiler, rust-ast, vector, recording]
issue: https://github.com/betsudo/compact/issues/750
---
# Certify finite for-loop index domains before Rust AST projection

This is an **unnumbered local ADR proposal**. The repository ADR register reserves 0367–0499 and the vault assigns later numbers; the register owner should assign an available number when integrating this decision. This document records a bounded implementation decision, not upstream language acceptance or a completed qualification.

## Context and observed failure

The pinned upstream source at `672bd0f226f47a7d036c5c074ce71fbe1af1925e`, `test-contracts/keywords/for/generic_bound/for_generic_bound.compact`, specializes `sum_range<1, 3>([1, 2, 3, 4])`. Its original runtime assertion expects `5`: `v[1] = 2` followed by `v[2] = 3`, with `total` initially set to zero. Its three original files remain outside the locally qualified nine-case `for` slice at `10214228`; the Rust target currently refuses `v[i]` with `no unique constant-valued Vector index for this occurrence`. That refusal is preferable to generating a wrong projection.

Compact defines `for` as a statement with a compile-time-known iteration count. For a range, the start is inclusive and end exclusive; generic natural-number parameters are valid bounds. Analysis resolves `S..E` into a literal tuple and lowers `for` into a unit-accumulator Fold (`compiler/analysis-passes.ss`). The circuit path unrolls and inlines that Fold before `resolve-indices/simplify` (`compiler/circuit-passes.ss`). The Rust path starts from the earlier analyzed Fold and emits one uniform `VectorFold` body (`compiler/passes.ss`, `compiler/rust-ir-passes.ss`). Its current source-occurrence bridge stores one constant index per original Vector-ref node. The circuit observes both 1 and 2 for that node, so the bridge correctly marks it ambiguous (`compiler/rust-vector-indices.ss`).

The two successful original `for` cases and seven original diagnostic refusals in the isolated slice do not establish this remaining stateful case or general loop parity.

## Decision

Keep `for` as a source statement lowered through the existing Fold machinery. **Do not add an unrestricted `Expr::For` or general dynamic Vector indexing to the Rust IR/runtime.** Admit a finite index domain only when the compiler proves all of the following for the exact analyzed Fold and Vector-ref occurrence:

1. The Fold source is an exact, effect-free literal sequence of unsigned items after generic specialization; its length and item type agree with the analyzed Fold. The bound values are within the declared item type and the indexed Vector width.
2. The Vector index is the Fold item formal in the same lexical reducer scope. The indexed operand is an effect-free reference or is otherwise lowered with one-time, source-ordered evaluation. No unrelated, copied or shadowed source occurrence may borrow this domain.
3. The circuit evaluator's per-iteration index observations and successful guard facts agree **exactly** with the literal iteration sequence. Conflicting, absent, failing or extra observations refuse. Numeric values alone are not authority for guards or evaluation order.
4. The generated Rust projection selects a constant, checked Vector position for every admitted item. A value outside the certified domain fails closed. Duplicate domain values may share a projection only when their repeated iterations and effects remain intact.

A bounded implementation may keep the existing uniform `VectorFold` and emit a checked finite dispatch over its certified literal domain. If a case cannot satisfy the rules above, use per-iteration specialization with distinct occurrence identities and the same circuit authority, or retain the existing refusal. The dispatch is an internal lowering of a certified source domain; it is not a public runtime indexing feature. Any temporary maximum-domain size is an explicit capability limit, never a claim of general `for` support.

Conceptually, the pinned case must have these ordered effects:

```text
total := 0
iteration i=1: total := total + v[1]  // 2
iteration i=2: total := total + v[2]  // 5
return total                            // 5
```

The compiler must evaluate a source iterable once, preserve each loop binding's lifetime, execute body effects in source order, and skip the body entirely for zero iterations. For `for (const x of vector)` the count is fixed but element values need not be constants; the existing `VectorFold` path remains valid for its supported uniform body. This decision does not turn those values into a static index domain.

## Ownership and boundaries

- **Frontend/analysis:** keep range/generic bound validation, inclusive/exclusive semantics, return-in-loop refusals and Fold origin. If needed, preserve a reliable generated-from-`for` marker so a literal Fold from an unrelated source cannot impersonate the range construct.
- **Circuit pass and fact bridge:** certify the exact `(Fold, iteration, Vector-ref occurrence)` relationship and guard result. Do not collapse multiple valid observations into one chosen constant or use source coordinates as identity.
- **Rust IR emitter:** lower only the certified domain to typed constant projections and ordered Fold effects. Preserve original source locations for diagnostics.
- **Rust renderer/recorded plan:** reuse existing `VectorFold`, ledger action, transcript and replay validation where the checked dispatch is representable. A new narrowly typed static-fold IR node is justified only if a uniform body cannot preserve the required per-iteration semantics. No runtime ABI or public generated API change is expected for the bounded case.
- **Packaging/qualification:** bind original source bytes, compiler revision, Rust/TypeScript output, raw ZKIR, runner inputs, keys and proofs to one exact head. Local Darwin checks do not establish native Linux qualification.

## Rejected shortcuts and alternatives

- **Take the first or last observed index:** miscompiles `generic_bound` (2 or 3 instead of 5), so it is forbidden.
- **Allow arbitrary runtime Vector indexing:** widens Compact's compile-time index rule and can diverge from the unrolled circuit; it is outside this decision.
- **A general runtime `For` expression:** duplicates an already bounded source construct and does not by itself solve the circuit/Rust index-identity split.
- **Unroll every Fold before Rust lowering:** can duplicate effectful source evaluation and widen compiler change surface. Per-iteration specialization is a valid follow-up when a certified uniform Fold cannot express a source, but it needs its own exact evaluation-order and source-identity proof.

## Acceptance and failure controls

1. Port all three intact `generic_bound` originals byte-for-byte. Source-built paired TypeScript/Rust compilation and original runtime assertion must return 5 and leave `total = 5`; ordinary and recorded Rust must agree with TypeScript on public state, transcript and gas.
2. Retain the nine locally qualified `for` cases, including both accepted runtime results and all seven exact original diagnostic regexes. Test literal and generic bounds, repeated values, zero range and empty iterable, nested Fold scope, out-of-range and malformed literal sources, and source evaluation once.
3. Keep genuinely runtime-dependent Vector indices refused. Add duplicate-source, unrelated-observation, shadowed-formal, mismatched-domain, unknown/false guard and effectful-operand controls. Each refusal must precede publication of generated output. Do not turn a missing fact into a default index.
4. Run focused Scheme/compiler and Rust renderer/recorded tests, source-built exact-head generated TypeScript/Rust comparison and Clippy. Preserve existing VectorFold, Bytes and Ledger 8 behavior. Document any size cap and its refusal.
5. For the proof-required stateful original, generate fresh keys and a standalone proof, verify independently, exercise wrong-verifier/corrupt/binding refusals, then prove a distinct positive-fee transaction and apply under default strictness. Exact replay must refuse without changing ledger state. Run the grouped native Linux gate separately; do not label Darwin arm64 evidence as Linux.

Only after these checks may the remaining original case be counted as Rust AST compatible. The original Vitest harness and broader `for` corpus remain separate coverage claims.

## Tracking

- Focused implementation: [betsudo/compact #750](https://github.com/betsudo/compact/issues/750).
- Coverage and compiler owners: [#727](https://github.com/betsudo/compact/issues/727), [#728](https://github.com/betsudo/compact/issues/728); native integration: [#732](https://github.com/betsudo/compact/issues/732).
- Pinned upstream fixture: [source](https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/keywords/for/generic_bound/for_generic_bound.compact), [original assertion](https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/keywords/for/generic_bound/runtime.pass.test.ts).
- Language authority: [Compact reference, `for` statements](https://docs.midnight.network/compact/reference/compact-reference#for-statements), [generic bounds release note](https://github.com/midnightntwrk/compact/releases/tag/compactc-v0.31.0).

Implementation remains local under the main session's grouped CI and publication sequence. No merge or ADR-register publication is implied by this proposal.
