## The attractive surface

A generated method that calls another contract looks like a small API feature. Its semantics include the callee address, declared interface, installed operation, expected verifier, entrypoint identity, arguments, result and communication commitment.

A call that accidentally reaches a same-shaped sibling entrypoint could produce the same host result while proving a different assertion.

## Start from the mainstream implementation

The owner directed the team to read and port mainstream's sequence carefully. Resolve the module and interface. Authenticate the operation and expected verifier. Enter the callee's context. Restore the caller. Propagate the result and commitment opening through the caller's witness.

This avoids inventing an independent interpretation of a cross-contract call.

## What is already available

Typed address comparisons, entrypoint hash binding, native claim recording and replay are implemented prerequisites. The runtime can prepare a supplied caller and leaf-callee recording from a shared ledger snapshot.

Preparation checks the installed verifiers and the exact claim before using Ledger 9's joint partitioner. The pair-preparation tests use synthetic verifier carriers and cover honest preparation and refusal controls.

The internal compiler statement probe also produced a selected standalone caller proof with an adapter-built preimage. Same-output sibling entrypoint, wrong verifier, binding and corruption controls passed for that bounded statement.

## What is still held

Ordinary generated invocation of the intact caller and a real joint funded two-contract transaction remain on hold. The canonical worktree preserves three uncommitted compiler files for this direction.

Runtime preparation of supplied recordings is a useful prerequisite. It does not establish that the generated source call constructs those recordings correctly.

## The eventual acceptance path

Resume with the original caller/callee fixtures, the sibling substitution test and the pinned reference implementation. Qualify generated execution in both targets. Prove both statements, independently verify them, apply the funded joint transaction, then check replay, rollback and invalid bindings.

The intact transfer model measures k=9 and belongs in an explicit slow lane.

## Why this boundary improves the API

An ergonomic contract method becomes trustworthy when its ordinary call path carries the same authentication and statement identity that the low-level runtime requires. Explicit types can prevent ordinary family mix-ups. Installed state and untrusted artifacts still require runtime validation.

### Source trail

- [Secure cross-contract calls #730](https://github.com/betsudo/compact/issues/730)
- [Pinned mainstream contract implementation](https://github.com/LFDT-Minokawa/compact/tree/672bd0f226f47a7d036c5c074ce71fbe1af1925e)
- [Compatibility plan snapshot](../sources/ledger9-upstream-compatibility.md)
