# Upstream Ledger 9 secp256k1 fields

The 15 fixture files under `base/` and `scalar/` retain the bytes from the
[pinned upstream tree][upstream]. This family declares six cast refusals and
one ledger roundtrip. Run the original diagnostic and runtime assertions
with the built compiler and the built TypeScript runtime. The runtime package's
`node_modules` must contain the pinned Vitest install.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT"
```

Use an absolute compiler path and a fresh output directory. The runner
exercises the exact six refusal diagnostics on TypeScript, Rust, and both
targets. It invokes the original base-field Vitest callback on generated
TypeScript with the synchronous runtime's positional context API. Compiler
logs, generated artifacts, and test-runner cache live inside the supplied
output directory. Key generation is skipped in this test.

The same command runs the [positive Bytes<32> cast fixtures](positive-casts/README.md)
and the [field arithmetic fixtures](arithmetic/README.md).

[upstream]: https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/stdlib/secp256k1
