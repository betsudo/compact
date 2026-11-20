# Ledger 9 P-256 group operations

The pinned LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e` supplies the three
`ecAdd`, `ecMul` and `ecMulGenerator` overloads and the
[original computed-identity fixture](../accessors/upstream-computed).
The TypeScript runtime tests retain its group, validation and coordinate
assertions. The fixture keeps its original body, identities and finite
assertions. The Rust runtime uses the Ledger 9 P-256 group.

The additional guarded fixture checks the active and inactive
operations, scalar boundaries, a finite point with x=0 and canonical
identity results. Compact checks reject mixed-curve points and scalars
and passing a base field to generator multiplication. Build
`compile-fail.rs` separately with the shared consumer manifest to check
three corresponding generated Rust E0308 errors.

```sh
for fixture in accessors/upstream-computed group/guarded; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/$fixture/ts-rust/.build/zkir"
done
node test-contracts/stdlib/secp256r1/accessors/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-group" group
```

The [shared P-256 consumer](../casts/README.md) accepts `group-parity`
and funded IDs selected in `preimages-group/vectors.json`. Each selected
case requires a fresh, independently verified proof. The selected
multiplication uses fixture state seeded by the original `storePoint`.
The funded call proves against that exact state and checks the
default-strict application and unchanged replay refusal separately.
The group exports use the explicit slow proof lane.
