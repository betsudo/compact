# Transient provenance: action-free stateful query summaries (#612)

Date: 2026-11-10. Parent: GPG-signed local `3f5adceb1` (nested pure
summaries). Branch/worktree: `codex/rust-ast-transient-provenance-state-query-summaries`
in an isolated checkout. No remote publication.

## Scope

The explicitly invoked typed-IR pass now materializes bounded return-origin
summaries for action-free stateful value helpers with `Expression`, `SetMember`,
`MapMember` or `MapLookup` returns. It preserves symbolic formal dependencies,
known transient producer routes, unknown ledger contribution and source paths.
Call-site reuse preflights node, expression-depth, call-depth and route budgets;
an ineligible call uses bounded body traversal. Stateful helpers with actions,
effectful returns, nested stateful calls, sinks, assertions, witnesses or
unsupported paths remain uncached. A missing known producer does not certify
stability. There is no emitter or recording hook.

## Evidence

The previously retained, genuine frontend-produced schema 20 IR fixtures from
`transient_provenance_map_lookup.compact` and
`transient_provenance_membership.compact` supply positive action-free query
helpers and ordinary Field controls. The new focused tests check cached versus
uncached findings, unavailable records and truncation for source returns,
synthetic direct `MapLookup`/`SetMember`/`MapMember` return variants, an
actionful fallback and tight budgets. Direct typed-return variants are
schema-valid synthetic IR, not claimed as source-emitted forms.

All 19 `transient_provenance*.rs` integration-test targets passed: 62 tests,
zero failures, with `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-state-query-summaries-20261110`.
The full focused run is retained as `provenance-tests.log`. The pinned
pre-existing frontend compiled both source corpora with the parent and current
backends. Generated `contract/lib.rs` and
`contract/rust-capabilities.json` compared byte-identical:

| Source | Rust SHA-256 | Capabilities SHA-256 |
| --- | --- | --- |
| Map lookup | `3ee6feff610a22a74223e54383f26871de091465507e0c5fe0bf6e9ce4d114fe` | `11b72e80d71add4b7ea0becb13f1e43ebf3acd4484b1f48646ef5c73fea149d1` |
| Membership | `8b6aca14dcf345397bba4477871940d3b791c5f5ef2ee91467ebafe5bea1d47d` | `08544d30fb5b4eeb8e845b25c678f4d7a9ee9d197eb1efc6c182eba36f55e92c` |

Independent read-only review and final re-review found no blocker. The
reviewer noted the direct typed-return variants are synthetic; source-backed
query and membership helper paths are covered. No shared fixture was
regenerated.

## Limits

Sinkful and actionful helper summaries are unmaterialized and continue through
bounded per-call traversal. No frontend warning transport, native/recorded
result or FAB-byte equivalence, proof, broad suite or Forge claim is made.
