# Nominal Uint arithmetic

The two Compact programs and their four compile/runtime declarations are
byte-for-byte copies of the pinned mainstream files at
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`, introduced by
`4d4753c4274b503b5775199afa3901f68e50662d`.

`same_range_arithmetic` retains nine original inputs and expected results
across multiplication, addition and subtraction on small nominal Uint types.
`addition_overflow` retains a boundary result of 255 and a refusal when the
sum is 256. The three upstream Scheme regression cases are adapted to the
current synchronous TypeScript harness in `compiler/test.ss`.

The generated TypeScript and Rust consumers execute the original assertions
in the local qualification lane. These exports are off-chain pure circuits;
they do not produce ZKIR statements or transaction proofs. The baseline
compiler's internal type-check failure and the exact source and generated
artifact identities are retained in the local evidence receipt.

Run the family from the checkout with the current compiler, built TypeScript
runtime package, Node and Rust 1.99. The output directory must be absent:

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_nominal_uint.py \
  --compiler "$COMPACTC" --ts-runtime "$TS_RUNTIME" --output "$OUTPUT"
```

The runner checks every copied file against the pinned Git object, compiles
both fixtures with the default and paired targets, executes the TypeScript
assertions, and runs a locked offline generated Rust consumer. It adds one
subtraction-underflow refusal on each target beyond the upstream cases.
TypeScript reports the compiler's subtraction assertion; generated Rust
reports `UnsignedUnderflow` from its checked subtraction primitive.
