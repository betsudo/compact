# Select versions and an execution target

The local Ledger 9 migration pairs compiler **0.31.213**, Compact language
**0.23.105**, TypeScript runtime **0.18.0** and
`@midnightntwrk/onchain-runtime-v4@4.0.1`. The Rust backend, runtime and
macros source packages are **0.2.0**. The generated crates require Rust
**1.99**, with **1.99.0** pinned for the local checks. The native graph
uses the published `midnight-ledger-v9@1.0.1` crate and the exact
dependencies in the [runtime compatibility record](../../../runtime-rs/compatibility.json).
The ordinary Rust target emits ZKIR v3. Local proof qualification used
the ZKIR v3 **3.2.0** CLI.

See the [Ledger 9 adoption plan](../plans/ledger9-adoption.md) for the
macOS arm64 checks, archive consumer rehearsal and remaining integration
gates. The Ledger 8 qualification and milestone 0.3.0 receipts below
retain their original source and toolchain identities.

## Keep these version domains separate

| Boundary | Retained 0.3.0 value | Meaning |
|---|---|---|
| Initiative milestone/artifact label | 0.3.0 / `rust-backend-v0.3.0` | Engineering milestone; no tag or published package is implied here |
| Compact compiler | 0.31.133 | Frontend/toolchain version |
| Compact language | 0.23.105 | Source language version |
| TypeScript runtime | 0.16.101 | `compactc --runtime-version` refers to this runtime |
| Backend / Rust runtime / macros | 0.2.0 | Exact source package pairing; registry availability is not asserted |
| ContractLab / default generated application | 0.1.0 | Separate unpublished package identities |
| Runtime ABI | 50 | Generated-source/runtime compatibility marker, not stable Rust binary ABI |
| Private Rust IR | 20 | Internal frontend/backend data format; historical “schema6” was our earlier private format |
| Capability / compatibility schema | 3 / 1 | Report formats |
| Native ledger | 8.0.3 | Pinned ledger graph |
| Declared native MSRV | Rust1.88 | Qualified only with the documented lock/features/host |

Ledger 9 generated Rust asserts `runtime::RUST_RUNTIME_ABI == 54`.
The assertion detects a runtime mismatch at Rust compile time. The
compatibility schema is 2, private Rust IR is 20, finalized capability
schema is 3 and private origin transport is 3.

### Before publishing the Rust crate cohort

The Ledger 9 source packages still carry version 0.2.0. Use the
compatibility record and package provenance when selecting the source
cohort. A future registry release needs its own version and consumer-lock
coordination. The local archive rehearsal verifies the packaged runtime
and macro sources and executes an offline six-contract Rust consumer.

## Obtain a matching source distribution

Use the Rust-capable compiler artifact built from this branch and its matching Scheme component, runtime/macros sources and compatibility record. This guide does not assert a public download URL, registry package or release tag. If working from source, use the branch's retained build/distribution workflow and record the actual compiler identity before accepting example results.

An installed compiler bundle supplies its Scheme component through the bundle layout. A development wrapper needs `COMPACTC_SCHEME` set to its matching executable. The examples require `compactc` on PATH. `--skip-zk` skips key generation, not contract code generation.

Ledger 9 Rust generation supports two source-selection modes:

| Mode | Behavior |
|---|---|
| Default Rust generation | Validates and bundles matching runtime/macros sources in the generated output |
| `--rust-runtime-root /absolute/source/root` | Uses a validated shared source pair; the root contains `runtime-rs` and `runtime-rs-macros` |

