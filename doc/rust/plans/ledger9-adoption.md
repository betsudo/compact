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

The migration branch is an integration worktree. The active Rust workspace resolves the published Ledger 9 package graph from crates.io. The existing proof runner still follows the v2 producer recipe, now pinned to the published ZKIR 2.2.0 package. Use `cargo test -p midnight-compact-runtime --all-features --all-targets --locked` for the Rust runtime.

Port the remaining fixture and proof consumers and pass the required Linux, package and end-to-end gates before switching the distribution default.

The current Scheme frontend can emit Rust IR and ZKIR v3 in the same invocation with `--feature-zkir-v3`. The official Compact 0.36.0 reference emits a newer ZKIR format and has no Rust IR entry point. Check the exact current frontend artifacts against the selected official ZKIR tool before generating keys. Preserve the compiler's source/runtime pairing checks and give the Ledger 9 pair a distinct ABI identity when qualifying generated Rust.

Use the released Ledger 9 transaction and proof APIs as the reference. Keep the observed operation and its installed verifier bound to the resolver artifact. The funded lane needs the released `dust/10/spend` material, which differs from the retained Ledger 8 `dust/9/spend` material. The Dust proof and the Counter's ZKIR v3 proof use different verifier roles.

Local macOS arm64 validation with Rust 1.99 passed all 281 runtime tests and 1,874 backend library tests. Seven backend tests remained ignored. The testkit library also type-checked with its proof feature enabled. The same-source retained Rust AST frontend emitted private IR and ZKIR v3 for the Counter. The bundled official ZKIR tool modeled both exports at automatic k=6. This was a source and model probe, with the Ledger 9 compiler pairing and genuine proof/application still open.

## Local compiler/runtime pairing

The ordinary Rust target selects ZKIR v3 and emits ABI 54 against the exact Ledger 9 runtime dependency matrix. The runtime uses the published `midnight-ledger-v9` crate name, so the existing manifest validator can continue to reject Cargo package aliases and alternate sources. Runtime and generated consumers declare Rust 1.99 for this local qualification. The legacy Ledger 8 functional CLI mode refuses on the migration branch. The historical fixtures retain their Ledger 8 ABI on the Ledger 8 line.

The standard `compactc --target rust --rust-require-recording --rust-runtime-root … --skip-zk` invocation generated an unedited Counter crate and its ZKIR. The generated crate compiled with the transaction feature. Native execution, the typed observed call, recorded replay, and the independently captured Ledger 9 TypeScript oracle agreed on the selected transition and full state bytes. Native and recorded gas matched. The official ZKIR model tool accepted both retained artifacts at k=6. CLI preflight refused a Ledger 8 source root and preserved existing output. Local Rust 1.99 arm64 validation included 30 CLI tests, 21 compatibility tests, and 281 runtime tests.

## Funded Counter qualification

The first standalone proof passed, but the Ledger 9 transaction prover exhausted its skip entries before producing a contract proof. The previous ZKIR v3 emitter grouped multiple expanded ledger operations into one impact. Ledger 9 consumes a skip entry for each transcript operation. The updated emitter keeps those operation boundaries and omits suppressed operations. Counter's three impact instructions match the released Compact 0.36.0 output. The generated artifacts retain this branch's ZKIR 3.0 JSON format.

The rebuilt Scheme frontend generated an unedited Rust Counter crate and its exact ZKIR. The regenerated crate still passed native, recorded and observed execution parity with the independent TypeScript oracle. Released ZKIR 3.2.0 generated fresh k=6 keys and a standalone proof for `increment`. A separate process verified the proof, refused a different circuit's verifier, changed binding and commitment fields, and corrupted proof bytes, then verified the original again.

A separate local transaction used the same exact IR and installed verifier with a fresh transaction-bound v3 proof. It proved one Dust spend using the released Ledger 9 `dust/10/spend` material, passed default strict validation, and applied the expected generated Rust state transition. The installed operation stayed intact. Replaying the verified transaction returned `IntentAlreadyExists` with byte-identical whole-ledger state. This qualification used Rust 1.99 on macOS arm64, with locally funded test state.

