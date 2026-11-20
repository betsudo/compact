# Upstream secp256k1 field arithmetic

[Upstream `compiler/test.ss`][upstream] lines 81763–81979 are retained
byte-for-byte in `upstream-arithmetic.ss.txt`. This excerpt contains the
six arithmetic circuits, their reduction-stage expectations, all 34
TypeScript runtime assertions, and the complete issue #866 regression for
self-subtraction.

`field-arithmetic.compact` is the original upstream Compact program.
`self-subtraction.compact` keeps the original secp256k1 scalar ledger, helper
and three exports from issue #866. The P-256 and Curve25519 cases remain in
the archived test. `cases.mjs` retains the upstream inputs and expected
values in their original order, with the pinned secp256k1 moduli.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT"
```

The runner emits TypeScript and Rust together and executes the original
result assertions on generated TypeScript through the local synchronous test
context. `alias-guard.compact` also checks repeated arithmetic inside an
active or inactive ledger write and subtraction through a nominal type.
Four negative fixtures check refusal of mixed foreign fields, a native
`Field`, a `Uint<8>`, and a mixed nominal/base pair. Each refusal is checked
in TypeScript, Rust, and combined target modes without a published output.

This runner uses `--skip-zk`. Its positive artifacts are in
`$OUTPUT/arithmetic/{field-arithmetic,self-subtraction,alias-guard}/ts-rust/.build` for
independent generated Rust execution and raw preimage checks.

[upstream]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81763-L81979
