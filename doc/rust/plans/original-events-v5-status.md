# Original Events v5 proof and SDK status

This local continuation carries the reviewed Rust caller runtime into the exact original Events pair. The accepted v4 package, its source-bound proofs and funded receipts, and the frozen v4 Linux handoff remain separate evidence. [Caller #778](https://github.com/betsudo/compact/issues/778), [cross-contract calls #730](https://github.com/betsudo/compact/issues/730) and [Linux #773](https://github.com/betsudo/compact/issues/773) remain open.

## Signed physical candidate

Reviewed source `6ac5b04bbd9be8e2c66b8bcc51e654810904d666` produces a signed v5 candidate at identity SHA-256 `635a6303044c59a1b38370fe5211b6117c24f03b4f1c12c0a7e81fec49eccb9d`: 3,278 files and 154 directories. It preserves the complete signed v4 parent, original source, generated Rust/TypeScript/IR/ZKIR, keys and checked identity bytes. Its runtime projection contains the four reviewed changed files and new caller test, bound to the reviewed 31-file Caller source delta and two candidate-owner files. Five local Cargo manifests are confined; no lock is introduced into the signed candidate. Exact verification and 19 signed tamper controls pass.

The detached qualification `qualification-6ac5b04bb.json` has SHA-256 `49c2a2d99271919445614bbabf8e88d9a68b5b1046c3faffa1a6388715c20389`. Independent review passed. The earlier 408f package is historical. The candidate's immutable `candidate-no-cli-admission` status records its preparation stage; later qualification is recorded externally rather than rewriting signed bytes.

## Current-runtime CLI admission

Reviewed signed source `73fb2ab802e34833b86c265b919d2236e94d09e3` adds a self-contained v5 verifier and shares the authenticated, atomic admission flow with v4. The actual compiler admits v5 against its selected caller runtime, passes 18 controls and strict all-targets Clippy, and publishes 3,293 files/159 directories. A second actual positive with the archived signed v4 runtime and package passes, with all 13 v4 controls preserved. The ordinary full Outer continues to refuse without changing admitted output.

The signed receipt `qualification-73fb.json` has SHA-256 `dec02cb4c96ec6677749a7ffd6ac49ca6d69586a4e7306ed1246a7fd1c9af2fd`. Its retained compiler binary has SHA-256 `fcaabcdfbf8f6360d5ea273703cee8c708914a3ab99a1af4be196097c4dcae4e`. Independent review verified both output inventories, exact signed artifact bytes, Cargo relocations, source-map normalization, provenance/lock hashes, five confined local manifests, commands and logs.

## Admitted-output native proofs and funded application

The source-only consumer at `3a5c40093` retained its PREP receipt and nine controls;
it did not execute proofs. A later material-path audit found that its binary-ZKIR
resolver omitted the v5 archive's `prior-v4` prefix. Signed local
`bffd8d788af65290a42007b6e4c3c8514cfae1bb` fixes only that v5 path and the exact
qualifier relocation rule, preserving the old source and v4 evidence.

The corrected consumer compiles the actual admitted `contract` and signed
`events-v5` Inner/runtime/identity/keys. Independent final review accepts:

- Seven genuinely computed contract proofs: three unfunded, two for the first
  funded transaction and two for the second.
- A fresh-process retained verifier for the three unfunded proofs, with nine
  proof/public-input/verifier mutations refused. Unfunded checks do not claim balancing.
- Two funded default-strict Real applications with balancing and contract proofs
  enabled, positive Dust fees, exact applied Outer `0x42` then Inner `0x07` logs,
  and persisted Inner state 1 then 2.
- Both replay attempts refused as IntentAlreadyExists with unchanged serialized
  ledger bytes. The second process independently restores the first funded state
  and matches the required post-block second prestate.

Signed qualification SHA-256 is
`725e3c638e895bb36da031ac1440a17025b7074f03129e438083de2b84d41dec`;
independent review SHA-256 is
`4b2f7c6635d683d99e928f42e2bb0755bb6f28e40ba5e24db1a5e47d77dac88c`.
The review reruns the retained three-proof artifact verifier. It checks 360 source
pins, the exact 3,303-file/162-directory execution fixture, 28 evidence files and
25 proof/transaction/state artifacts. Retained binary SHA-256 is
`016384052bf12a408194de7523d470fa9b6d641deb21788977a40e2b88769659`.
Freshness is native computation in new output paths; deterministic fixture RNG
is retained, without claiming bytes must differ from historical proofs.

The isolated runner pins its Python closure, actual admitted bytes, source, keys
and cached parameters before/after. Prelaunch review fixed audit-hash, ignored
module, ambient compiler override and isolation-argv gaps before the first Cargo
run. Fixed Rust 1.99 dev optimization level 2, debug 0, incremental 0 and two build/
Rayon workers were used throughout. Actual build took 7m51s; all command phases
totaled about 519 seconds, including checks. This is an environment/timing record,
not a controlled speedup claim. The 50 GiB disk and 3 GiB target guards remained
active. No node, proof server, Docker stack or parameter download was started.

The owned 1,050,475,658-byte target and clean worktree were removed after review.
Signed cleanup SHA-256 is
`c9555a1bc2a1aa90d9ed0e9b2f1724bca1ec1cea573087cb17e969684fb3d623`.
Binary, exact replay fixture, lock, source branches, logs and artifacts remain.

## Released Ledger 9.1.0.1 interoperability

Signed local `4715d06483b256c87bcd2534188f87edb5b43d72` independently consumes
the exact new v5 proofs and funded transaction/state artifacts on the released
registry SDK matrix. Its sibling consumer preserves reviewed 656's sequential
second-state algorithm byte-for-byte, including the post-block transition.
It generates no new proofs and does not compile source-SDK or generated-contract
crates into the released verifier.

All seven retained V3 proofs verify directly, with 21 proof/input/key mutations
refused. Both transactions pass default-strict Real verification and application;
six strict proof mutations refuse. First before/after, corrected second-prestate,
final and both unchanged replay ledgers match source-SDK bytes exactly. Independent
review replays all three retained-binary modes with byte-identical reports. This
closes independent second-funded artifact verification through the released SDK;
it does not invent an artifact CLI in the source-SDK consumer.

Signed qualification SHA-256 is
`fe41721824841dba885f10ba9ed332d357201993da020fddffd972b975ad8c6e`;
signed delivery SHA-256 is
`e729fe04121f3a4829ef6ac1916c94dc222b69b24384d3147a47410ddec5dff9`.
Independent review SHA-256 is
`ffeb89d574604ddb89de0e767b607b516ad083f8223a9e6760680e45358d4d84`;
replay report SHA-256 is
`4df4ca9c86540fa815c32b1db782d635393379523c1b90ee66f56f50e462b27f`.
The retained verifier binary SHA-256 is
`d301ecc75911edca5fa0f1f847f3f9be89e0f3ba7bd2f786cad42c507d9f4327`.

The locked/offline graph has 401 lock records: one local consumer and 400 exact
registry/checksum records, with no Git dependency. All ten required matrix
libraries actually compile; 26 Midnight library records and 314 compiled package
IDs are recorded separately from lock-only optional/platform packages. Source,
original/private admitted trees, 25 input artifacts, exact copied-consumer
membership, compiler features, retained binary and lock are checked before/after.
The exact execution copy contains 3,329 files/166 directories.

Build plus unfunded verification took about 301 seconds; each funded verification
phase about 2.1 seconds. Fixed opt2 was retained throughout; no controlled speedup
claim is made. The 50 GiB/3 GiB guards did not trip. The owned 949,216,754-byte target
and clean worktree were removed after final review. Signed cleanup SHA-256 is
`5bf0057bacf37382e0b5a7dee14c489593c7e29fa4499a190b2a3bd7d55ae228`.
Binary, lock, small execution fixture, logs, outputs and source remain retained;
a post-cleanup unfunded verifier smoke check passes. Existing bffd inputs, old v4
receipts, shared caches and other owners' worktrees remain untouched.

## Remaining qualification

Both v5 SDK results are local Darwin arm64. Packaged source-built compiler and
native Linux remain open; no generated EntryPoint invocation, live/public network
or broad generic cross-contract parity is implied. The frontend remains the source
Scheme wrapper. The archived v4 native/Linux handoff and all old source/proof
identities stay intact; a v5 dual-SDK Linux handoff is the next preparation task.

Caller viaInner and closed A/M/I preparation have separate coordinator/material
identities; their broader proofs are not covered by these original Events results.
The zero-width schema21/ABI55 work also has a separate unqualified source identity.
The broad milestone remains incomplete; source publication and hosted CI are separate.
