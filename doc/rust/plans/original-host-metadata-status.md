# Original host metadata qualification

The pinned upstream host tests are separate from generated Rust and transaction qualification. The local implementation checkout and the canonical documentation checkout also have different source inventories. The signed coverage audit `audit.json` (SHA-256 `082391e8e7de9afdc006b5b6f1e67f5c9fb9a7bc216d3d000098dd2b6f3a4249`) records the exact heads: accepted `3eafaf3eb` contains all 943 pinned `test-contracts` files byte-for-byte; among 45 E2E test modules, three are exact, 37 adapted and five absent. Presence does not establish execution. Focused receipts do not qualify the complete root lint/test or E2E package. [#727](https://github.com/betsudo/compact/issues/727), [#728](https://github.com/betsudo/compact/issues/728) and [#732](https://github.com/betsudo/compact/issues/732) retain coverage, frontend metadata and host/Linux ownership.

## Current-source expectedVk

The unchanged original five-case verifier-key host body passes at independently reviewed signed `94b97c691f51924fa02640735614ff951c3f6edb`, on a separate branch from accepted `3eafaf3eb`. It uses the private source Scheme profile selecting ZKIR v3, released Compact runtime 0.21 and physical verifier-key metadata. Three circuits have retained real key bytes, generated JavaScript and compiler manifests; the released onchain SDK 4.0.1 setter case passes. Thirty-one focused metadata guards also pass.

The metadata-enabled emitter now uses computed literal keys and immediately sets the table prototype to null. This preserves an own `__proto__` data entry and the null-prototype property while allowing the unchanged original Acorn reader to inspect the literal initializer. The original test and fixture remain byte-identical to the pinned upstream revision. Bare public CLI output is explicitly tested to omit metadata, as before; this slice does not change that default.

Signed `qualification-94b.json` has SHA-256 `06c4a3ce56ae8ed8c390d71cedd54db72854eb71aa1e83fe0e344a75e9d3619a`. The independent review `independent-review-94b.json` has SHA-256 `ad4803846a89ce2494f9c9028a89e5412648ac9782a48a5e056bba8838418cdb`, verifying six source, 85 artifact, ten log and five command hashes. An initial failed Node prototype-check command is retained; its corrected command passes. The older released Compact 0.36.0 five-case receipt remains a separate baseline, not additional current-source coverage.

These are private-profile Darwin arm64 host results. There is no public-default metadata, Rust descriptor, proof/funded or Linux claim. The changed emitter is not integrated into the accepted v4 branch and does not requalify immutable Events v4/v5 or Caller packages.

## Remaining host work

The original compiler-version test also asserts the compiler commit. The adapted local test omitted that assertion, and accepted source lacked a commit producer. A separate bounded slice is restoring the unchanged test and original counter fixture, with released and source results recorded separately. An unstamped source build must report the upstream empty/unknown commit convention explicitly; its evidence-bound Git head must not be presented as a compiler stamp.

Full root prepare/lint/test, the complete E2E package, default fuzzer corpus, remaining editor/formal tooling and native Linux qualification remain open. Existing focused CLI, version-first, fuzzer and released expectedVk results are retained under their own recorded source heads.
