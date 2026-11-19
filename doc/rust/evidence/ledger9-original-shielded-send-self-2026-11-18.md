# Original shielded self-send proof gate

The unchanged pinned `sendShieldedToSelfTestWithChange` case spends a
qualified 100-unit coin, sends 60 to the contract, and returns 40 change.
It claims one input nullifier, two shielded spends, and two shielded
receives, with all other Effects empty. The unchanged
`sendShieldedToSelfTestNoChange` case spends all 100, claims one nullifier,
one shielded spend and the same one shielded receive, with all other
Effects empty. The Rust gate checks these exact sets and result values
through generated direct and recorded calls.

Each case deploys a contract into a fee-funded Ledger 9 `TestState`,
seeds a real contract-owned 100-unit Zswap coin, obtains its Merkle index,
and constructs a matching qualified contract-owned input. The generated
output coins are bound to an authoritative Zswap offer. A wrong Merkle
index refuses at intent preparation. Without the offer, Ledger refuses
the exact claimed input nullifier and contract address against an empty
real nullifier set.

Each case generates a fresh standalone v3 proof, verifies it separately,
and refuses a changed public binding and a corrupted proof. A separate
transaction proof, positive Dust fee, funded default-strict application,
post-application output commitments and indices, spent nullifier, and
byte-identical replay refusal establish offline Ledger admission.

The generated Rust/TypeScript, ZKIR, and keys are copied byte-for-byte
from the prior signed `4091a72d2c6c4222688556ab63511a8218b0a77e`
qualification. The detached receipt binds the signed source, original
fixtures, generated artifacts, proofs, and Ledger outputs. Original
TypeScript 19/19 is inherited only by matching generated JavaScript
hash; it is not rerun in this slice.

This is local Darwin arm64 offline `TestState` evidence. Native Linux
and a live node remain separate gates. Ledger 8, Forge, and the original
fixtures are unchanged.
