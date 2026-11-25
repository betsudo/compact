# Transient provenance: symbolic caller parameters (#612)

Date: 2026-11-10. Parent: GPG-signed local `bb362ced4` (Field-to-bytes
evidence). Branch/worktree: `codex/rust-ast-transient-provenance-symbolic-inputs`
in a separate checkout. No remote publication.

## Scope

The opt-in typed-IR report now exposes sorted symbolic root-parameter
dependencies separately for sink data and branch control. A root circuit
parameter starts as unknown external origin with its circuit/name identity.
Bindings, arithmetic, conversions, aggregate projections, conditional joins,
transient producers and bounded call-site helper traversal carry those
identities. A helper's formal parameter is instantiated from its caller's
actual origin; a finding names the caller's external parameters, not an
unrelated callee formal. Known producer routes remain independent from the
unknown and symbolic contribution. Witness results remain unknown but retain
possible dependencies on their arguments.

The pass caps tracked root parameters at `max_parameters` (default 32).
Aggregate component trees are capped by `max_component_nodes` (default 64)
before retaining a summary; a repeated product-alias chain therefore reports
`aggregate component limit` instead of expanding exponentially. Exhausted
limits set `truncated` and record unavailable evidence. Assertions with a
known transient or unknown/symbolic condition report
`assertion execution control unavailable`; later sink findings do not claim
precise control dependence through them.

## Evidence

The tests reuse genuine source-produced schema 20 fixtures from
`transient_provenance_helpers.compact`, `transient_provenance_aggregates.compact`,
and `transient_provenance_state_sinks.compact`. They check caller parameter
instantiation through produced and forwarded helper routes, mixed known and
unknown arithmetic, product projection versus whole-product dependencies,
and separate data/control parameters at a ledger write. Focused synthetic
typed-IR regressions cover an unknown witness result, an ordinary assertion
gate before a known transient sink, parameter budget, and a well-typed
10-binding product-alias chain. The existing source assertion test now expects
unavailable execution control for its ordinary Field gate, without creating
a known-transient finding. No shared fixture was regenerated.

All 15 provenance integration-test targets passed: 45 tests, zero failures,
using `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-symbolic-inputs-20261110`.
The full focused run is retained as `provenance-tests.log` in that directory.
Exact-parent and current backends compiled the same helper source with the
same Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`3b00b28390af715951c4e9273123ac505d0c2a7e97eb04b8d104ad245b1c2ac4`;
capabilities
`0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review found two initial blockers (aggregate clone
growth and omitted ordinary assertion control). Both were fixed; re-review
found no remaining correctness blocker in this bounded slice.

## Limits

These are possible parameter dependencies at the selected sink. They are
not a proof of runtime value sensitivity, path reachability or stability.
The pass still traverses helper bodies at bounded call sites; it does not
materialize reusable cached helper summaries. Unsupported forms and
cross-transaction origins remain unavailable or unknown. Acknowledgements
remain exact for producer routes, while these new parameter lists are
supplementary evidence and are not an acknowledgement key. No frontend
warning transport, runtime/FAB-byte equivalence, proof, broad suite or
Forge claim is made.
