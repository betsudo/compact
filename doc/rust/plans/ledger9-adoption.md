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

The v3 emitter now bounds each active multiplication operand by the Jubjub
scalar order and converts it to the typed scalar consumed by ZKIR. An inactive
guard selects zero before the check. The explicit reduction builtin encodes its
result back into a Compact Field so it can still participate in Field arithmetic.
The released v3 tool models the unchanged DID `deactivate` source at k11 and
1,918 rows. The DID k11 recipe and the small-fixture limit stay in force.

Generated Rust retains canonical scalar rejection. The published Ledger 9
TypeScript provider reduces a raw scalar at or above the order. ZKIR refuses
the resulting noncanonical active call preimage. On the common canonical domain,
the generated Rust and TypeScript calls agree at zero and at the last canonical
scalar. The v3 checks also cover an inactive noncanonical operand and explicit
Field reduction.

Both original DID deactivation cases passed the maintained Ledger 9 proof
gate on macOS arm64 at `65568d356`. The cases retain the 200 and zero timestamp
witnesses and the complete original source, Rust IR and twelve checked programs.
Each case generated fresh k11 keys, a 4,208-byte standalone proof and a distinct
v3 proof in a funded transaction with one proved Dust spend. A separate process
verified the retained standalone proof and rejected corrupted bytes. The
changed binding and changed version input controls refused. Both transactions
applied with default strictness, the expected installed operation and state
were preserved, and exact replay returned `IntentAlreadyExists` with identical
serialized ledger bytes. The gate checks the modeled geometry and published
material before key generation. The workflow prepares k11 for this DID lane.

The full compiler suite passed 3,887 checks, including all 168 ZKIR v3
snapshots and the JavaScript suites in both ZKIR modes. All 284 Rust fixture
registrations were fresh. The locked workspace type-check passed with all
features and targets. The original DID native lifecycle and the 30-step
witnessed SDK gate also passed with the corrected packaged compiler.
Both Ledger 9 proof gates refuse a configured external proof server.

The reviewed original DID tests now use fresh Ledger 9 captures. The unchanged
capture producers ran with Compact Runtime 0.17.0 and Onchain Runtime 4.0.1.
The complete generated DID Rust library matched the checked-in library after
formatting. All 203 lifecycle steps and 24 composition rows retained their
case identities and changed only serialized contract-state fields. All 102
pure cases matched. The DID and reduced crypto composition suites passed all
48 tests on macOS arm64 with Rust 1.99, including full state and transcript
comparisons and the retained and current identity checks.

The [capture binding](../../../tests-rust-backend/did-adoption/oracle/ledger9/binding.json)
records the paired compiler, runtime package, generated JavaScript, input and
test hashes. The original capture files and review records retain their Ledger
8 identities. To reproduce the Ledger 9 observations, generate both targets
from the unchanged DID source with `--feature-zkir-v3 --rust-require-recording
--skip-zk` and the paired Rust runtime root. Generate the reduced
`unit_composition_crypto.compact` TypeScript target with
`--feature-zkir-v3 --skip-zk`. Run the original DID lifecycle, pure and
composition producers from a staged repository root, with the paired package
at `runtime` and both generated modules resolving that same package. Keep the
producers and case modules at their original repository-relative paths so the
capture can hash its inputs. Only the new Ledger 9 data enters these Rust
conformance tests. The existing Ledger 8 proof inputs stay bound to their
original evidence.

The original Passport fixture also has a
[Ledger 9 binding](../../../tests-rust-backend/vc-passport-adoption/oracle/ledger9/binding.json).
The paired compiler regenerated the complete Rust library and TypeScript
contract from the pinned 17-file source closure. The original TypeScript
producers recaptured all 202 named cases across 75 exports. The data matched
the historical branch captures exactly after comparing each profile label
separately. The Rust fixture tests check the new capture hashes and complete
generated library alongside the two historical TypeScript profiles. The
Passport fixture passed all 20 tests on macOS arm64. The original contract
exports only pure circuits.

