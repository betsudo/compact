# Ledger 9 P-256 field carriers

The 15 files under `base/` and `scalar/` match LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The runner checks the six
cast refusals through TypeScript, Rust and combined compilation. It
executes the original base-field callback and the local scalar and
default controls. The original sources compile with their intact bytes.

In the development shell, supply a built compiler, the built TypeScript
runtime with its pinned test dependencies, and the Ledger SDK ZKIR 3.2
CLI and WASM package. Use a fresh evidence directory.

```sh
FIELDS=test-contracts/stdlib/secp256r1
node "$FIELDS/check.mjs" "$COMPACTC" "$TS_RUNTIME" "$OUTPUT/family-certified"
"$ZKIR" mock-compile-many "$OUTPUT/family-certified/base/roundtrip/ts-rust/.build/zkir"
"$ZKIR" mock-compile-many "$OUTPUT/family-certified/controls/ts-rust/.build/zkir"
node "$FIELDS/check-preimages.mjs" "$OUTPUT/family-certified" "$ZKIR_WASM" "$OUTPUT/preimages"
```

The preimage checks cover zero, one, both sides of the 192-bit limb
boundary, and the canonical upper limits. They compare the shifted
FAB atoms with an independent byte calculation and check refusal
of changed ledger writes and return commitments.

Copy `consumer.Cargo.toml`, `consumer.rs` and `wrong-field.rs` to
`$OUTPUT/funded/consumer/Cargo.toml`, `main.rs` and `wrong-field.rs`
respectively. Seed the consumer from the workspace lock and build with
Rust 1.99 and the pinned offline dependencies. The `parity` mode takes
the evidence root and an unused output argument. It compares the
generated Rust result, FAB atoms, recorded effects and VM replay. The
`wrong-field` Cargo feature enables the compile-fail binary for the
P-256 scalar and K1 base inputs.

The three funded modes take an export name, the evidence root and a
fresh output directory. They require the matching tagged keys and
independently verified standalone proof under
`$OUTPUT/slow-proof/<export>/statement.{prover,verifier,proof}`.
Use verified Ledger 9 material for `MIDNIGHT_PP` and
`MIDNIGHT_LEDGER_TEST_STATIC_DIR`. The base and scalar roundtrips
use the canonical upper values. The reset uses its default input.
The consumer compares the TypeScript and Rust statement fields,
applies a separate funded transaction with default strictness, and
verifies unchanged state on replay. All three exports use the explicit
slow lane.

The [byte cast fixtures](casts/README.md) extend the same test command with
the upstream width refusals, conversions and guarded ledger cases.

The [arithmetic fixtures](arithmetic/README.md) cover the six original
P-256 addition, subtraction and multiplication circuits and the 34
original assertions.
