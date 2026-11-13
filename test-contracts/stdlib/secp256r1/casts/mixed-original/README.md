# Original mixed foreign-field casts

The two programs are exact evaluations of the two mixed K1/P-256/Curve25519
Scheme source cases at pinned mainstream
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`, apart from the repository
license header. Their full original Scheme source and JavaScript assertions
remain in `../constant/upstream.ss.txt` and `../unused/upstream.ss.txt`.
The runner checks both archives against the pinned `compiler/test.ss` Git
object and re-evaluates their source lines and numeric interpolations before
compilation. `check.mjs` retains all fourteen ledger assertions. The Rust
consumer checks the same original calls and outputs. The runner also executes
the original archived JavaScript callbacks directly, changing only their
TypeScript `: any` annotation and adapting the async call-context wrapper.

The original constant case contains `(x < y) == (x < y)` inside `castNative`.
The old stateful Rust emitter panicked while parsing that valid expression.
The fix retains the nested comparison syntax tree. A unit regression covers
equality and inequality; the intact mixed source qualifies the emitted API.

The explicit proof lane selects the original `castNative(zeros, 1, 2)` call.
It checks the TypeScript preimage against the emitted ZKIR, rejects a changed
cell write, generates a fresh k=8 proof, verifies independently with wrong
verifier, changed-binding and corrupt-proof controls, then separately proves
and applies a funded Ledger 9 transaction with default strictness. The Rust
recording agrees with the TypeScript input and transcript and rejects replay
without changing ledger state. The
[Curve25519 proof lane](../../../../../tools/compact-rust-backend/check_upstream_ledger9_mixed_curve_proofs.py)
also checks the two original `castC25519(false/true, 7, 8)` calls and the
`castWide()` reduction to five. Each has a separate honest and changed-write
preimage check, fresh proof with independent verification and three negative
controls, and fee-funded default-strict transaction with native state parity
and unchanged-state replay refusal. Its proof runner is rebuilt from the
archived locked Rust source. The other eight original exports have generated
execution and mock circuit coverage; their full proof and funded matrix remains
open. All reported results are local macOS arm64; native Linux remains required.

Run the complete lane with the signed source's Rust CLI and a Scheme compiler
built from the same unchanged frontend source, Compact Runtime 0.17.0, ZKIR
3.2.0 and Ledger 9.1.0.1 material:

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_mixed_foreign_casts.py \
  --compiler "$COMPACTC_RUST" --scheme-compiler "$COMPACTC_SCHEME" \
  --ts-runtime "$TS_RUNTIME" --zkir "$ZKIR" --zkir-wasm "$ZKIR_WASM" \
  --proof-runner "$PROOF_RUNNER" --material "$MATERIAL" \
  --wrong-verifier "$WRONG_VERIFIER" --output "$OUTPUT" \
  --cargo-target "$CARGO_TARGET_DIR"
```

For the extended Curve25519 lane, run
`check_upstream_ledger9_mixed_curve_proofs.py` with the same arguments and add
`--proof-source "$PROOF_SOURCE"`, pointing to the locked source directory of
the archived standalone proof runner, plus `--wrong-verifier-k10` and
`--wrong-verifier-k11` paths to different-circuit verifier keys from the same
respective k. The material directory's parent must contain its
`material-receipt.json`. The driver runs the original family lane first
and writes separate source, preimage, proof, funded and replay receipts.
