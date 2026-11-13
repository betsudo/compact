## Start with one state transition

A Compact contract is more than the text of a circuit. Its behavior includes the value returned, the ledger reads and writes, the witnesses called, the cost charged, the statement proved, and the transaction that eventually changes state. A second backend has to preserve those relationships.

Imagine a counter that increments and returns its new value. A generated Rust function can calculate the right number while recording the wrong ledger operation. A recorder can produce the right operations while preparing a proof for a different installed verifier. A valid proof can still fail a funded ledger application. Each of those failures occupies a different boundary.

Our journey began with a TypeScript-oriented Compact compiler and a Ledger 8 reference environment. The long-term question was whether Rust could become an ergonomic, typed way to use the same contract semantics. The answer emerged through repeated small deliveries rather than one replacement compiler.

## What this book preserves

This edition follows the original Rust text emitter, the Rust AST rebuild, native execution, recorded execution, proof and wallet integration, the 0.3.0 closure, compositional effects, the functional Ledger 8 experiment, and the move to Ledger 9. It records the detours, including Forge, because those decisions explain why the project has its current boundaries.

Every chapter asks four questions. What limitation triggered the work? Which owner changed? What behavior became possible? Which evidence established the result? A final section records what was still open at that point.

The commit explorer is the granular index. Phase assignments are editorial groupings over a frozen Git graph. Commit subjects are historical source material. They sometimes use shorthand or describe an intended result. The chapter narrative and its receipts supply the scope of acceptance.

## Three ways to read

Read the chapters in order for the engineering story. Follow the Architecture link to walk a circuit through the pipeline. Use the Commit explorer to move from a milestone to its individual steps, then follow a published commit back to code.

The historical atlas preserves the earlier October documentation as an explicitly dated snapshot. The later chapters explain which earlier limitations were subsequently resolved.

## The edition boundary

The mainstream compatibility reference is LFDT-Minokawa/compact at 672bd0f226f47a7d036c5c074ce71fbe1af1925e. Released Compact 0.36.0 and the Ledger 9.1.0.1 SDK package matrix supply a separate interoperability reference. The local compatibility checkpoint used for this edition is v73 on November 13, 2026.

PR #725 merged the migration foundation into the working branch. The broad upstream-compatibility work continued locally afterward. Two final isolated commits in this edition await canonical integration. A merged foundation and complete upstream compatibility are distinct milestones.

## Why the distinctions matter

A guide becomes misleading when a reader cannot tell whether an example was merely generated, executed natively, proved, or applied to a ledger. This book keeps those stages visible. It also keeps historical Ledger 8 evidence useful while explaining the changed Ledger 9 SDK and representation contracts.

The objective remains faithful Compact behavior with useful Rust types, explicit diagnostics, and runtime checks where values cross an untrusted boundary.

### Source trail

- [Original journey and evidence vocabulary](https://github.com/betsudo/compact/blob/51a079bd8598b9decb3df7de92ec10768ec9b8dc/doc/rust/journey/README.md)
- [Ledger 9 migration PR #725](https://github.com/betsudo/compact/pull/725)
- [Compatibility milestone #726](https://github.com/betsudo/compact/issues/726)
