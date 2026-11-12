# Ledger 9 secp256k1 public-key recovery and owner binding

`upstream-owner.ss.txt` preserves the original compiler test at the
[pinned upstream revision][owner]. `upstream-owner.compact` extracts its
contract source verbatim after the license header. The upstream
[runtime recovery cases][runtime] are archived in
`upstream-runtime-recovery.ts.txt` and run with the TypeScript runtime tests.

The [family runner](../README.md) executes the original owner contract with
both generated targets. It checks low-s and high-s signatures and both
original owner refusals. The host tests also check both recovery IDs for
the wrapped x-coordinate case.

Rust exposes `secp256k1_ecdsa_recover` with `FixedBytes<32>`,
`Secp256k1Scalar` for each signature component and `BoundedUint<3>` for the
recovery ID. The runtime uses the exact `k256` version already selected by
the Ledger SDK. It rejects zero signature components and impossible
recovery points and returns a checked `Secp256k1Point`.

In the Compact fixture, the recovery ID guides the witness. The circuit
authenticates the recovered point through the signature and its derived
address against the stored owner. Proof qualification uses fresh keys and
proofs for the exact emitted artifacts. The original verification circuit
uses the separate slow lane.

[owner]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L81659-L81761
[runtime]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/runtime/test/secp256k1.test.ts#L154-L219
