# Compiler192 runtime archive consumer handoff

Status: local bounded qualification in progress. Parent `0b1050a4d` preserves
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
