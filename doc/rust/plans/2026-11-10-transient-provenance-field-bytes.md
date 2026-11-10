# Transient provenance: Field-to-Bytes<32> conversion evidence (#612)

Date: 2026-11-10. Parent: GPG-signed local `a169b072a` (cumulative audit).
Branch/worktree: `codex/rust-ast-transient-provenance-field-bytes` in a
separate checkout. No remote publication.

## Scope

The analyzer already visits `Expr::FieldToBytes32` as a unary conversion.
This slice adds genuine source-backed evidence for that distinct typed-IR
variant. A transient hash cast to `Bytes<32>` remains a known producer at a
persistent hash input and at a persistent commitment opening. An ordinary
Field cast to `Bytes<32>` remains a valid unknown-origin input without a
known-transient finding. The test also checks deterministic structured
finding and conversion-dependency evidence.

## Source and evidence

`examples/rust_backend/transient_provenance_field_bytes.compact` compiled
with the recorded frontend pair using `--target rust --skip-zk
--emit-rust-ir`. The source-produced schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-field-bytes-source-ir.json`
(SHA-256 `a81cdbc602c34701df44f09356c302d20a03238bdddf7076fb4b47061a30e3da`).
The fixture contains `field_to_bytes32` in all three circuits. No shared
fixture was regenerated.

Three focused provenance test targets passed: 8 tests, zero failures, using
`CARGO_BUILD_JOBS=2` and a separate Cargo target under
`/Users/ysh/.codex/local-evidence/transient-provenance-field-bytes-20261110`.
Exact-parent and current backends compiled the same source with the same
Scheme frontend. `cmp` found generated `contract/lib.rs` and
`contract/rust-capabilities.json` byte-identical. SHA-256: Rust
`eb1b7458892727f8be5db156a11f75edb12c050bd19e4f117572e5efdb6824f2`;
capabilities
`0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review found no implementation blocker and requested
an explicit `FieldToBytes32` IR-variant assertion, which is included.

## Limits

This confirms the Field-to-Bytes<32> cast in the selected corpus; it does
not certify all byte encodings or unknown external values as stable. The
analysis remains opt-in and separate from generated APIs and capabilities.
No runtime, FAB-byte equivalence, proof, broad suite, or Forge claim is made.
