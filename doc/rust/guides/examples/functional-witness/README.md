# Isolated witnessed functional SDK consumer

The complete maintained Field-witness source generates with default bundled
runtime and the raw SDK passes clean local qualification with original compiler
0.31.143 on macOS arm64. This slice still requires published PR review and full
Linux CI before integration. An external copied testkit needs one explicit source-package adjustment:
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
including the repository Apache license as `testkit-rs/LICENSE`,
apply the copied-manifest path adjustment above, and seed `consumer/Cargo.lock`
from the qualified Compact source lock. Resolve with `cargo metadata --offline`,
check all local manifests and registry/Git/checksum identities, then execute:

```sh
cargo +1.99.0 test --locked --offline --manifest-path consumer/Cargo.toml
cargo +1.99.0 clippy --locked --offline --manifest-path consumer/Cargo.toml \
  --package isolated-functional-sdk-consumer --lib -- -D warnings
cargo +1.99.0 clippy --locked --offline --manifest-path consumer/Cargo.toml \
  --package compact-contract-field-witness-proof --lib -- -D warnings
cargo +1.99.0 clippy --locked --offline --manifest-path consumer/Cargo.toml \
  --package midnight-compact-testkit --lib -- -D warnings
```

These are the graph-selected direct owner commands executed by the gate. They
share the consumer's qualified lock; no separate SDK or testkit lock is needed.
Generated SDK bytes remain raw. The gate runs `rustfmt --edition 2024 --check`
on every handwritten consumer Rust file, including proof and refusal fixtures.
The executable consumer computes source and generated-file hashes from the
external files. Earlier development probes and the clean qualification retain
separate source epochs in the slice plan.

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

## Optional fresh proof and strict application

The actual `consumer/proof/prove.rs` uses ProofLab's official provider directly.
Enable `proof`; it selects the existing SDK transaction and testkit proof features.
This is a test composition, without a new provider facade or compiler API.

With the exact selected compiler, native `zkir` and source-built proof material
runner, the maintained gate executes the complete recipe from a fresh directory:

```sh
python3 tools/compact-rust-backend/check_functional_source_sdk.py \
  --compiler /absolute/path/to/compactc \
  --material-runner /absolute/path/to/compact-rust-proof-smoke \
  --output /absolute/new/qualification-directory
```

Run this command from the matching clean Compact source checkout. The isolated
consumer graph itself contains no checkout paths. `MIDNIGHT_PP` and
`MIDNIGHT_LEDGER_TEST_STATIC_DIR` must identify the selected public proof material
and pinned upstream ledger static fixtures. Prepare built-ins and the selected
small parameter before the gate with the matching source-built material runner:
`compact-rust-proof-smoke --prepare-proof-material --parameter-k 6`.
The gate freshly verifies their upstream SHA256 declarations in verify-only mode,
checks source/package membership and cohort locks, and generates new direct native
keys. This maintained fixture explicitly requires k6, matching the prepared CI
material; a changed geometry requires deliberate recipe/material qualification. Both explicitly supplied proof providers use synchronous local material
access, so missing additional parameters refuse rather than being fetched by
those providers. The pinned upstream fee fixture is separately bound to its
verified built-in material; this is not a process-level network sandbox. It retains original raw generated bytes; it caches no proof or verdict.

The external proof program can also be executed after the same material/graph
checks, using the retained fresh keys:

```sh
cargo +1.99.0 run --locked --offline --profile proof-checks \
  --manifest-path consumer/Cargo.toml --features proof --bin functional-sdk-proof \
  -- generated /absolute/new/proof-output
```

The checked profile retains assertions and overflow checks. Compiled Cargo output
may be reused; proof execution, mutation controls and ledger application rerun.
The source-built material runner and selected files are hash-bound; this does not
attest an arbitrary executable's build provenance or a hostile host's effective
configuration. The complete staged consumer (including its resolved lock) and copied testkit
(manifest, source, README and license) must have identical inventories at the end.
Direct Clippy selects each owner from the admitted graph, including
the copied testkit library without its repository-only development fixtures.

