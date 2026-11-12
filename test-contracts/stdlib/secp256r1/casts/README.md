# Ledger 9 P-256 byte casts

The original Scheme tests and stage snapshots are archived from
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The standalone Compact fixtures retain the original literals after the
license header. The four TypeScript callbacks use the current synchronous
query context. `constant` and `unused` extract the original P-256 cases
from the two mixed-curve tests. Their full Curve25519 cases remain in
the archives for later execution.

`Bytes<32>` converts to the selected field by little-endian modular
reduction. The reverse conversion emits the raw canonical bytes.
The checked host constructors and shifted ledger encoding still enforce
canonical values. The private IR retains the existing serialized
`secp256k1_bytes_cast` tag and selects the field through the typed
source and destination.

Run the [family runner](../README.md) with output
`$OUTPUT/family-first`. It checks four original width refusals, two
cross-field refusals and the original guarded and unguarded assertions.
Mock-compile the emitted ZKIR for the seven successful cast fixtures,
then run the SDK checks.

```sh
CASTS=test-contracts/stdlib/secp256r1/casts
for fixture in decode encode guarded-save guarded-decode guarded-encode constant unused; do
  "$ZKIR" mock-compile-many "$OUTPUT/family-first/casts/$fixture/ts-rust/.build/zkir"
done
node "$CASTS/check-preimages.mjs" "$OUTPUT/family-first" "$ZKIR_WASM" "$OUTPUT/preimages-first"
```

Copy `consumer.Cargo.toml` and `consumer.rs` to
`$OUTPUT/funded/consumer/Cargo.toml` and `main.rs`. Seed Cargo from the
workspace lock and build with Rust 1.99 and the pinned offline dependency
cohort. The `parity` mode takes the evidence root and an unused output
argument. It compares all generated Rust cases with the TypeScript
ledger expectations and replays the recorded VM operations.

The funded modes take a selected case ID from `preimages-first/vectors.json`,
the evidence root and a fresh output directory. Supply the matching
tagged keys and independently verified standalone proof under
`$OUTPUT/slow-proof/<case>/statement.{prover,verifier,proof}`.
Supply verified Ledger 9 parameters and Dust material. Each funded mode
compares Rust and TypeScript statement fields, applies a separate
transaction with default strictness, and checks unchanged replay refusal.
