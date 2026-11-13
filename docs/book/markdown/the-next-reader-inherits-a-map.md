## Delivered capabilities at the edition cutoff

PR #725 merged the Ledger 9 migration foundation. The subsequent compatibility work has local arm64 qualification for selected cryptography, hashing, serialization, collection operations, event semantics and generated Rust/TypeScript parity. Each capability retains its source and case-specific evidence.

The frozen compatibility source is 7a4fab772ee34c19053ce293d08052b501ce4b50. The isolated Jubjub identity-key assertion and Keccak opaque diagnostic follow-ups are retained at 6eef58425bb53883c8f3b417b601246820c7d675 and 30a2d76550beb7d920a92bab2be6b1180243f658. Their local review findings were resolved in checkpoint v73. Canonical integration and final qualification remained outstanding at this cutoff.

## Remaining compatibility work

| Area | Remaining obligation | Why existing evidence is insufficient |
| --- | --- | --- |
| Upstream source corpus | Account for every applicable original fixture and its required semantics. | Selected feature cases do not establish complete language admission or parity. |
| Language admission | Implement and test the semantics between the local 0.23.105 gate and the reference 0.28.0 language. | Changing a version declaration does not implement the intervening language behavior. |
| Proof and application coverage | Complete fixture-specific proof, verification and application matrices. | Native results and recording availability do not establish a valid applied transaction. |
| Platform qualification | Run required native Linux gates for the later local ports. | Local arm64 receipts qualify their named platform only. |
| Generated cross-contract calls | Complete generated caller/callee execution and the funded joint transaction path. | Runtime preparation of supplied recordings does not establish generated source-call behavior. |

These obligations are separate from the accepted Ledger 8 foundation and the merged Ledger 9 dependency migration. Their completion requires new evidence; it should not invalidate accepted earlier work without a concrete regression.

## Planning the next implementation step

Select one documented semantic or generated-API gap. Identify its frontend, checked-lowering, runtime and generated-consumer responsibilities. Define positive behavior, compile-fail or source-refusal controls, and applicable proof and ledger-application checks before implementation.

Use focused local checks during iteration, then run the complete required integration gates for the affected scope. Several commits may form one coherent step. The iteration ledger explains those groupings, while the commit explorer retains the original messages and identities.

## Maintaining an accurate guide

Update the edition manifest when selecting a new source snapshot. Preserve accepted reports and failed runs. Correct explanatory prose when evidence changes an interpretation, while retaining the original source records.

Keep implementation, generated compilation, native execution, recording, proof verification and ledger application distinct. State the platform, source and fixture population for every quantitative claim. Internal planning labels and calendar dates belong in provenance; the narrative should explain the capability and its dependencies.

The design sequence moves semantic decisions into explicit components: source admission into typed IR, syntax into Rust AST construction, ledger effects into checked operations, and proof/application identity into validated preparation and retained receipts. This division provides the basis for extending the backend without treating earlier coverage as a universal compatibility claim.

### Source trail

- [Upstream compatibility programme #726](https://github.com/betsudo/compact/issues/726)
- [Compiler compatibility #728](https://github.com/betsudo/compact/issues/728)
- [Cryptographic compatibility #729](https://github.com/betsudo/compact/issues/729)
- [Integration qualification #732](https://github.com/betsudo/compact/issues/732)
- [Frozen source register](../sources/edition.json)
