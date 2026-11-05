# Compiler192 runtime archive consumer handoff

Status: reviewed bounded local qualification passes at `21d4f9e9a`. Parent `0b1050a4d` preserves
the accepted current-package source-SDK/Passport/DID baseline; historical archive
acceptance and accepted semantic capabilities are not reopened.

## Concrete gap

The finite candidate profile still needs current package rehearsal and archive
consumption. The existing six-contract archive gate resolves version-only
runtime/macros from local Cargo archives, checks one shared runtime and exercises
typed ordinary APIs. Its temporary directory removes generated SDKs, vendor tree
and consumer on exit, preventing retention for current-candidate review and
failure diagnosis. Add an opt-in retained-output mode to that existing owner.
Keep its default command and all existing runtime/compile-refusal checks.

## Owners and public surface

- `check_release_packages.py`: existing two-crate local rehearsal, compatibility,
  lock identities and exact repeat verification.
- `check_archive_consumer.py`: complete Counter, Boolean Cell/Set,
  Boolean-to-Field Map, Field List and Merkle sources; raw generated crates with
  exact version-only runtime dependencies and no copied runtime.
- `consumer_safety.py`: existing two positive runtime methods and intended
  E0382/E0423 controls, with separate logs and result.
- Compiler, runtime/macros and checked-recording authorities remain unchanged.
  Generated APIs remain the existing ordinary typed Contract/state views.

The CLI addition is `--output <new directory>`. Existing output must refuse;
success and failure retain scratch artifacts, command streams and a source/tool/
package-bound receipt. This does not establish public registry availability.

## Acceptance and order

1. Preserve predecessors and use a new isolated package target so Cargo cannot
   overwrite earlier same-version archives. Reuse the owned shared build target
   for archive consumers.
2. Run focused package controls and retained-output positive/failure/existing-
   output refusal controls. Keep every original archive assertion active.
3. At clean signed source, resolve compiler192 with the existing-only original
   recipe recorder. Run unchanged two-crate rehearsal and `--verify-manifest`,
   retaining both runs and raw archives.
4. Run all six original contracts against these archives. Require six runtime
   methods, two safety methods, intended compile refusals and one shared runtime
   resolved as registry packages from the archive vendor directory. Retain raw
   SDK/vendor/consumer bytes, manifests, lock, metadata and command streams.
5. Archive/source/graph corruption must refuse. Inspect actual misuse diagnostics;
   unrelated compiler/dependency failure never counts as intended rejection.
6. Fresh independent CLI source/receipt review, Claude first with genuine quota
   Codex fallback; literal docs, knowledge base and CAS checkpoint. Complete
   required Linux/CI gates stay necessary for integration.

## Exclusions

No new semantic feature, facade, IR/ABI/runtime/dependency migration, functional
registry mode, source trimming, generated-source normalization, vendored offline
release, registry publication, wallet/network execution, CI optimization or Forge.
These ordinary consumers use `--skip-zk`; new proofs are not applicable to this
slice and existing functional proof epochs remain separate. Current wallet
handoff, full profile freeze, canonical integration and broader language work
remain open. All new work stays LOCAL ONLY; no push, merge, hosted trial, release,
shared installation or cleanup.

Evidence: `rust-ast-profile192-archives-20261105` (local locator).

## Initial execution and review correction

At `40fb07e57`, the original package rehearsal/repeat and six-contract consumer
finish successfully, but independent source review finds four P2 gaps in the
new retention mode. Its qualification remains withheld: safety fixture inputs
were omitted from the initial pins, preflight preceded receipt creation,
failed safety runs lost fixture bytes/partial command records, and Rustup proxy
hashes could describe the dispatcher rather than selected Cargo/rustc.

The correction starts retention before manifest/tool preflight, snapshots all
four safety fixtures, preserves the helper's original fixture bytes and partial
command receipt on failure, and resolves/invokes the selected Cargo/rustc paths.
Their bytes are checked before archive commands and at completion; child Cargo
receives the selected `RUSTC`. This is selected-input admission, not exhaustive
or continuous consumed-source attestation. The default temporary command and
all prior behavior/refusal assertions stay intact. Fresh clean package/consumer
execution and independent source/receipt review remain required after correction.

