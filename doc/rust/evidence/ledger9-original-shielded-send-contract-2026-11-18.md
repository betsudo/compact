# Original shielded contract-send proof gate

The unchanged pinned `sendShieldedToContractTest` case spends a qualified
100-unit contract coin, sends 60 to a distinct contract and returns 40
change to the sender. Its sender call claims one input nullifier, two
shielded spends and only the change commitment as a shielded receive;
all other Effects are empty. The recipient output is not auto-received
by the sender call. The Rust gate checks those exact sets and values
through generated direct and recorded calls.

The gate deploys both contracts into a fee-funded Ledger 9 `TestState`,
seeds a real sender-owned Zswap coin, obtains its Merkle index and
constructs a matching qualified contract-owned input. The generated
sent and change coins are bound to an authoritative Zswap offer with
separate recipient and sender addresses. A wrong Merkle index refuses
at intent preparation. Without the offer, Ledger refuses the exact
claimed input nullifier and sender address against an empty real
nullifier set.

The gate generates a fresh standalone v3 proof, verifies it separately
and refuses a changed public binding and a corrupt proof. A separate
recipient `receiveShieldedTest` proof and same-segment call, a separate
transaction proof, positive Dust fee, funded default-strict application,
post-application output commitments and indices, spent nullifier and
byte-identical replay refusal establish offline Ledger admission.
The sender-only transaction with the output offer is refused because
the recipient contract commitment lacks its own receive claim.
The qualified input is seeded directly into authoritative Ledger state
and its Merkle root is block sealed; the application uses the Ledger
state rather than the test wallet's unsynchronized event replay view.

The generated Rust/TypeScript, ZKIR and keys are copied byte-for-byte
from the prior signed self-send qualification. The detached receipt
binds the signed source, original fixtures, generated artifacts, proofs
and Ledger outputs. Original TypeScript 19/19 is inherited only by
matching generated JavaScript hash; it is not rerun in this slice.

This is local Darwin arm64 offline `TestState` evidence. Native Linux
and a live node remain separate gates. Ledger 8, Forge and the original
fixtures are unchanged.
