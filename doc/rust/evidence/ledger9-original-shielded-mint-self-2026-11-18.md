# Original shielded mint to self proof gate

This slice uses the unchanged pinned `test-center/compact/unshielded-tokens.compact`
and its unchanged `test-center/ts/unshielded-tokens.ts` assertion for
`mintShieldedToSelfTest`. The original test requires a 100-unit shielded mint,
one claimed shielded spend, one claimed shielded receive with the same
commitment, and empty unrelated Effects.

The generated Rust recording call uses the exact full original compiler
bundle, including the circuit's ZKIR and prover/verifier keys. It constructs
a contract-owned output from the returned coin information, binds that
output to the observed offer, and checks that both claimed commitment sets
contain the output's exact commitment. A missing output offer must refuse
Ledger effects reconciliation at the claimed receive commitment. The
positive path proves the call, verifies it separately, tests changed binding
and corrupt proof refusals, then applies a positive-fee funded transaction
with default strictness and requires byte-identical replay refusal.

The original TypeScript test covers Effects, while the Rust gate additionally
checks the physical offer, output index and funded transaction. Qualification
is local Darwin arm64 offline. Native Linux and live-node execution remain
separate gates. No generated source, original fixture, Ledger 8 or Forge
artifact is modified by this slice.
