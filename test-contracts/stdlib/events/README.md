# Standalone Ledger 9 events

`Inner.compact` is byte-identical to the standalone source in upstream
`test-center/composable/Events/Inner.compact` at
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. `Ordered.compact` covers a
conditional log, a second log and an assertion that fails after emission.

Generate both targets with the same current compiler and runtime. Make the
built TypeScript runtime package resolvable from the generated outputs.

```sh
for fixture in Inner Ordered; do
  "$COMPACTC" --target ts --target rust --rust-runtime-root "$PWD" \
    --rust-require-recording --feature-zkir-v3 --skip-zk \
    "test-contracts/stdlib/events/$fixture.compact" "$OUTPUT/$fixture"
  zkir-v3 mock-compile-many "$OUTPUT/$fixture/zkir"
done
node test-contracts/stdlib/events/check.mjs \
  "$OUTPUT/Inner" "$OUTPUT/Ordered" "$ZKIR_WASM_PACKAGE" "$OUTPUT/preimages"
```

The runner uses ZKIR SDK 3.2.0 to check the real compiled statements.
It tests event contents, emitter, order, guards, rollback and refusal of
altered VM transcripts. The untouched upstream fixture measures k=8.
The two-event control measures k=9 and this runner checks its model.
Standalone cryptographic verification and funded strict application use
separate proofs. Cross-contract events and the functional Rust target
remain outside this slice.

## ShieldedBurn

`ShieldedBurn.compact` retains the seven exports from the pinned
mainstream compiler test. The original Scheme and TypeScript assertions
are preserved in `shielded-burn-upstream.ss.txt`. The event uses version 1,
tag 3 and a 49-byte payload consisting of the nullifier, the optional
amount flag, and the little-endian 128-bit amount.

Run the generated TypeScript and SDK controls with:

```sh
node test-contracts/stdlib/events/check-shielded-burn.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/shielded-burn-preimages"
```

The runner checks event order, both conditional outcomes, and a
substituted transcript with the two events swapped. The proof lane
uses k=14 and is run explicitly.

## ShieldedMint

`ShieldedMint.compact` retains the seven original exports.
`shielded-mint-upstream.ss.txt` preserves their upstream compiler test.
The version 1, tag 2 payload contains a 32-byte commitment, a 32-byte
domain separator, an optional amount flag and a little-endian 128-bit
amount.

```sh
node test-contracts/stdlib/events/check-shielded-mint.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/shielded-mint-preimages"
```

The runner checks all event outcomes, preserves a structural inactive
amount in codec roundtrips, and rejects an invalid flag, changed domain
separator and swapped event order.

## ShieldedReceive

`ShieldedReceive.compact` retains the seven original exports and
`shielded-receive-upstream.ss.txt` preserves their compiler test. Upstream
`7489a7926` corrected the version 1, tag 1 payload to put the commitment
first, then the optional ciphertext, then the optional contract address.
The payload is 578 bytes. Each optional field retains its value bytes
when its presence flag is false.

```sh
node test-contracts/stdlib/events/check-shielded-receive.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/shielded-receive-preimages"
```

The runner checks the exact wire order, both flag positions, unchanged
input context, conditional emission and refusal of substituted or
reordered logs.

## UnshieldedSpend

`UnshieldedSpend.compact` retains the seven original exports and
`unshielded-spend-upstream.ss.txt` preserves their compiler test. The
version 1, tag 4 payload contains the sender selector and both sender
branches, the domain separator, the token type and the 128-bit amount.
Its structural encoding retains the inactive sender branch.

```sh
node test-contracts/stdlib/events/check-unshielded-spend.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/unshielded-spend-preimages"
```

The runner covers exact payload bytes, both sender variants, an invalid
selector, and refusal of a substituted inactive branch and reordered logs.

## UnshieldedReceive

`UnshieldedReceive.compact` retains the seven upstream exports.
`unshielded-receive-upstream.ss.txt` preserves the original compiler test.
Version 1, tag 5 uses the 145-byte layout with an `Either` recipient.
Both branches retain their bytes through a codec roundtrip.

```sh
node test-contracts/stdlib/events/check-unshielded-receive.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/unshielded-receive-preimages"
```

The runner checks both recipient variants and refuses substituted inactive
branch bytes, a malformed selector, and reordered logs.
