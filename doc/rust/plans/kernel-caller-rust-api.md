# Ledger 9 Kernel.caller Rust API qualification

The independently reviewed local slice is signed source `23af5a8e78a55c3a0d9aa51b672c34989fea8cd4`, based on `7cece56c5`. It adds genuine generated Rust support for the original Caller Inner contract. [#778](https://github.com/betsudo/compact/issues/778) retains the broader Caller and transaction work. This slice remains separate from the accepted Events v4 integration because it changes the signed runtime closure.

## Implemented behavior

The frontend emits a typed caller query only in the existing Ledger 9 released-runtime profile. The backend checks the complete canonical `Maybe<PublicAddress>` structure, including the nested Either and 32-byte address members. Native and recorded code execute the pinned 14-operation slot-6 VM program. They consume the constructed aligned read value, preserve its gas, and replay the exact read in Verify mode. A forged observed read refuses. A constructor Cell assignment containing the query takes the VM path; the functional target explicitly refuses this unsupported operation.

The unchanged original `Caller/Inner.compact` has SHA-256 `db593aeba03d3e5f738128f9444ece313b4f81494677a2e094031e46966bd466`. The retained exact-source compiler emits Rust and TypeScript successfully, and the generated crate passes an offline all-features Cargo check. Its constructor/root test checks `None`. A separate sealed A-to-B snapshot test derives B's `Some(Contract(A))` from the observed parent and checks native/recorded execution and replay gas; it does not supply caller identity as a transaction authority.

## Evidence and limits

The signed delivery index `delivery-23af.json` has SHA-256 `77c4209a33e29d76feafd821e846abb6597eb7d3f35886eede739a9f8fa1d9a8`. It binds the signed qualification `99d2618fc02df4632ac6e2b3ea9ed23c5cad799056f6ede7885f5e1a4a290d03`, the original source, 31 committed changed blobs, retained Inner output, commands and logs. The retained compiler binary has SHA-256 `9db533a34899c53af6d92496c34ba1d0c8dcec500ca4caec805a630b73de9953`.

Five focused tests pass: two runtime tests, one sealed-child test, one backend type guard and one generated constructor/root test. The retained Inner tree has 135 files; its scratch test and Cargo lock were added to the evidence copy after CLI emission and are disclosed separately. The signed Events frontend regression `37785eb9c3c1655ddc2843b46caeb31c90ef793a27f26c8cf888a35e235792b6` checks two byte-exact compiles and eight refusals.

These are local Darwin arm64 API/VM results. They do not qualify original Outer `viaInner`, nested/multiple calls, fresh Caller proofs, funded strict application/replay, `Some(User)` or native Linux. Constructor `None` and the tested root context are distinct: an actual top-level transaction with unanimously owned unshielded inputs can derive `Some(User)`, so root `None` is not unconditional.

The signed v4 package remains valid for its original runtime. Four signed runtime files differ from this slice, and a new caller test file is present. The current-runtime admission comparator must reject that old closure. Signed boundary note SHA-256 `e80b2d7e95b2573cb2ea856af5c6d68a140057eee470a86d3bd4482afdb71a75` records this explicitly. The separately reviewed [v5 continuation](original-events-v5-status.md) now provides the physical package and current-runtime CLI admission. Fresh v5 proof/funded qualification remains pending; the archived v4 package and frozen native Linux handoff remain intact. All owned Cargo targets and the finished source worktree were removed; signed source, compiler binary and evidence are retained.

## Private Outer continuation and physical material

Reviewed signed source `34494e61ea3cd84b2272d5c00b692fb9f5ef2eb7` adds a private generated `direct`/`viaInner` projection. It owns the parent frame across the receiver read, child claim and completion, consumes the actual Inner child method, and preserves the complete aligned output. The earlier continuation's receiver gap was fixed at `1c0d4a1a7`: the sealed observed child's address must equal the persisted receiver, including when two installed children share the same site. Backend 4/runtime 26 tests, generated all-features checking, actual `Some(Contract(Outer))` execution and changed-method/receiver/extra-operation refusals pass. The signed qualification `qualification-344.json` has SHA-256 `5f723ab992d6b84a43342263d1dcec602dd6b2741f746086b8796ddc66db572c`. This runtime coordinator extension is separate from both Events v4 and v5.

The reviewed physical candidate at signed `ac767d25a08ebb8db555926e261d43de99b11d5d` has identity SHA-256 `6c02231842e30090b616c67b8aa3e14680af9c8c69a351bc49909995362917f6`. Its 841 files/43 directories bind all original Caller sources and assertions, six genuine generated key pairs, full TypeScript/IR/ZKIR, private Rust, and the exact runtime plus macros closure. Four local Cargo manifests are confined and ten tamper controls pass. Signed `package-qualification-ac7.json` has SHA-256 `391102c51c800623260c35d7cf0cbbde17b410dab6c82a9e27d7c6b3ade53f3a`. The source keygen receipt has SHA-256 `7c78cd53d776bb4449128929273b753cf2610fb51d004208ffb2b6c963709a58`.

At ac7 this is physical material, without a checked link from installed verifier keys and method/entry hashes to the source/ABI. The later identity and token layers below bind those identities before execution. Packaged Cargo execution, generated API token wiring, public admission, Caller proofs, funded application/replay and Linux remain open. Ordinary full Outer still refuses; `viaMiddle` and `both` need later nested/multiple-call support. Finished owned worktrees and targets were removed; signed branches and evidence remain.

## Checked physical identity candidate

The next local layer is independently reviewed signed `1cff17c1babe900002a8a6184fbb85e37b0cde98`. Its new v2 stage embeds immutable ac7 and adds a deterministic augmented Inner manifest binding the reviewed Rust adapter. The identity crate checks exact entry/method hashes, Unit input, persisted slot-0 receiver, installed A/B verifier keys and resolver bytes before constructing the sealed coordinator. It does not create an identity dependency on Outer.

Identity SHA-256 is `8abbabf5353d63e8e818621d091649380a649396ad4876575367783600be569f`. The signed `qualification-1cff17c1b.json` has SHA-256 `3caa6958d65b4e9f746b2a70cdaf24216305ab7c36c9e99826997405d5486242`. Exact verification covers 850 files/47 directories. Eight physical checks comprise two positives and six refusals. An exact verified copy passes offline Rust 1.99 all-features Cargo checking; its generated lock is retained separately and the restored copy passes exact verification again. Independent review SHA-256 is `e75eb1f192aeb36455161b207e9f3c6da2e5c521d7afffaa361b1c448d4bef9b`.

At 1cff the identity rejection branches are typechecked and source-reviewed, not dynamically exercised; its private Outer still accepts a raw coordinator. The subsequent d246 layer below consumes the checked token and exercises binding/refusal cases. No public Caller admission, fresh Caller proof, funded application/replay or Linux result is claimed. The owned Cargo target was removed; original compiler manifests and signed parent packages remain intact.

## Token execution and bounded tree preparation

Reviewed signed `d2464cc844b4406e1ce23278ed64ef8f847f0308` wires the checked token into the private generated `viaInner` route and removes its raw-coordinator argument. Actual physical-key execution, the focused renderer test and ten negative binding branches pass. Its new v3 stage preserves the signed parents and binds 857 files/50 directories. Final signed receipt SHA-256 is `0c63aa879a4cdcc9ca29b151ca9f3fc435700d24d7aeb9716bff2651a3257b1d`. A signed count-correction supersession records an overwritten draft receipt/signature; final source, artifacts and logs are retained.

Reviewed `7386a431dc345c7b7e48e7fb498d596edf0c838d` derives one extra child context from the sealed Outer-to-Middle handle. Six focused tests include native/recorded `Some(Contract(Middle))`, replay gas and block/state/ancestor/depth refusals. Signed receipt SHA-256 is `e5b2b2104f4572a379c48561db4186a4fdf1b58117ccdaa8d8ff14717d169cf6`. This is context composition, without generated `Middle.forward` execution.

Reviewed `a41abb36d698a132d73f744f33bea31d87277417` adds a private unfunded three/four-call preparer. Ten VM tests pass. Typed recordings are checked before erasure; sealed contexts, exact claim tuples, source preorder, local sequence, call-key uniqueness and SDK partitioning are validated. The SDK partition graph uses commitment values alone, so equal commitments at different call keys can still fail SDK forest construction. Additive recording gas and a single batched replay have different overheads; the invalid equality guard was removed and diagnostic failures retained. Signed receipt SHA-256 is `008beaf65c199fecd5fe26abda509b7535c748964b02494347b52bd1509448a2`.

The tree checks zero net persistent-state change, unchanged call metadata and no events/nonclaim effects. It does not separately reject write-then-restore traces. Tests use synthetic verifier carriers and genuine recorded VM calls, without generated Middle/Outer adapters, physical-key admission, a transaction, proofs or funding. Source SDK 9.1.0.2 is executed; relevant registry 9.1.0.1 construction code is compared only. New runtime bytes require a new package identity. Source-owned Middle/Outer adapters and their physical binding are next; `viaMiddle`/`both`, public admission and Linux remain incomplete. Finished owned targets/worktrees are cleaned.

## Generated private Middle execution

Separately reviewed signed `b39adeec1640fc7152a9345bac1c049f8bc16138` generates private `Middle.forward` from the exact original source/IR. It derives Inner's context only through a sealed Outer-to-Middle handle, reads Middle's persisted slot-0 Inner receiver and compares it with the observed installed child before invocation. It bridges the complete aligned `Maybe<PublicAddress>` result and binds the recorded child claim. A narrow runtime wrapper consumes exactly three typed recordings and the two sealed handles, returning unfunded prepartition calls.

Backend source/IR refusal tests pass 2/2. The generated Middle crate passes offline all-features checking; its physical-key execution observes Inner's caller as `Some(Contract(Middle))` and refuses a wrong Inner verifier or persisted alternate receiver. Independent review verified all six source blobs, 40 evidence files and the exact signed ac7 parent closure, then replayed backend 2/2, generated Middle 1/1 and all 11 runtime tree tests. Signed `qualification-b39.json` SHA-256 is `e5274debeedf4a8f29eae85cb56696f1386b7a7f6e6dfa1bc86ee680c959d482`; independent review SHA-256 is `d7f694ebd69b5d609db0ecba3eef6bb33bab070183a3472fde0fec7cafb34bf6`.

The runtime wrapper test's root Outer recording remains manual with synthetic carriers. This is not generated `Outer.viaMiddle` execution or a physical three-call qualification. Generated Middle has no new signed package identity; installed Middle/Outer keys and prover-resolver authenticity need a later identity gate. Fresh proofs, funded strict application/replay, released SDK 9.1.0.1 execution, public admission, `both`, and Linux remain open. The owned 2 GiB target is cleaned; source and retained binaries, lock and evidence remain. Generated private Outer is the next source slice.

## Generated private Outer viaMiddle records

Reviewed signed `89bf52445519f42ef05bdb1c74406f7d23b37d02` replaces the manual root with source-generated private `Outer.viaMiddle`. Outer reads its persisted slot-1 Middle receiver and compares the sealed observed Middle before invoking generated Middle. Middle independently checks slot-0 Inner. Both bridges preserve the complete aligned `Maybe<PublicAddress>` output and claim the actual recorded child input/output/opening. Three generated crates pass offline all-features checking. Actual execution shows root `None`, Middle's caller `Contract(Outer)` and Inner's caller `Contract(Middle)`, with wrong persisted Outer receiver refusal. Root `None` is this fixture's context, not an unconditional ledger rule; owned unshielded inputs can yield `Some(User)`.

Signed `qualification-89b.json` SHA-256 is `5c52389f14a9134406b1125e9f001547e86be788d2e47b16dbf438d235757299`; independent review SHA-256 is `03554c76ba7ca651f2071c02a51f183357d4f213cc1a0e1d07c5a6d803f0c108`. Review checks four source blobs, 33 evidence files, source/fresh-IR pins and the exact signed ac7 parent closure. Retained-binary replay passes Outer backend 2/2, Middle backend 2/2, generated Middle 1/1, generated Outer 1/1 and runtime tree 11/11.

The private projection returns typed recordings and two sealed handles for inspection. Its public-field record bundle is not an authenticated package token. The generic runtime three-record wrapper stays `pub(crate)` and is not invoked externally by this generated crate. Installed physical test keys do not establish joint A/M/I key/resolver identity or fresh proof authority. A new immutable physical A/M/I package is next, followed by a source-specific checked token; no old Caller or Events identity covers these new bytes. Fresh three-call proofs, funded strict application/replay, released SDK 9.1.0.1 execution, public admission, `both`, and Linux remain open. The owned 2 GiB target and finished clean worktree are removed; signed source, binaries, locks and evidence remain.

## Physical A/M/I package

Reviewed signed `c1c61e77be547b7cdb319c657911ac2d7e2fe6fe` now binds the generated Outer/Middle/Inner projections and exact 89b runtime in a new immutable physical stage. Identity SHA-256 is `999d62b1c764f13b6aa9de60c09cb7f4b4ae089ac2b51d1c9f9d39e024e795ed`. The 1,652 files and 77 directories retain signed d246/ac7 parents, b39/89b qualification, original sources/IR/ZKIR/keys, six generated Rust/Cargo preimages and confined local manifests. One positive and 16 refusal controls pass, including re-signed mutations, nested Cargo escapes, extra files/empty directories, symlinks and changed keyring/digest. Earlier package drafts remain historical.

Signed `qualification-c1c61e77b.json` SHA-256 is `72728430826a8441cefac14a345a097a1fe5230629f01f6f0cc440ab56427f3f`; independent review SHA-256 is `3bdbffa572d10a059eadc03d29ebb9a6954efc456152643a0efeb1085828cd1f`. The reviewer ran the exact-head physical verifier and checked parent signatures, generated transformations and runtime/source binding. The clean package worktree is removed; the stage, branch and evidence remain.

This is physical packaging only, with no package Cargo execution, checked token, public admission, proofs, funded replay, SDK 9.1.0.1 or Linux qualification. The next bounded token design consumes the actual root, retains exact observed states and sealed handles, owns verified resolver bytes and makes the new unchecked Outer helper crate-private. Observation block hash/height must agree; per-contract transaction hashes can differ. Root times remain local inputs, without chain/finality authentication. This first route explicitly refuses `Some(User)` unchanged and qualifies only the no-owner root. Generic assembly remains internal, and later transaction authority must not accept bare mutable records.

## Checked A/M/I private execution

Reviewed signed `2c5f11330b071015b8d896ead0945a0da64b55ff` now implements that consuming token in a new 1,664-file/82-directory stage, identity `045913a99ee2d85965498b268158a9d2386d5ea306e4416799072d185964c0b2`. It binds the actual supplied no-owner root, immutable ledger/observations and sealed handles, installed A/M/I verifiers, fixed entry/method/ABI and exact resolved prover/verifier/ZKIR bytes. It owns the resolved material, so resolver mutation after binding causes no new resolver lookup or substitution. Execution accepts no replacement root, ledger, observation, proof binding or expected caller. The new unchecked Outer helper is crate-private.

Five VM tests cover actual nested caller recording and 26 dynamic refusals; 23 physical refusals and six downstream compile refusals also pass. Independent review replays the binary and all six compiler probes with their intended diagnostics. Signed `qualification-2c5f11330.json` SHA-256 is `0f7f65e2f324e323ef594b995eac18244563a16328257afcbe2d234f30e17b1c`; independent review SHA-256 is `b085167f595f211845b1192a00603931dbced74dd0fc8da7a6fc9f1b8c8d3af2`. Nine source blobs and 20 evidence files are bound. Expected build inputs now come from immutable validated Git objects with a final source/head/clean check, closing an observed concurrent source-read race before the final Cargo batch.

This qualifies private checked typed VM records on the source SDK 9.1.0.2 graph. It does not expose authenticated cross-crate assembly, proof preimages, a transaction, public admission, fresh proofs, funded replay, SDK 9.1.0.1 or Linux. `Some(User)` is refused unchanged in this first route; observed times remain local inputs. The next design makes checked execution produce an opaque outcome for closed source-specific unfunded prepartition preparation. Generic runtime assembly stays private. The owned 958 MiB target and clean worktree are removed; the retained execution fixture, binary, lock and evidence remain for replay.


## Closed unfunded preparation

Reviewed local `8707e3a8247261bcb4355c21c4d21101fa897cc2` extends the checked
A/M/I product to three SDK prepartition calls through its closed assembly path.
It retains opaque execution/provenance authority and validates source/material,
VM order/state/private output, installed keys, whole call-key uniqueness, SDK
forest structure, claims and batched gas. The generic runtime assembler remains
crate-private. SDK calls are mutable cloneable ordinary data; future product
proof/transaction constructors must consume the opaque prepared wrapper.

Thirteen test groups cover 80 refusal cases; 17 product compiler refusals and
one ordinary SDK-clone positive pass. Physical checks pass 1 positive/25 refusals
for 1,679 files/87 directories. Independent review replays all unit groups and
compiler probes under Rust 1.99. Signed qualification SHA-256 is
`4c5383bdccb6a7c5a761cfb58dd93e81de15358ea7e533270a0d6c1581fdcdba`;
review SHA-256 is `c2ab089306ed200b3539c6d3d4fe6ecf7b8f00428081a6ea939d7db6853b973f`.
The package identity is `7e16daa362524dddb9d42cb96d9f0f2ef38335147d85c9543e81e5447e4937a8`.
Initial 225 compile failures remain historical; corrected 870 was frozen and
reviewed before execution. Owned 936 MiB target and clean worktree are removed;
replay fixture source remains because the binary embeds its location.

This is private source-SDK 9.1.0.2 unfunded preparation, not actual ProofPreimage,
Transaction, proofs, funding, public admission, SDK 9.1.0.1 or Linux qualification.
No-owner root is the scoped route; Some(User) remains refused, and local time
observations do not authenticate chain finality. Existing v4/v5 packages keep
their exact identities; no source publication or services were started.
