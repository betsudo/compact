# Proposed next bounded slice: struct member projections in wider aggregate literals

Candidate R is admitted on codex/rust-ast-struct-member-literal-widening from the exact clean Q docs head85c95bb21e6b2f53833ea9fd53820f19c42c91c9. Q bounded source/receipt closeout is accepted locally at executione8d. No R implementation is accepted yet. Primary Rust AST goal ACTIVE/INCOMPLETE, LOCAL ONLY; Forge deferred/incomplete. Preserve canonical1bdf and all local predecessor refs. Reconcile actual checkpoint/refs and clean baseline first. Reuse the existing Q actual-type context owner through an explicit local shallow-stack if Q is not integrated; never duplicate it or call the stack canonical.

## Concrete gap and accepted controls

`next-struct-projection-characterization-v2/receipt.json` retains six full sources and36 original165/current166 TS/Rust/independentIR commands. Both compilers accept the TS and independent IR for `[input.right,input.left]` from a declared Input struct with two Uint8 fields into Vector2Uint16 or Vector2Field, but native Rust lowering rejects the actual narrow struct_field values against the desired widened elements. This is existing behavior under165 as well as166, not evidence of a Q regression.

Four controls already work in all lanes: standalone struct member scalar widening, explicit member casts, wider struct literals using indexed inputs, and wider struct literals using narrow variables. All16 full IR/raw-origin/Rust/capability assets for those four controls are byte-identical across165/166. Do not reopen struct literal or standalone scalar support. First v1 characterization incorrectly passed a Rust-only flag to the TS lane; retained as invocation failure, superseded by exact full-source v2 with correct lane arguments, not counted as language refusal.

## Owners and intended public API

`compiler/rust-ir-passes.ss` currently lowers elt-ref to struct_field in the ordinary expression owner. Q typed literal source metadata reaches tuple/vector projection arms but does not handle elt-ref. Actual original166 next-struct-source-trace-v1 retains safe-cast(Vector2Uint16, Tuple<Uint8,Uint8>, tuple(elt-ref right1,elt-ref left0)) through emit-rust-ir. Source type authority already exists; no target-derived guess or registry is needed. Preserve that trace before editing. Candidate change: route that one typed projection form through the existing authoritative source/target Coerce helper, preserving the complete base expression first. Native struct field legality, Coerce planning/certification/resource bounds and syn Rust AST emission remain their existing owners. No IR/runtime ABI, dependency, executor, normalization framework, nominal redesign or general aggregate rewrite.

Intended generated API keeps the declared exported Input struct and narrow BoundedUint255 fields; existing pure `value(Input) -> FixedVector<BoundedUint<65535>,2>` or `FixedVector<Field,2>` returns the declared output. Use consuming generated ordinary/recorded/functional Programs only for complete maintained stateful helper exports. Types do not authenticate untrusted external values or observed chain state.

## Scope exclusions

No new dynamic indexing, struct literal framework, structural-to-nominal conversion, general expression context normalization, compiler optimization, CI optimization, Forge or publication. No reopening already accepted standalone member/scalar/struct literal behavior. Existing complete CI and proof gates remain required for integration.

## Acceptance

1. Exact clean focused baseline and actual frontend source-type trace; distinguish original165/current166 behavior and accepted controls. Preserve all earlier evidence and unedited complete source.
2. Independently current TypeScript-aligned results and unmodified generated Rust for the admitted Uint/Field cases, reversed/different/boundary inputs and cohesive nested/alias contexts only if actual source proves their relevance. Original narrow field types and checks stay intact. Use source assertions to observe first-error/order where possible; inspect source-once lowering rather than inventing observable effects for pure code.
3. Ordinary wrong field type and result maximum/carrier/length compile-fail controls with correct-type positive twins and actual marked Rust error spans. Preserve implicit narrowing, nominal and missing-field frontend refusals. Native untrusted IR rejects field-name/index/type mismatches and invalid unselected branches at existing owners.
4. Preserve byte-identical accepted full controls; complete producer mutations and same-geometry changed expected results must fail actual consumers. Bind all source/imports, independent and golden IR, all Programs, original compiler/recipe/serialization/Rust/Cargo config/runtime/tool identities. Strict unedited SDK roots required.
5. For complete small stateful Uint/Field helper exports, inspect all models and verify locked material before fresh direct keys. Keep automatic k<=8 and every full source. Require actual distinct-carrier four-route behavior, fresh genuine proofs with immutable verifier and corrupt/binding/result/wrong-verifier negatives; separately independently proved funded default-strict application and replay refusal. No constructor/all-input or full-language proof claim.
6. Signed Conventional DCO/Assisted commits; actual Claude-first independent CLI source/receipt reviews with actual-limit Codex different-model fallback; docs and verified Obsidian process/CAS. Full required arm64/Linux CI and reviewed integration remain open until actually executed. No cleanup before verified GitHub merge of completed owned work.


## Admission decision

Actual current checkpoint6b763987 and all refs/worktrees were inspected before
creating the separate branch. Canonical1bdf185ea, reviewedQ85c95, P3d94, O03a114,
N857244 and Forgeexperiment e299 remain retained. Use an explicit local
shallow-stack on Q: the exact authoritative literal-source context helper is
already present there. Reusing it avoids duplicate owner logic; no candidate
is called integrated canonical. Focused current clean baseline is required
before the one cohesive elt-ref typed-projection extension.


## Focused baseline and implementation decision

At signed5273136e2 the focused22coercion/projection/order/historical tests pass.
Expanded23-source/69-command original166 characterization identifies18TS-valid
sources:13Rust mismatches and5acceptedcontrols. Five genuine frontend refusals
retain length/kind/missing-field/implicit-narrowing/nominal causes in every lane.
The complete cases include wide,tuple,nested member/literal,alias,local,conditional,
source assertion,empty-mixed and checked-narrowing contexts.

Existing frontend/native owners match the reviewed Q source. The change adds
one typed elt-ref arm through the existing source-type projection helper. That
helper lowers the complete original member/base expression first, then uses
authoritative source and target types for existing Coerce. Identity and absent
literal source context keep original IR. Native field-name/index/type validation
and ownership remain unchanged. Compiler167 records the source change; no IR,
runtime ABI, dependency or executor changes. Current compiler/API/proof and
independent review acceptance remain open.
