# Ledger 9 P-256 coordinate accessors

The [original roundtrip](../point/upstream-roundtrip) and the archived
direct, ledger and checked-assertion fixtures preserve their Compact
bodies and original assertions from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The runner adapts the original async calls to the current synchronous
generated runtime. It keeps the upstream x=0 point and generator.

Active extraction from the identity fails with a coordinate-specific
error. The inactive branch uses the finite P-256 generator for the
SDK's eager coordinate extraction and selects the original zero
result. The `guarded-writes` fixture checks both branches under proof.
The original computed-identity fixture runs intact in the
[P-256 group lane](../group/README.md).

```sh
for fixture in point/upstream-roundtrip accessors/upstream-ledger accessors/upstream-asserted accessors/guarded-writes; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/$fixture/ts-rust/.build/zkir"
done
node test-contracts/stdlib/secp256r1/accessors/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-first"
```

The [shared P-256 consumer](../casts/README.md) accepts
`accessor-parity` and the selected funded IDs from
`preimages-first/vectors.json`. Read-only accessors and coordinate
writes that read an existing point seed the isolated fixture state
using the original generated store. The funded transaction proves
the selected call against that state. Build `compile-fail.rs`
as a separate binary with the shared manifest and check its two
E0308 refusals.
