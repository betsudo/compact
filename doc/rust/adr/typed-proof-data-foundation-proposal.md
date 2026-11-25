# Typed time and standalone proof-data foundation

Status: **proposed for review**. No owner acceptance, proof qualification or release is implied.

This proposal has two independent compatibility boundaries.

The backend Cargo package advances from 0.2.0 to 0.3.0 under ADR0293. The new
`Expr::KernelBlockTimeLessThan` and `Expr::KernelBlockTimeGreaterThan` variants
break exhaustive downstream matches. `Expr` becomes `#[non_exhaustive]` in this
same migration; downstream matches must keep a fallback. Runtime, macros,
testkit, generated application, compiler and TypeScript runtime labels are
separate identities. Runtime/macros stay 0.2.0, generated ABI54, IR20 and
capability schema3 stay unchanged. Cargo.lock changes only the local backend
package identity; SDK/transitive versions, sources and checksums stay exact.

For the generated/runtime boundary, the proposed policy permits additive,
explicitly selected domains with independent linked versions. Existing generated
ABI54 operations retain their contract. Any breaking change to that existing
contract still requires an ABI migration. Every emitted native or recorded
typed-time operation carries a macro-free compile-time equality assertion on
`runtime::BLOCK_TIME_API_VERSION == 1`. Direct backend rendering has the same
assertion; source-shape inspection by the normal CLI is an additional mismatch
check, not a substitute for linkage or authenticated source provenance.
Functional typed-time projection remains refused and emits no new operation.
Generated proof-data helpers independently assert `runtime::proof_data::API_VERSION == 1`.
Missing or mismatched domain constants must fail compilation. Ordinary old
ABI54 nonfeature consumers remain compatible with the matching current runtime.

Standalone proof-data helper emission is **opt-in**. Existing `RenderOptions`,
`RenderedContract`, `RustCapabilityReport` and `RustCircuitCapability` struct
literal contracts remain unchanged. A separate render result/report wrapper
joins optional helper metadata without changing default serialization. The CLI
flag `--rust-standalone-proof-data` requires `--target rust` and
`--rust-require-recording`; it refuses the experimental functional combination
before output replacement. Default rendering does not run the helper classifier
or emit helper methods. Actual runtime source fingerprints and backend package
metadata necessarily reflect current bytes; raw metadata identity with an old
source snapshot is not claimed.

The helper families are bounded typed time comparisons, closed Field Set Unit
mutations, read-only Maybe<Field> getters and closed Counter Unit programs with
read/equality assertions. Unknown shapes and existing refusals remain unchanged.
Name collisions omit only the opt-in helper. Unit and Maybe bodies do not gain
fabricated assertions, traces or proof eligibility. Helpers capture an actual
recorded call exactly once, with typed input, private outputs and public Verify
operations, then use the public ordinary data bridge. Conversion requires
explicit caller binding and communication randomness; no parameters, RNG,
keygen, proof, verifier, transaction preparation or chain action is performed.
The runtime bridge alone is not compiler/circuit provenance.

The port targets the published Ledger9.1.0.2/transient-crypto3.1 graph. Retained
fixture IR and provenance belong to their donor epochs. Six original pinned672
Compact/TypeScript sources remain byte-exact. Original SDK/TypeScript behavior,
proving and Linux portability still require fresh evidence.
Historical receipts and fixture hashes are preserved rather than relabeled.

Local macOS ARM64 checks have passed: runtime 50, backend 2,030 with seven existing
ignored tests, compiler/go 4,039, ten compiler-profile tests and strict all-target
Clippy. The affected 12 classifier/runtime and 35 host/context checks passed as
overlapping repeats. A fresh linkage run completed 66 steps and 27 offline
dependency graphs, including genuine old54 baseline compilation, 13 positive
generated-runtime compilation controls, eight missing/mismatched-domain
refusals, and three backend migration controls (two positive, one refusal).
Native/recorded and direct-render linkage controls cover the emitted domains;
functional typed time remains explicitly unavailable. Independent stopped-run
evidence review and owner acceptance remain separate gates.

Typed-time normal CLI generation uses `--target rust --target ts --feature-zkir-v3
--ts-runtime-profile released-0.21.0`; Rust-only selection remains valid for
non-time families, and direct library rendering does not require a TypeScript
output. Emitted runtime0.21 TypeScript was not executed; compiler/go
used its default JavaScript runtime0.18. Validation against released Compact 0.36.0 and the Ledger 9.1.0.1 SDK matrix
remains pending, as do original SDK/TS behavior, fresh proof and Linux
qualification.
No proof, funded-chain, special EntryPoint or two-contract acceptance is implied.
The checks apply to the tested source snapshot; the following documentation-only
update preserves its code and fixtures and does not relabel historical proofs.
This ADR remains proposed for owner review, rather than an accepted ABI waiver.

A later maintainability change may split the private classifier by family. This
port keeps its bounded traversal/classification behavior together to avoid an
unrelated refactor while establishing the compatibility boundary.
