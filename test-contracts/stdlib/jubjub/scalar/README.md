# Ledger 9 Jubjub scalar

The files under `casts/` are from LFDT-Minokawa/compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The Uint fixture covers
zero, one and the maximum Uint<64>. The Boolean and enum fixtures
require the original frontend diagnostics.

The local `ledger/scalar.compact` fixture covers the one-field
encoding, default zero, Field and Uint reduction, and typed Jubjub
multiplication. Its stateful exports write and read the scalar cell.
Compile both targets with `--feature-zkir-v3 --skip-zk`. The local
qualification measured both stateful exports at k=11 with Ledger SDK
9.1.0.1 ZKIR 3.2.0 and proved them with an explicit slow harness. The
pure upstream cast emits no proof.