The maintained standalone Passport SDK gate passed on macOS arm64 at
`8c69103b3`. It binds the published Ledger 9 registry lock identity and
the resolved Onchain Runtime 4.0.1 JavaScript and WASM. The raw generated
SDK, isolated adoption consumer and documented consumer passed strict
Clippy. The adoption and guide consumers passed 18 Rust methods in total.
All 202 live TypeScript cases matched both the historical upstream capture
and the reviewed Ledger 9 capture. Both correct-type twins compiled, both
typed misuse cases refused at the intended E0308 spans, and a changed
TypeScript expectation failed the actual Rust assertion before the restored
expectation passed. The gate retains the original pure source closure and
the guide's historical lock separately. The gate checks the native Onchain
Runtime, State and VM pins and the actual resolved packages in each consumer.
The separate formatting copy uses a retained configuration.

The original Map size `put` models at k7 and 119 rows with released ZKIR
3.2.0. Fresh Ledger 9 generation matched the complete formatted Rust, linked
IR and all eight checked programs. The maintained generated API gate passed
eight Rust consumer tests, 19 live TypeScript cases, two boundary cases and
four refusals. Both source refusals, the E0308 and E0382 compile failures,
and the changed-TypeScript-result control passed. Both replacement and
insertion passed fresh v3 proving at `fe9e8d22b`. Each case retained a
3,824-byte standalone proof and a distinct 3,824-byte transaction-bound
v3 proof. A separate verifier process accepted both standalone proofs and
refused corrupted bytes. The changed binding and same-geometry changed
result refused. Both transactions included one proved positive-fee Dust
spend, applied under default strictness, and refused exact replay with
identical complete ledger bytes. Mutated IR and checked-program bytes
refused before keys or state.
The local orchestration expects the v2 receipt, and the Linux workflow
is configured to retain the generated proof and replay evidence and
verification logs. The constructor and wide-zero qualification results
below record the migration of the earlier workflow steps.

The original literal Vector projection and the separate constant-valued
Vector index source each passed both branches of the `witnessed` circuit
with ZKIR 3.2.0 at k7 and 71 rows. The complete Rust, IR and five checked
programs matched for each source. Their generated API gates passed live
TypeScript parity, source refusals and Rust compile failures. Fresh v3
standalone proofs passed separate verification, and the changed binding,
input, output and corrupted-proof controls refused. All four distinct
transaction-bound v3 proofs funded and applied with default strictness,
then refused exact replay with identical ledger bytes. The two Map cases
also passed a fresh rerun through the shared v3 application code.

Source preflight now checks the reviewed Ledger 9 bindings for 36 Rust
test and support files and 16 active TypeScript captures alongside the
original review and capture records. The Rust edits add the optional v3
operation argument or select fresh Ledger 9 captures, with formatting
adjustments. The gate passed all 37 pinned sources. Workspace behavior
remains a separate qualification.

## Constructor Vector qualification

The maintained generated API gate passed all 132 commands on macOS arm64
with the original compiler recipe at `a0d81c197`. It kept all 14 sources,
the independent Scheme comparison, and the 24 Rust and TypeScript cases.
The case roster covers 17 successes and seven constructor errors. The gate
also exercised 68 post-initialization routes, 34 public replays, generated
SDK and consumer Clippy, source mutations, and malformed IR refusals.

Released ZKIR 3.2.0 modeled the 42 original exports in a separate probe.
The 14 reads were k6 with 34 rows. Each source's `advance` and
`advance_other` were k7 with 78 and 79 rows respectively.

Port the maintained model observer, proof preflight, and isolated proof
consumer together. Bind the exact source, raw IR and checked programs to
the v3 model and verify the released k7 parameters and Dust spend material
before key generation. Retain all 17 proof cases, the changed statement,
wrong verifier and corrupted proof controls, and separate verification.
Each successful case still needs a distinct funded transaction proof,
default strict application and byte-identical ledger state after replay
refusal. Constructor equality and wide-zero retain their own qualification
gates.

## Remaining workspace qualification

The maintained generated functional gate now selects Ledger 9 for all
23 source fixtures. Its 56 Rust methods and 160 four-route scenarios
passed, along with generated-consumer Clippy. Seven additional ADT
fixtures now compare full Ledger 9 state bytes against fresh TypeScript
captures. Their eight Rust tests passed, with historical observations
retained alongside the new captures.

