# Ledger 9 P-256 field arithmetic

The original six circuits, typed ZKIR snapshot and Scheme test are
archived from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The Compact source
preserves the original body. The vector file expands all 34
upstream assertions in order, using the pinned field bounds.
The family runner executes those assertions with the generated
TypeScript and checks rejected mixed operands in all target modes.

Both Rust field carriers use typed `Add`, `Sub` and `Mul`.
The SDK gate checks the two input types and the return type,
subtraction lowered to negation and addition, shifted FAB atoms,
and the original ledger effects. It rejects altered public writes
and returns.

```sh
"$ZKIR" mock-compile-many "$OUTPUT/family-first/arithmetic/ts-rust/.build/zkir"
node test-contracts/stdlib/secp256r1/arithmetic/check-preimages.mjs \
  "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-first"
```

Use the [shared P-256 consumer](../casts/README.md) with
`arithmetic-parity` to compare the 34 generated Rust executions
with TypeScript and public VM replay. The six selected case IDs
use the original final operand pair for each export. Use each ID
for the separately funded default-strict application after
fresh standalone proof verification. These statements use the
explicit slow lane.

For the generated Rust argument refusal, add `compile-fail.rs` as
a separate binary to a copy of the consumer manifest. Check that
the generated calls each report E0308 for the indicated P-256
scalar, K1 base or native Field supplied to the wrong carrier.
