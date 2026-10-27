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
| Selected proof | Fresh direct deactivation key/proof, immutable verification and changed binding/version refusal | A Unit result has zero atoms; no result-component mutation is invented. |
| Local transaction | Separately funded, default-strict application, exact final state and replay rejection without mutation | This is local ledger application, not network submission. |

The maintained API and proof gates are separate commands in the
[fixture README](../../../tests-rust-backend/did-functional-lifecycle-source/README.md).
Each command requires a fresh absent output destination, binds its source/tool
and generated material identities, and retains its receipt. Verification uses the
same immutable verifier for positive and changed-input controls. No cached proof
or verification verdict substitutes for a run.

## Development and integration status

The current slice starts from verified canonical merge `acf529eff` of PR #692,
which passed full compiler/native/proof/package/wallet/E2E CI and independent
exact-head Claude review.
The maintained development API gate passes after a checked Unit pure-call
emission fix, with the first failed Clippy receipt retained. Final clean compiler
qualification, fresh selected functional proof, full required integration CI and
independent review must pass before this slice is called delivered. The
[bounded plan](../plans/did-functional-lifecycle-current-profile.md) records the
actual baseline and evidence stages. Forge remains deferred and is not required.


## Clean local qualification — compiler 0.31.138

Signed implementation `926e8de8c83667e6fd45f87a0fc0fe71147e1ccd` is based on
accepted canonical `acf529eff`. The original Nix compiler package builds from
that clean source. Its build has `doCheck = false`; the separate maintained
compiler suite passes both ZKIR lanes with 636/636 manifest and 1/1 JavaScript
checks each at its recorded earlier preparation epoch.

Fresh qualification with the clean original 0.31.138 compiler and its own
Scheme frontend passes:

| Gate | Result |
| --- | --- |
| Complete generated fixtures | 281 checked, zero stale or failed |
| Original DID generation | Full IR/origins/capabilities and all twelve programs match |
| Live TypeScript | 24 observations, including all twelve exact DID rows |
| Selected Rust behavior | Five test methods; all ten invalid inputs on four routes (40 refusal executions) |
| Ordinary and consuming APIs | E0308 wrong version/signature, E0382 Program reuse; strict consumer Clippy |
| Selected functional proof | Fresh direct key k11 / 1,804 rows; 3,296-byte proof; immutable verification |
| Proof controls | Changed binding and same-geometry version input refused; changed IR/program refused before keys/state |
| Local application | Separately funded default-strict apply, exact final state, IntentAlreadyExists replay with unchanged ledger |
| Source hygiene | Complete workspace format, zero new header violations and all three exact version checks |
| Public Python | 352 discovered: 332 passed and 20 existing explicit skips |

The Unit-fixed backend library also passed 1,846 methods with seven existing
explicit-run helpers ignored in its earlier development epoch. That result is
not relabeled as a new clean-source execution. Final PR review and complete
integration CI remain required. This proves one selected call; constructor
execution, timestamp-zero Rust success and all twelve exports are not proved.
Earlier development compiler/proof receipts retain their own literal identities.
