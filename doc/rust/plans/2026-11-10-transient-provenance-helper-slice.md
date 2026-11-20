# Transient provenance: bounded pure-helper slice (#612)

Date: 2026-11-10. Parent slice: signed local `fe7c5dd848a63ddb789da82125c57c4cb6f0bf49`.
Branch/worktree: `codex/rust-ast-transient-provenance-calls`, isolated from the
first-slice checkout and all other sessions. This remains local only.

## Scope

Extend the explicit typed-IR analysis to pure `Expr::Call` edges. At each
call, evaluate actual arguments once in source order, bind their provenance
to the callee's formal parameters, analyze its expression body and append
structured call-argument/return dependency steps. A transient producer in a
helper can reach a caller's persistent sink; a transient argument can flow
through an identity helper or into a helper's persistent sink. Unknown ordinary
arguments remain unknown, and a known route mixed with one remains known.
The finding's `circuit`/`source` identify the selected root declaration;
`sink_circuit`/`sink_source` identify the lexical helper containing the sink.
Producer evidence similarly names its lexical circuit. IR expression paths
identify the operation because schema 20 has no expression-level spans.

The existing expression-node and depth limits include inlined helper bodies.
A separate call-depth limit and active-call-stack refusal bound recursive
expansion. Missing or ambiguous helpers, duplicate formal names, arity mismatch, recursion and
exhausted budgets produce unavailable evidence. This is direct call-site
instantiation over the finite typed IR, not a cached whole-program summary.
It does not analyze
stateful/effectful helpers, aggregates, conditionals, control dependence or
ledger writes. A missing finding still does not certify stability.

## Source-backed corpus and evidence

`examples/rust_backend/transient_provenance_probes/transient_provenance_helpers.compact` is compiled by
the same pre-existing `compactc`/`compactc-scheme` pair documented in the
first-slice note, with `--target rust --skip-zk --emit-rust-ir`.
`tests/transient-provenance-helpers-source-ir.json` is the new resulting
schema 20 fixture. It contains helper-produced, forwarded, sink-in-helper,
mixed-input and ordinary-argument controls. Synthetic typed-IR tests cover
unavailable missing/recursive/ambiguous/duplicate-formal/over-depth helpers. No shared fixture is
regenerated.

Focused Cargo checks use
`CARGO_TARGET_DIR=/Users/ysh/.codex/local-evidence/transient-provenance-calls-20261110/target`
and `CARGO_BUILD_JOBS=2`. A standalone evidence directory holds the compiler
output and before/after generated Rust and capability JSON.

Results:

- `cargo check -p compact-rust-backend --lib` passed.
- `cargo test -p compact-rust-backend --test transient_provenance --test transient_provenance_calls`
  passed: six tests total, zero failures. Source-backed positives and ordinary
  controls are separate from synthetic malformed-IR and limit controls.
- The exact first-slice backend executable from `fe7c5dd84` and the current
  backend both compiled the same helper source with the same Scheme frontend.
  `cmp` found generated `contract/lib.rs` and `rust-capabilities.json`
  byte-identical. SHA-256 is
  `3b00b28390af715951c4e9273123ac505d0c2a7e97eb04b8d104ad245b1c2ac4`
  for Rust and
  `0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`
  for capabilities. Outputs are `base-output` and `branch-output-v3` under the
  evidence directory.
- Independent read-only review identified ambiguous duplicate helper names
  and duplicate formal names as malformed-IR gaps. Both now report
  unavailable, with synthetic regression cases. Final review found no
  remaining actionable soundness issue within this bounded helper slice.

The Scheme frontend executable predates the parent commit; it supplies the
source-backed schema 20 IR. The byte comparison uses the separately built
exact-parent and current Rust backend executables. No runtime/proof execution
or full #612 completion is claimed.
