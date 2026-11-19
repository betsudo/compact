# Original Events v5 runtime and CLI status

This local continuation carries the reviewed Rust caller runtime into the exact original Events pair. The accepted v4 package, its source-bound proofs and funded receipts, and the frozen v4 Linux handoff remain separate evidence. [Caller #778](https://github.com/betsudo/compact/issues/778), [cross-contract calls #730](https://github.com/betsudo/compact/issues/730) and [Linux #773](https://github.com/betsudo/compact/issues/773) remain open.

## Signed physical candidate

Reviewed source `6ac5b04bbd9be8e2c66b8bcc51e654810904d666` produces a signed v5 candidate at identity SHA-256 `635a6303044c59a1b38370fe5211b6117c24f03b4f1c12c0a7e81fec49eccb9d`: 3,278 files and 154 directories. It preserves the complete signed v4 parent, original source, generated Rust/TypeScript/IR/ZKIR, keys and checked identity bytes. Its runtime projection contains the four reviewed changed files and new caller test, bound to the reviewed 31-file Caller source delta and two candidate-owner files. Five local Cargo manifests are confined; no lock is introduced into the signed candidate. Exact verification and 19 signed tamper controls pass.

The detached qualification `qualification-6ac5b04bb.json` has SHA-256 `49c2a2d99271919445614bbabf8e88d9a68b5b1046c3faffa1a6388715c20389`. Independent review passed. The earlier 408f package is historical. The candidate's immutable `candidate-no-cli-admission` status records its preparation stage; later qualification is recorded externally rather than rewriting signed bytes.

## Current-runtime CLI admission

Reviewed signed source `73fb2ab802e34833b86c265b919d2236e94d09e3` adds a self-contained v5 verifier and shares the authenticated, atomic admission flow with v4. The actual compiler admits v5 against its selected caller runtime, passes 18 controls and strict all-targets Clippy, and publishes 3,293 files/159 directories. A second actual positive with the archived signed v4 runtime and package passes, with all 13 v4 controls preserved. The ordinary full Outer continues to refuse without changing admitted output.

The signed receipt `qualification-73fb.json` has SHA-256 `dec02cb4c96ec6677749a7ffd6ac49ca6d69586a4e7306ed1246a7fd1c9af2fd`. Its retained compiler binary has SHA-256 `fcaabcdfbf8f6360d5ea273703cee8c708914a3ab99a1af4be196097c4dcae4e`. Independent review verified both output inventories, exact signed artifact bytes, Cargo relocations, source-map normalization, provenance/lock hashes, five confined local manifests, commands and logs.

## Remaining qualification

These are Darwin arm64 physical/CLI/Cargo results using the source Scheme wrapper. They do not establish fresh v5 native execution, proofs, independent proof verification, funded strict application/replay, packaged compiler or native Linux. The old v4 proof/funded receipts retain their original source and identity; they are not relabeled as v5 results.

The next admitted-output consumer compiles `admitted-output/contract` and uses the signed `admitted-output/events-v5` Inner/runtime/identity/key closure. Its no-build preparation at signed `3a5c40093` passes independent review and nine controls. Signed preparation SHA-256 is `72d25003dabc506073322625763089eb386f812c1386d986e4fa22ea46431c5c`; signed delivery SHA-256 is `89499f55e46a08b6c1fe2678647773914fa49f64f56973779deb05321a2d58cf`. No native or proof execution is claimed by preparation. Proof/native modes retain the 50 GiB free-space guard. Caller `viaInner` uses another runtime coordinator extension and needs its own signed physical material; v5's runtime identity cannot silently cover that extension. The broad milestone remains incomplete.
