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

This first consumer qualifies local Darwin arm64 standalone proof and
serialization interoperability. It creates no keys or proofs and does not
verify transaction-level proof composition or apply/replay transactions.
The separate three-mint application gate below has its own source and receipt.
This does not qualify all generated Rust APIs, public compiler
admission, native Linux or the broad migration milestone. Historical source-SDK2
proof/funded qualifications retain their original source identities.


## Three retained shielded-mint applications

Signed consumer source `734031744634f2f3c4929c5d2aa5852051cc5a7c`, staged at
`8cd4aef1702ff1734d6e5a08c187ce9655f86667`, now qualifies three existing
funded transactions on the released SDK1 graph. The old signed deployment
snapshots are authenticated exact before states for these cases: source
inspection proves no intervening ledger assignment before the calls.
Every transaction and before/after/replay byte remains bound to the original
All20 source and receipt.

| Original case | V3 contract calls | Post-application block step |
| --- | ---: | ---: |
| mintShieldedToSelfTest | 1 | None |
| mintShieldedToUserTest | 1 | None |
| mintShieldedToContractTest with receiveShieldedTest | 2 distinct contracts | 10 seconds |

Default strictness explicitly requires Real verification, contract proofs
and balancing. All three transactions apply successfully, and replay refuses
with `IntentAlreadyExists` and unchanged serialized ledger bytes. Every
after/replay state is byte-identical to the old source-SDK2 result. The two
contract addresses are distinct and both installed contract states remain
unchanged, preserving the original mint/receive assertions.

Four targeted embedded contract-proof corruptions and three positive-fee
Dust-proof corruptions produce their specific strict verification refusals.
Wrong before state, late fee-intent time and corrupt transaction tag also
refuse for each case. Source time and block-context differences are preserved:
self/user use contract expiry minus 3600 seconds and direct time/time context;
contract mint uses expiry minus ledger global TTL and the original TestState
context, followed by its exact ten-second post-block update. Replay uses the
original time/time context. The block update is part of the signed after-state.

The offline locked build again reports 370 Cargo artifact records/325 distinct
IDs and all ten released SDK1 core versions. The metadata, build and consumer
commands exit zero. Fresh independent retained-binary replay repeats all
three cases and produces a byte-identical report without Cargo or proving;
source, proof, parameter, static and binary input maps stay unchanged.

Signed actual qualification SHA-256:
`76c0eaaf33afe0887d87c2c4a49fc51f571f8d15a1bbf061f5901602eaf6f589`.
Independent actual review SHA-256:
`0e43315f4181c48ae640e4a5dc7aff383f952e65f5eb1c28879bfddd07cc7e00`.
Fresh retained-binary replay SHA-256:
`37c184e1cdab8a95dcf05797e5fa12f5d16e21d6586c808c2340ac1c755871e5`.
Retained binary SHA-256:
`8dca43fb1df8f859f0be75f10563435b4f0604663df56af5f7ad21d6489d40b2`.

The owned rebuildable Cargo target is removed after independent review and
active-use audit. Signed source stages, binaries, full maps, logs, proof
artifacts and replay evidence remain. This adds no fresh keys or proofs and
does not rerun original TypeScript tests. The other 19 historical successful
transactions lack authenticated exact before states in this gate. Their
capture owners are mapped, but no application qualification is inferred.
Native All20 generated APIs, public admission, live chain and native Linux
remain separate requirements; the broad migration remains incomplete.

## Native All20 SDK1 compile qualification, 20 November 2026

Signed local source `d14b0786900b5f91ea50483b107cd108a46cb12e` now passes
offline locked Cargo checking for the original ABI54 generated All20 library,
both with default features and with `ledger-transaction`. The default check
reports 350 artifacts/310 distinct package IDs; the feature check reports
365 artifacts/320 IDs. The union is 320 compiled IDs, including the ten
released SDK1 core packages. This native graph uses `midnight-zkir` 2.2.0,
separately from the retained-proof consumers' 3.2.1 graph. Optional ledger
1.0.1 is compiled by the feature check, not the default check.

The original generated library and SDK2 compatibility metadata remain byte
identical. A separately identified private SDK1 manifest and lock select the
new graph; the old metadata is not relabeled. Signed actual receipt SHA-256:
`62b40a39d5b34293e2e5851d4e1a306d092994f5863afc9f73dfc604454f59fe`.
Independent actual review SHA-256:
`4384ef32403147c066205da8cc191b3f3517b3518d673ec9b0f1beffebf8c672`.

This is compile-only Darwin arm64 evidence. No generated native, recorded or
observed method is executed by these checks. Original TypeScript parity,
fresh current-producer material comparison, native Linux and broad public
admission remain open. Retained-proof and funded application results above
keep their separate source, artifact and graph identities.
