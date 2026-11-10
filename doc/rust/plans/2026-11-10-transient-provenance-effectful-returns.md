# Transient provenance: effectful value returns (#612)

Date: 2026-11-10. Parent: signed local `8dab25b39` (nested producers).
Branch/worktree: `codex/rust-ast-transient-provenance-effectful-returns`
in a separate checkout. No remote publication.

## Scope

The explicitly invoked typed-IR pass now returns an `Origin` from a bounded
`ReturnPlan`. `Value` visits its expression, `Let` carries lexical producer
bindings, `Sequence` analyzes actions before its result, and `Conditional`
joins both return values with the established possible-path control policy.
An effectful value-returning helper with no separate actions can therefore
pass known provenance to its caller's persistent sink. Branch-local state
sinks retain their separate control routes. Identical primitive literal
`Value` arms do not gain a false return-value control route. All existing
node, depth, call, route, step, and finding limits still apply; a malformed
effectful return with separate actions is still rejected before traversal.

## Evidence

The genuine source
`examples/rust_backend/transient_provenance_probes/transient_provenance_effectful_returns.compact`
was compiled with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. The new schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-effectful-returns-source-ir.json`.
It contains effectful helpers with conditional action branches, a produced
transient return, a forwarded transient argument, a plain Field control,
a transient-dependent condition, and a `ReturnPlan::Let` binding returned
through a helper. A synthetic identical-literal control case tests the
bounded equality exclusion. No shared fixture was regenerated.

Nine focused provenance test targets passed: 28 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-effectful-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend; `cmp` found generated `contract/lib.rs` and
`rust-capabilities.json` byte-identical. SHA-256:
`18a24187b11b878c0817b1d8122da4ea33b6f7ee21ac27ca7f57afea74a71058`
for Rust and
`8c5e267bd9f2317163ce3592779bf4cde0cf8c72311fcd5c6625b40dff5e186a`
for capabilities. No runtime/FAB equivalence, proof, broad suite, or Forge
claim is made. Independent read-only review found no actionable blocker in
this bounded slice.

Other explicitly unavailable/unknown boundaries remain as documented in
the preceding notes. Absence of a known transient producer does not certify
stability; this pass does not change generated public APIs or emission.