The follow-up review confirms those four fixes and finds an initial manifest
pin being replaced during preflight. The next correction extends the pin map
without replacing any first observation and rejects disagreement before vendor/
generation/build commands. A regression mutates the manifest during tool
selection and requires a failed receipt that preserves the original hash.
The successful `37f1eb079` runtime remains retained with qualification withheld;
its package/consumer results are not relabelled as the corrected source.

## Reviewed local result — 2026-11-05

Clean signed `21d4f9e9a03c9a290869780b1b846edc570b1d1a` passes all seven outer
qualification phases on Darwin arm64 / Rust 1.99.0. Nineteen focused methods
(nine archive, four safety and six package controls) pass. The build-disabled
original-recipe recorder resolves the same compiler192 package `pwb8lhc5ni8r9x2f9mnkbjpx9sh44bi7`,
executable SHA256 `e23cccf270c86f0af8840eabb6adacee3d71f42de3f9a70c539c2aacac7d2f54`.
This is an existing package realization, not another compiler build.

The unchanged original package rehearsal and exact repeat pass. Both archives
bind clean source `21d4f9e9a` and its tree: macros 18,755 bytes / 15 members;
runtime 884,380 bytes / 523 members. Every non-Cargo-generated member matches
source (12 macro and 520 runtime files), including the original manifest via
`Cargo.toml.orig`. Normalized Cargo manifests, generated locks and VCS metadata
are separate; VCS metadata names the actual execution commit. No prior archive
is overwritten or relabelled.

The full ordinary archive gate passes 22 outer commands and six safety commands.
Six complete raw generated crates use exact version-only dependencies, no
bundled runtime, and one shared registry-resolved runtime/macros pair from the
archive vendor directory. Six original native methods and two safety runtime
methods pass. Nine intended E0308/E0599/E0382/E0423 refusals retain their actual
consumer source and diagnostics. All original archive-install/graph checks,
negative maps and six-method generated test source remain unchanged.

Thirteen external control/audit rows cover the actual graph positive, four graph
corruptions, a copied bit-flipped archive rejected before generation/build, and
seven exact original diagnostic checks. Raw SDKs, vendor/consumer files,
negative sources, four safety fixture originals, Cargo metadata, locks and all
command streams remain retained. Parent and independent receipt audits rehash
29,116 recorded artifacts, 18 declared inputs and the streams without mismatch.
Selected input checks do not establish continuous or exhaustive consumed-source
attestation, external callback security or a hostile-host guarantee.

Fresh independent source review and separate actual receipt review report no
actionable P1/P2 or unsupported success claim. Both use authorized Codex CLI
fallback after retained genuine Claude quota; neither is a build/test/proof
rerun. Source review resolves all five recorded P2 findings. Earlier runtime
v1/v3 results remain withheld for new retention-mode acceptance; the v2 outer
log collision failed before children. Failed comparison harness assumptions and
malformed metadata serialization are preserved separately from the successful
final comparison; neither was a compiler/archive-content failure.

Evidence includes `qualification-v4.json`, `runtime-release-v4.json`,
`existing-recipe-v4/receipt.json`, `archive-consumer-v4/receipt.json`,
`archive-source-comparison-v4.json`, `adversarial-controls-v4.json`,
`parent-audit-v4.json`, `source-review-v3.json`, `receipt-review-v1.json` and
`bounded-closeout-v4.json` under the local evidence root above.

This closes the selected current ordinary archive baseline obligation. Proofs
and ledger application are N/A for this `--skip-zk` consumer. Functional registry
mode, public registry publication, a fully vendored release, current wallet
handoff/network acceptance, complete CI/Linux/profile freeze and canonical
integration remain open. The broader Rust AST goal stays incomplete; Forge
remains deferred. Documentation successors keep the literal `21d4f9e9a` archive
and execution identities.
