# Boolean serialization

`bool.compact` retains the two Boolean exports from the scalar serialization
test at upstream `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
`stateful.compact` retains the `d1` decoder and its ledger write from the
upstream #772 Boolean validation test. That upstream group also covers a
mixed struct. Its wider source and expected output remain separate from
this Boolean slice.

Generate both targets, and make the built Compact TypeScript runtime
resolvable from the generated outputs.

```sh
for fixture in bool stateful; do
  "$COMPACTC" --target ts --target rust --rust-require-recording \
    --feature-zkir-v3 --skip-zk --rust-runtime-root "$PWD" \
    "test-contracts/stdlib/serialization/$fixture.compact" "$OUTPUT/$fixture"
done
zkir-v3 mock-compile-many "$OUTPUT/stateful/zkir"
node test-contracts/stdlib/serialization/check.mjs \
  "$OUTPUT/bool" "$OUTPUT/stateful" "$ZKIR_WASM_PACKAGE" "$OUTPUT/preimages"
```

The runner checks both valid encodings and all 254 invalid byte values
against generated TypeScript and the ZKIR 3.2.0 SDK. It also checks the
public write, rollback on invalid input and rejection of a substituted
Boolean output. Use the two emitted preimages for independent proof
verification and use a separately proved transaction for strict application.

## Fixed Bytes serialization

`bytes.compact` retains the original `serialize_bytes` and
`deserialize_bytes` exports and adds nominal alias, padding and
zero-width cases. `bytes-stateful.compact` reads the first two bytes of
a three-byte input, pads them to four bytes and records the result in a
public cell. The stateful circuit currently measures k=11 and runs in
the local slow proof lane.

Compile both fixtures using the command above, substituting
`bytes bytes-stateful` for the fixture names. Then run:

```sh
node test-contracts/stdlib/serialization/check-bytes.mjs \
  "$OUTPUT/bytes" "$OUTPUT/bytes-stateful" "$ZKIR_WASM_PACKAGE" "$OUTPUT/bytes-preimages"
```

The runner checks the output and public write for all three cases and
rejects substituted prefixes and nonzero output padding in the SDK.
It writes the binary ZKIR and preimages into a new directory for proof
generation and independent verification.

## Native Uint serialization

`uint.compact` retains the six upstream Uint<8>, Uint<64>, and Uint<128>
exports. It also exercises nominal Uint<9>, wire padding, the zero-only
Uint range, and direct casts at their width boundaries.
`uint-stateful.compact` records a decoded Uint<9> in a public cell.
Compile both targets, then run:

```sh
node test-contracts/stdlib/serialization/check-uint.mjs \
  "$OUTPUT/uint" "$OUTPUT/uint-stateful" "$ZKIR_WASM_PACKAGE" "$OUTPUT/uint-preimages"
```

The runner checks little-endian encoding, range refusal and state
preservation. It exports the exact SDK preimages and binary ZKIR for
independent proof qualification. The Rust wire lowering in this slice
covers the native u128 carrier.

## Native Field serialization

`field.compact` retains both upstream Field exports and covers nominal
aliases, zero padding, and direct casts from shorter and longer buffers.
`field-stateful.compact` writes the decoded element to a public cell.
The TypeScript and ZKIR runner checks the modulus boundary and emits
preimages for independent verification:

```sh
node test-contracts/stdlib/serialization/check-field.mjs \
  "$OUTPUT/field" "$OUTPUT/field-stateful" "$ZKIR_WASM_PACKAGE" "$OUTPUT/field-preimages"
```

The stateful fixture measures k=9 and runs in the local slow proof lane.

## Aggregate deserialization

`products.compact` retains the five upstream struct, tuple and Vector
decoders. It also covers a nested nominal tuple and an empty Vector whose
input still evaluates a check. `aggregate.compact` retains the upstream
`d1` and `d2` source and their ledger write. Compile both targets and run:

```sh
node test-contracts/stdlib/serialization/check-aggregate.mjs \
  "$OUTPUT/aggregate" "$ZKIR_WASM_PACKAGE" "$OUTPUT/aggregate-preimages"
```

The runner checks Boolean offsets 0, 3 and 5 with bad bytes 2 and 255,
little-endian Uint fields, rollback and rejection of a changed output.
It exports binary ZKIR and three preimages for independent proof
verification. The stateful `d2` fixture runs in the local slow proof lane
at k=11.

## Aggregate serialization

`encode-products.compact` retains the five upstream struct, tuple and
Vector encoders, including the exact byte Vector assertion. It also
covers nested nominal products, zero padding and an empty Vector with a
checked input. `alias-roundtrip.compact` retains upstream's `foo` and
`unfoo` with their ledger operations. Compile both targets and run:

```sh
node test-contracts/stdlib/serialization/check-encode-products.mjs \
  "$OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/encoding-preimages"
```

The runner preserves the original alias roundtrip and tests ignored decode
padding, an invalid Boolean byte and changed proof output. The two stateful
circuits use the local slow proof lane at k=12 and k=13.

## Enum serialization

`enum.compact` retains the original Color encoder and decoder. Additional
cases cover nominal aliases, padding, the zero-byte single-variant encoding
and checked direct casts. `enum-stateful.compact` decodes a byte before
writing the enum to the ledger. The runner checks all 256 ordinal byte
values and emits three SDK preimages at k=7.

```sh
node test-contracts/stdlib/serialization/check-enum.mjs \
  "$OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/enum-preimages"
```

## Wide Uint serialization

`wide-uint.compact` covers little-endian Uint<129> and Uint<248> codecs,
nominal aliases, nested products and direct casts. Serialization pads
with zeros and deserialization ignores the trailing wire padding.
Direct casts check any bytes outside the destination width.
`wide-uint-stateful.compact` writes the decoded Uint<129> to the ledger.
The SDK runner exports three preimages at k=8. It checks that matching
overflow statements fail at the bound even when the ledger write and
return agree with the input.

```sh
node test-contracts/stdlib/serialization/check-wide-uint.mjs \
  "$OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/wide-uint-preimages"
```

## Secp256k1Base serialization

`secp256k1-base-wire.compact` covers nominal aliases, nested products,
zero padding and modular decode of 32 little-endian bytes. The runner
tests six boundary values and rejects changed proof outputs. The
stateful write uses the explicit local slow proof lane at k=10.

```sh
node test-contracts/stdlib/serialization/check-secp256k1-base.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/secp256k1-base-preimages"
```

`foreign-field-wire.compact` covers the other five 32-byte fields.
It exercises the distinct moduli, a mixed product with a nominal
Curve25519 scalar, output padding and ignored input padding. Each
stateful write uses the explicit k=10 slow lane.

```sh
node test-contracts/stdlib/serialization/check-foreign-field-wire.mjs \
  "$CONTRACT_OUTPUT" "$ZKIR_WASM_PACKAGE" "$OUTPUT/foreign-field-preimages"
```

`empty-point-vector.compact` covers the upstream zero-length
vector layout for an element type without a wire codec. Encoding
pads the result with zeros. Decoding ignores the supplied padding.
The compiler regression suite also checks a nested empty vector and
the refusal for a nonempty vector of points.
