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

## UnshieldedMint

`UnshieldedMint.compact` retains the seven upstream exports.
`unshielded-mint-upstream.ss.txt` preserves the original compiler test.
Version 1, tag 6 encodes the domain separator, token type and little-endian
128-bit amount in 80 bytes.

```sh
node test-contracts/stdlib/events/check-unshielded-mint.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/unshielded-mint-preimages"
```

The runner checks the wire layout with distinct byte fields and rejects
a substituted token type and reordered logs.

## UnshieldedBurn

`UnshieldedBurn-codec.compact` retains the original two codec exports.
The upstream type and codec-lowering assertions are retained alongside
it. `UnshieldedBurn-emission-derived.compact` adapts the upstream Spend
emission fixture to Burn and includes those two codec exports.

Version 1, tag 7 encodes the selector and both sender branches, the
token type, and a little-endian 128-bit amount in 113 bytes.

```sh
node test-contracts/stdlib/events/check-unshielded-burn.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/unshielded-burn-preimages"
```

Use the generated output of the derived emission fixture. The runner
checks both structural sender variants and refuses changed inactive
bytes, a malformed selector, and reordered logs.

## Paused and Unpaused

`Paused.compact` preserves the seven exports of the upstream Paused
fixture at `672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The compiler
suite runs the original eight JavaScript assertions. The original
type and codec pass assertions are retained beside the fixture.

`PauseTransition.compact` is a derived fixture with both tags in one
trace. It includes the original Unpaused codec sources.
`check-pause-events.mjs` checks exact empty payload alignment and
distinct tags, rejects a tag substitution and a reordered trace, and
retains the SDK preimages for proving. The mixed Unpaused/Misc pass
assertion runs against the local flattened IR.

```sh
node test-contracts/stdlib/events/check-pause-events.mjs \
  "$PAUSED_OUTPUT" "$TRANSITION_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/pause-preimages"
```

## Misc

`Misc.compact` preserves all seven upstream exports. The local compiler
suite runs the original standalone assertions and compares the mixed
Unpaused/Misc pass against the corresponding local IR. Both targets
serialize the 32-byte name followed by the 256-byte payload.

```sh
node test-contracts/stdlib/events/check-misc.mjs \
  "$MISC_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/misc-preimages"
```

The SDK checks cover both conditional branches and refuse changes to the
name, payload, trailing zero byte and event tag.
