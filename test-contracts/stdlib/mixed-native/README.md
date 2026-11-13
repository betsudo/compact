# Pinned mixed native collections

The `k1` and `r1` Compact bodies and callbacks are preserved as literal
strings in `compiler/test.ss`, ported from mainstream commit
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The exact-head preflight
gate extracts their original bytes, checks pinned SHA-256 digests and writes
the intact fixtures to its evidence directory. The local compiler suite
adapts the call-context shape at its test harness boundary; the original
source, assertions and expected values stay unchanged.

Both original `test(3, 4)` circuits compile for Rust and TypeScript. They
exercise a Jubjub point in a nested Map alongside secp256k1 (`k1`, k=17) or
P-256 (`r1`, k=18) values. Their generated Rust, Rust IR, capabilities, and
raw ZKIR matched the previously qualified outputs at the Runtime 0.18.0
compiler head. The TypeScript output now carries the Runtime 0.18.0 version
check and circuit-signature and declared-interface metadata.

`capture-ledger9-mixed-original-proofs.mjs` records the original call with
Runtime 0.18.0 and Onchain Runtime 4.0.1, checks its ZKIR 3.2.0 preimage,
and refuses a changed nested Map key. A zero `Uint<32>` key has an empty
canonical byte payload; the negative control replaces that payload with 1.

The source-bound k=17/k=18 proof and funded transaction gates are tracked
separately. Local macOS results do not replace native Linux qualification.
