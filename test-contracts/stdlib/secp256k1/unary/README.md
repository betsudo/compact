# Upstream secp256k1 unary field operations

The two original `wantProof` tests from [pinned upstream `compiler/test.ss`][scalar]
are retained byte-for-byte in `upstream-scalar.ss.txt` (lines 58986–59016)
and [`upstream-base.ss.txt`][base] (lines 59668–59698). Both retain the
original ZKIR 3.1 expectations. Each corresponding `.compact` file executes
the original source with its original `test` export. The runner checks the
typed input, output order, and operand links in the generated ZKIR 3.0.

`unary-controls.compact` covers `neg(0)`, an inactive inverse with zero,
an active inverse, and an unused inverse whose zero failure must precede a
ledger write. A pure export also discards an inverse and must refuse zero.
The fixture calls local Boolean overloads of `neg` and `inv`
alongside the native field overloads. `cases.mjs` supplies independent modular
results and zero refusal cases for both fields. The four invalid-source
fixtures cover unsupported operands and mismatched foreign field return
types.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT"
```

The three positive fixtures compile to TypeScript and Rust together. The
runner executes generated TypeScript and checks that the emitted ZKIR
retains discarded inverses. The invalid-source fixtures run in TypeScript,
Rust, and combined modes and must refuse without publishing output.

Key generation is skipped. Paired artifacts are available under
`$OUTPUT/unary/{upstream-base,upstream-scalar,unary-controls}/ts-rust/.build`
for independent Rust, preimage, and proof checks.

[scalar]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L58986-L59016
[base]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L59668-L59698
