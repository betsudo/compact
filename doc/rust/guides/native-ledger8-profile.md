# Native Ledger8 Rust candidate: capability and handoff contract

This is the finite **local candidate** profile, not a release, current hosted-CI
acceptance, or a claim that the primary Rust AST goal is complete. Each result
below retains its literal execution revision. The current local contribution
train has not been merged into the canonical branch by this handoff.

The [compiler192 application reconciliation](../plans/profile192-application-reconciliation.md)
qualifies one exact current package at `214acb658` for the original witnessed SDK,
pure Passport and selected DID cases. Earlier compiler183 application epochs,
the compiler184 request-field repair and later constructor/product qualifications
keep their literal source/package identities. This is per-stage evidence, not a
renewed monolithic profile or integration gate.

## Versions and hosts

| Identity | Selected value and boundary |
| --- | --- |
| Compact compiler / language | 0.31.192 / 0.23.105 for the selected SDK/Passport/DID baseline at `214acb658`, package `pwb8lhc5ni8r9x2f9mnkbjpx9sh44bi7`; other cohorts retain their literal recipe/execution epochs |
| Rust runtime / macros | `midnight-compact-runtime` 0.2.0 / macros 0.2.0 |
| Optional source testkit | `midnight-compact-testkit` 0.1.0; unpublished paired source recipe |
| Runtime ABI / private IR / finalized capability schema | 53 / 20 / 3; package versions are separate |
| Ledger | 8.1.3, pinned Git revision `b5d3e965a18e4cac17fac4f38446fc55c524439a` |
| Native ZKIR / TypeScript runtime | 2.1.1 (the v2 target) / 0.16.102 |
| Rust toolchain | 1.99.0 executed locally; manifests advertise 1.89, which this handoff does not establish as tested |
| Current local host | Darwin arm64 |
| Current Linux x86_64 | Configured integration workflow; no new hosted execution or acceptance in this handoff |

The runtime and backend compatibility documents must agree. A compiler version
string alone does not identify a package: the backend README is an explicit Nix
CLI source input, so documentation can change the package derivation without
changing Rust/Scheme implementation. Preserve recipe, source, binary and generated
artifact identities separately; do not relabel an old compiler execution.

## What consumers can use

The Rust target emits structured Rust AST after typed lowering and independent
checked-plan validation. Ordinary native/recorded methods remain available. The
opt-in `--rust-functional-ledger8` surface provides a consuming, compiler-issued
`Program::new(...)` with native and recorded execution over one checked body.
Constructing a Program performs no witness or ledger work. Supported operations
retain source evaluation order, typed values, first-error behavior and existing
frame/slot ownership. A checked body is not a guarantee that every runtime input
can prepare, prove or apply.

Use the maintained [witnessed source SDK example](examples/functional-witness/README.md),
[original DID guide](did-functional-lifecycle.md), [Passport example](examples/passport/README.md)
and [ContractLab testkit](../../../testkit-rs/README.md). The paired SDK/testkit
closure has one physical bundled runtime and an explicit copied-manifest path
adjustment. It is not a registry release or fully vendored offline distribution.
Raw generated Rust stays raw; a separate formatting copy is only a comparison.

Rust rejects the qualified wrong witness result, signature/version type and
consumed Program/context examples at their intended E0053/E0308/E0382 loci, with
positive controls. These are type/ownership checks, not authorization or privacy
proofs. Real signatures, arithmetic limits, state versions, target/path identities
and untrusted recording data still require explicit runtime checks. Witness
callbacks remain ordinary handwritten Rust; a typed signature does not certify
callback privacy or external effects.

## Stage-specific evidence

The populations below are different units, not additive coverage or a language
completion percentage. Compiling a whole contract does not prove every export.

