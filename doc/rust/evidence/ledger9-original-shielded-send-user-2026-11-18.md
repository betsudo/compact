# Original shielded send to User on Ledger 9

This local slice qualifies the unchanged pinned `sendShieldedToUserTest` in
`test-center/compact/unshielded-tokens.compact` and its original TypeScript
case in `test-center/ts/unshielded-tokens.ts`. It extends the signed
contract-send proof branch without modifying the source fixture or generated
91-file Rust/ZKIR/key bundle.

The original case spends a 100-unit qualified contract-owned shielded coin,
sends 60 to a User public key, and returns 40 change to the sender contract.
The sender call must claim one input nullifier, two shielded spends, and only
the contract-owned change receive. All other Effects remain empty. The
user-owned output must not be auto-received by the contract.

The Rust gate seeds a real contract-owned output and derives its Merkle index
and input proof from the resulting Ledger tree. It binds both generated
output commitments and their canonical offer indices to a physical Zswap
offer, checks the disclosed recipient key, refuses a wrong key and wrong
Merkle index, and retains the exact no-offer nullifier refusal. A fresh
standalone proof is separately verified; changed binding and corrupted proof
must refuse. The fee-funded transaction is checked and applied at default
strictness, then replayed against its updated Ledger bytes and refused
without mutation. Post-application checks assert that the 60-unit output is
User owned and the 40-unit change is owned by the sender contract.

The exact signed-head gate also reruns all sixteen previously qualified
unique original-token Rust circuits. The older original TypeScript 19/19
result is carried only when the generated JavaScript hash is identical;
it is not rerun in this slice. Qualification is local Darwin arm64 offline
TestState evidence. Native Linux, live node, indexer, proof server, hosted
CI, and publication remain separate gates. The detached-GPG receipt under
`~/.codex/local-evidence/ledger9-shielded-send-user-pr772-20261118/`
records exact source, artifact, proof, transaction and replay hashes.
