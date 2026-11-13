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


The wide scalar case preserves the pinned mainstream Scheme source,
callback and ZKIR expectations in `wide-cast/upstream.ss.txt`. The family
runner exercises the original values and verifies the three input limbs
and their ordering. It also refuses the wrong width and field. The wide
cast measures k=11 and 1,301 rows, so its original circuit uses the
explicit slow proof lane.

## Ed25519 verification

The family runner executes the original stateful callback from
`verify/stateful/upstream.ss.txt` on paired compiler output. It also runs
the original Wycheproof driver for 151 vectors. The driver uses the
upstream partition of 120 driven cases and 31 encoding exclusions.
At language 0.23.105 the intact corpus source refuses its 0.25.0 pragma.
The runner records that refusal and runs a derivative with only the
pragma removed. The sequential source and declarations are retained
intact and require the same language admission.

The two stateful exports require k=17. The two and three signature
sequential statements measure k=18. These statements belong in the
explicit slow proof lane.
