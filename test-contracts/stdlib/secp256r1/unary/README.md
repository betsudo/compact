# Ledger 9 P-256 negation and inversion

The two Compact fixtures preserve the original bodies from
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
Each complete Scheme test and ZKIR 3.1 snapshot is archived.
The local compiler's 3.0 output is checked for the same typed input,
paired returns, communication commitment and inverse and negation
dependencies.

The family runner covers nonzero operands and modulus boundaries,
zero negation, active-zero inverse errors, inactive-zero evaluation,
discarded inverse checks and user-defined overload resolution. The
tests inspect the actual private ledger cell and verify that an error
leaves the original state unchanged. Both original fixtures refuse
the legacy proof profile. The extra fixtures refuse invalid argument
and return types in all three target modes.

Run the SDK preimage checks after compiling the family.

```sh
for fixture in upstream-base upstream-scalar unary-controls; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/unary/$fixture/ts-rust/.build/zkir"
done
node test-contracts/stdlib/secp256r1/unary/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-first"
```

The [shared P-256 consumer](../casts/README.md) accepts
`unary-parity` and the four selected funded case IDs from
`preimages-first/vectors.json`. Verify each fresh standalone proof
before its separate funded run. The selected cases include both original
fixtures and one inactive-zero branch for each field. Use the explicit
slow lane for these proofs.

Build `compile-fail.rs` as a separate binary in a copy of the shared
consumer manifest. All three mixed-field calls must report E0308.