The terminal default-feature arm64 workspace run recorded 3,198 passes,
234 failures and seven ignored tests before the ADT update. Of the
234 failures, 224 showed a Ledger 8 state serialization mismatch and
ten showed retained DID reducer provenance mismatches. Rustdoc also
failed on 289 targets in the reused build directory. One direct
reproduction traced the error to a proc-macro metadata hash mismatch,
and the single-package doc check passed with its separately compiled
runtime. Rerun the full workspace doc qualification in a fresh,
exclusive build directory.

The broader workspace behavior, remaining proof consumers and required
Linux integration lanes remain open.

Ten DID reducer fixtures now bind separate Ledger 9 TypeScript captures.
The exact compiler regenerated all ten Rust libraries byte-for-byte.
The 59 cases retained their reviewed order, inputs, outcomes, transcripts
and gas. State bytes and capture provenance were refreshed using Compact
Runtime 0.17.0 and Onchain Runtime 4.0.1. The original Ledger 8 captures
and review manifests retain their exact hashes. The 26 locked Rust
integration tests passed, including the ten provenance tests.

The maintained Constructor Vector model observer passed all 42 v3 models
and its membership refusal controls. The first exact conditional proof
probe refused a prepared operation that had lost its installed v3 IR.
The proof consumer now uses the observed-call path in both typed routes
and keeps the malformed replay refusal. Full Constructor Vector proof
qualification requires a fresh run of the corrected gate.

Both conditional cases passed a focused run with the observed-call repair.
Each standalone proof passed separate verification, including the wrong
verifier and corruption controls. A distinct v3 proof funded each transaction
with one positive-fee Dust spend. The transactions applied under default
strictness and exact replay was refused with identical ledger bytes.
The complete 17-case gate passed at `3da993bba`. It retained 28 fresh
key pairs for the 14 source variants and 17 standalone verifications,
including the wrong-verifier and corrupted-proof refusals. All 17
separate funded transactions passed default strictness and refused
exact replay with byte-identical ledger state.

The Constructor Equality port retains the three cohorts and both source
scopes. The released v3 tool modeled all 96 exports at k6 and 34 rows for
`read`, k7 and 78 rows for `advance`, and k7 and 79 rows for
`advance_other`. Independent TypeScript captures using the Ledger 9
runtime pair passed all 16 base, 30 boundary, and 105 tuple cases.
The 54-command tuple scope gate and the 19-command base API gate passed
at `11769f654`. The required proof callers preserve all selected counts
and verify each retained standalone proof separately. The base cohort
passed all 16 cases at `11769f654`, with 16 fresh key pairs and separate
verification of all 16 standalone proofs. It refused corrupted proofs
and distinct verifiers. Sixteen distinct funded transactions applied
with default strictness and refused exact replay with byte-identical
ledger state. The boundary cohort also passed all 30 cases at the same
commit, with 16 fresh key pairs across eight sources. All 30 separate
standalone verifications refused corrupted proofs and distinct verifiers.
Each distinct funded transaction passed default strictness and refused
exact replay with byte-identical ledger state. The tuple cohort passed
all 105 selected cases at the same commit, with 32 fresh key pairs
across 16 sources. Separate verification refused corrupted proofs and
distinct verifiers. The input, output, binding, and preimage controls
passed for each selected case. All 105 distinct funded v3 transactions
applied under default strictness and refused exact replay with
byte-identical ledger state.

Wide-zero multiplication passed the maintained generated API gate for the
original and conditional sources. The two ordinary entries model at k7
and 102 rows, and the two witnessed entries at k7 and 82 rows. The Ledger 9
proof caller verifies k7 material before fresh key generation for the two
originally selected ordinary entries. Both cases passed at `df2d240a1`
with separately verified 3,824-byte standalone proofs and distinct
3,824-byte transaction proofs. Each default-strict transaction included
one positive-fee Dust spend and refused replay without changing ledger
bytes.

Six maintained API gates passed at `c8576565b`. Source-map publication
ran 32 commands with the original 28 path cases and three private
manifest controls. Direct Fold ran 48 commands, Field return wrappers
14, tuple return wrappers 22, Unit initializers 30, and the nested
product request field gate 64. Their original behavior and refusal
rosters remain in force.

The ten Direct Fold sources regenerate exact bound IR, origins,
capabilities, and checked programs with Ledger 9. Their released v3
models use k6, k7, and k8. The proof migration verifies those parameters
before fresh key generation and retains all eleven executions.
The 64-command Direct Fold proof gate passed at `9a3518c68`.
It separately verified all eleven standalone proofs and rejected their
corrupted forms. Eleven distinct funded v3 transactions applied with
default strictness and refused replay with byte-identical ledger state.

