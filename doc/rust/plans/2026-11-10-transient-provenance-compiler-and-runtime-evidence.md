# #612 local compiler warning and runtime parity slice

Date: 2026-11-10. Canonical base: `origin/codex/rust-backend-ast` at
`09c75e2891283fed65f2db385925a5a4b9409af6`. This local worktree is
`codex/rust-ast-transient-provenance-compiler-warnings`, stacked on the
signed `6a05c31203c9f8302782ccdd3d8200ad596141d0` typed-IR analysis
head. Nothing was pushed or published.

## Implementation

The public `compactc` Rust target accepts the explicit
`--rust-transient-provenance=warn` option. It consumes the already parsed
schema-20 typed IR after the existing IR and proof-info decode checks and
before rendering. It forwards neither this option nor its optional
`--rust-transient-provenance-acknowledgements <json>` policy file to Chez.
The bounded analysis and exact-route acknowledgement policy are the same
ones used by the standalone advisory command. The compiler prints one
source-positioned warning or acknowledged note per known route, explicit
unavailable notes, and a summary that says absence of a finding does not
certify stability. An acknowledgement changes severity, not evidence.
Unsupported or exhausted analysis remains advisory. Default compilation
does not invoke the analysis. Diagnostic text clips fields and escapes
untrusted filenames; an inaccessible stderr cannot turn a published
compilation into a failure. Malformed acknowledgement JSON fails before
staging or frontend execution.

## Focused evidence

All compiler artifacts are under
`/Users/ysh/.codex/local-evidence/transient-provenance-compiler-warnings-20261110`.
The pinned pre-existing `compactc-scheme` frontend from the
`rust-ast-zero-width-boundary-20261101/fresh-proofs-v1/bin` evidence
directory compiled all 16 `transient_provenance*.compact` source corpora in
both default and warning modes with `--skip-zk --emit-rust-ir`. Warning mode
emitted 84 known-route warnings and eight explicit unavailable notes; default
mode emitted no provenance warning. For each corpus, generated
`contract/lib.rs` and `contract/rust-capabilities.json` were byte-identical
between modes. The retained `corpus-summary.json` and per-source stderr logs
record the result. The direct source corpus produced five warnings including
local arithmetic, conversion, commitment opening, and transient commit;
ordinary Field argument and witness controls produced none. A genuine
exact-route acknowledgement changed one of those five warnings to a note,
left four warnings, and preserved both generated files byte for byte.
Malformed acknowledgement JSON was rejected without creating output.
The focused `compactc` unit target passed 34 tests with
`CARGO_BUILD_JOBS=2`, including deterministic formatting, controls,
acknowledgement and filename escaping. No shared fixture was regenerated.

Runtime evidence is under
`/Users/ysh/.codex/local-evidence/transient-provenance-runtime-parity-20261110`.
Identical external Cargo consumers, with separate build targets, linked
the exact canonical-base and local-stack generated crates for the genuine
map-lookup source corpus. Each checked four selected calls (`state_hit`,
`hash_hit`, `helper_hit`, `ordinary`): native and recorded result, state,
effects, private state, private transcript outputs, gas, replayed state and
effects, and tagged serialized ledger-state bytes. The hash and helper
cases also compared native and recorded aligned FAB representations and
tagged serialized FAB bytes, 67 bytes per return. Both consumer outputs are
byte-identical, SHA-256
`c93b1e43d81fa24d6677f3226d03bd01c250bf60223b4a779eb70ca2e8e067df`.
Independent read-only review found no remaining blocker after the stderr
and filename fixes; it confirmed the direct return FAB-byte assertions.

## PR CI qualification

After opening PR #724, the repository's header and public Python gates
identified two integration requirements for the new source probes. All 40
new Compact/Rust files now carry the repository's full license header. The
16 analysis-only Compact sources live in
`examples/rust_backend/transient_provenance_probes/`, outside the top-level
full parity fixture inventory. This keeps the source-backed probes visible
without claiming they are maintained flat generated libraries or changing
shared parity baselines. The branch-owned schema-20 IR fixtures were
recaptured with the pinned frontend solely for the 11-line source-position
shift. All 16 current sources reproduce those retained IR bytes exactly;
the parsed IR excluding source positions is unchanged. Focused tests pass
across 21 provenance integration targets and the `compactc` target. The
public Python gate passes all 519 selected methods (zero failures/errors),
and the new-header regression check reports zero violations. A fresh
canonical-base/current backend comparison over the relocated sources found
all 1,855 generated files byte-identical. These post-PR receipts are under
`/Users/ysh/.codex/local-evidence/transient-provenance-pr-20261110`.
The first full CI pass also exposed one module-order formatting issue and two
Clippy warnings in the new analyzer/test code. After those focused fixes,
`cargo fmt -p compact-rust-backend -- --check`, backend all-target Clippy with
`-D warnings`, and the 22 focused Rust test targets all pass locally. The
changelog gate still needs a maintainer decision because this slice excludes
a Scheme compiler version change.

## Limits

The warning flag is optional and Rust-target-only. This is a bounded
possible-flow diagnostic over supported typed-IR forms, not a stability
proof or default compiler policy. Schema 20 provides circuit declaration
positions but no expression-level source spans. The runtime comparison
covers four selected calls; nonempty tagged FAB returns were exercised for
two. Private FAB arrays were empty in all four, and no proof execution or
universal Compact language coverage is claimed. The secondary
`compact-rustc` wrapper and frontend-owned diagnostic transport remain
outside this local slice.
