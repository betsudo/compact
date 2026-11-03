# Positive zero-valued DID timestamp witness

Status: reviewed local API and separate proof/application qualification pass at
signed `4e49b59e58473201a71e5bc3d3b0ba78c911b875`. Parent: signed
`0f6e722df14f859e1f084eb536eedf7358cb1dcc`. Branch:
`codex/rust-ast-did-zero-timestamp`.

## Gap and API

The unchanged original DID capture includes successful `did/valid` and
`did/timestamp-zero` rows. The selected Rust success tests run only the first.
The zero is the timestamp witness return value, not ledger wall-clock time.
Consumers keep the existing typed witness, ordinary native/recorded methods and
consuming functional `deactivate::Program`. No public API change is needed.

## Owners and acceptance

1. Extend the two lifecycle test files with the original zero row. Reuse every
   existing success assertion, retain the original success and failure methods,
   and update the maintained API gate's exact accounting. Compare all four
   routes against original state, private/public transcripts, witness order,
   gas and independent replay. Run the full selected API gate first; fix a
   semantic owner only if an actual failure identifies one.
2. Extend the private proof runner and maintained proof gate with an explicit
   case choice, defaulting to the existing valid case. Bind current proof results
   to the exact selected capture and timestamp. Preserve explicitly historical
   receipt validation without admitting an unbound receipt as current evidence.
3. Keep the intact two-file source, twelve Programs and complete IR exact.
   Run valid and zero proof cases separately with fresh keys, standalone proofs
   and independently proved funded DEFAULT-strict transactions, immutable
   verification, source/input/binding/stale/replay controls. Use the existing
   explicit DID k11 recipe without changing the tiny/nested k8 limit.
4. Run focused protocol/refusal tests and fresh Claude CLI source/receipt review
   (Codex CLI only after an actual Claude quota failure). Record actual results,
   source/tool/material identities and limits in the existing knowledge base and
   checkpoint. Retain complete required CI/proof integration gates.

## Exclusions

No compiler/runtime refactor, new contract, fixture/oracle normalization, source
trimming, cap increase, proof/verdict cache, Forge, hosted run, push, merge,
publication, shared installation or cleanup. Constructor and all-twelve-export
proof coverage remain open; this edge does not complete the backend goal.

Evidence:
`/Users/ysh/.codex/local-evidence/rust-ast-did-zero-timestamp-20261103`.

## Reviewed local result — 2026-11-03

The native-first baseline passes at `783344d2a`. The final source at `4e49b59e5`
passes the existing-only compiler recipe with the same immutable compiler
0.31.183, seven focused receipt tests and the maintained ten-command DID API
gate. All twelve original live TS rows match. Seven Rust methods retain all five
previous methods and add the two zero-success methods. The two success cases
exercise eight ordinary/functional native/recorded routes; ten error rows retain
40 failing-route executions, alongside the existing late-witness controls.
Full state, transcripts, witness order, gas and independent replay assertions
remain intact. Strict consumer Clippy and three intended E0308/E0382 controls pass.
No compiler, runtime, witness, oracle or generated-fixture repair was needed.

Each proof case has a separate successful nine-command run and two changed-IR/
Program refusals before keys/state. Each directly generates fresh keys with the
existing k11 / 1,804-row model, verifies a fresh 3,296-byte standalone proof, and
independently proves and applies a funded DEFAULT-strict transaction: 7,413 bytes
for valid, 7,412 for zero. Binding, changed version input, same-geometry,
stale-version and unchanged IntentAlreadyExists replay controls pass for both.
Deterministic key bytes can match because both cases use the same circuit; the
actual key-generation commands run separately. Warm Cargo compilation reuse is
not proof or verdict reuse.

The runner checks the selected original capture and independently compares its
captured final data before proving. Current result protocol v2 carries the exact
case and timestamp witness (`"200"` or `"0"`), and every current gate requires
the requested case. Historical v1 receipts remain readable without a case claim;
they cannot satisfy current case-bound validation. Swapped, missing, wrong and
mistyped case fields are refused. Existing full source/IR/Program binding stays
exact, with no output normalization. Unit has no output atoms, so a changed-result
control remains inapplicable.

Authentic Claude CLI source review (90.77 seconds) and separate actual receipt
review (169.99 seconds) find no actionable P1/P2. The latter independently checks
the 1,507 API and 2,943-per-proof declared source pins, tool snapshots, generated,
material, result and command records, including intended compile-fail stdout and
stderr hashes. Author inspection diagnostic mistakes and the unstarted receipt
review attempt remain preserved separately; no actual gate failed.

This is trusted-host receipt inspection, not an independent cryptographic rerun,
continuous/atomic host attestation, exhaustive consumed-source inventory or proof
for every input. All original sources, captures, IR, twelve Programs and generated
Rust remain unchanged. The existing current-DID capture/tool limits remain as
documented in the [parent handoff](current-did-handoff.md).

Actual evidence: `controller-v2.json`, `original-existing-recipe-v2/receipt.json`,
`did-api-v2/receipt.json`, `did-proof-valid-v1/receipt.json`,
`did-proof-zero-v1/receipt.json`, `actual-receipt-inspection-v3.json`,
`local-claude-did-source-review-v1.json` and
`local-claude-did-receipt-review-v2.json` under the evidence root above.

The maintained API CI entry includes both new tests. The workflow proof entry
still selects the default valid case; zero proof qualification is currently
local only and needs explicit integration wiring. Constructor/all-export proofs,
current full CI/Linux and logical integration remain open. The next bounded task
is the [finite supported-profile handoff](current-supported-profile-handoff.md),
not a claim that this edge completes the backend goal.
