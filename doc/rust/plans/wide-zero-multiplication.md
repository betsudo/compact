<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Checked wide-Uint multiplication by zero

Status: local slice stacked on PR #699. Work began at reviewed `59072920a`.
The parent now includes signed fixture repair `6166ab906`, whose fresh Codex
source review passes under the authorized Claude quota fallback. Complete
parent CI/archive acceptance remains open. Unique edits were preserved before
normal signed parent reconciliation; no history was overwritten.
The owner explicitly authorized local progress while #699's long-running CI is
pending or red. Parent integration still requires its complete checks, proof
gates and retained evidence. A parent failure will be repaired on its branch;
this slice will incorporate a correction when it changes a shared premise.

## Gap and public API

Compact accepts `zero(x: Uint<200>): Uint<0..1> { return x * 0; }`.
The current Rust backend refuses the wide multiplication. Existing checked
`WideUint` inputs can represent the entire source domain without narrowing.
The generated API should accept
`WideUint<4722366482869645213695, 340282366920938463463374607431768211455>`
and return `Result<BoundedUint<0>, CompactError>` through the existing circuit
API. The same law applies with zero at either operand position.

## Owners and implementation boundary

- `pure_semantics/numeric.rs`: select a zero-product strategy only after all
  three bounds are canonical. One operand must have maximum zero, the other
  must be wide, and the result must have a canonical small maximum. Zero fits
  that result bound without narrowing the wide operand.
- `value_lowering.rs`: emit a structured Rust block evaluating both checked
  operands once, left to right, before constructing the zero result.
- `recorded/effect_plan/lower/binary.rs` and `validate`: admit the same bounded
  operation, with independent validation of SSA references and all three types.
- Existing native and recorded syntax adapters consume that strategy. Existing
  sequencing, carriers, codecs and ledger/proof test infrastructure remain the
  owners of execution and untrusted boundaries.
- `compiler/rust-ir-passes.ss`: carry the declared return type through the
  existing ordinary return owner and use its existing typed arithmetic lowering
  for multiplication returning the zero singleton. Preserve other historical
  return annotations.

The original stateful frontend rounds the internal product carrier to maximum
one; pure typed products carry maximum zero. Preserve either annotation, then
use the declared public zero type in the ordinary return owner. The typed
numeric policy also admits a canonical small intermediate carrier because zero
fits it exactly; it never narrows the wide operand.

No operand value is inspected to decide applicability. A wide value that happens
to be zero does not justify accepting a general wide product. Failed casts,
assertions and witness effects inside either operand must remain observable.

The original parent compiler accepts literal-zero stateful products, including
a wide witness, and zero-typed parameters in pure functions. Its downstream
`Lflattened` checker rejects stateful products with a nonliteral zero-typed
parameter or witness before either backend emits code. The isolated reproducer
and diagnostics are retained separately. This slice qualifies the accepted
source forms and tests the broader typed-plan law independently; it does not
claim to repair that upstream circuit-lowering issue.

## Acceptance

1. Establish the focused local baseline at the exact parent head and reproduce
   the supported TypeScript/refused Rust case with its original compiler.
2. Cover both operand orders, aliases/nesting, and wide values 0, 1, 2^128 and
   2^200−1. Preserve the existing small multiplication and wide-add behavior.
3. Exercise pure, ordinary stateful, recorded and functional routes, including
   observable witness order/count and first-error controls in both operands.
4. Keep historical compatibility tests. Independently test the newly admitted
   domain, malformed-bound precedence, general nonzero wide-product refusals,
   forged result/operand bounds, invalid SSA references and untrusted decoding.
5. Compile generated consumer APIs; wrong ordinary carriers must fail for their
   intended type error. Require precise TypeScript cast failures and a canary
   that rejects a wrong failure family.
6. Where a stateful selected call has checked programs, generate fresh keys and
   real proofs, verify/apply/replay them under the maintained funding policy,
   and reject exact-source/program/input/result/proof mutations. Pure functions
   and failed paths do not acquire a proof claim from those selected calls.
7. Use focused checks locally. Before integration, retain the complete required
   CI/proof gates, both fresh actual Claude and Codex CLI reviews, documentation,
   signed DCO/Assisted-by commits and source-bound receipts. Notify the owner
   when the logical PR is ready for review.

## Exclusions

General wide multiplication/subtraction/order, new public traits or bigint
carriers, new evaluators or recognizers, ABI/schema changes, effect erasure,
unchecked narrowing, broad refactoring, a CI optimization project and Forge
adoption are outside this slice. Cleanup follows verified merge and preserves
unpublished work, needed ignored files and historical qualification evidence.

## Maintained validation and current limits

`check_wide_zero_multiplication.py` emits the complete source afresh for Rust
and TypeScript. It checks the genuine wrapper at four wide boundaries and
compiles an external consumer covering ordinary, recorded and both functional
routes against a manual Counter transition/replay. A fallible wide witness is
observed once after that transition even when literal zero is the left operand.
It retains private state/transcript checks, distinct first-error assertions,
precise range errors, a positive API twin and two E0308 controls at their exact
ordinary misuse spans. A deliberately changed TS result must fail the actual
Rust consumer. No generated API is edited to make the tests pass.

The TS capture requires the actual selected runtime's CompactError class and
exact cast family, value and maximum; wrong family/value/maximum canaries run
against each observed cast. It retains the runtime production closure and Node
binary/external-library identities. Host OS and kernel mapping names are
recorded separately without a byte-qualification claim.

`check_wide_zero_multiplication_proof.py` separately owns fresh selected
`multiplyRight`/`multiplyLeft` keys, functional-derived proofs, independent
verification, default-strict funded application and replay refusal. The linked
fixture is the complete original compiler output, formatted only, including all
four checked programs and the complete raw IR. The Rust runner independently
checks that entire IR and program roster before keys/state. This selected proof
scope does not imply a pure-function, failing-path, constructor or witnessed
call proof. Full required integration CI remains mandatory.

The exact local proof receipt and review/integration status are recorded in the
main-session checkpoint. Development failures remain retained; parser-summary
controls do not stand in for cryptographic execution. Forge stays deferred.

### Zero-singleton proof boundary discovered during qualification

The first genuine selected proof attempt used stateful `Uint<0..1>` results.
It stopped at `ProofPreimage::check` with `Communications commitment mismatch`.
The original compiler emits a zero-width unsigned output descriptor, whose
aligned value contributes zero fields, while that source's ZKIR emits a zero
field output. The retained failed source, original compiler receipt, fresh keys
and exact failed check remain evidence; no binding is normalized or waived.

The two selected ordinary proof APIs therefore declare `Uint<8>` and return
`BoundedUint<255>` with value zero. That is an explicit source result type,
without a generated-interface rewrite. Pure APIs and witnessed behavior controls
retain `BoundedUint<0>`, including exact first-error and untrusted-boundary tests.
Those zero-singleton result routes have no successful proof claim. Repair of
the zero-width frontend/ZKIR protocol edge is a separately recorded capability
gap; this slice does not claim it fixed or infer whole-backend proof readiness.

Local development qualification now passes for the explicit selected `Uint<8>`
source: both genuine 2912-byte functional-derived call proofs verify, reject
changed binding/input/result/proof bytes, apply under default-strict funded
ledger rules, and reject replay without state mutation. Complete raw IR, all
four programs and the formatted linked library match fresh original compiler
output; both pre-key source/program negatives reject. These receipts are local
source-bound executions; fresh CLI reviews and complete current CI/archive
acceptance are still required before integration.
