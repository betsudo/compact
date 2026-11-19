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


## Owned Ledger 9 coin amount refusal

Reviewed local `5a7fc67db7fc9c3fb5b3c9e128bf503329f845d9` closes the remaining
original standard-library coin range/error-class case. The private Ledger 9
coin snapshot reads nonce, color and value once as own data, then validates a
BigInt in Uint128 before effects. Generic unchecked descriptor encoding and the
legacy four-argument context route remain unchanged; the existing generic
encoding assertion is retained verbatim. A global encoder change would have
broken that accepted contract.

The unchanged original stdlib module now passes 31/31. Across all four originals,
results are 898 passed / 26 failed out of 924, with no skips. Only the named
out-of-Uint128 case changes status from the bigint-helper parent. Existing 465
source assertions plus four new Ledger 9 recipient/range/state-identity cases
pass 469/469 in 24 modules; these overlap originals and are not additional
pinned-original coverage. Independent replay passes the one original target
with 923 filtered and all four new cases with 21 filtered.

Signed qualification SHA-256 is
`5f8114e3082b0a9a5cb0ec35d1ba5d84191769801429eb3bcbb3173cce65edb0`;
independent review SHA-256 is
`b4402942081732c4afe4436d3d893f5e27af787d957887c92cb9b759b03f1080`.
The signed build attestation is
`81d55c5105fc1da89501575629f8420aad942bbc9264b6fa9bb776685c3dd847`.
Fresh compilation checks exact 6,042-file/16-link source/build/tool membership
before and after; ignored source/config additions are absent. Of 84 emitted
files, only zswap.js and its map change from the helper parent. Wrong source,
build digest and emitted-code inputs refuse before a test child starts. All 159
evidence hashes are reviewed. Earlier v2 build evidence remains historical;
the v3 strict-closure run has fresh 469/924 results.

Owned Node, build, staging copies and clean worktree are removed; signed cleanup
SHA-256 is `41fc86afb2ec3c112446fb811c4cf58cd32bbc3e0d6634f85841894e752541af`.
Source branches, lock, emitted output, red results and receipts remain. This is
local arm64 JavaScript/API qualification; generated Rust/proofs/funded/Linux
are separate. The 26 compact-type alias/inventory/canonical-decoder differences
and byte-cast signature compatibility remain in #785.

## Opaque atom refusal and owned byte decoding

Signed local source `8da87375b06301d6a47ef9512848e396d76acded` ports the
pinned-main missing-atom refusals for both opaque descriptors and makes fixed
Bytes decoding return its own buffer. The existing leading-U+FEFF behavior,
unchecked encoder APIs, extra exports and canonical scalar/point rules remain.
All four original modules and their assertions are unchanged.

The actual original cohort improves by exactly seven named failures to
905 passed / 19 failed out of 924, with no skips. Existing source tests remain
469/469 in 24 modules; these overlap original coverage. Two fresh source builds
check the exact 6,042-file/16-link closure; only compact-types.js and its map
change among 84 emitted files. Three wrong-source/build/emitted inputs refuse
before a test child. The first negative-control harness incorrectly expected
no staging directories on artifact refusal; the runner had already refused
before the test child. Those logs and corrected controls are retained.

Signed qualification SHA-256 is
`7d6691b158ecc06dd79d0f1cd9b8e204e193e8b35b2383e9b354369750dc60b0`.
Independent execution review SHA-256 is
`d56b40e5c6e04e9b1e8ad725afe413a07719eb36388554d8b115e5eabe6468ce`.
A fresh original replay passes 7/7 with 917 filtered. Its first import-selector
setup failure remains historical. Signed final delivery SHA-256 is
`805cbd11f9bd98892b77ab6960e4f1842a38cb2eb6560cbe09083803e30afc28`.
All owned Node/build/staging copies and the clean worktree are removed; source,
emitted output, actual lock, raw logs and receipts remain. Signed cleanup
SHA-256 is `d247335b16f310917425a6a4726df3c59282dbeff1ad41008a569519be47c198`.
The full original cohort
remains red: three export-inventory assumptions, six original MAX_FIELD samples
for the stricter Jubjub scalar, six off-curve Curve25519 samples, and four
encoder/error API differences. These are failed original cases, not exclusions
or passes. A separate signed classification has SHA-256
`57e4b1ace85fe32191f8fd4e5df35aacfc8765ae64da40e7abbb46372febf4eb`.
This is local arm64 JavaScript/API evidence, without Rust, proofs/funded or
native Linux qualification.

## Enum and Bytes encoding refusals

The next independently reviewed local slice `035e85a11b44be8f12ab9e4d0ed032572beab853`
ports the pinned Enum decode diagnostic, Enum integer/range encoding guard and
fixed Bytes width encoding guard. Exactly three further unchanged original
cases change failed to passed: the same cohort is now 908 passed / 16 failed
out of 924, with no skips. Existing source tests remain 469/469; independent
original replay passes 3/3 with 921 filtered. Two builds retain the exact
6,042-file/16-link closure, with only compact-types.js and its map changed.

Signed qualification SHA-256 is
`b37eccb320f57f6c43d47246acc45cc78021fee46a44ba6d2a8bd9a815ead885`;
independent review is
`d8e2a5d75efed6d644d1dace823d82e7ff1820a61111c0c0387128887a0f3258`.
Signed final delivery SHA-256 is
`661bd8f8a4b8c6b8de8e7da63e7aab2e49305228181f38422aabc2138e17cd65`.
All owned build/tool/stage copies and the clean worktree are removed; actual
lock, emitted output, original red results and signatures remain. Cleanup
SHA-256 is `48bafdcf821864ee9cca73b13b782583988c3655d4d2a488f30e5533b73a7c2d`.

The remaining 16 original failures comprise three inventory assumptions,
six strict scalar and six strict point sample differences, and one generic
unsigned encoder expectation. The latter conflicts with the explicitly retained
accepted generic-encoding assertion and remains unchanged. No original failure
is relabeled as a pass or removed. Cast-helper signature/semantics work remains
separate: passing the original stdlib assertions does not establish its full
named diagnostic API. This slice is local arm64 JavaScript/API evidence only.
