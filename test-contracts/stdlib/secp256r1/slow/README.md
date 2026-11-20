# Ledger 9 P-256 SHA-256 and sequential signature suites

The original [SHA-256 KAT](../sha256_kat) and
[sequential witness circuits](verify_sequential) retain their Compact
sources and test declarations from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The family runner attempts
the intact language pragma first. On the current compiler it uses a
separate copy with only that line removed. Use `--upstream-kat` to
require intact language admission.

The SHA-256 runner invokes the unchanged upstream runtime callback.
The shared Rust consumer checks every driven case using both an
independent SHA-256 digest and the corresponding generated
`proveEcdsaN` function. Eleven pure exports cover all message widths,
including zero. The original 219 DER exclusions and all 265 driven
cases account for the 484-vector corpus. The tests also cover empty
input and a changed message.

```sh
"$ZKIR" mock-compile-many "$OUTPUT/family-first/signature-suites/sequential/ts-rust/.build/zkir"
node test-contracts/stdlib/secp256r1/slow/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-sequential"
```

The sequential controls check signature and key callbacks in source
order, private-state transitions, all ten single-witness substitutions,
and the identity-key and zero-s refusals. A correctly generated
substituted call writes zero. Replacing witness atoms in a statement
that still claims one fails SDK checking.

The [shared Rust consumer](../casts/README.md) accepts `ecdsa-corpus`
for the SHA-256 KAT, `sequential-parity` for the generated witness
and replay checks, and `two` or `three` for funded runs. The two
funded exports require separately verified fresh proofs and
verified k=17 and k=18 parameters. They use the explicit slow lane.