Next, migrate the maintained proof gate and TypeScript/Nix package pairing, then regenerate the affected workspace consumers. Keep historical v2 proof/source bindings and Ledger 8 fixtures available on their original line. A Ledger 9 check must bind its own generated Rust, ZKIR v3, keys and released package graph.

## Rust 1.99 toolchain

The workspace and Nix-packaged Rust CLI select Rust 1.99.0. Nix checks the official release manifest hash and uses the matching Cargo, compiler and standard library. Linux uses the matching musl standard library and keeps the static linker and ELF checks. The standalone compiler lock is selected from the workspace lock and its registry package identities, checksums and retained edges are audited before a locked build.

## TypeScript package pairing

This branch's synchronous Compact runtime uses minor `0.17.0` and the published `@midnightntwrk/onchain-runtime-v4@4.0.1`. The Nix package consumes the registry artifact from the npm lockfile. Compiler output checks the runtime minor at import. Both cross-version controls refused when swapping generated code and runtime between `0.16.102` and `0.17.0`. The official asynchronous Compact Runtime 0.21.0 remains the independent oracle.

Build `.#compactc` and `.#runtime.package`. Generate Counter with `--target ts --target rust --rust-require-recording --skip-zk`. With the packed runtime and its locked dependencies available to Node, run `runtime-rs/tests/fixtures/capture-ledger9-paired-counter.mjs` on the generated `contract/index.js`. The capture checks the shared Ledger 9 module identity and exact before/after contract bytes against the official oracle. The generated Rust consumer matched those bytes, all four gas fields and the typed return. Its two ZKIR files were byte-identical to the qualified standalone and funded proof inputs.

The native macOS arm64 Nix runtime and compiler packages built, the runtime tests passed, and the compiler suite passed in both existing JavaScript check modes.

## Released ZKIR tool pairing

The packaged native v3 compiler builds the published `midnight-zkir` 3.2.0 crate with a standalone, registry-only Ledger 9 lock on Rust 1.99.0. The JavaScript check harness installs the published ZKIR v3 3.2.0 and ZKIR v2 2.2.0 WASM packages from its lockfile. The existing v2 native producer and historical receipts remain available. Rust and both-target invocations report the v3 package version.

The v3 emitter now uses the released point and scalar conversion instructions. It lowers each hash to a typed Bytes32 result and splits the digest back into Compact's high-byte-first field representation. This keeps the typed Compact value, its hash alignment, and the public transcript ordering consistent. The v2 emitter retains its original snapshots and check mode.

On macOS arm64, the compiler suite passed all 3,883 checks, including all 168 v3 snapshots and the JavaScript checks in both ZKIR modes. The portable compiler reproduced the exact Counter and guarded Counter ZKIR and Rust libraries used in proof qualification. The packaged native CLI reproduced the straight-line Counter's binary IR, proving key and verifier key byte-for-byte. The published v3 WASM package agreed with the native binary IR for both the straight-line and guarded Counter. Both guarded Counter branches passed with a fresh transaction-bound v3 proof and real Dust funding. Each had 20 active transcript operations, one proved no-op, matching generated TypeScript state and typed return, and byte-identical ledger state after exact replay refusal. The reset branch starts and ends with the same contract value while the ledger records fees and replay protection. Both Linux musl tool derivations were evaluated locally.

The maintained v3 proof gate still needs to bind its producer, exact generated artifacts, separate verifier and transaction application. Current fixture freshness and the accepted functional SDK routes still need regeneration and qualification on Ledger 9.

## Earlier Compact Ledger 9 branch

The local `ledger9-rustbackend` ref at `6f4e5948f` records the Rust backend port and constructor rejection corpus. The retained `origin/ledger9-rustbackend` ref at `fc879310b` also records the TypeScript fixture update for the options-object `createCircuitContext` API and an independent Rust toolchain pin. Its runtime still uses the Ledger 9 RC3 Git graph and Rust 1.95.0. Use its negative cases and fixture recipes as references when auditing current gates. Keep this migration on the Rust AST line with released packages and Rust 1.99.0.