| Cohort and literal execution epoch | Local behavior and controls | Genuine proof/application scope |
| --- | --- | --- |
| [Compiler192 installed-default SDK](../plans/profile192-application-reconciliation.md), `214acb658`; preceding compiler183 `026f1109b` / `cd3f432c6` / `7baa4b774` stay separate | Three ContractLab methods, three positive/misuse pairs, three actual proof-feature library lints, installed-prefix/override/byte-identity and recording/snapshot refusals; 29-command gate plus 16 focused controls and three real selection refusals | One witnessed Cell call: fresh k6/49-row proof and independently proved funded DEFAULT-strict application; no constructor or wallet lifecycle |
| [Compiler192 Passport](../plans/profile192-application-reconciliation.md), `214acb658`; preceding compiler183 `9286fe749` stays literal | Intact 17-file source, all 75 pure exports/89 internal circuits, 202 sampled TS rows plus codec comparison, 16 adoption and two guide methods, Clippy, intended refusals and changed-oracle/restored controls | Recording/proof/application **not applicable** to this original pure library |
| [Compiler192 DID selected lifecycle](../plans/profile192-application-reconciliation.md), `214acb658`; preceding compiler183 `4e49b59e5` / `51616d5ce` stay literal | Intact two-file source, complete IR and twelve Programs; twelve live DID TS rows plus auxiliary crypto; seven Rust methods, eight success routes and forty error routes, late-witness/checkpoint/metering and typed controls | Timestamp witnesses 200 and 0 of the same `deactivate` export: separate fresh k11/1,804-row proof runs and independently proved funded DEFAULT-strict transactions; constructor and all-export proofs unclaimed |
| [Products below Vector components](../plans/vector-component-products.md), `5ef204e8e`; later documentation-only successors retain their own bridges | 24 complete generated SDKs, 35 sampled cases and typed/static/ingress/ordering controls | Three selected complete sources have fresh proofs and separate funded strict applications; 21 required exports remain unproved, including 18 over-limit models |
| [Constructor Vector operands](../plans/constructor-vector-operands.md), compiler192 producer `f09900533`, proof qualifier `a8133996b` | Thirteen complete SDKs, 22 TS cases, typed/static/ingress/identity/order controls and 15 selected constructor-derived success cases; all 39 models retained | Fifteen fresh later-entrypoint proofs and separately proved funded DEFAULT-strict applications/replay; not constructor cryptography or total-cost acceptance |
| [Compiler192 ordinary runtime archives](../plans/profile192-runtime-archives.md), `21d4f9e9a`, same `pwb` compiler package | Two-crate rehearsal/exact repeat; six complete raw registry-mode SDKs with one archive-vendored runtime/macros pair, six native and two safety methods, nine intended compile refusals; 19 focused methods and retained source/tool/graph/output controls | Proof/application N/A for this ordinary `--skip-zk` consumer; no functional registry mode, public registry or fully vendored release claim |

