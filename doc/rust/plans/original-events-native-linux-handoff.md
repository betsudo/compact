# Original Events native Linux integration handoff

Owner: [#773](https://github.com/betsudo/compact/issues/773). This gate is source-owned and has only been exercised in `--source-only` mode on macOS arm64. Native Linux qualification remains open until a Linux x86_64 owner runs the full gate and reviews its retained logs and artifacts.

The input is the frozen signed Events v4 physical package at identity SHA-256 `605312a3bc2dfbe82212921e402f2cc1705a015e0dc12e59ca6432af46facf8b`. Transfer the complete package, its detached `identity.json.asc`, and the public trusted GPG keyring over a trusted channel. Pin the keyring SHA-256 and signer fingerprint `F8851C118C66BF98DB9B1B5DB75A9FB8C6327844` against the prior owner handoff. No private signing key is needed or should be transferred. The gate calls the frozen v4 verifier, which checks exact physical membership, signature, recursive v3 parent, source/IR/artifact identity, Rust renderer and generated TypeScript binding. It also compares both Rust consumer templates and SDK2 proving static files to exact tracked Git bytes and membership. It does not regenerate or modify the signed package.

## Git history needed by the verifier

The v4 parent records the historical v3 source head `e6f94dedac2f795e16d9bd068e7afc0a5a7736ee`. This commit is **not** an ancestor of the integrated branch because the v3 source was cherry-picked. A shallow or public clone may lack it. The source-only preflight requires these exact local Git commit objects, independently of ancestry:

| Purpose | Commit |
| --- | --- |
| v4 package source | `8ea671f7978996e02ec9dab3f5266e9a6391b95e` |
| v3 package source | `e6f94dedac2f795e16d9bd068e7afc0a5a7736ee` |
| v2 package source | `2c7e4eaa3734c7912b6c98297c81428fb5b0d2e2` |
| v1 package source | `1ccb63150a409de58214fae8f8ff2f7b891d3f66` |
| original TypeScript Entry source | `b465dcce625497e5da7493de2ea29fcdb1f0dc89` |

The reviewed gate implementation is local and may be absent from public remotes. A maintainer with the complete local Git object store must include that independently pinned gate head as well as the historical package heads in the transfer bundle:

```sh
git update-ref refs/events-native/gate "$REVIEWED_GATE_HEAD"
git update-ref refs/events-native/v4 8ea671f7978996e02ec9dab3f5266e9a6391b95e
git update-ref refs/events-native/v3 e6f94dedac2f795e16d9bd068e7afc0a5a7736ee
git update-ref refs/events-native/v2 2c7e4eaa3734c7912b6c98297c81428fb5b0d2e2
git update-ref refs/events-native/v1 1ccb63150a409de58214fae8f8ff2f7b891d3f66
git update-ref refs/events-native/typescript b465dcce625497e5da7493de2ea29fcdb1f0dc89
git bundle create events-native-history.bundle refs/events-native/gate refs/events-native/v4 refs/events-native/v3 refs/events-native/v2 refs/events-native/v1 refs/events-native/typescript
sha256sum events-native-history.bundle
```

Deliver the bundle digest separately. The Linux owner verifies that digest, runs `git bundle verify`, then fetches each named ref into local `refs/events-native/*` and checks the five historical `git cat-file -e <sha>^{commit}` objects plus the independently pinned gate head. Check out that gate head in a separate clean source tree; the bundle supplies local implementation bytes without a source push or CI run. Do not replace an absent object with an integrated cherry-pick or edit the package identity to name a reachable commit. The current source head must also descend from integrated base `447f879aea19ba58a12d6a0746e18c46be4d5be5` and exactly match `--expect-head`.

## Inputs and run

Use a clean checkout at the reviewed native gate head, a clean SDK2 source checkout at `6fbc2e3900845a858f70eebf8a0deab1d4d3c0d6` with `ledger/static`, Rust/Cargo toolchain 1.99.0, Nix, GPG, and cached/offline Cargo dependencies for both SDK graphs. Obtain `REVIEWED_GATE_HEAD` independently from the owner handoff; do not derive it from the checkout being tested. Put the signed package, keyring, evidence and targets on volumes with at least 50 GiB free. On macOS the scripts additionally check the Data volume. Reserve an exclusive run directory outside both source trees. Use absolute paths:

```sh
python3 tools/compact-rust-backend/qualify_original_events_native_linux.py \
  --source-only --package "$EVENTS_PACKAGE" --keyring "$EVENTS_KEYRING" \
  --expect-head "$REVIEWED_GATE_HEAD" --report "$EVIDENCE/source-only.json"

python3 tools/compact-rust-backend/qualify_original_events_native_linux.py \
  --package "$EVENTS_PACKAGE" --keyring "$EVENTS_KEYRING" \
  --expect-head "$REVIEWED_GATE_HEAD" --ledger-source "$SDK2_SOURCE" \
  --run-dir "$EVIDENCE/native-run" --report "$EVIDENCE/native-result.json"
```

The native mode refuses non-Linux x86_64 hosts. It builds `.#compactc` from the exact clean source, hashes both packaged `compactc` and `compactc-scheme`, admits the original Outer source with the signed v4 callee package, checks generated Rust, TypeScript, ZKIR and key bytes, manifest and provenance, and exercises wrong-digest and ordinary-Outer atomic refusals. It verifies that `admitted/events-v4` is an exact signed copy. The admitted Outer contract members must equal the signed original bytes except for the exact two Cargo dependency relocations, an appended workspace marker, a relocated source-map root, and the output Cargo lock. Committed native consumer variants compile `admitted/contract` as Outer and the signed admitted Inner/identity closure. The native resolver reads its keys from that same signed admitted closure. Both variants are checked against the original consumer bytes with only exact path relocations. The gate sequentially runs this source SDK2 consumer for fresh proofs, independent retained verification, the first funded strict transaction, and the second funded strict transaction. It finally runs the separate released 9.1.0.1 registry consumer over both retained transactions, including the exact post-block second prestate and applied A/B event logs, payloads, state bytes and replay refusals. `MIDNIGHT_PROOF_SERVER` is unset; the source consumers use their own pinned proving inputs. Fresh generated transaction proofs and positive fee are checked by their source-owned Rust assertions.

The run directory retains admitted output, private package and consumer copies, Cargo locks, all proof/transaction/state artifacts, and command stdout/stderr. Record its exact `target-source` and `target-released` paths in the iteration resource ledger. The source target is removed before the separate released build; both exclusive targets are removed on exit. Keep `source-only.json`, `native-result.json`, the full run directory, Nix derivation/store reference, trusted package/keyring identities and the trusted Git bundle digest together for review. The native receipt records source/tree, package, keyring, compiler binaries, SDK2 static inputs, both Cargo locks, released registry matrix, commands, exit codes, environment selection, artifact hashes and Rust results. It clears inherited compiler, wrapper, Rust flags and target overrides. The host Nix installation, cache configuration and Cargo cache remain operator-controlled prerequisites; the receipt records Nix version, Cargo home and a digest of `NIX_CONFIG` without exposing possible credentials. The owner should independently verify the hashes and sign the reviewed receipt before external transfer. Historical macOS arm64 receipts remain separate evidence and do not count as this Linux run.

If the gate fails, preserve the run directory and logs as failure evidence; do not label it qualified. The source-only result asserts provenance only. The full gate requires native execution and independent owner review to close #773.
