# Original runtime resolution coverage

Reviewed signed `127f75a1293e7cd2c49955490d2abecf9f9fbf93` retains the exact original `runtime/test/cross-contract-resolution.test.ts` from pinned mainstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e`. Its 14 cases cover provider/load failures, missing or unusable exports, reentry and refusal before callee state is recorded.

| Profile | Original results | Meaning |
| --- | --- | --- |
| Published Compact runtime 0.21.0 / onchain runtime 4.0.1 | 14 passed, 0 failed | Original runtime baseline qualified locally. |
| Accepted source `3eaf`, runtime 0.18.0 | 0 passed, 14 failed | Public `crossContractCall` is deliberately absent; all 14 report the missing function. Source compatibility remains open. |

The test body and assertions are untouched. A private import alias outside the body selects the actual profile. Published execution authenticates the six-package 416-file graph. Source execution authenticates its separate emitted 84-file build and 275 runtime dependency files. The source runner records the nonzero child exit, zero passes and 14 failures, and requires the observed closed export; its successful boundary check is not an original-suite pass. Internal preflight analogues are also separate from original coverage.

Independent review reran published 14/14 and source 0/14. Original-byte and runtime-code mutations refuse before execution. Review removed a preset success label from diagnostic reports: a genuine failed child now retains its nonzero exit/counts without an `original-tests-passed` field. The earlier signed `362cfcb3e` receipt remains historical.

Signed `qualification-127-original-cross-contract-14-v2.json` SHA-256 is `a0386b76938b460c634549d8b3e6f9b2d190ea82b1f52d88771cde39afffcf14`; independent review SHA-256 is `6c53fec6c2915a5013558e166ae68f4a1fe7b2aa60d0806729a72e58b9064320`. Source secp256k1 30/30 remains a separate regression result.

## Existing source runtime suite

The existing accepted `3eaf` runtime suite separately passes all 465 cases across 24 modules, with no failures, skips or todos. These include adapted and internal tests. Direct Vitest executes source TypeScript modules; the emitted 84-file build and 275 dependency files are authenticated before and after, alongside all 145 archive source files. Three exact Git read fixtures are temporarily staged and removed; no generated EntryPoint invocation occurs.

The final script requires the actual summaries `24 passed (24)` modules and `465 passed (465)` cases. Earlier discovery-only module counting and an emitted-path scope description were corrected in a new script/receipt, preserving the historical pair. Independent review reran the full suite and verified fixture cleanup.

Signed `qualification-3eaf-full-runtime465-v2.json` SHA-256 is `b0ee68f3398cbea33a23be137932db318a11cb4430a131e5ede2904c8207a0a3`; independent review SHA-256 is `dc65fd559090e1ea546b9e0c91cf582d44815a6af996b9cfe59b6359e242a854`. This count overlaps other source tests and must not be added to the [115 intact original curve cases](original-runtime-curves.md) as unique original coverage.

These are local Darwin arm64 host results. Generated Rust, public compiler compatibility, fresh proofs/funded replay and native Linux remain separate. The source export boundary is retained while secure admission is developed in [#730](https://github.com/betsudo/compact/issues/730); [#727](https://github.com/betsudo/compact/issues/727) tracks the complete census. Finished owned Node dependencies, build scratch and clean worktree were removed; source refs, signed evidence, build outputs and dependency snapshots remain.


## All twelve original modules on one current source graph

Signed runner `82cea75977c194b52d12689ba0265f7fc8655c3b` compares the unchanged
pinned original bodies against released runtime 0.21.0 and source runtime
0.18.0 at `96b4ce8035654025076477a6ad1f52251e6b16d2`. Each profile collects
all 1,091 cases with no skips, todos or load gaps. Twenty-four raw module
reports and their case/status multisets are retained. The complete comparison
supersedes the earlier mixed-head census for current runtime coverage.

| Original module | Cases | Released passed/failed | Source passed/failed |
| --- | ---: | ---: | ---: |
| compact-types | 833 | 829 / 4 | 817 / 16 |
| conformance | 58 | 58 / 0 | 58 / 0 |
| cross-contract-resolution | 14 | 14 / 0 | 0 / 14 |
| cross-contract-zswap | 9 | 9 / 0 | 9 / 0 |
| curve25519 | 44 | 44 / 0 | 44 / 0 |
| error | 5 | 5 / 0 | 5 / 0 |
| module-resolution | 14 | 14 / 0 | 14 / 0 |
| schnorr | 2 | 2 / 0 | 2 / 0 |
| secp256k1 | 30 | 30 / 0 | 30 / 0 |
| secp256r1 | 41 | 41 / 0 | 41 / 0 |
| stdlib | 31 | 29 / 2 | 31 / 0 |
| verifier-key-hash | 10 | 10 / 0 | 10 / 0 |
| **Total** | **1,091** | **1,085 / 6** | **1,061 / 30** |

Both original suites are **red**. Released failures are four original BOM
assertions and two wide-coin assertions. Source failures are three accepted
extra-export inventory differences, six strict scalar boundary differences,
six strict off-curve Curve25519 differences, one accepted generic unsigned
encoding difference, and the deliberately absent public crossContractCall
export in all fourteen resolution cases. No original fixture, assertion or
refusal was removed to produce these counts. Secure public admission remains
tracked separately; this report does not authorize opening that export.

This is fresh current-head evidence for all 115 original curve cases as well.
The separate 474-case source suite overlaps originals and adds internal/API
controls; it must not be summed into unique original coverage. Historical
924-case subset results remain valid at their named source revisions.

Signed qualification SHA-256 is
`4c5457c6ff878c87993f58159311d7fe0f0096bb1180d698451cf32acb16c135`;
independent exact-source/raw-report/closure review is
`20fda95baad27633d49239d254409fa6cc4f07328597e3df2b5b330197bb1944`.
The reviewer verified all raw reports and inventories without rerunning Node.
The owned 148MiB tool/stage copies and clean worktree are removed after audit;
cleanup SHA-256 is
`b474a80e3e7f4f587bf2104e7039a08908c97bbab2ef8b0d522cdf10bf645f28`.
Source branch, actual runtime/build graph, raw reports and signed receipts
remain. These are Darwin arm64 host results; generated Rust, proof/funded and
native Linux qualifications remain separate.
