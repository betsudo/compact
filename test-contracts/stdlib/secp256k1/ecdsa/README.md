# secp256k1 ECDSA verification

The two contracts and their original test blocks come from
[LFDT-Minokawa/compact at 672bd0f](https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e).
The ordinary fixture retains the original false signature and exact
identity-key assertion. The #609 archive retains the optimizer snapshot.
The paired suite runs the intact double-verification contract with two
distinct signing keys and checks each failed assertion independently.

The additional deterministic controls use Noble to sign and verify the
same digest independently. They cover valid low-s and high-s signatures,
a wrong message, a wrong key, and the zero denominator refusal.
