# Compiler192 local wallet serialization handoff

Status: source-first baseline qualification in progress. Parent `b00b89ad4`
preserves reviewed current192 application and ordinary archive stages. Historical
ADR33/35 wallet acceptance keeps its original versions, hosts and scope.

## Concrete obligation and owners

The finite native profile still needs current wallet handoff evidence. Reuse
the existing proof-smoke positional entry point with its three required roots:
complete original Counter, Boolean Cell and Boolean Cell-read sources. Do not
introduce a selector, wrapper API or generated wallet protocol. The existing
`positional_counters::check_binding_and_handoff` owns observation/decoder/key/
address/stale-state refusals, direct Counter proof, proven sealed deployment and
call export. `positional_cells` retains both existing companion scenarios.

`check_wallet_handoff.mjs` and its exact ledger-v8@8.1.3 lock own canonical JS
decoding, action/segment count, shared address and exact byte round-trip. Rust
ledger transaction adapters own preparation, proving, tagged encoding and ledger
application. Compiler, runtime, handwritten callbacks and wallet/network remain
separate owners. No generated public API change is proposed.

## Baseline before proofs

1. Preserve all predecessor branches, targets and evidence. At clean signed
   source, resolve the exact existing original compiler192 package; do not
   relabel an older package or implicitly rebuild it.
2. Generate all three complete original sources without keys into new retained
   outputs. Keep raw Rust/IR/capabilities/origins/Programs untouched. Compare a
   separate formatting copy against the maintained static fixture libraries;
   independently build/test those consumers and bind their actual Rust inputs.
3. First close any concrete baseline regression at its existing owner. Keep
   historical compatibility checks independent from current output, without
   fixture repinning, IR normalization, source trimming or cap changes.
4. Build the original proof-smoke target with current source/tool/dependency/
   actual consumed-fixture identities, bounded jobs and Cargo reuse. Snapshot the
   exact executable before proving. Observe every original model through the
   official typed IR model owner before direct keys; automatic tiny k<=8 remains.
5. Check locked official parameters and required built-ins before cryptography.
   Independent source/preparation review must precede fresh keys/proofs.

## Actual handoff acceptance

Generate fresh direct keys from the same unedited current IR, then invoke the
existing three-root proof-smoke suite with new deployment and call output paths.
Keep all observation, wrong-verifier/address/state and decoder controls. Require
the standalone Counter proof and changed-binding refusal, independent transaction
proving, expected Counter/Cell state assertions and the original export sequence.
Capture exact source/tool/material/key/verifier/binary/stream/transaction identities.

Decode the exact fresh bytes with the pinned JS package. Require exact byte
round-trips, sole deploy/call in segment 1 and matching address. Keep independent
swapped-role, malformed/trailing-byte, mismatched-package and changed-address
controls where the actual canonical decoder supports them; do not count general
module/dependency failures as intended refusal. Decode success is not signature,
proof, fee, finality or consensus verification.

Inspect receipts and obtain separate independent actual receipt review, Claude
first with genuine quota Codex fallback. Record literal stages in docs, midnight
knowledge base and CAS checkpoint. Every original required CI/proof gate remains
mandatory for integration; this selected suite does not replace the full gate.

## Explicit limits

The original `check_transaction_with_handoff` smoke uses
`WellFormedStrictness::default()` with `enforce_balancing=false`, offline network
`local-test` and timestamp zero. Preserve and disclose those existing settings.
This slice is not DEFAULT-strict fee-funded, current-TTL wallet balancing,
network submission/observation, constructor cryptography, full-language parity,
release or full profile/integration acceptance. Previously accepted strict
functional SDK/DID/constructor-derived entrypoint proofs remain separate epochs.

All new work is LOCAL ONLY: no push, merge, hosted run, release, shared install,
Forge reconstruction, optimization or artifact cleanup. The broader goal remains
active and incomplete. Evidence: `rust-ast-profile192-wallet-20261105`.
