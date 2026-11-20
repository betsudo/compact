# Ledger 9 secp256k1 known-answer suites

The `ecdsa_verify`, `bitcoin_kat`, and `ethereum_address` siblings
preserve the original sources and compile and runtime declarations from
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The original [support files and vectors](../../../support/crypto/README.md)
include the Wycheproof and Ethereum data and their provenance.

The first command compiles the original contracts for both targets and
runs the original TypeScript callbacks. The second command uses Rust
1.99 and the workspace's locked dependencies to run the same inputs
through the generated Rust APIs.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT" --upstream-kat &&
  python3 tools/compact-rust-backend/check_upstream_ledger9_secp256k1_rust.py "$OUTPUT"
```

Run in the development shell with a built TypeScript runtime and its
pinned test dependencies, an available offline Cargo cache, and a fresh
output path. The upstream corpus classifies all 463 Bitcoin cases. It
drives 244 cases per suite, excludes 217 encoding cases and excludes two
malleability cases. The adapter checks the case partition by ID. Rust
checks canonical scalar admission, the exact circuit refusal categories,
all ten message widths including empty bytes, and recovery for all 162
valid signatures. The Ethereum suite checks two published addresses.

The current compiler reports language 0.23.105. For a diagnostic run,
substitute `--allow-pragma-probe` and use a fresh output path. The runner
checks each intact fixture's version diagnostic and confirms that the
compiler publishes no output on refusal. It then removes exactly the
0.25.0 pragma line in separate copies and executes the original
assertions against both generated targets. The intact-source gate
remains open. These pure circuits generate no proof statements.