## Nested Jubjub ledger values

The read-only fixture audit found five v3 compiler crashes in DID, its digest reducer, a Jubjub Cell, a nested enum and a nested point map. This branch adapts the physical ledger encoding from the earlier Compact fix at `ee345ee15` to the current ZKIR v3 emitter. It writes each Jubjub point as two field coordinates and binds each public read as a typed point. The default path encodes the identity point as `(0, 1)`. The regression runs the generated TypeScript through round trip, both guarded read branches and reset with the v3 preimage checker. The v2 checker covers round trip and guarded reads. A separate probe reproduced the existing v2 default-reset transcript mismatch.

Mixed native values with a zero-width unsigned field require a separate layout fix. The current compiler rejects the ambiguous alignment at the source. Generic native-type hash inputs and Merkle path recomputation also remain separate from the repaired ledger operand and leaf encoding.

## Ordinary Rust fixtures

The first regeneration covers Counter, Boolean Cell, Jubjub Cell, Map and List. The current compiler emits ABI 54 in each library. The isolated macOS arm64 workspace used the published Ledger 9 graph and passed all 20 existing tests with all features and targets. The compiler regenerated these libraries from their Compact sources with Rust 1.99 formatting. The fixture test sources and coordinate oracle remain the same.

All 255 ordinary fixture libraries have been regenerated. The 202 fixture manifests with direct Ledger 8 Git dev dependencies now use the runtime's exact published crate pins. The resolved macOS arm64 workspace contains one `midnight-ledger-v9` 1.0.1, Onchain State/Runtime/VM 4.0.1, Storage 3.0.0, Serialize 1.2.1 and Zswap 9.0.1. Both published ZKIR majors remain available for their distinct check modes. The registry records the 66 changed manifests used by generated-consumer build bindings. The Ledger 8 transition tests reconstruct and verify their historical manifest bytes against the recorded hashes.

## Functional Ledger 9 entry

`compactc --target rust --rust-functional-ledger9` emits source-bound checked programs labelled `9.1.0.1` and enables the generated crate's functional runtime feature. The emitted compatibility record and ABI 54 identify the paired source runtime. The experimental typed adapter API retains its existing names. The Ledger 8 CLI flag still refuses before publication.

The Nix-built Rust CLI paired with the same Scheme frontend reproduced the generated Counter Rust library, checked programs, TypeScript and ZKIR byte-for-byte. Its functional native and recorded results matched ordinary Rust state and gas. Preparing the functional trace against the observed contract retained the same installed operation and produced the same call prototype as the ordinary recording. The generated TypeScript matched the independent Ledger 9 oracle and the applied contract's serialized state.

On macOS arm64, the functional recording generated a fresh ZKIR v3 proof and passed verification in a separate process, including the wrong-key, wrong-binding, wrong-commitment and corrupted-proof controls. A separately proved transaction spent one Dust input, applied under default strict validation, and refused replay with `IntentAlreadyExists` and byte-identical ledger state. This local gate covers Counter. The remaining maintained proof consumers and required Linux integration gates still need migration and qualification.

The source registry now uses the Ledger 9 flag for its 29 functional fixtures. Their complete checked-program sidecars and formatted Rust libraries were regenerated from the registered sources. The fixture producer completed all 284 ordinary and functional registrations with no generation failures.

The CI static-fixture resolver now authenticates the Ledger 9 registry archive against its pinned Cargo checksum and compares the extracted manifest, Dust IR, verifier and digest declarations against that archive. The real resolver passed against the locked macOS arm64 workspace metadata. The separately distributed Dust prover stays subject to the upstream digest check.

## Workspace checks and parity captures

On macOS arm64, Rust 1.99 type-checked the locked workspace with all features and targets. Ledger 9 fixture setup uses the two-argument contract operation constructor, the transaction signature enum, and the upstream global TTL for Zswap. The synthetic v2 IR uses the published v2 minor-version type. The packaged compiler's fixture freshness check passed all 284 registrations.

