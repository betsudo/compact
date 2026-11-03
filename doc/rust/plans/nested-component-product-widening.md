# Nested scalar products below Vector components

Status: source characterization, before implementation. Based on reviewed
request-field baseline `d9e9cb2cb`; the primary Rust AST goal remains incomplete.

## Gap and owners

The intact historical `nested_product_below_component.compact` (SHA256
`33c091f6cc2907b05c08156a129a067be8ae15ad062c2c61473538e51a8177c7`)
is valid Compact but Rust and independent IR refuse its ordered Cell.write/return.
The source is retained at its original declared path. The previous baseline
slice closed its separately declared same-type private-field Clippy defect.

`compiler/rust-ir-passes.ss::vector-value-widening?` owns selection of ordered
frontend-supplied conversions. Its scalar-product relation currently checks only
scalar children. The existing `pure_semantics/coercion.rs` recursively validates
tuple/vector shape, bounds and resource work; `value_lowering/coercion.rs` emits
structured Rust AST. Independent recording validation remains authoritative.
No new evaluator, IR, ABI, type erasure or source-name admission is proposed.

## Intended API and bounded relation

Keep the original exact nested `FixedVector`/tuple/`BoundedUint` public carriers,
ordinary methods and consuming functional Programs. Admit only tuple subtrees
below the already admitted component Vector position, preserving each arity and
Vector dimension and applying existing monotone Uint / Uint-to-Field / Boolean
identity leaves. This is structural composition under existing resource limits,
not an arbitrary new carrier or general aggregate-conversion framework.

The whole new `nested_component_products.compact` has parameter, helper, local,
checked-helper, witness-before/after, branch, deeper-product, Boolean/Field, empty
Vector and empty/single tuple exports. It is an independently declared complete
application, not a smaller replacement for the original regression. Any selected
proof qualifies only its actual export/source; the original's proof scope stays
separate. No source extraction, normalization, cap increase or proof/result cache.

## Acceptance order

1. Original compiler184: retain complete TS-positive/Rust-IR refusal for the
   original and whole new source. Use explicit same-type and pure conversion
   controls, and trace actual frontend-supplied casts/ordered owners. Close a
   concrete baseline defect before dependent feature work.
2. Change only the classified tuple leaf composition; retain original input
   bounds, once-only helpers/witnesses, lexical and effect/error order. Keep the
   entire original source and its historical refusal authority. Preserve every
   unrelated nested-Vector/identity/narrowing refusal; do not blanket-reclassify.
3. Build the original maintained compiler185 recipe from frozen clean signed
   source, with all version locations/generated docs coherent. Independently
   emit exact whole IR and unedited SDKs; require each declared export's APIs.
4. Run independent live TS and four typed Rust routes over asymmetric/boundary,
   helper error/cast order, both branches, witness order/failure/private state,
   empty and deeper cases. Preserve complete public transcripts and VM replay.
   Lint complete generated roots and consumer test code strictly.
5. Mark separate ordinary wrong leaf/bound/arity/dimension/result/ownership
   misuses, each with a matching positive; retain relevant malformed IR, dead
   branch and static-empty validation controls. Producer/input mutations must
   be consumed and refused by unchanged typed controls.
6. Observe all intact export models before selection. Require fresh real keys,
   proof/verification with immutable verifier and identity-bound negatives plus
   a separately proved funded DEFAULT-strict application/replay for applicable
   selected intact k<=8 sources/exports. Preserve over-cap results and unproved
   export status; never use one selected proof as all-export acceptance.
7. Fresh authorized Claude source and actual-receipt review; Codex fallback only
   after actual quota. Record exact epochs/limits and CAS/Obsidian progress.
   Full required CI/Linux and logical integration remain separate local-only
   exclusions, not waived gates or goal completion.

## Exclusions and restart

No nominal Struct, new identity carrier, Vector newly nested inside a tuple
subtree, narrowing legalization, constructor coverage, broad refactor, CI
optimization, Forge work, hosted trial, push, merge, release, shared installation
or cleanup. Accepted crosswalk/retirement and application work is not reopened.

Worktree: `/Users/ysh/.codex/worktrees/rust-ast-nested-product-widening/compact`.
Branch: `codex/rust-ast-nested-product-widening`.
Evidence: `/Users/ysh/.codex/local-evidence/rust-ast-nested-product-widening-20261103`.
Start with `admission-v1.json`; preserve the parent style/SDK branch and all
canonical/protected/Forge refs. The prior goal turn is progress: it changed
source, closed a real baseline regression and updated authoritative evidence.
