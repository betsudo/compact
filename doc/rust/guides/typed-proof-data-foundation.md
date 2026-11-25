# Typed time and opt-in standalone proof data

This proposal adds typed block-time comparisons for the released Ledger9
compiler profile and an ordinary recorded proof-data bridge. It targets the
published Ledger9.1.0.2 runtime graph. Local macOS ARM64 validation is described
below; donor proof/runtime receipts do not qualify this graph.

Typed-time normal CLI generation requires both output targets and the released
profile; direct library rendering does not require a TypeScript output:

```sh
compactc --target rust --target ts --feature-zkir-v3 \
  --ts-runtime-profile released-0.21.0 --rust-require-recording \
  --rust-standalone-proof-data INPUT OUTPUT
```

Legacy v2 and legacy v3 profiles retain their refusal. Bounds are canonical
Uint<64>; native/recorded operations use the official VM comparison and gas.
The VM observes a Boolean comparison, not an exact timestamp commitment.

Default Rust output keeps its existing APIs. Non-time closed helper families can
also use Rust-only output:

```sh
compactc --target rust --feature-zkir-v3 --rust-require-recording \
  --rust-standalone-proof-data INPUT OUTPUT
```

The new library entrypoints are `render_with_standalone_proof_data` and
`FrontendArtifacts::render_standalone_proof_data`. Their separate result/report
wrappers preserve the old public struct literals. Helpers return public
`RecordedProofData`, then callers explicitly supply binding and randomness to
`to_preimage`/`serialize`. No proof or transaction is produced by these calls.
No optional helper is promised for an unsupported family or colliding name.

The backend Cargo package is now 0.3.0: downstream `ir::Expr` matches need a
fallback. Runtime/macros remain 0.2.0 and generated ABI54 remains. Independent
time/proof-data linked version assertions reject old or mismatched runtime APIs.
See the [proposed compatibility decision](../adr/typed-proof-data-foundation-proposal.md).
Compiler/language/TypeScript labels are distinct from backend Cargo versions.

Local checks passed: 50 runtime tests; 2,030 backend tests (seven existing tests
ignored); 4,039 compiler/go tests; ten compiler-profile tests; and strict
all-target Clippy. The affected 12 classifier/runtime checks and 35 host/context
checks also passed as overlapping repeat checks, not additional coverage totals.

The fresh generated-linkage run completed 66 steps with 27 actual offline
dependency graphs: 13 generated-runtime positive compilation controls, eight
missing/mismatched-domain compile refusals, and three backend migration controls
(two positive and one exhaustive-match refusal). Genuine old54 baseline
compilation ran first. Normal and direct outputs were generated from current
source; independent stopped-evidence review remains a separate gate.

These results describe local ARM64 validation of the tested source snapshot.
This documentation update changes no code or fixture bytes. Emitted released
TypeScript runtime0.21 output was not executed; compiler/go used its default
JavaScript runtime0.18. Validation against released Compact 0.36.0 and the Ledger 9.1.0.1 SDK matrix
remains pending, as do original SDK/TS behavior, fresh proofs and Linux
qualification. No funded-chain,
special EntryPoint or two-contract acceptance follows. Original asynchronous
tests and refusal messages remain intact; supplemental mutations stay labeled.