At `119d114de`, the maintained Field wrapper, tuple wrapper, and Unit
initializer gates passed all two, four, and four selected cases on
macOS arm64. The gates verified k7 material before fresh key generation
and preserved the complete modeled export rosters. Separate processes
verified all ten standalone proofs and refused corruption. Each distinct
funded v3 transaction passed default strictness with one positive-fee
Dust spend and refused replay with byte-identical ledger state. The
Unit gate exercises later calls and state committed by initialization.

Twelve constructor fixtures now use fresh TypeScript state captures
produced with Compact Runtime 0.17.0 and Onchain Runtime 4.0.1. Only
serialized state fields changed in the captures, and the historical
Ledger 8 captures retain their original bytes. The Rust List fixture
now includes all five source exports in its installed operation map for
the exact state comparison. All 15 locked Rust integration tests across
the twelve packages passed on macOS arm64 at `ae48c02a6`.

The released v3 model probe covered all 70 stateful vector, nested
product, component-vector, and outer-vector exports. All 36 selected
exports fit the existing automatic k8 limit. The maintained product
proof callers still need migration and fresh qualification. Required
Linux integration and distribution qualification remain open.

The vector component gate now uses the released ZKIR v3 tool and binds
complete checked programs captured with the maintained repository-relative
source operand. The three selected raw IR files match the original bytes.
The corresponding programs carry Ledger 9.1.0.1 and preserve all other
content. The Ledger 8 program and model files remain available.

The maintained vector component API protocol passed all 170 commands on
macOS arm64 at `72aa8d16a`. The 24 complete sources produced 35 TypeScript
cases and 35 Rust tests, with 136 stateful routes, 58 public transcript
comparisons, 14 compile-fail controls, 13 malformed-IR controls, and
three typed source mutations. The complete proof gate retains three
selected k8 exports and separate standalone verification. The native
runner, complete model check, and pre-key refusal controls passed. The
corrected run reached material verification and refused a local cache
that lacked the k8 SRS. With verified upstream k8 material, all three
selected cases passed at `72aa8d16a`. Each standalone proof was
3,824 bytes and passed separate verification, including corruption and
wrong-verifier refusals. Each distinct funded v3 transaction applied with
default strictness and refused exact replay with byte-identical ledger
state.

The nested component and component Vector product callers now select
the released v3 tool. Their complete source models retain 14 and 16
exports respectively. The four and fourteen original proof selections
retain their complete checked-program bindings and typed negative
controls. The nested component gate passed at `5e99bec69` on macOS
arm64, including the 73-command API protocol and 16 pre-key byte and
membership refusals. It generated four fresh key pairs at k7 and k8.
Each standalone proof passed separate verification and refused
corruption. The four distinct funded v3 transactions applied with
default strictness and refused exact replay with byte-identical ledger
state.

The component Vector API protocol passed all 88 commands at
`d16987d11`. The proof gate stopped before key generation because the
original 14-case selection omitted the family `branch`, which now
fits k8 at 213 rows. The corrected selection covers all 15 in-cap
exports and includes the existing typed `branch_left` case. The
original direct export remains at k9. At `15bd923c2`, all 15
selected proofs passed on macOS arm64. The gate checked all 21 pre-key
refusals and separately verified each fresh standalone proof,
including the corruption refusal. Each distinct funded v3
transaction applied with default strictness and refused exact replay
with byte-identical ledger state.

The outer Vector port also includes `branch_left` and keeps both
branches in the parity tests. All 16 exports now fit k8, including
`family/branch` at k8 and 209 rows. The gate generates the unchanged
vector component `helper/run` with the same admitted compiler into a
private fixture, checks its k9 and 288 rows, and uses it for the
over-cap refusal before key generation. The API protocol passed all 91
commands at `15bd923c2`. The proof gate stopped before key generation
when the model command wrote its binary IR alongside the intact donor.
The corrected gate authenticates that binary and binds the original raw
IR in the refusal. The local retry at `221d9a690` passed all 91 API
commands and 22 pre-key refusals. It generated fresh keys for all 16
exports on macOS arm64. A separate invocation verified every standalone
v3 proof and refused corruption. All 16 independently proved funded
transactions applied with default strictness and refused exact replay
with byte-identical ledger state. The artifact readback checked the
standalone and transaction proof bytes, the per-call event records, and
the unchanged raw donor.

