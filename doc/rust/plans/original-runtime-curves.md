# Original curve runtime interoperability

Three intact runtime modules from pinned mainstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e` now pass independent review against both the published SDK graph and the emitted accepted source runtime. These are 115 unique original cases, run on each profile in separately bound slices.

| Original module | Published runtime 0.21.0 | Accepted source runtime 0.18.0 |
| --- | --- | --- |
| `runtime/test/secp256r1.test.ts` | 41/41 | 41/41 |
| `runtime/test/curve25519.test.ts` | 44/44 | 44/44 |
| `runtime/test/secp256k1.test.ts` | 30/30 | 30/30 |

Published P-256/Curve25519 qualification is signed `3f10698336653055b060dbdf9054a3fcb0e85e3c`. Its graph uses Compact runtime 0.21.0, onchain runtime 4.0.1 and Noble curves/hashes 2.2.0. Source qualification at `38a554df3f869aba76791f7915331ad662e4e2f2` builds the exact accepted `3eafaf3eb92d6c7a39ba93ff6d41df11eb352643` runtime/compiler/third-party archive with its own npm lock: runtime 0.18.0, onchain runtime 4.0.1, Noble curves 2.3.0/hashes 2.2.0, TypeScript 6.0.3 and Vitest 4.1.4. The older canonical docs checkout's runtime 0.17 is not this accepted implementation.

Original bodies, assertions and refusals are byte-identical. A private Vitest plugin redirects only their original `../src/index.js` import to the physical published runtime entry, outside the test bodies. The runner verifies an externally signed parent receipt, the exact six-package 416-file graph and actual Node module resolution before execution. Independent review re-executed all 85 cases at the final signed head.

The separate source gate redirects that import to actual emitted source JavaScript. Two builds produce the same 84-file output. An externally signed build attestation binds those files and the 275 runtime dependency files; the runner authenticates the complete physical maps, directories and absence of symlinks before execution. Source, emitted-helper and same-version dependency-code drift refuse before staging output. An earlier `de75` runner checked only source bytes and package version labels; its genuine positive logs remain historical, while `38a` closes that artifact-binding gap. Independent review re-executed the source 85 cases.

Review found that the first runner recorded child failures without propagating them. The final runner writes diagnostic logs and its report, then requires successful child exit, the exact expected passed/total count and zero failures. A genuine failed child exits 7 and now makes the runner exit 1 while preserving diagnostics. Wrong keyring, changed receipt and changed runtime member are refused before staging output. The earlier signed `4fc222f5b` receipt remains historical and immutable; its positive logs remain valid, but it lacks this failure guard.

Signed `qualification-3f-original-runtime85-v2.json` SHA-256 is `dc2ff55dba211dd846b8ef32aeaec09388ae9b7d5877a79a5c0c8d32b3423dc4`. Independent review SHA-256 is `090049e73bd63ad4f87c00ba66cc4a09583430818fecf5efaf82792cd68dcafc`. Four source files and 39 evidence files are bound. The published graph parent is the [combined host gate](original-host-metadata-status.md), signed receipt SHA-256 `35f14e82ead10ef4ea01926157d63b5e44b28fb31ec508293c866b1bf15188cb`.

Signed source `qualification-38a-source-runtime85-v2.json` SHA-256 is `937bb8443c31352a392da1bf20528d99a6ffa22931a87bc79ff20f87922bf4c5`; independent review SHA-256 is `3aa2d6825d953cbc23f5c671d6c2bb3244cee3d7a16c565a3c8893fb72060082`. It binds 145 archive source files and 456 evidence files. The separate signed build attestation SHA-256 is `bc4515d649ca4e3a6389494776f8595e15da3e8a420108fa3632196fa72c9732`.

Reviewed signed `2ccfcb86c9cf4309adf8913348d3bc8514ae945d` adds the unchanged original secp256k1 30-case module. All 30 pass on each graph, with changed published/emitted-code refusals. Independent review reran both 30-case suites and the source P-256/Curve25519 85-case selector regression. Signed `qualification-2cc-original-k1-30.json` SHA-256 is `f23e722398e8ba8474958fdc83a96786a38f351a05157aad8a64edbeb26dbf36`; independent review SHA-256 is `8be48bfe0b677d3c064152723f039986f669def8309066997bd357dca3e502bd`. Earlier 85-case receipts keep their own source identities.

This is local Darwin arm64 pure JavaScript runtime qualification. It does not qualify generated Rust, ZKIR, proofs, funded transactions, replay, a public compiler, or native Linux. Malformed foreign-field host atoms remain a separate compatibility limitation: both TypeScript profiles accept trailing-zero atoms that Rust currently refuses; these originals do not test that case. Canonical ledger/proof decoding remains unchanged pending a paired boundary decision. [Crypto compatibility #729](https://github.com/betsudo/compact/issues/729) and [coverage #727](https://github.com/betsudo/compact/issues/727) retain the broader work.

## Six original serialization callbacks through Rust

A separate reviewed local slice at signed `df89e5c8282cbd5a3cdd888e3f12c38562451a2b`
runs six unchanged original Curve25519 serialization callbacks through an actual
Rust runtime API bridge. The other 38 original callbacks are filtered and remain
unqualified on this Rust path. Independent review replayed 35 raw Rust calls and
two additional trailing-zero diagnostic calls against the retained native binary.
The JavaScript bridge marshals values and errors; it supplies no cryptographic
fallback or expected constants.

Signed qualification SHA-256 is
`33d25fefd22d554bba533bbebbdd7b0a6e423164a34b2d735695b4672245f676`;
independent review SHA-256 is
`eb4bbf6d3616e6094159b3200b0948900508f3e272875b1654d66d1224fe36cb`.
The stage binds 780 files/21 directories, 770 unchanged non-manifest runtime/macro
files from accepted `3eafaf3eb`, a manifest-only released SDK1 pure dependency
overlay, a 393-package lock and the exact 416-file published Node graph. Optional
ledger is not compiled. Earlier source-preparation races were corrected with
immutable Git inputs and staged Node imports before the final build.

This qualifies six direct runtime API callbacks on Darwin arm64. It does not
qualify generated Rust contracts, all 115 original curve tests through Rust,
full Ledger SDK ABI, proofs, funded replay or Linux. The trailing-zero diagnostic
still records TypeScript acceptance and Rust canonical-decoder refusal. Owned
Cargo, Node, build scratch and the clean worktree were removed; source, binary,
lock and evidence remain retained.

The newly confirmed [Rust zero-width encoding gap](zero-width-ledger9-encoding.md)
is a separate profile-specific serialization issue.
