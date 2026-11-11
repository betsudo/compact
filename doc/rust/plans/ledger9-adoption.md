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

The migration branch is an integration worktree. The workspace still has generated fixtures with direct Ledger 8 dependency pins. The compiler compatibility record and Nix compiler/TypeScript pairing still describe Ledger 8. The existing proof runner still follows the v2 producer recipe, now pinned to the published ZKIR 2.2.0 package. Use `cargo test -p midnight-compact-runtime --all-features --all-targets --locked` for this runtime slice.

Next, qualify one current Rust AST-generated Counter with the paired Ledger 9 compiler and runtime. Bind the exact ZKIR v3 artifact, prover and installed verifier, then independently verify a fresh proof. Separately prove and apply a funded transaction under default strict validation and check replay leaves the complete ledger unchanged. Port the remaining fixture and proof consumers and pass the required Linux, package and end-to-end gates before switching the default.

The current Scheme frontend can emit Rust IR and ZKIR v3 in the same invocation with `--feature-zkir-v3`. The official Compact 0.36.0 reference emits a newer ZKIR format and has no Rust IR entry point. Check the exact current frontend artifacts against the selected official ZKIR tool before generating keys. Preserve the compiler's source/runtime pairing checks and give the Ledger 9 pair a distinct ABI identity when qualifying generated Rust.

Use the released Ledger 9 transaction and proof APIs as the reference. Keep the observed operation and its installed verifier bound to the resolver artifact. The funded lane needs the released `dust/10/spend` material, which differs from the retained Ledger 8 `dust/9/spend` material. The Dust proof and the Counter's ZKIR v3 proof use different verifier roles.

Local macOS arm64 validation with Rust 1.99 passed all 281 runtime tests and 1,874 backend library tests. Seven backend tests remained ignored. The testkit library also type-checked with its proof feature enabled. The same-source retained Rust AST frontend emitted private IR and ZKIR v3 for the Counter. The bundled official ZKIR tool modeled both exports at automatic k=6. This was a source and model probe, with the Ledger 9 compiler pairing and genuine proof/application still open.
