# Transient provenance: standalone typed-IR advisory (#612)

Date: 2026-11-10. Parent: GPG-signed local `f4babcc16` (action-free stateful
query summaries). Branch/worktree: `codex/rust-ast-transient-provenance-cli`
in an isolated checkout. No remote publication.

## Scope and invocation

`compact-transient-provenance` is a separate opt-in Rust binary. It consumes a
schema-20 typed IR JSON file and writes the structured advisory JSON to stdout:

```text
compact-transient-provenance typed-ir.json
compact-transient-provenance typed-ir.json --acknowledgements exact-routes.json
```

The optional acknowledgement file is a JSON array of exact
`Acknowledgement` records, including a nonempty reason and full route copied
from an advisory finding. A matching record changes that route's warning
status without removing producer, dependency or sink evidence. Unknown input
is not certified stable. Findings and unsupported-analysis records do not
cause a nonzero exit; malformed input and unsupported IR schemas exit 2 with
no partial JSON report. The command does not invoke the Compact frontend,
renderer, recording admission, runtime or proof helpers; compiler diagnostic
transport and strict policy remain separate work.

The CLI uses default analyzer budgets and adds preflight input limits: 2 MiB
raw typed IR JSON, 16,384 JSON nodes, 512 bytes per JSON string, 512 KiB total
string bytes and 1 MiB acknowledgement JSON. More than 128 acknowledgement
entries produce an explicit truncated/unavailable advisory record; none are
silently applied.
The bounded raw JSON is parsed to a generic value for preflight, then the
original bytes are deserialized into `Contract` so duplicate typed-IR fields
remain errors. The source schema must equal 20 before analysis.

## Evidence

The source-produced map-lookup schema-20 fixture drives the CLI integration
tests. Eight tests cover advisory JSON equivalence with the library, positive
warnings and ordinary Field control, exact acknowledgement with unchanged
finding evidence, acknowledgement count limit, malformed acknowledgement
rejection, schema mismatch,
oversized string, JSON node budget and duplicate-field rejection. The
standalone evidence output `advisory-map-lookup.json` contains three warnings,
zero unavailable records and `truncated=false`.

All 20 `transient_provenance*.rs` integration-test targets passed: 70 tests,
zero failures, with `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-cli-20261110`.
The full focused run is retained as `provenance-tests.log`. The pinned
pre-existing frontend compiled the same map-lookup source with exact-parent
and current backend binaries. All 117 generated files other than per-run
`compile.log` matched byte-for-byte, including Rust code, capabilities,
compiler contract info and manifest. SHA-256: Rust
`3ee6feff610a22a74223e54383f26871de091465507e0c5fe0bf6e9ce4d114fe`;
capabilities
`11b72e80d71add4b7ea0becb13f1e43ebf3acd4484b1f48646ef5c73fea149d1`;
contract info
`8b0708f3bc12b2d29343f7172d637d250507e18b888c3c67b50966fb814bd5e6`;
manifest
`f8bb106a8d2bd6b4772beb78b80b5962626145b438d1bd7f9e580ca660db0caa`.
No shared fixture was regenerated.

Independent read-only review found a missing schema gate, loose input bounds
and a duplicate-key parsing issue. All were fixed and covered by focused
tests; final re-review found no remaining blocker.

## Limits

This command consumes already produced typed IR. It does not surface a warning
in the Compact source compiler, choose a default/strict frontend policy, or
prove native/recorded runtime result and FAB-byte equivalence. Its report is
bounded by the explicit input and analyzer caps; a missing finding never
certifies stability.
