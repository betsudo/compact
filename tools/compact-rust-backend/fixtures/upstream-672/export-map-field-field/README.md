# Original exported Field Map fixture

`original-map-field-field.compact`, `original-map-field-field-test.compact`, and
`original-exports.e2e.test.ts` are byte-for-byte copies from mainstream Compact
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`:

- `examples/adt/exports/map_field_field.compact`: 1,364 bytes, SHA256 `174cfca103ed3f9e057f4256c966943ecb6f8a9fd269db8031ac6ea974a0a101`; all eight exports retained.
- `examples/adt/tests/map_field_field.compact`: 1,500 bytes, SHA256 `2dfdc33fd2e3c9e7460850ea64a2ef0fc7a1962cd6b149faac8d572b4917f541`; all ten original assertions and casts retained.
- `tests-e2e/src/tests/compiler/compiler.adt.exports.e2e.test.ts`: 1,621 bytes, SHA256 `f26b79ab8af31452a0abad01b73b945ef867709c2ab2ab5f4be2b99b507708f7`; original skip-ZK compilation and valid-JS assertions retained.

`compact-rust-ir.json` is actual current frontend emission of the intact export
fixture after the bounded terminal Unit Map-return repair. It retains all eight
exports, typed Field key/value declarations and original source positions.
The four returned mutations each use one existing action followed by Unit.
This is a current emission, not a claimed upstream Rust artifact.

Only read-only `member(Field) -> Boolean` receives an optional proof-data facade.
Other Map output types, nested paths, effectful keys, arbitrary helpers and
witnesses remain outside that facade family. The prerequisite terminal Unit
return repair is likewise bounded to root `Map<Field, Field>` with pure arguments;
it does not claim all Map types or returned control-flow mutations.
The host tests and supplemental generated-API checks do not execute the original
TypeScript host suite, establish proofs, or qualify Linux or the released SDK matrix.
The original missing-lookup negative remains an expected-cell refusal; its separate
bare-assert parser limitation is not normalized here.
