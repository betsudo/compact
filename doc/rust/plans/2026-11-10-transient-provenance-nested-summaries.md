# Transient provenance: bounded nested pure summaries (#612)

Date: 2026-11-10. Parent: GPG-signed local `008ac9337` (pure leaf helper
summaries). Branch/worktree: `codex/rust-ast-transient-provenance-nested-summaries`
in a separate checkout. No remote publication.

## Scope

The opt-in typed-IR pass now materializes a pure helper's symbolic return
summary when its body calls other pure helpers and the complete nested
traversal remains supported and sink-free. Formal routes, helper-local
producers and dependency paths include the nested call chain. Summary
construction records the deepest visited pure-call stack; call-site reuse
requires that depth to fit the caller's remaining `max_call_depth`, alongside
the existing node, expression-depth and route preflights. If one of these
preflights fails, the call uses bounded body traversal, preserving its unavailable
diagnostic. Recursion is analyzed during summary construction and remains
unavailable at call sites. Stateful value calls, sinkful helpers, assertions,
`Expr::WitnessCall` and unsupported forms remain uncached.

## Source and evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_nested_summaries.compact` was
compiled by the pinned pre-existing `compactc`/`compactc-scheme` pair with
`--target rust --skip-zk --emit-rust-ir`. The new retained schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-nested-summaries-source-ir.json`
(SHA-256 `829414f44f5203aeb45fddf9dc8a09f2098ed8bd4bfbf8ee647f8f50d1331476`).
No shared fixture was regenerated. Four new tests check nested forwarding
and helper-produced positive findings, ordinary Field negative control,
caller-root parameter/producer path evidence, sinkful nested-helper fallback,
recursion, call-depth/node/expression-depth/summary-budget behavior, and
exact cached-versus-uncached diagnostics. A focused synthetic schema-20 IR
case excludes a pure wrapper calling a stateful value helper from caching,
independently of sink detection.

All 18 provenance integration-test targets passed: 58 tests, zero failures,
with `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-nested-summaries-20261110`.
The full focused run is retained as `provenance-tests.log` there. Exact-parent
and current backends compiled the same source with the same Scheme frontend;
`cmp` found generated `contract/lib.rs` and `contract/rust-capabilities.json`
byte-identical. SHA-256: Rust
`d7a53b7b35a0fd92161375ad1e5289bc64642810c2658d8e4ff5f361dd4156cb`;
capabilities
`0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`.
The parent/current outputs are retained in the evidence directory.

Independent read-only review found no code blocker and requested an explicit
stateful-value-call exclusion regression. The synthetic case is included.

## Limits

These are reusable return summaries for supported, sink-free pure-call
chains. Stateful and sinkful helper bodies still use bounded call-site
traversal; the absence of a producer does not certify stability. No
frontend warning transport, native/recorded result or FAB-byte equivalence,
proof, broad suite, or Forge claim is made.
