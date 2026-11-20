# Current-generated composite-zswap-transfer-oracle

`build.rs` renders the complete unchanged
`examples/rust_backend/composite_zswap_transfer_oracle.compact` retained IR using the current backend.
The existing tests preserve independent TypeScript observations for input/output intents, mandatory Kernel claims and private witness/Unit order.

```sh
cargo test -p compact-rust-composite-zswap-transfer-oracle-fixture --all-features
```

For proof qualification, `compact-rust-proof-smoke --composite-zswap ORIGINAL TRANSFER`
binds both complete source IR files, linked Rust, v3 raw and binary IR, and
verifier keys before either proof. The original planned output uses value zero.
The transfer consumes and emits 42 from an explicitly seeded input. Both cases
independently verify a fresh standalone proof, fund deployment and a distinct
call proof with Dust, apply under default strictness, and verify unchanged replay
refusal. The checks retain the exact -42 output-only deficit, extra wallet input
mismatch, missing contract receive claim, unbound intents and spent-input replay.
