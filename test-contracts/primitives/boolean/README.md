# Upstream Ledger 9 Boolean casts

The entire [pinned upstream Boolean family][upstream] is retained under
`casts/`. Both `.compact` files and their four upstream TypeScript test
definitions are byte-identical to the upstream tree.

| Upstream fixture | Compile expectation | Runtime expectation |
| --- | --- | --- |
| `casts/to_boolean` | Compile with default flags | Return `true`. Internal assertions cover both literals and a comparison result. |
| `casts/to_field` | Compile with default flags | Return `1n`. Internal assertions cover true as one and false as zero. |

Run from the Compact checkout with the local compiler, built TypeScript runtime,
Node, and Rust 1.99.0 and its populated Cargo registry.

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_boolean.py \
  --compiler "$COMPACTC" --ts-runtime "$TS_RUNTIME" --output "$OUTPUT"
```

`TS_RUNTIME` is the package directory containing its `package.json` and
compiled JS. The output directory must be absent. An existing isolated
`CARGO_TARGET_DIR` can be reused.

The local runner uses each original source with default compilation and with
both targets enabled. The TypeScript consumer repeats the upstream assertions
on the synchronous generated contract wrapper. The Rust consumer uses the
generated typed pure circuits and resolves registry packages against the
checkout's Cargo lock. Both-target generation skips key generation. The
runner checks that the original exports are pure and require no proofs.
This family has no upstream refusal cases.

[upstream]: https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/primitives/boolean
