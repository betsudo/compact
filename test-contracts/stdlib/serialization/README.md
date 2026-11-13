# Boolean serialization

`bool.compact` retains the two Boolean exports from the scalar serialization
test at upstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
`stateful.compact` retains the `d1` decoder and its ledger write from the
upstream #772 Boolean validation test. That upstream group also covers a
mixed struct. Its wider source and expected output remain separate from
this Boolean slice.

Generate both targets, and make the built Compact TypeScript runtime
resolvable from the generated outputs.

```sh
for fixture in bool stateful; do
  "$COMPACTC" --target ts --target rust --rust-require-recording \
    --feature-zkir-v3 --skip-zk --rust-runtime-root "$PWD" \
    "test-contracts/stdlib/serialization/$fixture.compact" "$OUTPUT/$fixture"
done
zkir-v3 mock-compile-many "$OUTPUT/stateful/zkir"
node test-contracts/stdlib/serialization/check.mjs \
  "$OUTPUT/bool" "$OUTPUT/stateful" "$ZKIR_WASM_PACKAGE" "$OUTPUT/preimages"
```

The runner checks both valid encodings and all 254 invalid byte values
against generated TypeScript and the ZKIR 3.2.0 SDK. It also checks the
public write, rollback on invalid input and rejection of a substituted
Boolean output. Use the two emitted preimages for independent proof
verification and use a separately proved transaction for strict application.
