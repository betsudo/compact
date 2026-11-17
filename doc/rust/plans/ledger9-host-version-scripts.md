# Ledger 9 host version scripts

This local slice ports `scripts/read-version.sh` and
`scripts/stamp-compiler-version.sh` byte-for-byte, including executable modes,
from pinned LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The SHA-256 manifest in `tools/compact-rust-backend/ledger9-host-version-scripts.sha256.json`
and its focused test bind the imported files to that revision.

## Qualification and owner disposition

- `read-version.sh` works against this checkout's committed compiler and
  language versions (`0.31.213` and `0.23.105`). The test also retains a custom
  source case and missing-argument/missing-feature refusals.
- `stamp-compiler-version.sh` is exercised with an isolated temporary Scheme
  config. The test checks release, prerelease and development output, complete
  commit/date provenance, and refusal without overwrite for malformed inputs
  or the wrong config library.
- The grouped #747 source does not yet contain `compiler/version-config.ss` or
  the matching `compiler/program-common.ss` version contract. Its release
  workflow does not invoke the stamp script. Compiler/release workflow owners
  must port and qualify that integration before the default stamping path can
  be used. This slice makes no claim about a built compiler's `--version`.
- The four remaining pinned CLI goldens (`compile/compact_help.txt`,
  `list/err_dangling_default.txt`, `update/std_reinstall_incomplete.txt`,
  `update/std_repair_corrupt_archive.txt`) are referenced by pinned upstream
  CLI tests but those assertions are absent here. The CLI owner should port
  their tests with the associated behavior; importing only outputs would give
  no coverage.
- After these two scripts, the host-only inventory still has 63 missing
  original paths: 2 workflows, 4 CLI goldens, 7 fuzzer files, 6 E2E tests,
  and 44 test-center paths. Host-only tooling needs its own owner and Linux
  qualification. The tests here run locally on arm64 macOS only.

Run the local focused checks with
`python3 tools/compact-rust-backend/test_upstream_ledger9_version_scripts.py`
and `bash -n scripts/read-version.sh scripts/stamp-compiler-version.sh`.
