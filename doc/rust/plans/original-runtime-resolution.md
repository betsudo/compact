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
