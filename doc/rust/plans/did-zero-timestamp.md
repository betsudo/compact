# Positive zero-valued DID timestamp witness

Status: local implementation and qualification pending. Parent: signed
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
