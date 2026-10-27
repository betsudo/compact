# Testing an original DID functional lifecycle

This bounded application check uses the unchanged midnight-did v0.7.0 source
at `4e7f6b0f69bf4e2c8506a9693f8d0c3dfe68e550` with the current
Ledger8.1.3 / ZKIRv2 / runtime ABI53 / IR20 graph. The complete two-file import
closure and original manifest are retained under
`examples/rust_backend/third_party/did_functional_lifecycle`. The original ordinary DID
fixture and its historical evidence keep their own identities.

## Generated API

Generate the complete original source, including all twelve checked programs:

```sh
compactc --target rust --target ts --rust-functional-ledger8 --skip-zk \
  --rust-runtime-root /absolute/path/to/compact \
  examples/rust_backend/third_party/did_functional_lifecycle/packages/contract/src/did.compact \
  ./generated-did
```

The generated argument types are the original `SchnorrSignature` and
`BoundedUint<18446744073709551615>`. Execution needs an initialized
`CircuitContext`, a real signature and an implementation of the generated
witness interface. The consuming API is:

```rust,ignore
let recorded = did::functional::deactivate::Program::new(signature, version)
    .execute_recorded(context, &witness)?;
```

Use `execute_native` for native execution. Creating `Program` performs no ledger
or witness work; consuming it selects the adapter for one checked execution body.
The compile-fail controls reject a wrong signature, an unbounded ordinary version
argument and reuse after consuming a Program. These are Rust type/ownership
properties, not signature authentication. The circuit still validates signatures,
versions, activity and arithmetic at execution.

The [complete generated fixture](../../../tests-rust-backend/did-functional-lifecycle-source/README.md)
contains the executable integration tests and tutorial witness implementations.
The Compact sources are unmodified; do not replace the reduction witness or
signature check with a Boolean success callback.

## Behavior, failure and proof are separate checks

| Layer | Selected check | Boundary |
| --- | --- | --- |
| Fresh source generation | Complete Rust/TS, IR, origins, capabilities and twelve programs | Metadata availability does not execute twelve proofs. |
| Live TS oracle | All twelve captured DID rows, with exact state, outputs, witness order and gas observations | Only the stated Rust cases are selected for Rust execution. |
| Rust lifecycle | Real constructor-derived valid deactivation, four routes, recording replay, private/public state and metering | Constructor execution supplies deployment state; a call proof does not prove the constructor. |
| Failure controls | Ten invalid inputs through all four routes, preserving first error and callback prefix | Seeded invalid prestates are test inputs. An inaccessible failed context is not evidence of rollback. |
| ContractLab | Successful publication and late-witness failure keep owned checkpoint semantics | The external witness journal is observable and is not rolled back. |
| Selected standalone proof | Fresh direct deactivation key/proof, immutable verification and changed binding/version refusal | A Unit result has zero atoms; no result-component mutation is invented. |
| Local transaction | Independently proved, separately funded, default-strict application, functional applied-state stale refusal, exact final state and replay rejection without mutation | This is local ledger application, not network submission. |

The maintained API and proof gates are separate commands in the
[fixture README](../../../tests-rust-backend/did-functional-lifecycle-source/README.md).
Each command requires a fresh absent output destination, binds its source/tool
and generated material identities, and retains its receipt. Verification uses the
same immutable verifier for positive and changed-input controls. No cached proof
or verification verdict substitutes for a run.

## Current qualification status

The accepted baseline is canonical `acf529eff` from PR #692, which passed full
compiler/native/proof/package/wallet/E2E CI and exact-head Claude review.
Clean original compiler0.31.138 and fresh DID API/proof gates pass at signed
reviewed implementation `bf221d060`: complete generation, five Rust methods, ten invalid
inputs across four routes, three compile-fail controls, immutable proof
verification and funded default-strict application/replay. The first failed
Clippy/header attempts and earlier development epochs remain retained.

See the [qualification table](../plans/did-functional-lifecycle-current-profile.md#clean-local-qualification--compiler-031138)
for exact source, counts and limits. Independent review and the complete required
integration CI for this new slice remain open. One selected call is proved;
constructor execution and all twelve exports are not proved. Forge is deferred.

## Review follow-up and retained artifact scope

Both maintained gates require a clean checkout before and after execution. They
share the immutable original-source closure check and consume the same source-fixture DID oracle in live TypeScript, Rust behavior
and functional proof preparation. The historical ordinary selector retains its
separate capture without coupling its refresh to this fixture. A dirty checkout or changed oracle is a
refusal, and a failed proof gate writes `status: failed` with a nonzero exit.

The standalone `deactivate-input-control.proof` exercises positive verification
and changed binding/version controls with one immutable verifier. The ledger
transaction is proved independently, verified under default strictness, applied,
and replayed. Its exact serialized bytes are retained separately as
`did-functional-applied-transaction.bin`. These are different proof artifacts;
the standalone mutation controls do not claim to mutate the transaction proof.
The named applied-transaction file is written only after strict verification,
application, replay refusal and post-application checks succeed.
The post-application flags and stale-version first-error check use the functional
fixture and the actual applied contract state, with no witness callbacks.

Receipt booleans report conclusions of live Rust assertions, not independent
cryptographic measurements. Python receipt tests validate protocol completeness
and types; real proof, verifier refusals and strict application come from the
native runner. The historical `--did-deactivate` path retains its original
capture and ordinary checks without acquiring new output files or controls.
