# Original curve runtime interoperability

Reviewed signed `3f10698336653055b060dbdf9054a3fcb0e85e3c` runs two intact runtime modules from pinned mainstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e` against the published SDK graph.

| Original module | Unchanged cases passed | Runtime graph |
| --- | --- | --- |
| `runtime/test/secp256r1.test.ts` | 41/41 | Compact runtime 0.21.0, onchain runtime 4.0.1, Noble curves/hashes 2.2.0 |
| `runtime/test/curve25519.test.ts` | 44/44 | Same declared graph |

Original bodies, assertions and refusals are byte-identical. A private Vitest plugin redirects only their original `../src/index.js` import to the physical published runtime entry, outside the test bodies. The runner verifies an externally signed parent receipt, the exact six-package 416-file graph and actual Node module resolution before execution. Independent review re-executed all 85 cases at the final signed head.

Review found that the first runner recorded child failures without propagating them. The final runner writes diagnostic logs and its report, then requires successful child exit, the exact expected passed/total count and zero failures. A genuine failed child exits 7 and now makes the runner exit 1 while preserving diagnostics. Wrong keyring, changed receipt and changed runtime member are refused before staging output. The earlier signed `4fc222f5b` receipt remains historical and immutable; its positive logs remain valid, but it lacks this failure guard.

Signed `qualification-3f-original-runtime85-v2.json` SHA-256 is `dc2ff55dba211dd846b8ef32aeaec09388ae9b7d5877a79a5c0c8d32b3423dc4`. Independent review SHA-256 is `090049e73bd63ad4f87c00ba66cc4a09583430818fecf5efaf82792cd68dcafc`. Four source files and 39 evidence files are bound. The published graph parent is the [combined host gate](original-host-metadata-status.md), signed receipt SHA-256 `35f14e82ead10ef4ea01926157d63b5e44b28fb31ec508293c866b1bf15188cb`.

This is local Darwin arm64 pure JavaScript runtime qualification. It does not qualify the current source runtime, generated Rust, ZKIR, proofs, funded transactions, replay, a public compiler, or native Linux. The current-source original P-256 runtime run remains a separate next gate. [Crypto compatibility #729](https://github.com/betsudo/compact/issues/729) and [coverage #727](https://github.com/betsudo/compact/issues/727) retain the broader work.
