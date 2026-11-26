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

Read-only `member(Field) -> Boolean` and `lookup(Field) -> Field` receive optional
proof-data facades for the admitted direct-key root Map shape. Other Map output
types, nested paths, effectful keys, arbitrary helpers and witnesses remain
outside those facade families. The prerequisite terminal Unit
return repair is likewise bounded to root `Map<Field, Field>` with pure arguments;
it does not claim all Map types or returned control-flow mutations.
The host tests and supplemental generated-API checks do not execute the original
TypeScript host suite, establish proofs, or qualify Linux or the released SDK matrix.
The original missing-lookup negative remains an expected-cell refusal; its separate
bare-assert parser limitation is not normalized here.

## Original negative fixture and syntax disposition (#798)

`original-negative-map-field-field.compact` is the exact 876-byte file from
`672bd0f226f47a7d036c5c074ce71fbe1af1925e:examples/adt/negative/map_field_field.compact`,
SHA256 `33a44f1eae73850cbcb9d5687f5dac59fa7937683828eb04379da6fb312d83c2`.
Its `//EXECUTION_FAIL` / `//CompactError: Error: expected a cell` annotations and
bare `assert c.lookup(0) == 0 "What should happen here";` remain unmodified.

Official Compact 0.36.0 and the local Rust-target frontend both actually exited255
at line24,char10: `parse error: found "c" looking for "("`. The exact positive
1,500-byte ten-assert fixture compiled with the release. This is the syntax versus
annotation mismatch tracked by [#798](https://github.com/betsudo/compact/issues/798),
under [#727](https://github.com/betsudo/compact/issues/727) and
[#726](https://github.com/betsudo/compact/issues/726).

The existing `check_rejections.py` maintains the current TS/Rust exact-byte syntax
refusal and positive compilation controls, without rewriting the source into a
parenthesized assertion. Its positive compile does not execute those ten
assertions. Supplemental generated-JS missing-key checks independently exercise
runtime refusal; they do not execute this parser-rejected original or establish
its annotated runtime result. No globally new original case is claimed.

The opt-in source also admits the direct root `is_empty() -> Boolean` read,
including its equivalent direct expression-return encoding. Its zero-input
facade calls the existing recorded method once. Nested or composed expressions,
arguments, witnesses and preceding writes remain outside this family. Generator
coverage is separate from future generated-API execution and SDK qualification;
no prior member/Lookup proof receipt qualifies the new helper.

The opt-in source also admits direct `size() -> Uint<64>` on the same
root `Map<Field, Field>`, including its exact expression-return encoding. The
Unit-input facade retains `BoundedUint<18446744073709551615>` and captures the
existing recorded call once. Arbitrary casts, composed expressions, arguments,
witnesses and preceding writes are excluded. Separate source-bound native,
model/fresh-proof/independent-verifier and staged TypeScript qualifications are
recorded in the Clear/Set/Map plan; no earlier qualification transfers. Original
fixtures and their ten assertions remain byte-for-byte unchanged, with zero new original
case credit claimed by this supplemental helper family.
