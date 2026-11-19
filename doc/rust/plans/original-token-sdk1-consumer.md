# Original-token retained artifacts on released SDK 9.1.0.1

The reviewed local consumer at signed source
`95179c777450fd30854bfe21ed3a3fe03297b251` verifies retained artifacts from
original-token source `447ff2a13c270f9d7fbe0fa7cd287ce7640057d9` against the
released Ledger 9.1.0.1 SDK matrix. These artifacts were produced on the source
SDK 9.1.0.2 graph. Original fixtures and assertions remain unchanged.

The bounded offline locked build emits 370 Cargo artifact records covering
325 distinct compiled package IDs. Actual compiled core versions are:

| Package | Version |
| --- | --- |
| midnight-base-crypto | 1.1.1 |
| midnight-ledger-v9 | 1.0.1 |
| midnight-onchain-runtime, midnight-onchain-state, midnight-onchain-vm | 4.0.1 |
| midnight-serialize | 1.2.1 |
| midnight-storage | 3.0.0 |
| midnight-transient-crypto | 3.0.1 |
| midnight-zkir | 3.2.1 |
| midnight-zswap | 9.0.1 |

All 31 distinct retained standalone proofs verify with 20 circuit verifier
keys. Each proof rejects a wrong key, a changed public input and corrupt proof
bytes: 93 refusal controls in total. The consumer also decodes 23 complete
tagged transactions without trailing bytes: 22 historical successful cases and
one historical without-offer refusal case. This classification preserves the
source result; it does not execute a new refusal under SDK1.

The actual build, metadata and consumer commands all exit zero. A fresh
independent retained-consumer process repeats the proof checks, refusal controls
and tagged decodes without Cargo or new proving. Complete source, locked
registry graph, physical binary, parameters and retained artifact bindings
are checked. The old receipt's historical signer remains explicit and is
linked through a separately authenticated current-key correction.

Signed actual qualification SHA-256:
`57ea511e3fe02ddc2f17d46e1cc48c133f61c70b046d126f284ee3fe7c3af0ad`.
Independent actual review SHA-256: `29a9c8e4b8b8ccf24451c7d1fa84474646813484d712db4fcf6b78f870f27330`.
Fresh retained-consumer replay SHA-256: `254054aa7036bde8d860e60319a4ac3fa5f48909a1c1b92036f273ab32a3675e`.
Retained binary SHA-256:
`e25f79def1ffcdf01463766d9a961750bb5d2b1b27f91a48ee3916a937a880c5`.

This is local Darwin arm64 proof and serialization interoperability. The
consumer creates no keys or proofs, does not verify transaction-level proof
composition, and does not apply or replay transactions. Authenticated exact
before-ledger reconstruction and SDK1 default-strict application/replay remain
a separate gate. This does not qualify all generated Rust APIs, public compiler
admission, native Linux or the broad migration milestone. Historical source-SDK2
proof/funded qualifications retain their original source identities.