The preceding [DID package-specific check](../plans/current-supported-profile-handoff.md#actual-final-package-local-result)
at `51616d5ce` rebuilds and records the exact README-bound compiler package,
then passes the DID API gate and both fresh proof/application cases. The zero
case uses the literal workflow shell locally under Nix; the valid case is a
separate maintained invocation. Nix and Rustup runner snapshots retain their own
identities. That DID check did not rerun the SDK, Passport or products. The later
compiler183 `026f1109b` SDK epoch has its own package run. The compiler192 baseline
separately reruns the SDK, Passport and selected DID cases at `214acb658`; it does
not relabel those earlier results, the product cohort or constructor proof epoch.

The current SDK qualification gate refuses `COMPACT_RUST_RUNTIME_DIR`, including
an empty value. Use an installed package for this gate. It admits the CLI's
installed prefixes in order, refuses checkout fallback, and compares 105 declared
installed runtime source hashes with selected sources before generation and at
completion. This is trusted loader admission, not an OS file-consumption trace.
Earlier SDK receipts without this admission record retain their narrower scope;
they cannot establish this newer installed-default qualification.

The preceding [selected DID epoch](../plans/current-did-handoff.md) at `67cbded3e`
and accepted compiler138/PR#693 scope are preserved; the later zero extension does
not rewrite their execution populations. Historical compiler143 SDK and
compiler142/144 Passport evidence are similarly distinct. The [delivery
roadmap](../ledger8-delivery-and-completion.md) records integrated scalar, pair,
Fold, Map-size, constant-projection and statement extensions. Do not reopen those
accepted slices merely because an older proposal calls them pending.

## Boundaries and explicit exceptions

- Functional Keccak is excluded by the selected Ledger8/ZKIRv2 profile. Native
  support is not a qualified v2 proof/application flow. Ledger9/ZKIR3 and additional
  signature targets remain separate work, not silently enabled capabilities.
- Constant fixed-vector evidence does not establish arbitrary dynamic indexing
  or every source/type combination. Typed lowering and representation/resource
  refusals still govern each program. Broader source-language parity and the
  whole of #516 remain unproven; this handoff does not redefine them as complete.
- The nested-product k8 policy remains unchanged. Do not trim an intact source,
  normalize its geometry or raise the cap to conceal an unproved export. The
  existing explicit DID k11 recipe is a separate maintained profile.
- Constructor-derived or privileged fixture prestate is not constructor/deployment
  proving. Fee/Dust funding, strictness and replay controls are part of each
  application receipt, not network submission or production wallet acceptance.
- Untrusted IR/recording admission remains explicit: checked types/order,
  canonical private atoms, replay state, target/slot and identity checks at their
  actual owners. The optional encoded-byte decoder limit does not establish
  aggregate decoded heap/CPU containment; that policy remains separate.
- Source origins may be unavailable. Structural diagnostics are not guaranteed
  exact nested source ranges. A generated capability flag is not an executed
  proof, and caller-declared snapshot hashes are not provenance or finality.
- Trusted-host source/tool/material/receipt inspection is not continuous/atomic
  attestation, exhaustive consumed-source inventory, arbitrary callback security,
  an independent cryptographic rerun or formal compiler/spec equivalence.

The historical [capability and retirement book](../recording-effect-algebra/capability-and-retirement-book.md)
and [acceptance crosswalk](../evidence/0.3.0/closure/crosswalk.md) keep their original
authorities and 84 retirement rows. This page neither rewrites them nor renews
historical whole-suite/Linux results for the present candidate.

## Configured integration gates and current state

Local iteration uses focused affected checks. Candidate integration still needs
the complete configured workflows, including:

1. Authenticated same-run compiler producers for Linux x86_64 and Darwin arm64,
   source/tool/closure checks, normal compiler tests, source maps and source
   occurrence controls. The existing original-recipe recorder checks the exact
   supplied compiler package; it does not build a missing package implicitly.
2. Public Python and refusal controls; complete native workspace tests, formatting
   and strict Clippy. Early lint/refusal checks retain their later complete gate.
3. Intact Passport, DID and isolated SDK gates; the existing constant-vector,
   Map-size, direct-Fold, Field/pair-return, Unit-initializer and nested-product
   API/proof cohorts. Every required assertion and negative control stays active.
4. Verified locked built-ins and selected parameters; fresh direct keys/proofs,
   immutable verification and applicable separate strict application/replay gates.
   The workflow now has an explicit zero-DID case after the preserved default
   valid case, a distinct absent output directory, and failure/success diagnostics.
   This is local wiring; hosted zero-case proof acceptance remains open.
5. Runtime package rehearsal, archive consumers, wallet handoff, normal E2E,
   version/header checks and retained artifacts. Success of a small proof cohort
   does not substitute for these integration obligations.

The ordinary package/archive stage now has reviewed local evidence at
`21d4f9e9a`, with full retained outputs and its own clean archive VCS identity.
That result does not establish the current wallet handoff, renew hosted
package/archive checks or replace the complete required integration gate.

See `.github/workflows/build-compiler.yml`, `rust-backend-checks.yml`,
`compiler-producer.yml` and `tools/compact-rust-backend/check_native_ci.sh` for the
executable commands. Full current CI/Linux, exact-head review and logical stack
integration remain open. No hosted run, push, merge, release or shared installation
is performed by this handoff. Cleanup waits for verified GitHub integration and
preservation of unique evidence.

Forge is deliberately deferred on `experiment/zkir-forge`, with stores, sources,
benchmarks, signed results and failures retained. Backend development does not
depend on it. Selected contract-level reuse benefits remain valid; whole-CI value
is unproven. The old 46-second measurement covered Forge-managed generation only;
direct bypass generation was not separately timed, so the total ZKIR/prover-key/
verifier-key share remains unknown.
