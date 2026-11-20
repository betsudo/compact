# Original shielded mint to contract proof gate

The pinned `mintShieldedToContractTest` source passes
`right<ZswapCoinPublicKey, ContractAddress>(disclose(address))` to
`mintShieldedToken`. Its original TypeScript test asserts a 100-unit shielded
mint and one claimed shielded spend, while all other Effects, including
`claimedShieldedReceives`, stay empty. This slice retains those original
fixtures and checks the same result and Effects in generated Rust.

The Rust gate funds and deploys distinct minting and recipient contracts.
It binds one real contract-owned output to the generated mint call, checking
its recipient address, commitment and ledger index. A missing output offer
refuses the claimed spend, and a mint-only transaction with that output
refuses because no recipient claims its receive. The recipient's separate
`receiveShieldedTest` call then claims the same commitment. Both calls are
recorded with offer-bound contexts and checked against their installed v3 IR
and their independent original Effects.

The paired recipient receives the minter-derived token color. The original
standalone `receiveShieldedTest` TypeScript fixture instead supplies a token
color derived from the receiving contract's own address. A separate Rust
offer-bound self-color call checks that original receive Effects and proves
and independently verifies it; the unchanged original TypeScript suite
retains the exact fixture and assertions.

Both calls produce fresh standalone v3 proofs with separate verification,
corrupt-proof and changed-binding refusals. A same-segment, two-contract
transaction produces distinct transaction proofs, spends positive Dust for
fees, passes default-strict Ledger 9 application, and refuses replay without
changing serialized ledger bytes. The original mint circuit still has no
auto-receive; only the recipient call claims the output.

The local Linux gate also supplies both `zkir` and `zkir-v3` executable names
for the pinned ZKIR v3 binary. The compiler probes the former before invoking
the latter; without the alias it exits successfully after 31 generated files
and no keys. The gate already enforces 91 generated files and 20 prover and
verifier key pairs. The failed 31-file attempt is retained only as a rejected
diagnostic.

Qualification is local Darwin arm64 with the Ledger 9.1.0.1 SDK matrix and
offline `TestState`. Native Linux and a live node remain separate gates. No
original source, TypeScript assertion, Ledger 8, or Forge artifact changes.
