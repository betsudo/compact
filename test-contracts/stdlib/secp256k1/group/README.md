# secp256k1 group operations

The Compact contract and TypeScript runtime vectors come from
[LFDT-Minokawa/compact at 672bd0f](https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e).
The adjacent text files preserve the original computed-identity test from
`compiler/test.ss` and group test blocks from `runtime/test/secp256k1.test.ts`.
The runtime port supplies the P-256 generator coordinates directly for the
off-curve control, so that test also runs before the P-256 native port.

The computed-identity contract preserves the upstream failures when an active
coordinate accessor receives a point produced by zero multiplication or inverse
addition. The additional fixtures cover storing canonical identity results,
known finite results, the existing Jubjub overload, and refusal of mixed
carriers in TypeScript and Rust compilation.
