# Pinned Bytes-to-Vector casts

The four `to_vector*` directories contain all 11 original source, compile and
runtime test files from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`, unchanged. The adjacent
`original-to-vector-sha256.json` binds their bytes to that revision.

The original `to_vector`, `to_vector_default`, and
`to_vector_wide_element` Compact circuits include their own element
assertions. `check_to_vector.mjs` and `consumer_to_vector.rs` repeat the
original result assertions in generated TypeScript and Rust. The checker
compiles each source as TypeScript-only and paired TypeScript/Rust, compares
TypeScript output, then checks the original `to_vector_underflow` compile
refusal and absence of published output on both targets. It requires a clean
exact-head compiler recipe and the exact-head Nix TypeScript runtime output.

The Rust IR now emits a checked Vector conversion after decoding Bytes into
`Uint<8>` elements when the accepted cast widens the element type. This keeps
`Uint<16>` and `Uint<32>` carriers and preserves the original values. The
`Uint<7>` refusal remains in the frontend.

The slice covers pure circuits on local macOS arm64. It does not qualify
stateful widening, proofs, funded transactions, or native Linux execution.
