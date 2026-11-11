# Ledger 9 adoption on the Rust AST line

The local `rust-ast-backend-ledger9` branch starts at `09c75e289`, the published Rust AST line after PR #723. The Ledger 8 line and its qualification receipts remain available for historical compatibility and the pending tuple-spread application review.

This first slice pins the published [Ledger 9.1.0.1 package set](https://github.com/midnightntwrk/midnight-ledger/releases/tag/ledger-9.1.0.1) in the runtime and aligns the core consumers' direct Ledger package pins. The runtime uses the Ledger 9 transaction signature enum, accepts Ledger 9 observation metadata, and uses the ledger's global TTL in the existing Zswap test setup. Observed call preparation preserves the complete installed operation after checking the verifier. The direct fixture path still constructs a verifier-only operation. The proof consumer must check its resolver artifacts against the observed operation.

The runtime parity tests use fresh byte-for-byte TypeScript captures for Counter, Map, Set and List. The Ledger 8 JSON oracles remain intact. The captures use Compact 0.36.0, Compact Runtime 0.21.0 and Onchain Runtime 4.0.1, with the release's bundled ZKIR v3 compiler. The fixtures have no attached keys or IR in the installed operation.

| Fixture | Compact source | Revision |
| --- | --- | --- |
| Counter | `examples/rust_backend/counter.compact` | `09c75e289` |
| Set constructor | `examples/rust_backend/set_oracle.compact` | `09c75e289` |
| Map constructor | `examples/rust_backend/map_oracle.compact` | `09c75e289` |
| List constructor | `examples/rust_backend/list_oracle.compact` | `09c75e289` |
| Set transitions | `examples/rust_backend/set_boolean.compact` | `29ba39d0138ce66293c70d1d0fddb2ca2ed8e7011` |
| Map transitions | `examples/rust_backend/map_boolean_field.compact` | `bd2415e156e0f63189aebba8258aff43eeba6545` |
| List transitions | `examples/rust_backend/list_field.compact` | `09c75e289` |

The Set and Map transition sources retain the export sets of the existing Rust fixtures. Capture them with `compactc --skip-zk` in a directory that resolves the pinned npm packages. Run `runtime-rs/tests/fixtures/capture-ledger9-counter.mjs` or `capture-ledger9-collection.mjs` with the generated contract entry and matching Compact runtime entry. The collection script also takes `initial`, `set`, `map`, or `list`. The tests compare full serialized contract states.

The migration branch is an integration worktree. The workspace still has generated fixtures with direct Ledger 8 dependency pins. The Rust compiler compatibility record now selects Ledger 9. The Nix frontend/TypeScript distribution still needs its own migration. The existing proof runner still follows the v2 producer recipe, now pinned to the published ZKIR 2.2.0 package. Use `cargo test -p midnight-compact-runtime --all-features --all-targets --locked` for this runtime slice.

Port the remaining fixture and proof consumers and pass the required Linux, package and end-to-end gates before switching the distribution default.

The current Scheme frontend can emit Rust IR and ZKIR v3 in the same invocation with `--feature-zkir-v3`. The official Compact 0.36.0 reference emits a newer ZKIR format and has no Rust IR entry point. Check the exact current frontend artifacts against the selected official ZKIR tool before generating keys. Preserve the compiler's source/runtime pairing checks and give the Ledger 9 pair a distinct ABI identity when qualifying generated Rust.

Use the released Ledger 9 transaction and proof APIs as the reference. Keep the observed operation and its installed verifier bound to the resolver artifact. The funded lane needs the released `dust/10/spend` material, which differs from the retained Ledger 8 `dust/9/spend` material. The Dust proof and the Counter's ZKIR v3 proof use different verifier roles.

Local macOS arm64 validation with Rust 1.99 passed all 281 runtime tests and 1,874 backend library tests. Seven backend tests remained ignored. The testkit library also type-checked with its proof feature enabled. The same-source retained Rust AST frontend emitted private IR and ZKIR v3 for the Counter. The bundled official ZKIR tool modeled both exports at automatic k=6. This was a source and model probe, with the Ledger 9 compiler pairing and genuine proof/application still open.

## Local compiler/runtime pairing

The ordinary Rust target selects ZKIR v3 and emits ABI 54 against the exact Ledger 9 runtime dependency matrix. The runtime uses the published `midnight-ledger-v9` crate name, so the existing manifest validator can continue to reject Cargo package aliases and alternate sources. Runtime and generated consumers declare Rust 1.99 for this local qualification. Registry output and the legacy Ledger 8 functional CLI mode refuse on the migration branch. Historical generated fixtures still pin ABI 53 and need regeneration in the workspace migration.

The standard `compactc --target rust --rust-require-recording --rust-runtime-root … --skip-zk` invocation generated an unedited Counter crate and its ZKIR. The generated crate compiled with the transaction feature. Native execution, the typed observed call, recorded replay, and the independently captured Ledger 9 TypeScript oracle agreed on the selected transition and full state bytes. Native and recorded gas matched. The official ZKIR model tool accepted both retained artifacts at k=6. CLI preflight refused a Ledger 8 source root and preserved existing output. Local Rust 1.99 arm64 validation included 30 CLI tests, 21 compatibility tests, and 281 runtime tests.

## Funded Counter qualification

The first standalone proof passed, but the Ledger 9 transaction prover exhausted its skip entries before producing a contract proof. The previous ZKIR v3 emitter grouped multiple expanded ledger operations into one impact. Ledger 9 consumes a skip entry for each transcript operation. The updated emitter keeps those operation boundaries and omits suppressed operations. Counter's three impact instructions match the released Compact 0.36.0 output. The generated artifacts retain this branch's ZKIR 3.0 JSON format.

The rebuilt Scheme frontend generated an unedited Rust Counter crate and its exact ZKIR. The regenerated crate still passed native, recorded and observed execution parity with the independent TypeScript oracle. Released ZKIR 3.2.0 generated fresh k=6 keys and a standalone proof for `increment`. A separate process verified the proof, refused a different circuit's verifier, changed binding and commitment fields, and corrupted proof bytes, then verified the original again.

A separate local transaction used the same exact IR and installed verifier with a fresh transaction-bound v3 proof. It proved one Dust spend using the released Ledger 9 `dust/10/spend` material, passed default strict validation, and applied the expected generated Rust state transition. The installed operation stayed intact. Replaying the verified transaction returned `IntentAlreadyExists` with byte-identical whole-ledger state. This qualification used Rust 1.99 on macOS arm64, with locally funded test state.

Next, migrate the maintained proof gate and TypeScript/Nix package pairing, then regenerate the affected workspace consumers. Keep historical v2 proof/source bindings and Ledger 8 fixtures available on their original line. A Ledger 9 check must bind its own generated Rust, ZKIR v3, keys and released package graph.
