# Rust zero-width encoding on Ledger 9

[Issue #784](https://github.com/betsudo/compact/issues/784) tracks a confirmed
local compatibility gap against pinned mainstream Compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`
and released Compact 0.36.0. Preserve the separate Ledger 8/ZKIRv2 boundary
qualification in [#702](https://github.com/betsudo/compact/issues/702).

Accepted local Rust source `3eafaf3eb92d6c7a39ba93ff6d41df11eb352643` assigns
zero-byte alignment to `BoundedUint<0>` and a single-variant `CompactEnum`.
The pinned issue-588 attestation places its one-variant `Kind` before Uint64;
the actual released v3 `attest.zkir` uses persistent-hash alignment [1,8].

For amount 7, a direct native tuple/enum probe produces
`aae89fc0f03e2959ae4d701a80cc3915918c950b159f6abb6c92c1433b1a8534`.
The actual released runtime 0.21.0 with the original ZKIR alignment produces
`7dd1657a36197d82bfae690c3212ef9b3c0d7bbaef12ddb8ef8f9f30b2373185`.
The audit mirrors the original struct field order through direct Rust APIs;
it does not execute a generated Rust original contract.

Signed audit v2 SHA-256 is
`b2fa9eaa8b3fc1f886ffbb4d4a90bc1c1615add0aeb000525b875a467b02c2ac`;
independent review SHA-256 is
`22c9bf90d2de28304231c4831df7a536cecaf1e18c599d6774aa88121b014dc0`.
It authenticates 770 unchanged runtime/macro source files, the private
manifest-only released SDK1 pure dependency overlay and 393-package lock.
A fresh Node oracle checks the exact 416 published SDK files and its
script/ZKIR/native-binary inputs before and after execution. Both processes
exit successfully and reproduce the mismatch. The earlier v1 post-run-only
graph attestation remains historical. Optional ledger is not compiled; no full
SDK ABI, generated-contract, proof, funded transaction or Linux result is claimed.
Owned Cargo and Node copies are removed; source, binary, lock, original ZKIR,
oracle and logs remain retained.

The next bounded implementation is explicit runtime and derive encoding
selection: legacy/default zero-byte behavior must coexist with an opt-in
minimum-one-byte carrier and enum representation in the same binary. A global
width change or Cargo feature would risk altering legacy callers through
feature unification. The compiler IR profile, generated carriers and runtime
compatibility handshake require a later separately qualified slice. Existing
ABI54 packages and Ledger8/ABI53/Forge evidence keep their exact identities.

Owners are runtime primitive alignment/binary hash representations, enum
and cell derives, IR extraction/schema, generated carrier/literal/coercion/
arithmetic/stateful lowering, compatibility metadata and original-host gates.
Retain the original issue-285 and issue-588 fixtures and assertions. Complete
generated Rust/TypeScript behavior, source-bound fresh proofs with independent
verification, funded strict application/replay and native Linux remain open.
