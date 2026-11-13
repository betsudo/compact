## The design problem

A Compact contract defines more than a circuit's return value. Its behavior includes ledger reads and writes, witness calls, execution costs, the statement proved and the transaction that changes state. A second backend must preserve those relationships while providing an appropriate interface for its host language.

Consider a counter that increments and returns its new value. A generated Rust function can calculate the correct number while recording the wrong ledger operation. A recording can contain the correct operations while preparing a proof for a different installed verifier. A valid proof can still fail ledger application. These are separate correctness obligations.

The Rust AST backend addresses this problem through a shared Compact frontend, typed internal representations, checked lowering, structured Rust emission and a runtime backed by the official ledger primitives.

## Implementation sequence and rationale

1. **Establish a Ledger 8 reference.** Pin the protocol family and reference behavior so representation and execution differences are reproducible.
2. **Introduce typed IR and structured Rust emission.** Separate source admission and semantic decisions from text formatting, then qualify native execution.
3. **Record execution and prepare transactions.** Retain ordered effects and bind observed state to the installed operation and expected verifier before preparing a proof.
4. **Qualify generated APIs with application cohorts.** Exercise realistic consumers and integration boundaries before generalizing their implementation.
5. **Compose checked operations.** Replace the need for a whole-circuit profile for each new arrangement with reusable operation-level checks.
6. **Share execution and complete semantic gaps.** Preserve bindings, effects and return conversions across native and recorded execution, producing a maintained baseline for migration.
7. **Migrate to Ledger 9 and port upstream capabilities.** Adapt the package, representation and transaction interfaces, then qualify each feature against original sources and applicable proof/application tests.

The chapters follow this order. Each explains the motivating limitation, the component responsible for the change, the resulting behavior and its validation boundary. Implementation stages describe capabilities; they are not a package release series.

## Navigation and source records

Read the chapters sequentially for the design history. Use the pipeline explorer to follow a circuit through its components, the iteration ledger for bounded implementation steps, and the commit explorer for original messages and code links.

The technical reference explains the Ledger 8 architecture and API at named revisions. Its source notes distinguish the checked-effect pilot from subsequent integration. The Ledger 9 chapters describe changes to that foundation.

## Scope of this edition

The mainstream compatibility reference is LFDT-Minokawa/compact at 672bd0f226f47a7d036c5c074ce71fbe1af1925e. Released Compact 0.36.0 and the Ledger 9.1.0.1 SDK package matrix provide a separate interoperability reference. The edition freezes local compatibility checkpoint v73; its date and source identities are recorded in [Sources and boundaries](../sources.html).

PR #725 merged the migration foundation into the working branch. Subsequent upstream-compatibility work has separate local qualification. Two isolated follow-ups await canonical integration at this edition's cutoff. Migration of the dependency graph does not establish complete upstream feature compatibility.

The objective is faithful Compact behavior, ergonomic generated Rust types, compile-time rejection of invalid ordinary usage, and explicit diagnostics and runtime checks at untrusted boundaries.

### Source trail

- [Original architecture and evidence vocabulary](https://github.com/betsudo/compact/blob/51a079bd8598b9decb3df7de92ec10768ec9b8dc/doc/rust/journey/README.md)
- [Ledger 9 migration PR #725](https://github.com/betsudo/compact/pull/725)
- [Upstream compatibility programme #726](https://github.com/betsudo/compact/issues/726)
