## A curve name is only the beginning

Supporting P-256 or secp256k1 means more than exposing a verification function. The compiler needs source types, defaults, casts and operations. Generated Rust needs distinct carriers. The runtime needs checked decoders and ledger layouts. ZKIR must receive the intended operands. Witnesses and public interfaces must preserve their representation.

The migration proceeded through those layers in small families.

## Fields before points

Base and scalar fields have different moduli. Rust types preserve that distinction, so an ordinary call cannot accidentally substitute one carrier for another. Explicit source casts have their defined modular behavior. An untrusted host decoder still needs a canonicality check.

Foreign field ledger transport also uses shifted values and split atoms. For the selected K1 and P-256 layouts, the low and high parts occupy 24 and 8 bytes. A plain integer comparison would miss a representation error.

## Before and after · Make the curve part of the type

### Before · A byte string needs context

Consider a host program with bytes from two curves. Both values can be held in a byte array. The caller has to remember which one belongs with the next operation.

### After · A generated call can express the distinction

~~~rust
// Schematic Rust. Names illustrate the roles in the API.
fn multiply_k1(point: K1Point, scalar: K1Scalar) -> K1Point;

multiply_k1(k1_point, k1_scalar);   // The carriers match.
multiply_k1(k1_point, p256_scalar); // Type error.
~~~

The example uses schematic names because the exact API depends on the generated interface. The ordinary typed call rejects the mix-up during compilation. Data arriving from outside Rust still needs the runtime’s canonical decoding and point validation.

## Unary and guarded operations

Inversion of zero must fail on an active path, including when the result is discarded. A proof VM may evaluate an operation eagerly even when source control flow makes its result inactive.

Guarded lowering therefore supplies a typed neutral input to the inactive operation while preserving active-path refusal. These cases show why the source semantics, host runtime and constraint model need to be checked together.

## Points and group operations

The K1 and P-256 families added typed point construction, equality, coordinate access, addition, multiplication and generator multiplication. Tests cover identity, off-curve input, coordinate-specific refusal and cross-curve rejection.

Curve25519 adds subgroup and encoding obligations. Its scalar transport splits at bit 204, and its point transport uses four shifted Base atoms. The original generator and its double passed ordinary Rust, recording and VM replay.

## Signatures, hashes and recovery

K1 ECDSA and Ethereum address derivation retain the original message and byte-order rules. P-256 ECDSA uses the upstream Compact implementation, with known-answer and Wycheproof accounting. Ed25519 uses SHA-512 and canonical scalar conversion, with identity and mixed-order key refusals.

The host recovery helper returns a checked point. In the K1 owner-binding fixture, the recovery ID guides a witness, and the circuit authenticates that point through its signature and derived address. P-256 host recovery remains a host API with its own tests. Applications requiring a specific signer must compare the recovered key to their trusted key.

## Honest corpus accounting

The P-256 report accounts for 484 Wycheproof vectors. It drives 265 through the selected circuit interface and records 219 encoding exclusions. The driven outcomes include true, false, scalar-boundary refusal and abort.

An excluded encoding is part of the coverage report. It should not be counted as a successful in-circuit test. The same discipline applies to intact fixtures whose newer language pragma remains refused.

## Selected proof depth

Local arm64 lanes include independent proofs and funded transactions for selected field, point, group, signature and recovery statements. The sequential signature fixtures also preserve witness order and substituted-key refusals.

The complete original group and mixed-circuit proof matrix remains open. The available typed APIs and selected proof evidence are substantial, with scope determined by each receipt.

### Source trail

- [Cryptography compatibility issue #729](https://github.com/betsudo/compact/issues/729)
- [Pinned upstream](https://github.com/LFDT-Minokawa/compact/commit/672bd0f226f47a7d036c5c074ce71fbe1af1925e)
- [Ledger 9 compatibility plan snapshot](../sources/ledger9-upstream-compatibility.md)