Vector and tuple cells, effectful returns, shielded merge, and transient receive and send use fresh Ledger 9 TypeScript captures generated by the paired compiler with `--target ts --feature-zkir-v3 --skip-zk` and Compact Runtime 0.17.0 with Onchain Runtime 4.0.1. The existing capture scripts produce the full state and applicable transcript and gas records. The Ledger 8 capture files are retained. The focused arm64 run passed 44 tests across these four fixtures, the historical ABI boundary control, and the testkit.

The bounded Uint and Bug 11 cases also pass against freshly generated Ledger 9
TypeScript captures. Both keep their exact state, gas, transcript and replay
checks. The initial workspace run, built before these capture updates, reported
3,109 passes, 322 failures and seven ignored tests. The failures comprise 309
state-byte or state-tag mismatches, 12 reviewed-capture provenance mismatches,
and one witness Cell gas-limit assertion. Each current capture still needs
independent regeneration and the gas-limit case needs a separate diagnosis.

## Maintained witnessed SDK proof

The witnessed Field Cell consumer uses the published ZKIR v3 compiler and
the Ledger 9 package graph. The selected circuit requires k7 and 68 rows.
The proof material resolver verifies the published Dust and Zswap files using
the upstream decoder for their tagged v2 IR. The v3 contract IR and the v2
built-in IR use their respective published decoders.

A local macOS arm64 probe produced a fresh 3,824-byte standalone v3 proof and
verified it in a separate process. Its independently proved transaction used
one Dust spend, passed default-strict validation, applied the expected Field
Cell value 42, and refused replay with byte-identical full ledger state. The
probe also passed the preparation, input, output, binding, proof-corruption,
and private witness refusal controls. The complete isolated source-SDK gate subsequently passed all 30 steps on
macOS arm64 at `4028a7b4b`. It used the original packaged compiler with no
runtime or Scheme override, ran three positive ContractLab tests and the three
typed compile-fail controls, and linted the consumer, bundled SDK and copied
testkit. It generated fresh k7 keys and proved, separately verified, and
strictly applied the selected call with a distinct funded transaction proof.
The remaining required proof workflows and native Linux checks are open.

## Expanded TypeScript parity cohort

The next 18 fixture packages compare fresh Ledger 9 TypeScript captures for
collections, hashes, arithmetic, control flow and witnessed state transitions.
The Map, nested Map, Set and witness recording cases use a separate nine-case
capture. The original nine-case capture and capture script remain bound to
their reviewed Ledger 8 versions. The witnessed Cell state comparison includes
the generated `write_offset` entry. Its gas-limit test uses a zero minimum
deletion credit, as required by the Ledger 9 cost-bound comparison, and retains
both the aggregate-meter assertion and the insufficient-read-budget refusal.

The locked macOS arm64 retest passed all 57 tests across these 18 packages,
with all features and targets enabled. The broader workspace behavioral suite
still needs fresh captures and the remaining provenance and proof migrations.

## Original DID lifecycle

The original two-file DID source closure regenerated byte-identical raw Rust IR,
source origins, capabilities, all twelve checked programs and the formatted
Rust library. The fresh Ledger 9 TypeScript capture retains all twelve DID
cases. Compared with the historical capture, the changed fields are serialized
contract states. The seven Rust lifecycle tests pass across the ordinary and
functional routes. The original Ledger 8 capture remains available.

The complete native lifecycle gate passed on macOS arm64 at `c2c73023f`.
It checked all twelve live DID observations, three typed misuse controls and
Clippy. The two positive cases ran across four Rust routes, and the ten failure
cases exercised all four routes.

The released native ZKIR v3 tool rejects this source's `deactivate` circuit
because `EcMulGenerator` receives a native scalar. A model-only probe with
explicit conversion at both multiplication instructions reached k11 and 1,871
rows. The next compiler change must preserve canonical scalar rejection.
`jubjub_scalar_from_native` reduces modulo the Jubjub order, while Compact's
multiplication builtins reject an out-of-range scalar. No DID keys or proofs
were produced during the Ledger 9 lifecycle qualification.
