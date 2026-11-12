# Ledger 9 P-256 point representation

The original equality, identity hash, leaf tag and roundtrip tests
are archived from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. Their Compact bodies
are intact. The equality and identity callbacks retain the original
assertions with the current synchronous context path. The pinned
noble curve supplies their original host point multiples.

The point codec uses two shifted coordinate pairs and a native field
flag. The TypeScript host boundary and Rust finite constructor check
curve membership. The Rust FAB boundary accepts in-range ignored
identity coordinates and yields the canonical identity. The native
ZKIR boundary requires the canonical encoding. The tests check the
identity hash with an independent encoding and SHA-256 computation.

The leaf tests check the three original type names in the current
contract info and Rust IR. The upstream JavaScript `circuitSignatures`
export is an open metadata gap. The archived roundtrip also calls
coordinate accessors and will run intact when those are available.

```sh
for fixture in upstream-equality upstream-identity; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/point/$fixture/ts-rust/.build/zkir"
done
node test-contracts/stdlib/secp256r1/point/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-first"
```

The [shared P-256 consumer](../casts/README.md) accepts
`point-parity` and the five selected funded IDs from
`preimages-first/vectors.json`. They cover both point comparisons,
identity hashing and storing finite and identity values. Verify each
fresh proof independently before its separate funded application.
Build `compile-fail.rs` as a separate binary with the shared
consumer manifest and check its two E0308 refusals.
