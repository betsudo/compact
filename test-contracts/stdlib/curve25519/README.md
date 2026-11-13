# Curve25519 fields

The original round-trip, cast-refusal and Maybe/Either files come from
mainstream revision `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
Run the field fixtures with a compiler from this tree and its built
TypeScript runtime.

```sh
node test-contracts/stdlib/curve25519/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT/family"
"$COMPACTC" --target ts --target rust --rust-runtime-root "$PWD" \
  --rust-require-recording --feature-zkir-v3 --skip-zk \
  test-contracts/stdlib/curve25519/controls/field-defaults.compact "$OUTPUT/defaults"
node test-contracts/stdlib/curve25519/check-preimages.mjs \
  "$OUTPUT/family" "$OUTPUT/defaults" "$ZKIR_WASM_PACKAGE" "$OUTPUT/preimages"
```

The preimage check uses the pinned ZKIR SDK 3.2.0. Run the generated
artifacts with the same built runtime, resolvable from both output paths.
It tests six inputs per field and refuses modified inputs, ledger
writes and outputs. The tests cover shifted zero and the boundaries
at bit 192 for Base and bit 204 for Scalar.

Both original round trips measure k=11 and belong in the explicit
slow proof lane. Each funded check uses its own transaction proof.
The retained Maybe/Either fixture also names Curve25519Point. Run it
with the point port. The standalone field reset checks the two field
defaults.
