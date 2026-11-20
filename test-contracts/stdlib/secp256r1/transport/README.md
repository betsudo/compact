# Ledger 9 P-256 witness and tuple hash coverage

The Compact bodies and Scheme blocks are preserved from
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`,
`compiler/test.ss:82729–82822` and `compiler/test.ss:83672–83690`.
The original callbacks run with the current synchronous context API.
The witness tests use the finite x=0 point and the generator. Additional
controls check the callback and private-state transition and reject
malformed points at ingress.

The tuple hash uses two P-256 points in order. The independent test
encoder produces two 96-byte point representations with shifted
coordinates and a 32-byte identity flag per point. It checks the
original pair, the reverse order, and an identity with ignored
coordinates next to the finite x=0 point.

```sh
for fixture in upstream-witness upstream-hash608; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/transport/$fixture/ts-rust/.build/zkir"
done
node test-contracts/stdlib/secp256r1/transport/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-transport"
```

The SDK admits the honest preimages and rejects altered writes,
changed returns, substituted or malformed witnesses, and reordered or
malformed hash arguments. The [shared P-256 consumer](../casts/README.md)
runs `transport-parity` and the three selected funded IDs. The two
witness exports measure k=12 with 2,153 rows each. The original tuple
hash measures k=14 with 11,554 rows. All use the explicit slow lane.
