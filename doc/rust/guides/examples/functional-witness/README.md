# Isolated witnessed functional SDK consumer

Local slice in progress. The accepted compiler141 baseline generates the complete
maintained Field-witness source with default bundled runtime and compiles the
raw SDK. An external copied testkit needs one explicit source-package adjustment:
change only its copied Cargo manifest runtime path from `../runtime-rs` to
`../generated/contract/runtime-rs`. Preserve the original manifest separately.
The original repository manifest and compiler-generated files stay unchanged.
This is an unpublished paired source-SDK recipe, not registry publication or a
fully vendored offline distribution.

The consumer uses one physical bundled runtime shared by contract and testkit.
The testkit's repository-only dev fixtures are not part of this consumer graph;
no testkit development-workspace test claim follows. All resolved local manifests
must stay within the external closure, and registry/Git/checksum identities must
match the selected source lock.

## External layout and execution

Copy this complete `consumer/` and `source/` into a fresh directory. Generate with
the matching Rust-capable compiler and its standard library:

```sh
compactc --target rust --target ts --rust-functional-ledger8 \
  --rust-require-recording --skip-zk source/field_witness_proof.compact generated
```

Stage the matching reviewed testkit library sources/manifest as `testkit-rs/`,
apply the copied-manifest path adjustment above, and seed `consumer/Cargo.lock`
from the qualified Compact source lock. Resolve with `cargo metadata --offline`,
check all local manifests and registry/Git/checksum identities, then execute:

```sh
cargo +1.99.0 test --locked --offline --manifest-path consumer/Cargo.toml
cargo +1.99.0 clippy --locked --offline --all-targets \
  --manifest-path consumer/Cargo.toml -- -D warnings
cargo +1.99.0 clippy --locked --offline --all-targets \
  --manifest-path generated/contract/Cargo.toml -- -D warnings
```

The SDK's own lock also needs the same qualified seed/metadata resolution before
its direct locked Clippy command. Generated SDK bytes remain raw; formatting
applies only to handwritten consumer files. The actual documentation manifest/source passed local development execution with
all three tests, direct consumer/SDK Clippy and independent positive/refusal
controls. It computes source and generated-file hashes from the external files.
This pre-commit development result is distinct from a clean full proof/integration
qualification; the earlier literal-hash probe retains its own source epoch.

## What the tests cover

- Ordinary native/recorded and functional native/recorded direct generated-call
  adapters preserve outputs, Cell/private state, witness order, complete public
  state/effects, execution gas and recorded replay program/gas for this source.
- A failed witness leaves committed lab state at its checkpoint. External witness
  activity remains visible; private state is owned `u64`, not shared mutable data.
- Matching snapshots restore state; changed runtime identity is refused without
  committing the changed snapshot. Caller-declared hashes do not authenticate
  provenance or finality.
- Three independent correct-type binaries compile. The corresponding negative
  binaries reject wrong witness result type and reuse of consumed Program/context
  at their intended error locations. Their features deliberately fail; do not use
  `--all-features` as an ordinary success command for this refusal fixture.

These are local execution/replay and static API controls. Genuine ProofLab proof,
immutable-verifier/input/output negatives, separately proved funded default-strict
application and replay refusal remain open in the [slice plan](../../../plans/functional-isolated-source-sdk-consumer.md).
No constructor/all-export, arbitrary callback privacy, network, hostile-host or
whole-language coverage is claimed. Existing complete CI/proof gates remain
mandatory before integration.
