## The state at this edition

The Ledger 9 migration foundation is merged through PR #725. The later compatibility programme has local arm64 evidence for large portions of cryptography, hashing, serialization, collections, event semantics and generated Rust/TypeScript parity.

The frozen local compatibility cutoff for this edition is 7a4fab772ee34c19053ce293d08052b501ce4b50. The signed isolated Jubjub identity-key assertion and Keccak opaque diagnostic commits are recorded separately at 6eef58425bb53883c8f3b417b601246820c7d675 and 30a2d76550beb7d920a92bab2be6b1180243f658.

Their local review follow-ups were resolved in checkpoint v73. They await canonical integration and final qualification.

## The remaining milestone

The programme still needs complete applicable upstream corpus accounting, the semantics required for intact newer-language admission, remaining fixture-specific proof matrices, required native Linux qualification, and the held generated two-contract path.

The local frontend's language gate remains 0.23.105 while the pinned main reference uses 0.28.0. Editing a version number would not establish the intervening semantics. The history inventory identifies the changes that must be ported and tested.

## How commits become iterations

Use the commit explorer to find the implementation and its follow-up repairs. A logical iteration usually includes a semantic gap, affected owners, positive and refusal tests, documentation, independent review and applicable proof/integration results.

Several commits can therefore belong to one iteration. A milestone includes several accepted iterations and an explicit scope. Git merge commits identify integration points, while author dates describe the original work. The explorer labels its date field accordingly.

## Keep local history durable

The book includes a frozen commit catalogue and public narrative snapshots for local work. Published commits link to GitHub. Commits known only from local refs retain their identities and metadata in the catalogue, with publication status shown explicitly.

The source snapshots carry selected plan and qualification summaries. They provide a durable public account without embedding machine paths, private witness values, keys or complete internal notes.

## The maintainers' continuation rule

Update the edition manifest when taking a new snapshot. Preserve accepted historical reports. Add a correction when new evidence changes an earlier interpretation. Keep the book's status vocabulary aligned with the actual evidence.

A claim that was locally valid in October may remain useful in November. Its source, platform and scope let a future engineer decide whether it applies to the next change.

## Why this journey is worth preserving

The strongest thread is the steady movement of hidden assumptions into explicit owners. Source semantics moved into checked IR. Rust syntax moved into syntax trees. Ledger operations moved into typed slots and effect programmes. Proof and application identities moved into retained receipts.

That makes future work easier to reason about. The next engineer receives a map of decisions, tests and boundaries, with the unfinished work still visible.

### Source trail

- [Milestone #726](https://github.com/betsudo/compact/issues/726)
- [Compiler #728](https://github.com/betsudo/compact/issues/728)
- [Cryptography #729](https://github.com/betsudo/compact/issues/729)
- [Integration #732](https://github.com/betsudo/compact/issues/732)
- [Frozen source register](../sources/edition.json)
