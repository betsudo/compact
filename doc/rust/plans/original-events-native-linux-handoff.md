# Original Events native Linux integration handoff

## Git object binding hardening

Reviewed local `b3e677de6c559076e3dd7bf0f8ea4a21f0fa7c14` follows the source-only
13d handoff below. It refuses ambient Git authority overrides, disables Git
replacement objects in bootstrap and transitive source/SDK reads, and hashes
the raw framed commit object to independently supplied H before accepting its
F885 signature. The isolated replacement-ref control does not mutate shared
Git refs. This addresses [#787](https://github.com/betsudo/compact/issues/787)
locally; source integration/publication and native use remain separate.

The author and independent reviewer each run all twelve source-only tests and
77 refusal cases successfully. Signed qualification SHA-256 is
`c72c2494c02ccdac8c7c867760026cb8cd2e01b5c3ca6e5e5a2035fade3b39cd`;
independent review is
`c737142a4373b539166f37122c2d7464342543b53eeb716a663ba6b00dea298e`.
The externally authenticated bootstrap SHA-256 is
`f28144c41234352b2e4c579ede9a99785d142d8ee492b34f96152590811491e7`.
No packet, Cargo/Nix, fresh proof, funded application or Linux execution ran.
Use this reviewed follow-up for future v5 owner preparation; retain the old
13d signed delivery and v4 packet as immutable history.

## Later v5 dual-SDK source preparation

The additive local handoff at signed
`13d5fffbc7716172382aa47bc1f9fb45d36254cc` is independently reviewed in
source-only mode: 19 added files, 3,217 exact tool-source pins, 11 controls and
72 refusal cases. It uses the qualified bffd source-SDK and 4715 released-SDK
consumers, closed complete admission inventory and actual compiled-library
matrix checks. Entry requires an externally verified signed Git-blob bootstrap;
executing a live script does not authenticate its own earlier execution.
The original v4 handoff below and its transfer packet remain unchanged.

Signed PREP SHA-256 is
`83c5797d6687a3106edc864a93fed8f8106cb5056274e5b9dfd18b8d548fea2f`;
source delivery is
`98ebb800cce5c645593d3eff26e99dbcd30f4ce89a529cb520ed39233e5c2a23`;
independent review is
`753c5606f0d3728785a9de569c2993bf103a422d28445b505daa4e19f5b57cbb`.
The externally verified bootstrap SHA-256 is
`672b4a8d6851e164a318c979b4386e61faf4778c23523fd4ab5002a8f4ce774e`.

This is Darwin source-only preparation. No v5 packet, Cargo/Nix build, Linux
proof or application ran. Native owner, result signer, compiler budget and
signed execution context remain unassigned. A future native run must build a
genuine Rust/Scheme compiler pair, compute fresh proofs, apply both original
funded calls strictly, refuse replay, and independently verify with the released
SDK. The clean owned worktree is removed; branch, bootstrap, input snapshots and
receipts remain. The first empty-directory preflight refusal is retained.

## Historical v4 handoff

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