The workspace rustdoc rerun passed at `15bd923c2` with Rust 1.99,
the locked Ledger 9 graph, and a fresh exclusive target directory.
Rustdoc completed all 303 targets and ran seven passing doctests.

Eight expression and assertion fixtures now use fresh paired
TypeScript captures at `093467ed0`. Their 82 serialized contract-state
fields use Ledger 9. The three wide-add diagnostic locations track the
current source lines and preserve the error text and values. Historical
captures retain their original bytes. All 14 focused Rust tests passed
with all features on macOS arm64. At this head, the behavioral
workspace run completed 733 targets with 3,244 passes, 182 failures,
and seven ignores. The 99 failing targets all appear in the earlier
failure roster. All 182 failure messages show a v6/v8 contract-state
encoding mismatch. Further assertions can run after those comparisons
are repaired.

Another 31 fixtures now bind fresh captures from their original
TypeScript producers at `061640db6`. These captures refresh 177
serialized contract-state fields. Witness effect order also records
the current compiler, Scheme, and runtime provenance. The canonical
output capture preserves spend-set membership and the exact ordered
query trace. Its Rust test keeps the established set comparison and
all execution-order checks. The all-features native run completed
66 targets with 88 passes and one failure. The failed test also read a
separate Ledger 8 capture for the untaken impure call argument.
The original TypeScript producer supplied fresh state and current tool
provenance for that case at `3af6ccba3`. Its full public program,
ordered query trace, and other scenario values agree with the
historical capture. All 12 package tests passed on the focused rerun.

Eleven additional scenario captures at `6e324f9f9` refresh 124
serialized state fields. The native tests completed 13 targets with 18
passes and two failures against a separate historical Zerocash
constructor, mint, and spend capture. At `4991d2a73`, the fresh
Zerocash capture reproduces the historical leaf, all 32 siblings,
and their directions. The three DID relation reducers preserve all
22 scenarios, including their exact ordered programs, query costs,
and refusal cases. All nine focused tests across the four packages
passed with Rust 1.99 on macOS arm64.

At `514df1d5e`, Coracle withdraw preserves exact spend and nullifier
membership under the existing set comparison. Micro DAO vote commitment
preserves exact commitment-to-index pairs and ordered offer outputs.
For Coracle guess, the fixed per-query budget that succeeded on Ledger 8
now fails on Ledger 9 after six successful TypeScript queries. Native
and recorded Rust both report the corresponding out-of-gas error and
match the captured witness order. The zero-budget refusal and the
unlimited positive paths also pass. All three focused tests passed.

At `d5747ec1f`, five Election and seven conditional captures refresh
61 state fields and preserve their non-state behavior. The original
conditional producer also records current tool provenance for the
opposite-state case. The focused run passed 16 tests and reported five
remaining v6/v8 comparisons. Those tests still reference the distinct
Election owner-operation, base ternary, conditional Counter, and
shared direct behavior fixtures. At `7fc3f055a`, all four captures
refresh 42 state fields. The Election owner scenario reproduces both
ten-level Merkle paths from actual execution, including the leaves,
siblings, and directions. For the shared direct capture, the staged
producer changes only the required runtime version from 0.16.101 to
0.17.0. Its reviewed original bytes and case inventory are retained.
Both the full Election and ternary package tests pass. All three Rust
consumers pass the shared direct tests.

At `cea38ce25`, 14 Registry, assertion, literal, and pure-call captures
refresh 93 state fields. The shared Call Registry producer uses the same
version-only staging adjustment as the direct behavior producer.
All 43 focused tests across the five affected packages pass, including
the three older state comparisons exposed by the shared direct run.
A new workspace run is executing on this signed head with the same
locked offline behavioral scope as the earlier 733-target run.

At `f6ae8e51c`, 16 collection, conditional, Merkle, and shielded
effect captures refresh 111 state fields. Their captured programs,
effects, and other scenario values agree with their historical
counterparts. The focused tests across 13 packages follow the workspace
run on the same target directory. Workspace and integration
qualification remain open.
