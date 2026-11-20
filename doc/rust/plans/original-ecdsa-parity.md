# Original ECDSA API parity

The pinned `compiler.ecdsa.e2e.test.ts` and `examples/ecdsa/example_one.compact` remain byte-identical to mainstream `672bd0f`. The ten original assertions cover valid and tampered Ethereum/Bitcoin signatures, public-key recovery, Ethereum address size and known value, identity refusal and scalar reduction modulo the secp256k1 order. [#729](https://github.com/betsudo/compact/issues/729) retains broader curve/signature coverage.

## TypeScript and published runtime

Reviewed signed `b38761551a3d7aaedfc9f43fc7bcec7d9ffc1b3e` passes all ten unchanged original cases on released Compact 0.36.0 and the private source v3/released-runtime profile. The source profile uses published runtime 0.21.0's `keccak256` and five-argument `convertBytesToField` for the relevant secp casts. The source-runtime helper route remains separate. Six earlier missing-helper failures are retained as historical diagnostics.

Each original run retains its actual five generated files, copied before test import. The declared SDK graph includes compact runtime 0.21.0, onchain runtime 4.0.1 and Noble curves/hashes 2.2.0; 416 package files and actual resolution paths are bound. Signed `qualification-b387-same-run.json` SHA-256 is `4bda6dfa62870ab396c89bc95512e32c7d544581848c3f15d7376fd888d9467c`. Independent review SHA-256 is `026f7f65e50d8057b97be34057862d1ecb68a8eb8e9cb19ce8f6af7ef955b960`. The older cecf receipt retains only diagnostic generated copies and is not same-run artifact evidence.

## Generated Rust

Reviewed signed `50df7ede8cc90abdd945f78bd777291b1d4aa92c` passes the same unchanged ten-case body through an explicit Rust test adapter: nine cases invoke compiled generated Rust functions; off-circuit recovery remains a host case. The adapter only marshals the original arguments and results. It has no TypeScript crypto fallback or constant answers. Actual TypeScript compiler output is preserved before the temporary test-module overlay; the adapter is distinct from compiler-generated JavaScript.

The seven-circuit IR, eight generated Rust files, five pre-overlay TypeScript files, 771 runtime files, SDK graph, Cargo lock, binaries and raw public arguments are bound. Independent replay of all nine Rust calls passes, including identity refusal. Signed `qualification-50df-rust-original.json` SHA-256 is `51b09911a6a96be08a090f04629e424d31b26cfd59b2232bb2bf4ba580a23608`; independent review SHA-256 is `2a387a68bc4560da26b75df58597f9b41635669f96296edc7f581176b5dfe457`.

The source SDK 9.1.0.2 result retains its own identity. Separately reviewed signed `180e68be51976f832a4bf6e9a24d1751ae89c63b` passes the same ten original cases against the registry-only released SDK matrix. All nine retained SDK2 calls also replay byte-identically against the released-matrix binary, including identity refusal. Only the staged generated and runtime Cargo manifests changed; generated Rust and runtime code stayed identical. The 24 resolved Midnight packages match a subset of the signed released baseline, with zero Git dependencies. Optional `midnight-ledger-v9` 1.0.1 is pinned but is not compiled by this pure fixture; this does not qualify the full ledger transaction dependency graph.

Signed `qualification-180-released-rust-matrix.json` SHA-256 is `3c708fbe6fb6069abaffc7ac2c683cd481321ab079d5d6b0dc004d71ac3cf387`; independent review SHA-256 is `cfd87d9c2da7af7f69cb4a0c47a24ac9c3f00678a3ed763dc06cd459e93e4ea7`. Review checked 782 staged files, 771 runtime files and the 416-file Node SDK graph, and independently replayed both sets of nine raw calls.

These are Darwin arm64 results. Four exported circuits are pure, so the normal original compile produces no ZKIR or proving keys. This fixture does not require proof/funded qualification. Broader stateful crypto proofs, public Rust transport, combined-source qualification and native Linux remain open. Owned build targets and finished test worktrees are cleaned; source refs, binaries and evidence remain.
