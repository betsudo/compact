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

## Runtime and derive API qualification

The reviewed first runtime slice is signed local
`3e5625f148e269ce067ae84591803a991cbc8735`. It introduces explicit
`BoundedUint<MAX, ONE_BYTE_MIN = false>` encoding selection and the scoped
`#[compact_encoding(ledger9_v3)]` enum derive attribute. Legacy/default types
retain zero-byte behavior; selected types use at least one byte. Both profiles
coexist in one binary without a Cargo feature. The global ABI54 marker remains
unchanged; an additive ABI55 marker reserves a future generated-profile handshake.

Actual final-head execution passes 7 existing bounded-unsigned tests, 3 existing
enum tests, 3 new mixed-profile/hash tests and 33 macro unit tests. Macro strict
Clippy passes. A downstream valid control and four compiler refusal controls pass.
Independent review replays the retained test binaries and checks those controls.
The explicit v3 attestation hash now equals the published oracle `7dd1657a...`;
the legacy hash remains `aae89fc0...` in the same consumer.

Signed final qualification SHA-256 is
`f7f0c41e317e4726a0334d101f5ca978b0a62cfbd4e01e992587cbde033c7bea`;
independent review SHA-256 is
`132ac73e54caa591e006019145ba6eef8d667177f05e4ff7f5e7a71e0513e354`.
The stage contains 776 files/20 directories. Its manifest-only pure registry SDK1
overlay resolves 24 Midnight packages matching the signed baseline; optional ledger
is absent from the compiled tree. Source compatibility metadata retains its SDK2
labels. This is not a full SDK1 ledger ABI result. Earlier compile failures at 4df,
6ee and 470 remain historical; each fix was signed and reviewed before rerunning.
Owned 845 MiB Cargo target and clean source worktree are removed; source, locks,
binaries, stage and evidence are retained.

The next slice must carry the encoding profile explicitly through compiler IR,
carrier/literal/coercion/arithmetic/stateful lowering and runtime compatibility.
Generated issue-285/588 contracts, the actual ABI55 handshake, new physical proof
packages, fresh proofs/funded replay and Linux remain unqualified. Existing ABI54
packages and Ledger8/ABI53/Forge evidence keep their exact identities.

Owners are runtime primitive alignment/binary hash representations, enum
and cell derives, IR extraction/schema, generated carrier/literal/coercion/
arithmetic/stateful lowering, compatibility metadata and original-host gates.
Retain the original issue-285 and issue-588 fixtures and assertions. Complete
generated Rust/TypeScript behavior, source-bound fresh proofs with independent
verification, funded strict application/replay and native Linux remain open.


## Optimized backend regression checkpoint

Signed local `8b5e9619e90fffa7d5dc6554c576136c8c0052c1` passes the unfiltered,
locked offline optimized library suite under Rust1.99/default worker stack:
1,970 passed, zero failed, seven ignored. Independent receipt review verifies
44 changed source blobs, four authority manifests, lock, retained test binary
and log. Qualification SHA-256 is
`ca93d39d0347af6a82f1c67df79cf115cecf150f6dacd873076e1c31b9e08e42`;
review SHA-256 is
`04ca0838f1a441c2eecd4e15d2a701d82831a51d95994976a340072551ecef19`.

The run exposed inherited authority drift from accepted cross-contract edits.
[Issue786](https://github.com/betsudo/compact/issues/786) adds digest-pinned
current-to-predecessor projections, preserving old manifests and owner bodies.
Earlier red runs and the test-only borrow compile failure remain historical.
Unoptimized default-stack baseline aborts are not relabeled as passing runs.

A separate narrow lint cleanup at signed
`beeb2199597a088c1b0ae4b459425b4119f3155e` passes strict library-only Clippy.
Signed lint receipt SHA-256 is
`be01e8676da0f4c6347f1b8a23499a3c074dfc034ad26b6d0ecd8fdeaa3359cb`;
independent review SHA-256 is
`f6078126d2cdb1ea131bfcb4c7fe69249178b735e5a67578da1e2066c10fec49`.
This is not a fresh full library-test run at beeb, nor passing test-target lint:
frozen contract_calls.rs test diagnostics remain unchanged. The owned 775MiB
Cargo target and clean comparison worktree are removed; signed sources, locks,
binary and logs remain. Recorded/profile propagation continues as source-only
work. Public schema21 still refuses; compiler emission remains20. Generated
original285/588, physical ABI55 compatibility, proofs and Linux remain open.


## Recorded checkpoint and Cargo graph correction

Signed `167de8b701178c3343bd2bd848cde0b704fedb93` passes a fresh full optimized
library run (1,972 passed, zero failed, seven ignored) and strict library-only
Clippy. Receipt SHA-256 is
`867e9aa96e0d3496c547c7344e78668750d5145c965061fb8a1ae6382aed1b6f`;
independent review is
`b4cefcb637f04157ae4a4171fcacb9c153a1d84d77e95c877c5f467d8101d225`.
This qualifies the bounded recorded renderer preparation; schema21 public
admission and generated original contracts remain closed/unqualified. The
owned build target is removed. Functional preparation is a separate source-only
slice, explicitly refusing v3 through the archived Ledger8 adapter.

The earlier 8b5 test and beeb lint receipts labeled the inactive member lock as
the Cargo lock. Actual workspace builds use root Cargo.lock, SHA-256
`1871651dc10800a3bc94c81f9427293abcbcb73309c21093ec4a255f0626513e`.
New immutable signed addenda bind that lock, both manifests and the inactive
child lock. 8b5 addendum-v2 SHA-256 is
`4a71d4b383b2a5940fcd87a481d73aade43462d90c2de69da867b752765d4556`;
beeb addendum-v1 is
`9e25d99c9891b9602e00e05f211d14f206b8a80d874b9bcd47d0497934a96048`.
The original signed receipts/logs are unchanged. This corrects graph labels,
not execution results; no rerun is claimed for the addenda. Fresh a383/167
receipts identify both lock roles directly. Full test-target lint remains a
separate historical red scope in frozen contract_calls.rs.


## Functional checkpoint and original compiler material

Signed `9ca0e9c8791914d8508578edd212823e62058ce7` passes the full optimized
library suite: 1,977 passed, zero failed, seven ignored, zero filtered. Strict
library-only Clippy also passes. Functional schema21 is explicitly refused by
the archived Ledger8 adapter. Signed qualification SHA-256 is
`847fcd3034db611f4f0fff980b1ee7ed566d6e57bdbec145957fbc287c2efcfc`;
independent reviews are
`17e062971717fb57381aa087bb2aa5257ab3add68bb8393cdd9be6f116d76cb5`
and `2fa0a412773e0780df5dc55e81c947066e9d62873da6a8b22651e459d812b294`.
The owned 781MiB build target is removed. This still does not qualify public
schema21/ABI55 generated contracts, proofs, funded application or Linux.

A new explicit `--rust-ir-encoding-profile ledger9-zkir-v3` selector preserves
schema20/ABI54 for prior invocations. Actual original285 material exposed a
nested Cell.write lowering refusal; a focused generic ordered scalar-widening
fix is under execution qualification. Original588 emits both IR profiles, but
the source branch still emits `[0,8]` hash alignment rather than the accepted
host99d/released `[1,8]`. Equality between those two source profile outputs is
not mainstream parity. The already-qualified host minimum-byte frontend fix
must be integrated as a distinct semantic slice, then new artifact identities
and qualification recorded. Archived Events/CCC packages and proofs remain
unchanged; neither selector-only captures nor unit-suite results replace that
gate. Failed Scheme imports/pass loads and partial fixture captures are kept
as historical execution evidence.
