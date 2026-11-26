# Original gas-cost fixture

`gas-cost.compact` and `gas-cost.ts` are byte-exact copies from
LFDT-Minokawa/compact at `672bd0f226f47a7d036c5c074ce71fbe1af1925e`,
under `test-center/compact` and `test-center/ts`. The existing repository
`test-center` versions are adapted: they omit the explicit Field casts and
convert the original asynchronous tests to synchronous calls. Keep these
original copies, including both async bodies and the promise rejection assertion.
`provenance.json` records the original hashes and the genuine frontend JSON.

The original casts produce a typed Field-literal `let` before each Set
mutation. The generator's optional proof-data helper supports this closed
form and direct Field literals on declared Field Sets: no public parameters,
a Unit result, then one remove and one insert. Each let has one Field-literal
binding used as the mutation operand. Other let expressions, coercions,
witnesses, calls and action sequences do not acquire this optional capability.
The existing recording gate still applies, and helper-name collisions omit
only the new helper.

The emitted `recording().testGasCost_proof_data(context)` captures empty Unit
input before one recorded execution and retains the actual Unit result and
transcripts. Explicit binding and communication randomness are supplied when
converting the result to a proof preimage. Runtime API version 1 and ABI 54
remain unchanged.

Host tests use the genuine emitted IR plus clearly supplemental mutations.
They check rendering and admission; they do not execute the original TS tests,
qualify gas behavior, generate proofs, or establish Linux compatibility. Those
require separate generated SDK consumers and qualification evidence. Ordinary `initial_state` supports this constructor. No private full-state constructor surface is ported.

## Current port scope

The retained IR/provenance files describe the donor frontend epoch, not a new
Ledger 9.1.0.2 compiler/runtime/proof qualification. Current host tests inspect
that immutable schema-20 input. Helpers are emitted only through the explicit
standalone-proof-data opt-in. Original Compact/TypeScript bytes are unchanged.
Fresh compiler, generated-consumer, proof and Linux validation remain separate.
