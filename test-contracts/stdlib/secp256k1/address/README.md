# secp256k1 Ethereum addresses

The two contracts and their original test blocks come from
[LFDT-Minokawa/compact at 672bd0f](https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e).
The issue #608 fixture writes an Ethereum address and independently
hashes two nested secp256k1 points. The byte-order fixture returns and
stores the upstream 20-byte known answer. The paired tests also compare
the stored values, distinguish reversed coordinate bytes, and reject
identity representations with the original assertion.

The Rust backend handles fixed `Bytes<N>` to `Vector<N, Uint<8>>`
conversion and constant-offset fixed-length byte slices for these
contracts. The checked IR validates the input carrier, vector length,
and slice range before emitting Rust.
