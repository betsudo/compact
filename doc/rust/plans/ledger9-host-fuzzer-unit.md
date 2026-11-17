# Ledger 9 host fuzzer grammar unit coverage

This local slice copies seven original files byte-for-byte from pinned
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`:
the TypeScript fuzzer grammar, generator, and
`tests-e2e/src/tests/fuzzer.unit.test.ts`. The SHA-256 manifest in
`tools/compact-rust-backend/ledger9-host-fuzzer-unit.sha256.json` identifies
each original source. The upstream test retains all 68 assertions, including
grammar validation refusals, count fallback cases, generated type bounds,
and ZKIR v3 call/type coverage.

The upstream test is selected by the existing `tests-e2e/vitest.config.ts`
`**/*.test.ts` pattern and the normal `test` script. A focused local run used
Vitest 4.1.10 from the existing npm cache and an isolated config that only
selected the original unit file; it did not build or run a compiler. The
package manifest on this base requests Vitest `^3.2.4`, while pinned upstream
requests `^4.1.11`. Package-resolution and Linux CI remain open.

## Owner disposition

- The E2E fuzzer still imports the older `fuzzers.cjs` path. Its pinned
  replacement, test script concurrency, and README belong to the compiler
  E2E owner. This slice qualifies grammar generation as host-only unit logic,
  not generated contracts against either compiler backend.
- The four CLI goldens remain with the CLI owner. Their pinned assertions
  require absent CLI behavior: compiler help forwarding, dangling-default
  refusal, and interrupted/corrupt archive repair. The base test harness also
  differs from upstream. Copying the output files alone would not exercise
  those cases.
- After this slice, the original host-only inventory has 56 missing paths:
  2 workflows, 4 CLI goldens, 1 fuzzer README, 5 E2E tests, and 44
  test-center paths. These counts include only originally missing paths;
  modified existing files such as the E2E fuzzer integration are separate
  coverage work.

This local result is on macOS arm64. It is not Linux qualification.
