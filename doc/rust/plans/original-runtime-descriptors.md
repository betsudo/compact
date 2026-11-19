# Original runtime descriptors, conformance, Schnorr and standard library

Four unchanged modules from pinned mainstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e`
are retained in signed local original-gate slice `06e22b5aafe69a0dc5a63bd16f8202df108b966e`.
They expand into 924 parameterized cases per profile. Both runs are red; independent
review confirms faithful evidence rather than compatibility completion.

| Original module | Cases | Published runtime 0.21.0 | Accepted source runtime 0.18.0 at 3eaf |
| --- | ---: | --- | --- |
| compact-types | 833 | 829 passed, 4 failed | 807 passed, 26 failed |
| conformance | 58 | 58 passed | 58 passed |
| schnorr | 2 | 2 passed | 2 passed |
| stdlib | 31 | 29 passed, 2 failed | 27 passed, 4 failed |
| Total | 924 | 918 passed, 6 failed | 894 passed, 30 failed |

No original bodies, assertions or refusals were changed. There are no skips or
todos. Both the test child and gate exit 1. Signed differential receipt SHA-256 is
`65bf2d21d132654758df79c159d3d99049baedc36bedc9137af689891c152d5c`;
independent review SHA-256 is
`d05a351a87ae53794e6ba552a9fa328b98926a2eddf165f1df0fbaa823aba30a`.
Physical source/emitted/runtime dependency and tool maps are checked before and
after execution, alongside actual module resolution. The published graph uses
416 SDK files; the source path authenticates the separately built accepted 3eaf
output. Earlier Vite-cache input-guard diagnostics remain historical.

Published differences are four leading-U+FEFF opaque-string roundtrips and two
coin values exceeding Uint64. Source differences include the original closed
export-inventory expectations versus additional accepted descriptors, strict
field/point canonicalization, a missing bigint-to-bytes helper, and coin error
classification. [Issue #785](https://github.com/betsudo/compact/issues/785) maps
these owners and remaining decisions. Preserve accepted additional APIs and
canonical ledger decoding; do not erase assertions or remove existing exports
to manufacture a green original suite.

## Bounded bigint-to-bytes API repair

The follow-up uses accepted 3eaf/runtime 0.18 source, rather than the older runtime
in the original test-adapter branch. Signed local source
`d79f397fe8808d37c9285fef1d6b2324ef02fed1` adds the pinned-main
`convertBigintToBytes(n, x, src)` function verbatim. Existing
`convertFieldToBytes` and all legacy casts and diagnostic text remain unchanged.
Runner `b1041130f77d7370fadb796efd47128b1029065e` requires a signed build selected
by explicit digest/head and verifies the current runtime Git component matches.

All three unchanged original helper cases now pass. Across the same four full
modules, source results improve to 897 passed / 27 failed out of 924; only those
three named cases change status. The standard-library module is 30/31, with its
coin error-class case still failing. Existing accepted source unit tests pass
465/465 across 24 modules; these overlap the original tests and are not additional
original-case coverage. Three exact Git fixtures are read as text and removed;
this does not invoke generated EntryPoint contracts.

Signed qualification SHA-256 is
`d1278b53814f36864ab8a3a1a441c48e835597230788891794ece4d5bc201236`;
independent review SHA-256 is
`93febfdff9c514f610f8613c718ccc7845b04244aa4426d0555470140c1aab71`.
A focused independent replay passes exactly the three original helper cases,
with 921 filtered. Replay review SHA-256 is
`0ff752f2870a66fea358c6ba195d36d18944d933b0e594ddc21f837b6046313b`.
Two source builds produce identical 84-file JavaScript output. Only the three
cast artifacts differ from the signed parent; 81 output files and the generated
version remain identical. Source, dependency, complete tool maps and emitted
output are bound. Wrong source, build digest and emitted code refuse before a
test child starts.

These are local Darwin arm64 pure JavaScript/API results, not generated Rust,
proof, funded transaction, public compiler or Linux qualification. Byte-to-Field/
Uint signature migration and coin/error policies remain separate. Finished owned
Node/build/staging copies and the clean worktree are removed; source branches,
lock, generated version, emitted output, red diagnostics and signed receipts remain.
Implementation stays local; docs-only publication does not publish these source slices.
