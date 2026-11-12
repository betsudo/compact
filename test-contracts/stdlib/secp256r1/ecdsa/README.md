# Ledger 9 P-256 ECDSA verification

The standard library preserves the P-256 signature type and verification
body from LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The [original Scheme fixture](upstream-ordinary) binds the message by
hashing it inside the proof. It accepts both s values, returns false for
a different key, and refuses the identity key. Local controls also check
the wrong message, a zero s and an off-curve key.

The [original Wycheproof fixture](../ecdsa_verify) retains the upstream
language requirement. The P-256 family runner tries it intact and
retains its result. With the current compiler, a separate copy removes
only the pragma so the original runtime callback can run. Use
`--upstream-kat` to require the intact compile. The pinned corpus has
265 driven cases and 219 DER encoding exclusions. The shared Rust
consumer independently hashes each message and checks all driven
cases with `ecdsa-corpus`.

```sh
"$ZKIR" mock-compile-many "$OUTPUT/family-first/ecdsa/upstream-ordinary/ts-rust/.build/zkir"
node test-contracts/stdlib/secp256r1/ecdsa/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-ecdsa"
```

The [shared consumer](../casts/README.md) accepts `ecdsa-parity`
and funded IDs selected in `preimages-ecdsa/vectors.json`. Build
`compile-fail.rs` separately with that manifest for the scalar,
base-field and public-key type refusals. The original, twin and
wrong-key transactions use the explicit slow proof lane. The
alternate verifier for refusal checks comes from a separate circuit.