The development prototype and actual documented proof program pass locally:

- A fresh 2,912-byte standalone call proof verifies against the unchanged selected
  verifier. Changed binding, input, output and proof bytes refuse. A changed
  private witness result inconsistent with the public transcript refuses too.
- An independently proved 6,607-byte transaction, including fee/Dust proof,
  passes `WellFormedStrictness::default()`, applies the expected Cell state, and
  refuses replay as `IntentAlreadyExists` without ledger mutation.
- Constructor-state insertion and upstream Night reward/Dust registration are
  privileged fixture setup. This is not constructor/deployment proving or a
  wallet lifecycle. The standalone proof and transaction proof are separate.
- This source does **not** constrain a witness to return its argument. Input 43
  with witness/output 42 remains a valid different statement. The original proof
  refuses its changed communication commitment; successful checking of the new
  statement is source-faithful. A typed callback does not prove an unstated
  algebraic relation or certify arbitrary callback privacy/external effects.

The maintained gate passes clean local qualification on original compiler
0.31.143 at source `fb704896bee4abd596ce81f84975e8b2840ac6ef`, including all
28 commands, genuine malformed-recording refusals, fresh proof execution and
strict application. The combined branch also passes 401 public Python methods;
the Passport compatibility gate retains its separate qualified source epoch. Earlier compiler141 receipts retain
their own source identity. These are finite selected results. The accepted Passport parent has been
reconciled with the SDK tree unchanged; this SDK slice still requires published
PR review and complete Linux CI as recorded in the
[slice plan](../../../plans/functional-isolated-source-sdk-consumer.md).
No all-export, network, hostile-host or whole-language coverage is claimed.
Existing complete CI/proof gates remain mandatory before integration.


### Preparation refusals and result evidence

After the declared privileged setup and before selected-call proving/application,
the example takes
fresh genuine generated recordings and separately changes their final state or
inserts a noncanonical private-output atom. Preparation must return precisely
`ReplayStateMismatch` or `NonCanonicalPrivateOutput { output_index: 0, atom_index: 0 }`,
respectively. `prepare_call` receives only an owned recording and `CallSpec`;
these controls invoke no ledger application. This is a structural separation,
not an observed rollback or shared-store atomicity guarantee. The untouched
recording then prepares and proves successfully. These controls exercise the
existing untrusted recording boundary without introducing a new runtime API.

Mutation, private-state, replay and geometry fields in the result are derived
from actual execution/check results. The gate also independently requires the
applied Cell's fixed-width little-endian Field encoding to equal 42 and the
applied state data to match the separately executed expected state. Rust's actual
proof/verification and strict application assertions remain enabled; receipt
parsing does not substitute for them or attest an arbitrary hostile executable.

Communication-commitment randomness is freshly sampled from `OsRng` for the
selected application call. It is deliberately retained for the ordinary/functional
same-call comparison and the non-submitted local refusal/mutation controls, so
those controls isolate their changed input/output identities. Never submit those
controls. Do not replace the randomness with zero or reuse it for a different
application call. Other
seeded RNGs in this executable are deterministic ledger/prover test fixtures,
not application privacy guidance.


The source folders must contain exactly their tracked regular files before copy
and again at completion. Ignored build files under `consumer/target/`, a stray ignored lock, added
files and symlinks refuse admission; they are not silently copied or filtered.
With the selected k6 recipe, actual k6/built-in material content verification now
runs before source compilation or Cargo work, while the later keygen still must
confirm k6. Required paths and the single selected ZKIR executable are admitted
even earlier. No network or hostile-concurrent-host guarantee is inferred.

The second-review source-admission and scope corrections pass that literal clean
qualification, including twelve focused controls. Earlier source receipts retain
their own narrower epochs. Current-head PR review and full CI remain required.
