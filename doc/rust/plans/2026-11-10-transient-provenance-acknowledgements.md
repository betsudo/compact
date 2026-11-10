# Transient provenance: exact advisory acknowledgements (#612)

Date: 2026-11-10. Parent: signed local `0d3f6af4e` (value calls).
Branch/worktree: `codex/rust-ast-transient-provenance-acknowledgements`
in an isolated checkout. No remote publication.

## Policy

The opt-in `advise` post-pass accepts caller-supplied acknowledgements. Each
entry requires a nonblank reason and an exact root circuit, sink kind,
sink circuit, sink path, sink input, dependency role (data or control), and
full producer/dependency route. It marks one known route acknowledged; a
finding stays a warning until every known data and control route is
acknowledged. Acknowledged findings remain in the report with all evidence.
An unmatched or blank entry is returned as unused. An acknowledgement list
over the explicit limit is ignored, marked unavailable/truncated, and leaves
every finding a warning. `warning == false` is a policy status for known
routes; it does not certify unknown inputs as stable or alter cryptography.

The source-backed test uses the genuine `evolveNonce` standard-library route
from `transient_provenance_value_calls.compact`. Its explicit acknowledgement
marks only the selected persistent-hash use, while unrelated findings remain
warnings. A stale sink kind, wrong input, blank reason, over-budget policy,
and partial data/control route acknowledgements all leave warnings active.
This is a caller-side policy API; schema 20 has no source annotation or
frontend transport for acknowledgements. Intentional conversions in
`compiler/standard-library.compact` remain visible as transient producers
until the caller explicitly acknowledges a specific sink route.

## Evidence

Seven focused provenance test targets passed: 23 tests, zero failures, with
`CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-acknowledgements-20261110`.
The exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`rust-capabilities.json` byte-identical: SHA-256
`a2acccac18ba0a753c366150d3355e103cc6be667d5f512b2070c6c2dcae7b59`
for Rust and
`072d0af0641bcf3adf9825ca0b7a1976b7b82b1c3c931dab5dde5e20f1ed5565`
for capabilities. No proof, runtime, broad suite, shared-fixture regeneration,
or Forge work was run.

Independent read-only review identified two policy key gaps: missing sink
kind and missing data/control role. Both were added with regressions; final
re-review found no remaining actionable blocker in this bounded slice.

The stacked local work now supplies the issue's opt-in typed-IR prototype and
policy evidence. Unsupported forms still report unavailable; absence of a
finding is never a stability certificate. No compiler CLI/frontend plumbing
or strict error mode is claimed.
