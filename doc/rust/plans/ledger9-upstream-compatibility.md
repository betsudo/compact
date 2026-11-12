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

The standalone suite accounts for 943 files. It includes keyword and primitive
semantics, stdlib cryptography, coverage and test-runner support. Compiler
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

### Initial port

The [upstream assertion fix][assert] gives a successful `assert` the empty
tuple value. Port its two conditional witness-analysis cases, generated
TypeScript execution and refusal case, and the runtime unit assertion.
The baseline compiler failed internally when a conditional combined
`assert(...)` and `[]`. Generated Rust keeps the typed `()` result and
`CompactError::AssertionFailed` for refusal. This slice is tracked under
[#728][compiler].

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
