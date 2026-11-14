# Upstream secp256k1 point representation

The pinned [upstream equality][equality] and [issue #795 identity][identity]
tests are archived byte-for-byte in `upstream-equality.ss.txt` and
`upstream-identity-795.ss.txt`. The corresponding Compact programs are
executed with their original circuit names. The tests use independently
checked 5G and 7G coordinates and the identity in place of calling upstream's
generator multiplication. The #795 fixture covers the original identity
equality, persistent hash, and Cell canonicalization statements.

The exact [SHA-512 test][sha512] and [issue #608 test][keccak] are archived.
The SHA-512 fixture runs the original Compact source. The Keccak fixture
keeps the original `test1` and hash Cell. Ethereum address derivation and
`test0` remain in the archived upstream block. Both maintained tests read
the real generated ledger Cell and check OpenSSL digests of the concatenated
96-byte aligned point representations for 5G and 7G, reversed inputs, and
an identity with canonical or ignored coordinates.

`point-ledger.compact` adds an explicit Cell store, read, and reset sequence.
Malformed generated TS inputs include non-curve finite coordinates,
out-of-range coordinates on either finite or identity points, and invalid
host shapes. Two invalid-source fixtures enforce the point's distinct type
against Jubjub and a structurally similar type.

`upstream-descriptor.ts.txt` preserves [upstream runtime descriptor][descriptor]
lines 59–68 and 115–150. `upstream-descriptor-tests.ts.txt` preserves the
[upstream protocol tests and relevant examples][descriptor-tests] at lines
16–22, 30–34, 121–125, 139–149, 199–207, 263–318, 356–360, and 373–379.
The maintained checks compare five FAB atoms and prefix consumption for the
point descriptor, a vector with two points, and a Merkle path with a point
leaf. This includes the upstream structural codec sample whose five atoms
are empty, independently refused at the generated host point boundary.

```sh
node test-contracts/stdlib/secp256k1/check.mjs "$COMPACTC" "$TS_RUNTIME" "$OUTPUT"
```

The runner emits paired TypeScript and Rust under
`$OUTPUT/point/<fixture>/ts-rust/.build`
and executes generated TypeScript. Invalid-source controls run in
TypeScript, Rust, and combined modes and require no published output. Key
generation is skipped. Group operations have their own compatibility slice.

[equality]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81498-L81536
[identity]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81538-L81581
[descriptor]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/runtime/src/compact-types.ts#L115-L150
[descriptor-tests]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/runtime/test/compact-types.test.ts#L119-L149
[keccak]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81583-L81607
[sha512]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81609-L81626
