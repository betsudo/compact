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
