# Ledger 9 upstream compatibility

The Rust AST backend must accept the same Compact programs as mainstream
Ledger 9, preserve their observable behavior, and expose the capabilities in
typed generated Rust. The Rust AST development line retains its existing
backend-specific guarantees and historical Ledger 8 coverage.
Track completion in [#726][tracking].

## Baselines

- The [upstream test and language baseline][upstream] is
  `672bd0f226f47a7d036c5c074ce71fbe1af1925e` on
  `LFDT-Minokawa/compact` main, with compiler 0.36.103 and language 0.28.0.
  The upstream development lock currently uses Onchain Runtime 4.0.0-rc.3.
- The [released Compact 0.36.0][release] and [Ledger 9.1.0.1][ledger9]
  matrix define SDK interoperability. Use the published Ledger 9 package pins,
  Rust 1.99 and the qualified ZKIR toolchain from the
  [Ledger 9 adoption plan](ledger9-adoption.md).
- The initial candidate is the [PR #725][pr725] head
  `9fb0ca9c7c7e075461d3122b9f606a00c88de06a`. The compatibility branch
  starts from that revision in an isolated local worktree.

## Owners and coverage

| Area | Source owners | Coverage to port or add | Issue |
| --- | --- | --- | --- |
| Upstream corpus | `compiler/test.ss`, `test-center`, `test-contracts`, `tests-e2e`, and the suites below | Match assertions and diagnostics per case. Retain compile and runtime pass/fail fixtures, inputs, generated-interface checks, and slow cases. Run a small portable fixture family through both targets first. | [#727][coverage] |
| Frontend and ZKIR | Scheme parser, language and analysis passes, circuit and ZKIR v3 passes, Rust typed IR and checked lowering | Language and disclosure semantics, empty tuples, operation and value packing, v3.1 output and commitments, Rust and TS interfaces and source refusals. Preserve historical v2 checks while migrating the current producer. | [#728][compiler] |
| Curves, signatures and hashes | Standard library, primitive typing and lowering, ZKIR, TS runtime and Rust runtime primitives | secp256k1, secp256r1 (P-256), Curve25519 and Ed25519, SHA-512 and Keccak, upstream known-answer and Wycheproof vectors. Check wrong keys/messages, identity and malformed encodings where defined by upstream. | [#729][crypto] |
| Cross-contract calls | External interfaces, contract-call desugaring, checked Rust recording, generated APIs, ZKIR commitments, ledger transaction | Original composable tests, call-site circuit-name binding, wrong address/entrypoint/verifier, atomic rollback, transcript integrity, reentry and kernel caller. Apply a real two-contract transaction and refuse its replay. Include [#72][transcripts] and [#73][names]. | [#730][calls] |
| Public state and ABI | Runtime representations, generated Rust and TS ABI, ledger ADTs, event sinks and checked decode | Preserve upstream vectors, alias and encoding cases, malformed-input refusals, public event visibility and order, and serialized state parity. | [#731][abi] |
| SDK and host tooling | Runtime packages, CLI and package resolution, editor, source maps, formal specification, release workflow | Port or verify host-only suites on their owners. Check clean SDK resolution, artifacts and packaging, optional macOS and required native Linux. Run the complete existing and new backend gates. | [#732][integration] |

### Test inventory and dispositions

Run `python3 tools/compact-rust-backend/upstream_ledger9_inventory.py --candidate 9fb0ca9c7c7e075461d3122b9f606a00c88de06a`
from a checkout with the pinned upstream Git object. It compares all 1,964
tracked upstream files at their original paths. At the initial candidate, 556
match by mode and object identity, 287 differ and 1,121 are absent at those
paths. These are file counts. A matching test file still needs its compiler,
runner and dependency admission verified.

The standalone suite accounts for 943 files, including 372 Compact sources,
185 compile-pass, 173 compile-fail, 174 runtime-pass and 6 runtime-fail
declarations. Six declarations live under a `slow` path. An individual
declaration can drive many inputs, including cryptographic vectors. Compiler
Scheme cases, runtime tests, generated contract and composable tests, compiler
E2E, source-map tests and host tools have separate runners. Tests embedded in
CLI, editor, release actions and specification owners also belong in the
inventory. Check upstream workflow entry points as well as path names.

For each upstream test or parameterized family, record the pinned source path
and case, original input and assertion, port path, lane, and disposition.
Verify an existing equivalent by matching its input, expected behavior and
execution. Record unresolved or genuinely host-specific cases with a reason
and owner. Keep upstream fixture bytes and their provenance when the source
parses as-is. If an adapter is needed for the synchronous runtime, keep the
inputs and expected values, diagnostics and refusals. Keep original slow and
platform-specific cases runnable in their own lanes. The ledger of cases must
account for every suite before integration can be called complete.
The candidate reports language 0.23.105. A probe that removes a newer
language pragma is diagnostic evidence. Admission of an intact fixture
requires implementing and qualifying the language features it uses.

### Sequence and integration gate

1. Port small frontend semantic fixes with their original tests. Establish a
   reusable test mapping with one standalone primitive family and a generated
   Rust consumer. Port the remaining compatible fixture families in bounded
   groups.
2. Align language, generated ABI, ZKIR and runtime primitives as required by
   failing upstream cases. Qualify curves and signature operations in stages.
   Bring up a two-contract call starting with authenticated interface and
   transcript binding. Gate transaction support on the original composition
   suite and funded acceptance.
3. Run the entire mapped corpus, including SDK, CLI, editor, source-map and
   formal checks, against the release interoperability matrix. Run all
   existing required native Linux checks. Keep macOS optional.

For a proof-bearing feature, bind each receipt to exact source closure,
compiler recipe, raw IR, generated interface, tool, SDK, and verifier
identities. Generate fresh proofs, verify independently, and use a separate
funded transaction under default strict verification. Retain replay refusal
and unchanged state, verifier and binding negatives, and atomic rollback
where applicable. The upstream composable in-memory test harness checks call
preimages using synthetic keys. The real two-contract transaction has its own
gate. Compare complete serialized state and public outputs and retain
historical Ledger 8 fixtures and proof receipts. Keep the existing
`k <= 8` automatic test limit and route larger upstream proof cases to a
declared slow lane.

### Cryptography and cross-contract sequencing

The pinned ZKIR tool provides Keccak, SHA-512 and the foreign-curve
instructions. ZKIR v3 generation now selects upstream fixed-width Keccak
encoding in both Rust and TypeScript. The v2 profile and archived
fixtures retain normalized FAB atoms.

Seven trailing-zero inputs passed through freshly generated Rust and
TypeScript. Native and recorded Rust agreed on the stateful digest.
The ZKIR interpreter accepted the generated TypeScript preimage and
refused a public write using the historical digest. The
[upstream Keccak corpus](../../../test-contracts/stdlib/keccak256/README.md)
retains all nine original fixture families and their assertions. The local
arm64 corpus gate checks nine compile outcomes and seven runtime families
per target, including the original ADT refusal. At language 0.23.105 it
first verifies each intact fixture refuses its 0.25.0 pragma, then runs
pragma-only derivatives. Intact language admission and the k=14 stateful
proof lane remain open.

The ordinary SHA-512 port follows the upstream native signature and
fixed-width input encoding. Both generated targets return `Bytes<64>`.
ZKIR v3 uses the pinned three-limb digest split. The
[ordinary fixture](../../../runtime/test/fixtures/ledger9-upstream/sha512_ordinary.compact)
covers fixed-width and empty bytes, tuples, a Field, and public writes over
64-byte input and a tuple containing a Jubjub point. It shares the existing
ledger alignment rules for point coordinates. The local ZKIR tool measured
both public writes at k=14, so they require a slow proof lane. The intact
[secp256k1 point-pair fixture](../../../test-contracts/stdlib/secp256k1/point/upstream-sha512.compact)
uses the upstream 5G and 7G inputs. Both runtimes produced the same digest
and byte-identical serialized preimages. The current and upstream v3 IR
agree on the ten aligned point atoms and ledger write after accounting for
the version and temporary names. The SDK refused swapped inputs and a
changed digest. On macOS arm64, a fresh k=14 proof passed separate-process
verification and wrong-verifier, changed-binding and corrupt-proof controls.
A separately proved funded transaction built from generated Rust passed
default-strict Ledger 9 verification, applied the digest and refused
replay with byte-identical ledger state.

The secp256k1 base and scalar carriers now use distinct checked Rust types
and the pinned TypeScript descriptors. Their ledger encoding stores the
value minus one modulo the field modulus in a low 24-byte and high 8-byte
atom. The unchanged upstream fixtures exercise the base Cell roundtrip and
six invalid casts across Rust, TypeScript and paired compilation. The
[scalar and default fixture](../../../runtime/test/fixtures/ledger9-upstream/secp256k1_field_defaults.compact)
adds typed write, read and reset cases. Both targets agree on the encoded
values. The Rust VM replays all twelve checked runs, and the pinned ZKIR interpreter
refuses changed ledger writes. The base and scalar roundtrips are k=11 and
the reset is k=12. Fresh proofs and funded application for these statements
remain in the slow lane.

Explicit secp256k1 casts accept exactly `Bytes<32>`. Casts into either
field reduce the little-endian integer modulo its field order. Casts out
return the canonical little-endian bytes. Host decoders still reject
noncanonical values, and ledger values still use shifted encoding. The
[cast fixtures](../../../test-contracts/stdlib/secp256k1/positive-casts)
cover the pinned guarded constants and zero, dynamic modulus boundaries
and width refusals in all target modes. On local arm64 the generated Rust
VM replay matched native execution for 31 cases. The ZKIR interpreter
accepted the same cases and refused changed transcripts or commitments.
All seven exports are k=10–12. Fresh proofs and funded application remain
open in the slow lane.

The secp256k1 Base and Scalar types now support same-field addition,
subtraction and multiplication through both generated targets and ZKIR v3.
The [arithmetic fixtures](../../../test-contracts/stdlib/secp256k1/arithmetic)
preserve the pinned upstream test blocks, including typed-zero
self-subtraction. Generated Rust and TypeScript agree on all 37 original
K1 cases. Rust VM replay and the pinned ZKIR preimage check cover each
case. The interpreter rejects a changed ledger write for each one.
All nine stateful exports are k=11. Fresh standalone proofs for base
addition, scalar multiplication and inlined self-subtraction verified in
separate processes. The verifier refused changed binding, different
verifier and corrupted proof controls. A separate funded base addition
transaction applied under default strict verification and replay returned
`IntentAlreadyExists` with byte-identical ledger state. The remaining
arithmetic proof and application matrix is open.

The secp256k1 Base and Scalar `neg` and `inv` overloads now lower to
typed Rust, TypeScript and ZKIR. The
[unary fixtures](../../../test-contracts/stdlib/secp256k1/unary)
retain both original upstream ZKIR 3.1 expectations alongside checks
against the current 3.0 profile. Active inversion of zero fails even
when the result is discarded. Guarded ZKIR inversion supplies a typed
one to the eager VM operation when the branch is inactive. The pinned
interpreter accepted all 30 stateful preimages and refused six
active-zero inputs and 30 changed writes. Five fresh standalone proofs
covered both original fixtures, guarded branches and a discarded
inverse. A separately proved, funded inactive-zero call applied under
default strict verification. Replay returned `IntentAlreadyExists`
with byte-identical ledger state. Fallible constructor inversion and
the rest of the proof and Linux matrix remain open.

The secp256k1 `ecAdd`, `ecMul` and `ecMulGenerator` overloads now use
typed points and scalars in both generated targets and ZKIR v3. The
[group fixtures](../../../test-contracts/stdlib/secp256k1/group)
preserve the original computed-identity refusals and cover canonical
identity storage and the existing Jubjub overload. All seven stateful
exports had fresh proofs verified in separate processes on macOS arm64.
The generated Rust `storeXOfMul` call also passed funded default-strict
application and refused replay with identical ledger state.

The upstream `secp256k1EthereumAddress` circuit and its byte-order helper
now compile through both targets without changing the Compact source.
The [original fixtures](../../../test-contracts/stdlib/secp256k1/address)
include the exact address vector and the complete issue #608 contract.
Rust validates fixed Bytes-to-vector conversions and constant byte-slice
bounds in checked lowering. The paired Keccak circuit uses the original
nested point encoding and the second ledger slot.

The upstream `secp256k1EcdsaVerify` circuit and typed signature now
compile through both targets. The [verification fixtures](../../../test-contracts/stdlib/secp256k1/ecdsa)
preserve the original false and identity cases and the #609
two-signature optimizer regression. The executable coverage binds the
message digest and distinguishes each private signature witness.
The fixed vector-to-Bytes conversion preserves width and byte order.

The [owner-binding fixture](../../../test-contracts/stdlib/secp256k1/recovery)
uses the original Compact contract with generated TypeScript and Rust.
The Rust host recovery API accepts a fixed 32-byte digest, checked scalars,
and a recovery ID in 0–3. It uses the Ledger SDK's pinned `k256` version
and accepts the high-s signature with its matching recovery parity. The
recovery ID selects the host witness. The circuit verifies that point's
signature and checks its derived address against the recorded owner.
The [upstream known-answer suites](../../../test-contracts/stdlib/secp256k1/known-answer)
retain the original Bitcoin Wycheproof and Ethereum address contracts,
test callbacks and support data. At language 0.23.105 the three intact
fixtures refuse the 0.25.0 pragma. The opt-in probe removes only that
pragma in separate copies. On those copies both targets cover 244 driven
cases per Bitcoin suite, including 57 scalar-boundary refusals and ten
circuit aborts, 162 recovery round trips and two Ethereum addresses.
The [slow fixture runner](../../../test-contracts/stdlib/secp256k1/slow)
preserves the original address-write and sequential-signature contracts
and compile declarations. The current compiler refuses both intact
0.25.0 pragmas. On pragma-only derivatives the SDK measured k=16 for
the address write and k=17 for both signature folds. The paired harness
checks ordered witnesses, independently substituted signatures and
keys, and the exact address return and ledger effects. Intact-source
admission remains open.

The [P-256 field fixtures](../../../test-contracts/stdlib/secp256r1)
exercise the original base-field roundtrip and all six cast refusals.
Generated Rust has separate base and scalar types with checked
construction. Both runtimes preserve the shifted 24-byte and 8-byte
ledger atoms. The explicit slow lane covers the original base roundtrip,
the scalar roundtrip, and the two-field reset.

The [Curve25519 field fixtures](../../../test-contracts/stdlib/curve25519)
retain the two upstream round trips and six cast refusals. The Base and
Scalar types have distinct Rust carriers. The Scalar ledger representation
splits at bit 204 into 26- and 7-byte FAB atoms. Both intact round trips
run in the explicit slow proof lane at k=11.

The [Curve25519 point fixtures](../../../test-contracts/stdlib/curve25519/point)
retain the original ledger roundtrip, coordinate projection, equality and
point serialization refusals. Both targets also pass the original
Maybe/Either defaults. Generated Rust accepts canonical affine subgroup
points and uses four shifted Base atoms for ledger transport. The original
generator and its double pass Rust recording and VM replay. The write and
projection statements have independently verified arm64 proofs at k=12
and k=11, respectively. The write also passes a separate funded
default-strict application and replay refusal. The intact upstream point
serialization and deserialization fixtures refuse at the codec boundary
with the same operation and point type diagnostics as mainstream.

The [wire codec fixtures](../../../test-contracts/stdlib/serialization)
cover Boolean, Bytes, native Uint and Field encoding and decoding.
The aggregate slices cover the five upstream product encoders and
decoders, the intact `d1`/`d2` checked-Boolean fixture and the original
`foo`/`unfoo` alias roundtrip. The local proof lanes cover `d2` at k=11,
`foo` at k=12 and `unfoo` at k=13. Enum codecs use checked declaration-order
ordinals, with stateful coverage at k=7. Wide Uint codecs cover 129-bit
and 248-bit inputs. The stateful 129-bit write has a local k=8 proof
lane. The six secp256k1, P-256 and Curve25519 field codecs use mainstream's
32-byte encoding and modular decode. The stateful lanes measure k=10.
Empty vectors occupy no wire bytes even when their element type lacks a
codec. The pinned mainstream compiler and Compact 0.36.0 both compile
the empty point-vector fixture. Both references fail internally for the
tested JubjubScalar wire codecs. The Rust AST branch reports a source
error for those codecs.
Broader aggregate proofs and native Linux qualification remain open.

The [event fixtures](../../../test-contracts/stdlib/events) cover
`ShieldedSpend`, `ShieldedBurn` and `ShieldedMint`. The intact Burn and
Mint fixtures exercise single, ordered, conditional and absent emissions
plus their 49-byte and 81-byte wire codecs. The SDK checks both outcomes of
conditional emission and rejects swapped event order and Mint domain
separator substitution. These checks use the original upstream circuit
exports.

`ShieldedReceive` follows upstream `7489a7926` and preserves commitment,
optional ciphertext, then optional contract address in its 578-byte
payload. The original seven exports cover both conditional outcomes,
single and ordered emissions, and structural preservation of inactive
fields. Additional controls reject invalid optional flags, substituted
addresses and swapped event order.

`UnshieldedSpend` carries the full `Either` sender in its 145-byte
payload. The original fixture emits with each sender variant. The
codec controls distinguish both sender branches, the domain separator,
the token type and the amount, and refuse a malformed selector.
`UnshieldedReceive` uses the same structural shape for its recipient
and emits with its own tag. Both targets preserve inactive recipient
bytes and reject an undisclosed witness in either branch.
`UnshieldedMint` emits tag 6 with the domain separator, token type and
little-endian amount. Its codec checks use distinct byte fields and the
maximum `Uint<128>` value.
`UnshieldedBurn` uses tag 7 and a 113-byte payload consisting of both
sender branches and their selector, the token type, and the amount.
The fixture directory retains the original type and codec cases and
identifies the standalone emission test as derived.
`Paused` and `Unpaused` have empty payloads and distinct tags 8 and 9.
The intact upstream Paused fixture preserves all seven exports and its
eight JavaScript assertions. A derived transition fixture checks that
both tags survive recording and replay in order.
`Misc` uses tag 10 and encodes its 32-byte name followed by its
256-byte payload. The intact standalone fixture exercises all seven
exports. The mixed Unpaused/Misc assertion runs against the local
flattened IR, with the same operation order and payload operands.
The whole codec assertions for all three types run at local type
inference, where serialization is expanded.

The supplemental group fixture covers addition, scalar multiplication,
generator multiplication and zero and identity behavior. Its model is
k=17. The remaining original point fixtures and proof coverage for the
group fixture remain open.

[Ed25519 verification](../../../test-contracts/stdlib/curve25519/verify)
uses the mainstream Compact implementation, with the original stateful
callback covering sign-bit parity, changed messages, signatures and keys,
and identity and mixed-order key refusals. Both original stateful exports
compile for Rust and TypeScript with checked recording, and have fresh
independently verified arm64 proofs at k=17. The RFC 8032 one-byte case
passes a separate funded default-strict application and replay refusal.
The 151-vector corpus and sequential witness fixtures are preserved.
Their original 0.25.0 pragmas refuse on the current language version.
Pragma-only derivatives exercise the corpus and the ordered two- and
three-signature witnesses in both generated targets.

The [P-256 byte cast fixtures](../../../test-contracts/stdlib/secp256r1/casts)
cover modular little-endian conversion at the `Bytes<32>` boundary and
guarded ledger writes. Both targets reject the original invalid widths and
direct casts between different fields. The two original mixed-curve
Scheme tests are retained in full. Their P-256 cases execute in separate
fixtures. The [full mixed K1/P-256/Curve25519 cases](../../../test-contracts/stdlib/secp256r1/casts/mixed-original/README.md)
now execute all original assertions in generated TypeScript and Rust. Their
`castNative` Boolean comparison has a fresh k=8 independently verified proof
and separate fee-funded default-strict application with replay refusal.
The original Curve25519 `castC25519` false and true branches and `castWide`
reduction also have [fresh k=11/k=10 proofs and separately funded strict
applications](../../../test-contracts/stdlib/secp256r1/casts/mixed-original/README.md),
each with changed-write, verifier, binding, corrupt-proof and replay refusals.
These results are local macOS arm64 only. Remaining original exports retain
open proof and funded transaction coverage, and native Linux remains required.
The [P-256 arithmetic fixtures](../../../test-contracts/stdlib/secp256r1/arithmetic)
retain the original six stateful exports and 34 assertions. The generated
Rust and TypeScript preserve separate base and scalar operands and
returns. The proof lane checks changed ledger writes and returns,
then independently verifies a selected statement per export and
applies a separate funded transaction. The [P-256 unary fixtures](../../../test-contracts/stdlib/secp256r1/unary)
retain both original negation and inversion pairs. They verify field-specific
zero refusal, inactive branches, discarded inverses and overload resolution.
The explicit proof lane covers both original fixtures and both inactive-zero
branches.

The [P-256 point fixtures](../../../test-contracts/stdlib/secp256r1/point)
retain the original equality and identity-hash assertions. Rust and
TypeScript encode the same five atoms and reject off-curve finite inputs
at their typed boundaries. The three point-family tags are checked in
contract info and Rust IR. The upstream JavaScript circuit signature export remains open.
The [P-256 accessor fixtures](../../../test-contracts/stdlib/secp256r1/accessors)
run the intact point roundtrip and original direct, ledger and checked
assertion cases. The proof checks cover guarded active and inactive
paths with the correct curve and both coordinate-specific identity errors.
The [P-256 group fixtures](../../../test-contracts/stdlib/secp256r1/group)
port typed point addition, multiplication and generator multiplication.
They run the intact computed-identity test and check canonical identity,
finite coordinates, guarded results and cross-curve rejection. The
explicit slow proof lane covers all four original exports and both
branches of each guarded group operation. The
[P-256 transport fixtures](../../../test-contracts/stdlib/secp256r1/transport)
preserve both original point witnesses and the issue #608 tuple hash.
They check the five private FAB atoms, witness selection and private
state, ordered hashing, and refusal of altered preimages. The
[P-256 ECDSA fixtures](../../../test-contracts/stdlib/secp256r1/ecdsa)
preserve the upstream signature type and verification body. They
check the original in-circuit message binding and both s values,
and account for the complete Wycheproof corpus. The original
standalone pragma remains a release gate. The
[P-256 signature suites](../../../test-contracts/stdlib/secp256r1/slow)
retain all eleven SHA-256 exports and both sequential witness circuits.
They account for the original corpus, check witness ordering and
substitutions, and use the explicit slow proof lane. Nested point
collections remain open.

The P-256 host recovery helper follows the pinned upstream TypeScript
API and preserves all nine original recovery tests. Rust exposes
`secp256r1_ecdsa_recover` with a fixed 32-byte prehash, P-256 signature
scalars and a recovery ID bounded to 0–3. Both runtimes cover the
alternate recovered key, high-s signatures, wrapped nonce coordinates
and refusal of a recovered identity. Applications that require a
specific signer must compare the recovered key to their trusted key.

The [Jubjub scalar fixtures](../../../test-contracts/stdlib/jubjub/scalar)
preserve the upstream Uint cast and the Boolean and enum refusals. Generated
Rust takes a checked `JubjubScalar`. Generated TypeScript uses bigint and
checks the scalar range. Both targets preserve the one-field ledger encoding.
Explicit Field and Uint casts reduce modulo the scalar order, and casting a
scalar to Uint checks the target bound. The local qualification runs
the ledger fixture through typed cell access, recorded replay and
separate default-strict transactions.

The two pinned mixed Jubjub/K1 and Jubjub/P-256 collection cases run their
original TypeScript assertions with an adapter for this branch's call context.
The Rust port gives a declared child Map typed initialization, insertion and
lookup operations. A scalar read emits the same physical array path and typed
outer key in one VM instruction, followed by the typed inner key. Both
intact mixed cases generate ordinary and recorded Rust and retain the same
ZKIR. Local Rust checks cover generated values, state, gas and VM replay.
The focused child Map write and lookup measure k=8 and k=7. Their local
qualification includes fresh proofs, independent verification, rejected key
and result substitutions, and separate funded default-strict transactions
with unchanged-state replay refusal. The original k=17 and k=18 mixed
circuits remain open for proof and funded transaction qualification.

Port foreign field and point layouts, defaults, arithmetic and checked
decoding before signature circuits. Separate K1, P-256 and Ed25519
verification and test their distinct scalar, identity, subgroup and encoding
rules. Preserve upstream vector exclusions with reasons and test malformed
encodings at the decoder boundary. Use SHA-512 and canonical scalar
conversion for Ed25519.

Start cross-contract calls with the pinned EntryPoint caller and callee
fixtures and one provable fixed callee. Bind the address, compile-time
`EntryPointBuf::ep_hash` and arguments/result commitment to the caller's
claim. Preserve the same-shape sibling-entrypoint refusal before enabling
general calls. Authenticate the observed installed operation and expected
verifier, and combine both calls using the Ledger 9 transaction API's joint
partitioning. Prove and independently verify both calls, then apply the
funded transaction and test replay and invalid-binding refusals.

Both Rust entry points look for the callee's `compiler/contract-info.json`
beside the requested output. The interface-only gate covers missing,
malformed, stale and mismatched metadata and preserves the previous complete
output on refusal. Direct TypeScript compilation with a trailing output
separator remains a CLI portability case.

The exact upstream EntryPoint caller, callee and sibling-substitution tests
are retained under `test-center`. The derived `AddressCompare.compact`
fixture exercises constructor arguments, typed ledger reads and equality of
contract references. ZKIR v3 uses one 32-byte atom backed by two scalar limbs.
Generated TypeScript and Rust share byte-identical ZKIR for the comparison.
Generated Rust exposes a distinct address carrier for the declared interface.
The internal compiler probe now emits the intact caller's statement with the
immutable entrypoint hash. A standalone arm64 proof with an adapter-produced
preimage verified independently. The verifier refused a same-output sibling
entrypoint, a different verifier, a changed binding and a corrupt proof.
Both ordinary targets still refuse the intact caller at the call site.

The Rust recording runtime emits the same kernel claim as the TypeScript
compiler, including the callee's address, entrypoint hash and communication
commitment. Replay preserves the order of repeated claims. Single-call
preparation rejects any trace with such a claim. The local runtime can
prepare one caller and one leaf callee from the same ledger snapshot. It
checks the installed verifiers and exact claim before invoking Ledger 9's
joint partitioner. Preparation tests cover the honest pair and refusal
controls with verifier carriers.

Generated cross-contract invocation and funded proof qualification are on
hold while the remaining Ledger 9 compatibility work continues. The
intact EntryPoint transfer model requires k=9. For generated invocation,
follow the pinned mainstream module resolution, interface conformance and
verifier checks before entering the callee context. Restore the caller and
record the callee's result and commitment opening in the caller witness.
Use an explicit slow lane for the transfer proof, verify both proofs and
exercise strict application, replay and rollback.

The upstream compiler runs 11 composable TypeScript suites with 47 active
tests and five explicitly skipped reentry cases. Four composable compiler
E2E paths have 48 active upstream cases. Also track the expected-verifier
compiler E2E suite and 105 direct cases in upstream conformance, contract
resolution, cross-contract Zswap, module resolution and verifier-key hash
runtime suites. Preserve the original tests and add transcript immutability
and own-property name controls to address [#72][transcripts] and
[#73][names].

### Initial ports

The [upstream assertion fix][assert] gives a successful `assert` the empty
tuple value. Port its two conditional witness-analysis cases, generated
TypeScript execution and refusal case, and the runtime unit assertion.
The baseline compiler failed internally when a conditional combined
`assert(...)` and `[]`. Generated Rust keeps the typed `()` result and
`CompactError::AssertionFailed` for refusal. This slice is tracked under
[#728][compiler].

Local macOS arm64 qualification passed 3,891 Scheme harness checks across
both ZKIR modes, six source-map tests, 82 runtime tests, and a Rust 1.99
consumer compiled from fresh both-target output. The consumer exercised both
conditional orderings, bound, direct and nested empty tuples, and the typed
assertion failure. The fixture used the off-chain pure-circuit API.

The [vector-spread disclosure fix][spread] preserves nested Boolean analysis.
Before the fix, the intact upstream refusal fixture compiled to TypeScript.
The patched compiler diagnoses the undeclared disclosure on both targets
without publishing output. The original fixture and refusal metadata are
retained at their upstream paths. The existing capability gate runs its
diagnostic against both targets. The port includes all six Scheme forms from
the upstream change, and the local run passed 3,898 compiler checks, six
source-map tests and the capability gate. General Rust vector spreads remain
open.

The [deep Merkle port][merkle] aligns the v3 immediate with Ledger 9's
depth and entry count fields. The two intact upstream families exercise
plain and historic depth-32 trees and defaults in nested maps. The v3
interpreter checked eight TypeScript preimages and refused mutated depth
values. Generated top-level Rust insert and reset tests also passed.
The intact plain and historic fixtures generate typed Rust slots for the
nested map declarations. `insertDefault` and explicit default insertion
both preserve the declared depth and the SDK's nested value shape.
Historic map defaults begin with an empty root history. Top-level reset
seeds the blank root as before. Nested tree values have no scalar codec.
The eight original ZKIR models are unchanged. Insert measures k=13,
reset k=7 and the map-default exports k=6 in each family.

The [Boolean cast family](../../../test-contracts/primitives/boolean/README.md)
preserves two upstream sources, two compile-pass declarations and two
runtime-pass declarations byte-for-byte. The local adapter ran four
TypeScript executions and two Rust 1.99 tests against the integrated
compiler. Compiler metadata identifies both exports as pure circuits.

The [nominal Uint arithmetic family](../../../test-contracts/primitives/uint/nominal/README.md)
preserves the two upstream programs and their compile/runtime declarations.
The frontend omits a downcast when nominal operands and arithmetic result
have the same range. The original small-range values and overflow refusal
run through both generated targets as pure circuits; no proof or funded
transaction is associated with these exports.

The [opaque-string port][bom] preserves leading U+FEFF on decode. The
intact upstream ledger fixture exercises receipt, archive, equality and map
membership. The macOS arm64 run passed all 88 TypeScript runtime tests,
the compiler suite in both ZKIR modes, and fresh generated ordinary and
recorded Rust consumers. The six raw v3 ZKIR files matched across targets.
Proof, transaction and Linux qualification for this stateful fixture remain
open.

[upstream]: https://github.com/LFDT-Minokawa/compact/commit/672bd0f226f47a7d036c5c074ce71fbe1af1925e
[release]: https://github.com/LFDT-Minokawa/compact/releases/tag/compactc-v0.36.0
[ledger9]: https://docs.midnight.network/relnotes/ledger-9
[pr725]: https://github.com/betsudo/compact/pull/725
[tracking]: https://github.com/betsudo/compact/issues/726
[coverage]: https://github.com/betsudo/compact/issues/727
[compiler]: https://github.com/betsudo/compact/issues/728
[crypto]: https://github.com/betsudo/compact/issues/729
[calls]: https://github.com/betsudo/compact/issues/730
[abi]: https://github.com/betsudo/compact/issues/731
[integration]: https://github.com/betsudo/compact/issues/732
[transcripts]: https://github.com/betsudo/compact/issues/72
[names]: https://github.com/betsudo/compact/issues/73
[assert]: https://github.com/LFDT-Minokawa/compact/commit/7517ecef491cd9655cf288cd7ae96d1927e12070
[spread]: https://github.com/LFDT-Minokawa/compact/commit/3a560ed6827510b13cc177d44fe80eb6d674b6a8
[bom]: https://github.com/LFDT-Minokawa/compact/commit/9370308cf
[merkle]: https://github.com/LFDT-Minokawa/compact/commit/ded731230264e6db7f8383a434d9ac51ba9efff8
