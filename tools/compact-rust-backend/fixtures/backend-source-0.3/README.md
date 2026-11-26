# Backend source migration controls

`old-exhaustive.rs` preserves the complete 105-variant `Expr` match of backend
0.2.0 at published9db7. Compile it against that exact old backend as the positive
baseline, then against current0.3.0 as an expected non-exhaustive-match refusal.
`migrated-fallback.rs` handles typed time explicitly and retains a wildcard;
compile it against current0.3.0. These are Rust-source compatibility controls,
not generated/runtime ABI, SDK execution or proof qualification.

All compilations are pending. Use a fresh owned temporary consumer/target,
Rust1.99, `--offline --locked`, and the authenticated published/current source
roots. Preserve the actual diagnostics. A dependency/build error is not the
expected exhaustive-match refusal. Keep the old source unchanged.
