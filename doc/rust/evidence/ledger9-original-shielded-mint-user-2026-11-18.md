# Original shielded mint to user proof gate

The unchanged pinned `mintShieldedToUserTest` source sends a 100-unit
shielded mint to a disclosed `ZswapCoinPublicKey`. Its original TypeScript
test expects one claimed shielded spend and no claimed receive or other
Effects. The Rust gate retains those fixtures and checks the result and
negative-space Effects from the generated recording call.

The gate deploys the source contract with a funded Ledger 9 `TestState` and
binds the call to a real user-owned Zswap output. It checks the same public
key in the call and output, the output commitment, and the post-application
Merkle index. A different user key refuses against the authoritative offer;
a missing output offer refuses the claimed spend.

A fresh standalone v3 proof passes verification, a changed public binding
refuses, and a separate verifier process refuses a corrupted proof. A second
transaction-bound proof, a positive Dust fee, default-strict funded
application, and byte-identical replay refusal establish Ledger admission
for this original circuit. The mint itself still has no auto-receive.

The generated bundle is copied byte-for-byte from the prior signed
`b06bcd4ee183a9bf3dee5ffc6c96124d89386ee7` qualification and its
91-file hash map is checked before proving. The final receipt binds the
signed source, original fixtures, generated bundle, proof and Ledger output.
The original TypeScript 19/19 result is inherited only where its generated
JavaScript bytes match; it is not rerun in this slice.

This is local Darwin arm64 offline `TestState` evidence. Native Linux, a
live node and the remaining original shielded circuits need separate gates.
No Ledger 8, Forge or original fixture bytes are modified.
