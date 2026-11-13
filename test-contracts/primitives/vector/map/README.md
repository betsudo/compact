# Upstream Ledger 9 Vector map

The six files under `semantics/` and `empty_tuple/` are byte-identical copies
of the [pinned mainstream Vector map fixtures][upstream]. Both compile-pass
and runtime-pass declarations remain at their original paths.

The `semantics` circuit zips two length-three vectors, checks all three sums,
then checks unary mapping over the first vector. It returns `[11, 22, 33]`.
The `empty_tuple` circuit maps two empty operands and returns an empty vector.
Neither upstream family declares a refusal case. Both exports are pure and
have no proof statement.

`review_regressions.compact` is a local supplement for two reproduced review
findings: binary map over `Bytes<2>` operands and element widening from
`Uint<8>` to `Uint<9>` or `Uint<16>`. The gate checks three parameterized
circuits on generated Rust and TypeScript. It is not an upstream fixture.

Run the local source-bound adapter from a clean signed checkout:

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_vector_map.py \
  --compiler-receipt "$COMPILER_RECEIPT" \
  --ts-runtime "$TS_RUNTIME" --output "$OUTPUT"
```

It verifies each original file against the pinned Git object, compiles with
the default TypeScript target and with paired Rust/TypeScript targets, runs
the original return and internal assertions through generated TypeScript,
and runs a locked generated Rust consumer. An exact-head Nix compiler recipe
binds the source and binaries. The paired target uses `--skip-zk` because
these pure circuits have no proof statement.

[upstream]: https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/primitives/vector/map
