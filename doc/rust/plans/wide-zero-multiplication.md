<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Checked wide-Uint multiplication by zero

Status: local slice stacked on PR #699. Work began at reviewed `59072920a`;
the parent now includes signed fixture repair `2bdfefc0a`, whose new proof,
review and CI gates remain pending. Unique local edits were preserved before
fast-forwarding that parent correction.
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
