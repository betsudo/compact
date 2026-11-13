# Upstream Ledger 9 Bytes map

The twelve files under `reordering/`, `slice_composition/`, `to_bytes/`, and
`transformation/` are byte-identical copies of the [pinned mainstream Bytes
map fixtures][upstream]. Each original compile-pass and runtime-pass
declaration remains at its source path. The generated adapters execute the
original Compact assertions and compare all return bytes with the original
runtime expectations.

The Rust AST backend lowers constant Bytes indexing through a checked one-byte
slice and Uint conversion, without converting the whole Bytes value. Unary map converts Bytes
operands to their byte vectors and coerces elements to the map formal type.
The `to_bytes` case also requires checked Field-to-Uint downcasts. The local
`field_to_uint_checked.compact` supplement verifies that `255` succeeds and
`256` refuses on generated Rust and TypeScript.
The `bytes_index_large.compact` supplement reads the final byte of a
`Bytes<4096>` value and checks that generated Rust does not convert the entire
value to a vector of bounded integers.

Run the source-bound local gate from a clean signed checkout:

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_bytes_map.py \
  --compiler-receipt "$COMPILER_RECEIPT" \
  --ts-runtime "$TS_RUNTIME" --output "$OUTPUT"
```

The gate checks an exact-head offline Nix compiler recipe, all original file
bytes against SHA-256 pins taken from the upstream commit, default and paired
TypeScript output, ordinary and recorded generated
Rust execution, and a locked offline Rust consumer. It realizes the exact-head
Nix TypeScript runtime package without builds or substitutions, checks every
supplied runtime `dist` file against it, and executes the Nix package. Paired compilation uses
`--skip-zk`; these four stateful circuits have proof statements, but this gate
does not produce or verify proofs or apply funded transactions. Dynamic Bytes
indexing remains a separate Rust lowering gap. Native Linux qualification is
still required.

[upstream]: https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e/test-contracts/primitives/bytes/map