`COMPACT_RUST_RUNTIME_DIR` is also an explicit root selection. Missing or inconsistent compatibility records are rejected rather than silently falling back. Use the matching source pair. For archive consumers, follow the
[offline archive rehearsal](../../../tools/compact-rust-backend/README.md#offline-archive-consumer).
It generates against a verified shared source root before resolving the
exact runtime and macro archives. The Ledger 9 compiler rejects the
historical `--rust-runtime-registry` flag.

For ContractLab, use one runtime identity in Cargo: generated contract and testkit must resolve the same source package. The validated tutorials stage runtime/macros/testkit together in `examples/sdk/` and generate with that absolute root. `prepare-sdk.py` was executed in ADR0324's isolated external validation; it preserves source manifests and copies the testkit's five declared fixture path dependencies. See `VALIDATION.md` for the source checkpoint, qualified locks and three passing tutorial tests. No registry publication is required.

## Retained Ledger 8 target evidence

| Target | Retained evidence | Limits |
|---|---|---|
| Native macOS ARM64 | Candidate2c798d2d: external W2 ContractLab executes on Rust1.99 and1.88; all-feature testkit including optional ProofLab checks on1.88; relocated portable TS/Rust generation and default/transaction Cargo checks on1.99 | Warm caches; manually assembled debug compiler archive; no arbitrary future-lock or every-feature guarantee |
| Native Linux | Candidate57fab775 CI37748332319: bounded core tests/format/Clippy pass; selected Rust1.88 backend, Jubjub all-feature generated fixture and isolated backend compilation pass | No Linux external ContractLab execution or whole-workspace MSRV claim; full local qualification remains separate |
| Other native hosts | Earlier milestone history only | No new candidate qualification claimed |
| Node24.14 and Chromium151 via `wasm32-unknown-unknown` | ADR0256 ran four bounded generated examples with default runtime, Rust1.99/wasm-bindgen0.2.129/wasm-pack0.13.1 | Prototype only; no `ledger-transaction`, browser prover, live wallet or all-contract support claim |
| DID ledger8.1 public transaction interchange | Separate finite JS roundtrip/malformed-refusal receipts | Does not change native ledger8.0.3 or establish a continuous8.1 chain |

The tutorial commands target native Rust. They are not Node/browser build instructions. Retained WASM receipts include exact wrapper/build tooling and host Clang requirements.

## Migrate consumers

1. Preserve old generated code with its original matching runtime, or regenerate source with the current matching ABI54 compiler/runtime/macros pair. Do not edit the ABI assertion.
2. Prefer the generated crate's `runtime` re-export for value types, avoiding duplicate crate identities.
3. Backend0.2 exposes non-exhaustive `RenderError`. External matches need a fallback; resource diagnostics expose `{resource, limit, observed}`. Display text/resource labels are human diagnostics, not a stable machine protocol.
4. Keep the qualified Cargo.lock. A successful build of one workspace lock does not qualify a different standalone or target-specific resolution.
5. Positional generated signatures remain current; named Args/facade probes are research dispositions, not a required API migration.

The ADR0293/0304 migration receipts already contain old ABI49 refusal, working ABI50 controls, non-exhaustive compile failures, migrated callers and standalone Rust1.88 lock qualification. Those are historical qualified executions, not new runs of these drafts.

## Retained 0.3.0 compatibility checkpoint

The historical policy was accepted at `5efa91c2`; the accompanying `feature-source-policy.json` retains the Ledger 8.1.3 source pairing. The retained MSRV receipt covers actual Rust1.88 workspace backend/Jubjub all-feature and isolated standalone compilation at49898aed; the only later source changes before5efa91c2 are two test files. The accepted feature profiles are default native, exact-evidence `ledger-transaction`, source-only ContractLab, experimental bounded WASM and registry **manifest emission only**. Default native includes crypto/proof dependencies.

DID's original compiler profile (ledger8.0.2/compiler0.31.1/runtime0.16.0), its npm ledger-v8@8.1.0 wire profile and the isolated coherent Rust8.1 receiver are separate from native8.0.3. No native promotion follows from those receipts. See [candidate qualification](candidate-qualification.md) for current source, validation and documentation dispositions. The resumed ADR0285 consumer-safety controls passed at4c8aebc3. Independent audits and final-candidate checks retain their separate dispositions.

## Candidate consumer follow-up

At2c798d2d, the current-side API migration runner accepts a migrated caller and rejects a match enumerating every known RenderError variant without a wildcard (E0004). W2 is freshly generated in an external SDK copy and passes its witnessed commit/exhaustion checkpoint test on both Rust1.99 and1.88. Candidate lock source/version/checksum identity and one runtime package are checked. These are warm-cache bounded receipts, distinct from the original ADR0324 tutorial cohort. [Candidate consumers coverage and ADR0362 — 2026-10-08](../evidence/0.3.0/candidate-consumers-adr0362/review-note.md) retains commands, hashes and limits. Successor57fab775 changes only qualification harnesses and metadata; its source-equality receipt preserves the original consumer identities. [Bounded candidate CI — 57fab775](../evidence/0.3.0/candidate-ci-57fab775/review-note.md) records that earlier remote success. See [candidate qualification](candidate-qualification.md) for the current overall status; the receipts above retain their original checkpoints.

## Source build instructions

Use the existing [portable compiler archive workflow](../../../tools/compact-rust-backend/README.md#portable-compiler-archives). This links the branch workflow; no published release URL is implied.

## Later candidate receipts —57fab775

The [bounded CI receipt](../evidence/0.3.0/candidate-ci-57fab775/review-note.md) records exact57fab775 core checks and the selected LinuxMSRV scope. The [source-continuity record](../evidence/0.3.0/candidate-baselines-adr0363-0364/candidate57fab/candidate-continuity.json) and [baseline maintenance receipt](../evidence/0.3.0/candidate-baselines-adr0363-0364/review-note.md) distinguish later harness/metadata changes from earlier compiler/runtime/consumer measurements. None qualifies Linux external ContractLab or every host/feature combination.

At the recorded2c798d2d cohort, [coverage](../evidence/0.3.0/candidate-consumers-adr0362/coverage-delta/coverage-reconciliation.json) is367/375 changed runtime lines (97.87%) and411/421 testkit lines (97.62%); unchanged runtime files use exact-source joins, while decoder/testkit mappings are fresh. These are line metrics, not branch or formal guarantees. [Portable packaging](../evidence/0.3.0/candidate-consumers-adr0362/current-portable-fixed/receipt.json) is manual assembly with a debug Rust compiler, followed by actual relocated archive checks; no full Nix distribution or optimized release-build claim. See [candidate qualification](candidate-qualification.md) for the overall validation status.

## Qualification pointers

The [Ledger 9 adoption plan](../plans/ledger9-adoption.md) records current local validation. The [Ledger 8 candidate qualification page](candidate-qualification.md) retains the earlier candidate results.
