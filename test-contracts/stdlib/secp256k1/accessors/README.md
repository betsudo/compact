# Ledger 9 secp256k1 point accessors

`upstream-pure.ss.txt`, `upstream-ledger.ss.txt`, and
`upstream-assertion.ss.txt` preserve the complete
pinned upstream [compiler fixtures] and [assertion case]. The corresponding Compact files use
the original source. The tests reuse their identity cases and add the
coordinates of fixed, independently checked multiples of the generator.

`guarded-accessors.compact` makes both coordinate reads proof-bearing.
The tests cover active and inactive identity inputs and a discarded
access whose failure must still be observed. A constructor fixture checks
the same discarded-result behavior while initializing a Cell. The refusal fixtures
exercise the distinct Jubjub point and secp256k1 base field in both
generators. Run the existing family harness from the repository root.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT"
```

[compiler fixtures]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81982-L82066
[assertion case]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L82114-L82140
