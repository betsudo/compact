# Root Set read proof-data qualification

This grouped follow-up to the exported Map facade covers root `Set<Field>`
reads and the terminal Unit return fix in [#800](https://github.com/betsudo/compact/issues/800).
The migration baseline remains pinned mainstream
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`, released Compact 0.36.0 and the
Ledger 9.1.0.1 SDK matrix. Results below are local Darwin arm64 subsets; the
Ledger 9 migration remains incomplete.

## Behavior and maintained tests

The intact upstream fixture retains all six exports: reset, insert, emptiness,
membership, remove and size. Its 1,166-byte source and separate 1,459-byte,
ten-assert test are preserved in
[`fixtures/upstream-672/export-set-field`](../../../tools/compact-rust-backend/fixtures/upstream-672/export-set-field).
The frontend now keeps terminal Set mutation results as Unit while executing
the original operation exactly once. Reset dispatches the actual upstream
`remove` primitive. Assertions and refusals are not weakened.

Opt-in `is_empty_proof_data`, `member_proof_data` and `size_proof_data` use
one real recorded invocation each. They preserve the private context without
requiring `Clone` or an unrelated witness implementation. Native and recorded
ordinary APIs, proof-data API1 and runtime ABI54 remain intact. A compile-only
consumer type checks all three helpers and the ordinary membership/size methods
against the complete freshly generated crate, with `Private(91)` deliberately
implementing neither Clone nor witnesses.

The backend admits only the exact supported root Set shape. Membership pins
its own 751-byte raw program with `%value.0`; it does not normalize Map SSA
names. Size pins the genuine 597-byte raw program and returns a bounded Uint64.
Absence returns false. Size carrier boundary tests do not construct a Set with
u64::MAX members. Existing one-helper emptiness output and the exact
three-helper successor roster are handled separately.

Classifier/generator tests live in `tools/compact-rust-backend`; SDK projection
and refusal tests live in `tools/compact-rust-proof-data/tests`. The three native
capture adapters remain separate domain fixtures; shared capture/codec,
existing Map profiles, runtime source, dependency manifests and Cargo.lock are
unchanged.

## Local validation

Focused and full counts overlap and must not be added together.

| Gate | Result | Scope |
| --- | --- | --- |
| Unit-return frontend | 24 focused checks; 4,041 full compiler checks; 20 native/recorded/VM calls | Earlier source-bound frontend qualification |
| Current backend | 2,501 tests across 59 harnesses; seven existing ignored tests; strict Clippy | Includes the three Set helper families |
| Current SDK tools | 77 Rust tests across 11 harnesses; 28 Python tests; strict Clippy and both tool builds | Current source graph; no full SDK-matrix claim |
| Normal generation | 17 export artifacts and 12 original-test artifacts; all four phases pass | Both original inputs remain byte-exact; all six ordinary APIs and three helpers retained |
| Actual generated client | Locked/offline Cargo check passes; 308 selected packages, four exact local packages | Complete generated crate, retained runtime and non-Clone client; no method execution inferred |

The first new generator tests failed two textual return-type expectations.
The reviewed correction inspects structured Rust types while preserving exact
return types, invocation counts and refusal assertions. No production behavior
was changed for those expectations. A separate client-driver attempt stopped
before compilation because its metadata assertion expected only the client as
a workspace member; Cargo includes its generated path dependency too. The
corrected assertion admits exactly those two members, retains the same four
allowed local manifests, checks the consumer resolve root and keeps every
external lock tuple/checksum within the trusted seed. First failures are retained.

## Fresh proofs and separate verification

| Read | Measured k / rows | Complete fields / None skips | Fresh proofs | Returned SDK calls |
| --- | --- | --- | --- | --- |
| Emptiness | 6 / 41 | 18 / 6 | Four, 3,845 bytes each | Eight accepts, twelve refusals |
| Membership | 7 / 62 | 17 / 5 | Four, 3,845 bytes each | Eight accepts, twelve refusals |
| Size | 6 / 35 | 12 / 4 | Four, 3,845 bytes each | Eight accepts, twelve refusals |

Emptiness is the earlier separately qualified helper epoch. Membership and
size are new runs against the retained three-helper compiler. Each public run
passes all 18 phases within the automatic max-k8 lane. Model/check evaluation
precedes parent parameter access; the k13 opt-in policy is unchanged.

Each native lane uses four fresh contexts through ordinary, recorded and helper
routes. It compares supported context/state/gas, canonical typed I/O, every
ordered query operand/alignment, helper-once behavior and official VM Verify
replay. Membership covers absent zero, present zero, present wide value
2^128+1, and removed zero while retaining the wide value. Size covers 0/1/2/reset0
and verifies duplicate insertion preserves the complete native state.

The separately authored verifier reconstructs complete statements from public
typed I/O, raw program and official SDK operations. All twenty SDK calls per
read return: own/default-parameter positives accept; changed binding,
outer-canonical proof corruption and a current initialized distinct same-k key
refuse. The verifier reads no producer PI, preimage or prover-key bodies.
Before-state authenticity is admitted rather than regenerated independently;
SDK cryptographic primitives are shared. These are standalone proof checks,
not funded transaction application or replay qualification.

## Released runtime parity and SDK1 scope

All three staged TypeScript lanes pass strict TSC with library checks enabled
and four Vitest cases each against released runtime 0.21.0. Emptiness uses
26 genuine JS/WASM checks and awaited flushes; membership uses 30 and size 33.
Membership compares all twenty query operations and size all sixteen, including
operands/alignment and all four query-only gas dimensions. Complete logical
Set states agree; whole-ledger rendering invariants are checked within each
runtime. Whole cross-runtime ledger encodings and full PI equality are not
inferred from checker vectors. Worker network/independent heap observations
remain unknown. No TypeScript cryptographic proof generation is claimed.
Raw observation flags such as `nativeParity=PENDING`, `qualified=false` and
`SDKRuntimeConfirmed=false` remain unchanged. Separate stopped-run comparisons
reconcile the ordered operations and gas; they do not rewrite those observations.

The earlier released SDK1 proof-wire subset verifies all four retained
emptiness proofs with eight accepts and twelve refusals; all eighteen fields
and six skips agree with current SDK. That registry graph uses transient-crypto
3.0.1, proofs 0.8.3, serialize 1.2.1, stdlib 2.4.0 and curves 0.3.1. It generates
no new proofs. Membership and size released-SDK1 verification remain open.

## Artifact binding and remaining work

Normal generation observed compiler 0.31.222, retained Rust `e9da06ea`,
Scheme `488484c2`, a closed 773-file package and 771 unchanged runtime files.
The new backend batch guards 9,915 source files; integrated SDK qualification
guards 9,923. The earlier emptiness proof compiler uses Rust `f7a92c06`.
Original ten-assert native/TypeScript execution remains its earlier epoch;
these supplemental four-case lanes add no original-case inventory credit.
The released 0.36.0 interoperability target does not relabel local executables.

Detached signatures, source manifests, raw results, original failures, proofs,
keys and parameters are retained locally. Binaries reused after tests within
one unchanged-source batch are identified as warm reuse, not cold final builds.
Completed exclusive Cargo caches are removed after executed binaries are
retained; qualification stores and source snapshots remain.

Full SDK1 interoperability, [native Linux #797](https://github.com/betsudo/compact/issues/797),
remaining original suites and host-only tooling remain open. Held EntryPoint,
funded application/replay and real two-contract gates stay governed by the
authoritative checkpoint. Ledger 8, archived Forge evidence, existing stores
and the canonical three-file EntryPoint WIP are preserved. This PR closes none
of those broader milestone requirements.
