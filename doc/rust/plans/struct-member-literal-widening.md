# Struct member projections in wider aggregate literals

## Status

Bounded local qualification is reviewed at execution revision
`886a69bc53952d9ef944c6eede74688b8d7fabc9` on
`codex/rust-ast-struct-member-literal-widening`. The primary Rust AST goal remains
active and incomplete. Full required CI and reviewed integration remain open.
Forge remains deferred.

This is an explicit local stack on Q's clean documentation revision
`85c95bb21e6b2f53833ea9fd53820f19c42c91c9`. Canonical
`codex/rust-backend-ast` remains at
`1bdf185ea35929292a79d35946d4a98fb344d11c`. The candidate stack is not integrated
canonical. All predecessor refs and evidence are preserved.

## Gap and accepted controls

Original compiler 166 and current compiler 167 accept the TypeScript source and
independent IR for narrow struct fields used in wider vector literals. Before
the repair, Rust rejected the actual narrow `struct_field` values against the
wider element type. Standalone member widening, explicit member casts and wider
struct literals were already supported and remain controls.

The initial six-source comparison contains 36 commands. Expanded characterization
contains 23 complete sources and 69 commands per compiler:

| Cohort | Original 166 | Current 167 |
| --- | --- | --- |
| 13 valid sources exposing the gap | TS and IR accept; Rust rejects | All three lanes accept |
| 5 accepted control sources | All three lanes accept | All three lanes accept |
| 5 invalid sources | Frontend refusal in every lane | Same refusal causes |

All 20 full IR, raw-origin, Rust and capability artifacts for the five accepted
controls are byte-identical across 166 and 167. The original invocation that
passed a Rust-only flag to the TS lane is retained as an invocation failure; it
is not counted as a language refusal.

## Ownership and implementation

A focused clean baseline passed 22 tests at signed plan revision `5273136e2`.
The actual frontend trace retains both desired `Vector<2, Uint<16>>` and source
`[Uint<8>, Uint<8>]` types around the literal's `elt-ref` expressions through
`emit-rust-ir`. The source types are supplied by frontend analysis.

Implementation `f7c9235f2` adds one typed `elt-ref` arm to
`compiler/rust-ir-passes.ss`. It calls the existing projection helper, which
lowers the complete base/member expression before applying existing `Coerce`
when the source and target types differ. Identity and absent literal-context
behavior remain unchanged.

Existing native owners continue to validate the complete base struct, field
name at its declaration index, actual field type, ownership, coercion legality,
recording certification and resource bounds. Rust syntax remains structured AST
emission. Compiler 167 identifies this source change. No IR schema, runtime ABI,
dependency, executor or conversion relation is added.

## Generated API and qualified behavior

Inputs remain the actual generated `types::Input` and `types::Envelope` structs
with their declared narrow fields. Examples return the declared
`FixedVector<BoundedUint<65535>, 2>`, `FixedVector<Field, 2>`, wide integers,
tuples and nested carriers. Ordinary invalid use is rejected by Rust types;
untrusted IR is checked explicitly at the existing native boundary.

The maintained gate is
`tools/compact-rust-backend/check_struct_member_literal_widening.py`. Its 18
complete sources and 28 cases cover 26 pure calls and eight stateful route
executions. These include wide, product, nested, alias, lexical, conditional,
empty-mixed and checked-narrowing cases. Distinct source assertions expose
first-error order. Stateful helper exports exercise ordinary, recorded,
functional native and functional recorded routes with genuine Uint16 and Field
results.

The complete source/API protocol at execution revision `886a69bc5` passes 99
commands, including:

- 18 strict, unedited generated SDK roots;
- five marked Rust refusals and five frontend refusals;
- six malformed native-IR controls for geometry, conversion, field name,
  field index, non-struct input and an invalid unselected branch;
- actual consumer failures after changing expected output or either complete
  producer;
- exact source/import, independent and golden IR, all Program, compiler, runtime,
  tool and Cargo configuration identities.

The first API execution at `c198` failed because the test named `wide` collided
with a constructor helper named `wide`. Only the helper was renamed to
`wide_value`. The failed epoch remains retained. Development checks passed all
28 cases and strict consumer Clippy; the final clean-head protocol independently
reran and passed the complete gate. Early preflight refusal before the compiler
build completed admitted no execution.

## Genuine proof and application evidence

`check_struct_member_literal_widening_proof.py` includes the whole unmodified
helper SDK and both checked Programs. Its include root exposes the actual
`types` module. Inputs are constructed as the declared structs; generated code
is not rewritten into vector inputs.

Both complete helper export models are k=7 with 85 rows. Locked material is
verified before fresh direct keys. Each selected standalone proof is 2,912 bytes
and verifies with its immutable verifier. Separate, independently proved funded
transactions are 6,553 and 6,561 bytes and pass default-strict ledger application.
Replay refuses `IntentAlreadyExists` without changing the ledger.

Corrupt proof bytes, changed public binding, a changed result using its real
carrier and unchanged geometry, and the other immutable verifier are rejected.
Complete IR, Program and membership controls refuse before keys. A separate
complete Fold source supplies a k=9 diagnostic that refuses before keys; it does
not replace admitted proof inputs or waive the older N k=9 obligation.

## Review, provenance and limits

Actual independent source and receipt CLI reviews used `gpt-6-astra` with high
reasoning after verified Claude quota. Neither found an introduced or blocking
P1/P2 issue within its scope. The source review independently confirms the 20
unchanged control artifacts.

The receipt review checks 1,609 protocol inputs, 1,994 proof-source entries
against original and frozen locations, 202 generated assets, 146 strict-root
files and 18 locks, 219 runtime files and 20 Node libraries, plus tool, material,
key, result and log bindings. These scopes overlap and are not added together.
The own closeout audit checks 4,345 binding records across 3,941 paths.
Closeout log pins are observations at closeout, not retrospective host trust.

Historical validation at `c198` passed 283 fixture freshness checks, 1,830 backend
tests with seven pre-existing ignored tests, and strict backend Clippy. The five
relevant source owner directories are unchanged through execution `886a69bc5`.
This is not current whole-CI acceptance. The signed documentation successor
bridges literal receipts by rechecking source/evidence and signature history.

The accepted scope is pure typed member context and the maintained stateful
exports calling those pure helpers. [Direct stateful Vector literal
returns](direct-stateful-vector-literal-returns.md) remain a separately
characterized gap. No constructor execution proof, all-source/all-input proof,
universal language support, full CI, integration or broader-goal completion is
claimed. No publication or cleanup is due before verified GitHub merge.

Local evidence root: `rust-ast-struct-member-literal-widening-20261102`.
Authoritative records include `original-native-v3/receipt.json`,
`proof-v1/protocol/receipt.json`, `proof-v1/receipt.json`, `receipt-audit-v1.json`,
the source and receipt review outputs, `qualification-summary-v2.json` and the
documentation bridge. Failed and predecessor epochs retain their original scope.
