# Transient provenance: bounded pure leaf helper summaries (#612)

Date: 2026-11-10. Parent: GPG-signed local `3664f769e` (list and Merkle
sinks). Branch/worktree: `codex/rust-ast-transient-provenance-helper-summaries`
in a separate checkout. No remote publication.

## Scope

The opt-in typed-IR analysis now materializes and reuses symbolic return
summaries for eligible pure leaf helpers. Each summary records helper-local
transient producers, ordered formal-parameter dependency routes, nonformal
unknown contribution, source location and analysis node cost. At a call site,
the pass substitutes caller origins for the formals and rebases helper paths
into the caller's IR path. It retains expression order when a helper combines
an argument producer with its own producer. Aggregate component trees retain
hit/miss projections through both returned products and forwarded aggregate
arguments. Advisory reports carry the same summary evidence.

The summary pass uses `max_summary_nodes` (default 4096) as a total
precomputation budget, plus the existing circuit, node, depth, route, step,
component and parameter limits. A summary is used only when the call fits
the caller's node/depth/route budgets. Otherwise analysis traverses the
helper body at that call site. Pure helpers containing nested calls, sinks,
assertions, `Expr::WitnessCall` or unsupported forms are not cached and
retain the established traversal and unavailable behavior. Stateful helper
summaries are not materialized by this slice. A missing producer in a summary
does not certify stability: formal and unknown contributions remain separate.

## Source and evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_helper_summaries.compact` was
compiled by the pinned pre-existing `compactc`/`compactc-scheme` pair with
`--target rust --skip-zk --emit-rust-ir`. The new retained schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-helper-summaries-source-ir.json`
(SHA-256 `c48d78518f0802743c20f30963793239e10aa39a76c2999831664d9b1761c7fd`).
No shared fixture was regenerated. Five new tests cover materialized
formal/producer evidence, product return and aggregate forwarding hit/miss
projections, conversion, conditional selection, mixed producer route order,
cache-budget fallback, advisory evidence preservation, and exact cached versus
uncached findings/unavailable/truncation on seven source-produced corpora and
under selected tight analysis limits. Ordinary Field inputs remain accepted.

All 17 provenance integration-test targets passed: 54 tests, zero failures,
with `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-helper-summaries-20261110`.
The full focused run is retained as `provenance-tests.log` there. Exact-parent
and current backends compiled the same source with the same Scheme frontend;
`cmp` found generated `contract/lib.rs` and `contract/rust-capabilities.json`
byte-identical. SHA-256: Rust
`b3d6b61feb4f11efc791932002600f4f6a655e87978196ec8bf44f8be8bf86e7`;
capabilities
`0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`.
The parent/current outputs are retained in the evidence directory.

Independent read-only review found three gaps: mixed known/formal route
ordering, aggregate return/projection coverage, and ambiguous unknown-field
wording. The implementation now preserves interleaved order, the new source
cases test returned and forwarded products, and the field is named
`has_nonformal_unknown` with a no-stability comment. A subsequent tight-limit
test found a route-limit diagnostic-path discrepancy; route preflight now
falls back to body traversal. Final read-only re-review found no remaining
bounded-slice blocker and verified the retained test log and output parity.

## Limits

These are materialized return summaries for eligible pure leaf helpers, not
whole-program or stateful summaries. Helper sinks and nested calls still use
bounded body traversal. Source spans remain circuit-level where schema 20 has
no expression span. No frontend warning transport, native/recorded result or
FAB-byte equivalence, proof, broad suite, or Forge claim is made.
