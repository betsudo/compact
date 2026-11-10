# #612 cumulative generated-output parity

Date: 2026-11-10. Parent: GPG-signed local `bfa7c06c2` (standalone
advisory). Branch/worktree: `codex/rust-ast-transient-provenance-cumulative-parity`
in an isolated checkout. No remote publication.

## Question and method

Does the complete local typed-IR provenance stack change generated artifacts
relative to the confirmed canonical base? The comparison uses the exact
detached base `09c75e2891283fed65f2db385925a5a4b9409af6` backend retained
under `/Users/ysh/.codex/local-evidence/transient-provenance-20261110/target`
and the signed `bfa7c06c2` backend under
`/Users/ysh/.codex/local-evidence/transient-provenance-cli-20261110/target`.
Both are invoked by the same pinned `compactc`/`compactc-scheme` pair from
`/Users/ysh/.codex/local-evidence/rust-ast-zero-width-boundary-20261101/fresh-proofs-v1/bin`
with `--target rust --skip-zk --emit-rust-ir --compact-path compiler` and the
same source files and runtime directory. Each invocation writes only into
its own path under
`/Users/ysh/.codex/local-evidence/transient-provenance-cumulative-parity-20261110`.
No Cargo target was built for this comparison; it reused the separately built
base and head binaries. No shared fixture was regenerated.

The exact invocation and file comparison are retained as `reproduce.py` in
that evidence directory (SHA-256
`380e5db9ab5562413bb93fb93e5fb1f29ebc7e3794552ddd7838f9a7cf769c69`).
It requires a fresh `--output-root`, verifies both worktree commits and all
backend/frontend binary hashes before compiling, and rejects an existing
output directory. A run into `verified-rerun-01` reproduced the same 16-corpus,
1,855-file result and byte-identical `comparison.json`. Backend
binary SHA-256 values are
`1e8bda7d9db19021eda107bc991f462930ef0d9c701c1cec1640c59fe3cebc2b`
(base) and
`91c56108dadf5635439bbf3f55033be150ce2391f5b44380780cbb5be735b87e`
(head). Frontend SHA-256 values are
`233d8e4a4112e906f03db7a0cdc733f3ffce74ff7273538b7983860258bd3289`
(`compactc`) and
`59e590a4178088c2e332ea2ef54eb87f6ab1d9f61433f07e5d06c2dcf7dae661`
(`compactc-scheme`).

For every `examples/rust_backend/transient_provenance*.compact` source, the
comparison checks the relative file set and SHA-256 of every generated file.
Only per-invocation `compile.log` is excluded because it captures invocation
paths. The retained `comparison.json` records per-source file counts and Rust,
capability and contract-info hashes; its own SHA-256 is
`fc1be4ceb4e02a35a9c0adcd530ef8b8cff11c594f23c6a80534c9d1100cccc5`.

## Result

All 16 source corpora compiled with both backends. All 1,855 generated files
had matching relative paths and bytes. This includes generated `contract/lib.rs`,
`contract/rust-capabilities.json`, compiler contract info and manifest, typed
IR, ZKIR files and copied runtime sources. The per-source file counts were:

| Source suffix | Files |
| --- | ---: |
| base transient provenance | 113 |
| action calls | 116 |
| aggregates | 113 |
| assertions | 116 |
| effectful returns | 118 |
| Field bytes | 113 |
| helper summaries | 113 |
| helpers | 113 |
| map lookup | 117 |
| membership | 117 |
| nested | 114 |
| nested summaries | 113 |
| pure actions | 116 |
| state collections | 128 |
| state sinks | 121 |
| value calls | 114 |

The parity result is stronger than the earlier exact-parent, two-file checks
for these sources: it compares the full local stack directly with the canonical
base and the full retained generated tree. It is still generated-artifact
evidence. It does not execute native/recorded calls, compare runtime result or
FAB bytes, exercise proof behavior, or establish parity for unselected Compact
sources.

## Advisory corpus pass

The `compact-transient-provenance` binary built from the signed local commit
was invoked separately
on each of the 16 retained `contract/compact-rust-ir.json` files from the head
output tree. `advisory-corpus/` retains every structured JSON report and a
`summary.json` (SHA-256
`ecd48dbe24838aba64728faeb4fa36f9b01e135db8fde14861ed808d579f46bb`).
All 16 commands exited successfully. They produced 80 findings, all warning
under the empty acknowledgement policy, eight explicit unavailable records
and no truncated reports. The unavailable records occur only in the
assertions and pure-actions corpora, with reason
`assertion execution control unavailable`. This is a selected-corpus coverage
observation, not a claim that every possible Compact flow is analyzable or
that an absence of known producer evidence certifies stability.

A fresh `advisory-corpus-verified/` rerun produced byte-identical advisory
reports. Its `run-log.json` (SHA-256
`7c287f4f446275d9b446ec86cf2ab4b0c06cea6619fc6380416cf03db461617e`)
retains each process exit code, typed-IR input hash, report hash and counts.
The CLI binary SHA-256 for that run is
`4d8d23bec28ee71e1731ca7e6588445fc94fb65425fb8b5684e9cb97ba847257`.

Independent read-only review verified the base/head worktree commits, their
separate Cargo build dep files, binary hashes, pinned frontend hashes, complete
generated-tree equality, manifest counts and selected artifact hashes. It
found no parity blocker and requested the now-retained reproduction script
and binary hashes for clearer provenance. Its follow-up identified stale-output
and unverified-input risks in the first script; the fresh-output/hash checks
above close that reproduction gap.
