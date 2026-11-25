# Transient provenance: bounded first slice (#612, ADR-0463)

Date: 2026-11-10. Base: `origin/codex/rust-backend-ast` at
`09c75e2891283fed65f2db385925a5a4b9409af6`, confirmed both locally and
by `git ls-remote` before branch creation. Work is isolated in
`codex/rust-ast-transient-provenance`.

## Plan and scope

Expose an explicit `transient_provenance::analyze(&Contract, Limits) -> Report`
entry point over typed schema 20 IR. It records direct `transientHash` and
`transientCommit` producers, paths through local bindings, arithmetic and
representation conversions, and the value/opening inputs of `persistentHash`
and `persistentCommit` sinks. The result is structured producer, dependency,
sink and circuit-location evidence. A mixed ordinary argument/witness input
remains unknown while known transient evidence is retained. No result type
claims that a value is stable.

Resource limits cover circuits, parameters, expression nodes/depth, producer
routes, dependency steps, findings and unavailable records. Exhaustion or an
unsupported expression is reported as unavailable; `truncated` records any
resource limit that curtailed analysis or omitted diagnostics. Analysis is
opt-in and has no rendering or capability hook.

This slice covers pure expression bodies and stateful circuits with no actions
and a direct expression return. It does not summarize helper calls, stateful
actions/effectful returns, constructors, aggregates, conditionals, control
dependencies or persistence into ledger state. Unsupported expressions are
unavailable even when their arguments might contain a producer. This scope
does not establish safety for any unreported circuit. Source locations are
circuit declarations because schema 20 has no expression-level source spans;
expression paths distinguish producer and sink occurrences.

## Source-backed corpus

`examples/rust_backend/transient_provenance_probes/transient_provenance.compact` was compiled with the
pre-existing `compactc`/`compactc-scheme` pair from the isolated
`rust-ast-zero-width-boundary-20261101/fresh-proofs-v1/bin` evidence directory,
using `--target rust --skip-zk --emit-rust-ir`. Its new
`tests/transient-provenance-source-ir.json` is the resulting schema 20 IR,
retained as a new fixture; no shared fixture was regenerated. The corpus
contains direct hash-to-hash, commit-to-commit, local/arithmetic/unknown-input,
conversion, commitment opening, ordinary Field argument, ordinary witness
Field and transient-only cases. The older frontend executable is used to
create source-backed IR; the
exact base and branch backend renderers are compared separately below.

## Evidence

Focused commands use separate `target`/`branch-target` directories under
`/Users/ysh/.codex/local-evidence/transient-provenance-20261110` and
`CARGO_BUILD_JOBS=2`. No broad proof suite, workflow or shared fixture was
run. The separate evidence directory also holds source compiler output and a
detached, unmodified base checkout for exact backend comparison.

- `cargo test -p compact-rust-backend --test transient_provenance`: four tests
  passed. The tests check five source-backed positive sink inputs, ordinary
  argument and witness controls, conversion, arithmetic mixed with unknown,
  deterministic reports, witness-boundary unavailability, hard node limit and
  unchanged rendering when analysis is invoked. No runtime/proof execution is
  claimed.
- Exact base and branch backends compiled the same source with the same
  `compactc-scheme` executable. `cmp` found both generated `contract/lib.rs`
  and `contract/rust-capabilities.json` byte-identical. SHA-256:
  `d8b3bdccea10b5f59eff19570d7be89bd0746a38fef16b91a9065a13aedcd6e7`
  (Rust) and
  `940e7e8089134b4c92361f0b7cd230a6f093c5f5c3d74600225d9b1b95cb3721`
  (capabilities). Final comparison outputs are in `base-output-v2` and
  `branch-output-v4`.
- The source compiler pair SHA-256 is
  `233d8e4a4112e906f03db7a0cdc733f3ffce74ff7273538b7983860258bd3289`
  (`compactc`) and
  `59e590a4178088c2e332ea2ef54eb87f6ab1d9f61433f07e5d06c2dcf7dae661`
  (`compactc-scheme`). The frontend binary predates this base; the backend
  byte comparison uses binaries built separately from the exact base and
  branch source.
- Independent read-only review found dropped unknown evidence and unbounded
  traversal after node exhaustion; both were fixed and covered by tests. A
  follow-up review found no remaining producer-flow blocker within this
  stated slice and suggested the now-explicit `truncated` semantics.
