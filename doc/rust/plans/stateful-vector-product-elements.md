# Stateful Vector products with Vector elements

Status: reviewed bounded local source/API stage, continuing from V documentation head
`0bd77849cae0cf907e54c361ce92a9c04e45f5d3`, on
`codex/rust-ast-vector-product-elements`. The first change repairs the observed
strict signature baseline. Compiler180 implements the bounded component conversion.
The current stage passes 17 complete SDK roots / 20 TS/Rust cases, with fresh
source and separate receipt review reporting no P1/P2. Full feature acceptance,
static/untrusted/producer controls, model/key/proof qualification and integration
remain open. Forge is deferred and does not block this backend work.

## Strict signature baseline repair

A fresh check after restart reproduced all eight `type_complexity` diagnostics
in the complete unedited compiler177 identity SDK. The signature AST metadata
owner omitted a Vector whose product has a fixed-Vector component, such as
`FixedVector<(FixedVector<BoundedUint<255>,2>,BoundedUint<65535>),2>`.
Compiler178 extends only this existing function/method-scoped style matcher.
It preserves the complete carrier, signature, body and once-only effects; no
module/crate allowance, alias erasure or semantic admission change is introduced.
Focused controls check ordinary functions, Program methods and witness methods,
the previous supported shapes and unrelated/simple carriers. At signed
`b006f92313a821283c9609c032045403b575b84e`, both focused tests and strict
all-target/all-feature backend Clippy pass. Original compiler178 builds through
the unchanged Nix recipe. All three newly emitted, complete unedited control SDK
roots pass strict all-target/all-feature Clippy. Their complete IR, raw origins
and checked Programs are byte-identical to their compiler177 counterparts;
generated Rust differs only by the exact reasoned signature attributes (one on
the pure control, nine on stateful identity, zero on the prior V control).
Only the separate Cargo.lock is resolved after generation. This repairs the
observed strict-compilation baseline; it is not new SDK route execution or proof
acceptance. Actual Claude CLI source review reports no P1/P2. Original failures
and V's literal execution/proof epochs remain preserved.

Local receipts are retained under
`rust-ast-vector-product-elements-20261102`: `baseline-identity-v1.json`,
`original-native-v1/receipt.json`, `signed-source-tests-v1.json`,
`focused-checks-v1.json`, `fresh-control-roots-v1/receipt.json` and
`local-claude-source-review-v1.json`. Broader fixture and separate receipt-review
results remain separate gates with their own recorded outcomes.

### Next semantic owner

Read-only inspection confirms that `vector-value-widening?` checks product
components with `scalar-widening?`, excluding a fixed-Vector component. The same
relation selects ordered local binding and returned-value evaluation. Extend
this existing relation only for characterized Vector components after tracing
the actual supplied casts. Check whether selecting the existing ordered owner
also resolves the two legacy plain witness-call failures before adding a dispatch
rule. Preserve original child bounds, exact dimensions, single evaluation and
witness identities; arbitrary tuple/nominal support remains excluded.

### Initial compiler179 implementation (historical)

Actual original178 pass traces of the full parameter, local, alias and witness
sources show frontend-supplied `safe-cast` nodes with the original nested Vector
bounds. The witness initializer is a cast of the declared witness call; the
legacy IR loses that cast and emits a plain call. Compiler179 extends only the
existing product-component relation to recurse through fixed Vectors. Return
and local-binding selection reuse the existing ordered lowering and typed
witness dispatch; no separate call-dispatch change is made.

The 15 complete original probes, including both frontend refusal controls and
the pure/identity/prior-V controls, are adopted byte-for-byte under
`examples/rust_backend/stateful_vector_product_elements/`. These maintained
physical source paths are explicit new compiler inputs, not rewritten origins
or normalized Programs. At signed `e02b52f5d9077ad9a3e93a00a7f32f1946acb35f`,
all 45 original-path commands match the declared outcomes: 13 sources emit through
TS/Rust/independent IR and both frontend refusals remain. The pure/identity/prior-V
control IR, origins and complete Programs remain byte-identical. This includes
successful native emission for both witness positions without a new dispatch rule.

The initial maintained `check_vector_product_elements.py` gate declared 13 complete
positive SDK roots and 15 TS/Rust cases (56 stateful routes plus one pure route),
including both helper branches, the Field result and observed witness order. It
compares independent current IR, authenticates whole manifests, retains original
source/tool/config/generated inputs, executes unedited consumers and full strict
SDK roots, and checks both complete source-bound frontend refusals. At that checkpoint, execution was pending. Compile-fail/positive twins, malformed IR, complete producer
mutations, broader sources, model inventory and genuine proof/application remain
separate open gates; this first execution gate is not full acceptance.

### Source review findings and compiler180 successor

Claude's actual review of `a971993b0` found three P2 issues. The compiler179
component recursion also reached products below a component Vector; public TS
transcripts were not compared with Rust; and copied TS scope text claimed
assertion-refusal checks absent from this first cohort. That review is not an
approval, even though the corrected witness-literal successor `182243c07` passed
the first 58-command execution gate (15 cases, 56 stateful routes plus one pure).

Compiler180 restricts component recursion to fixed Vectors ending in scalar
leaves. Products below that component retain their previous owner. A complete
deeper-product source records the original178 TS success and exact Rust/IR
refusals, alongside the positive component-vector sources. The successor gate
compares complete TS public transcripts exactly with serialized Rust
verification ops, without operation/geometry normalization. It also removes the
unsupported assertion claim, adds a real 255/65535 boundary case, uses the
declared witness outcomes and derives route counts from the case roster.
These revisions require fresh source/SDK execution and independent review;
the earlier successful gate is retained as development evidence, not renewed
acceptance. Compile-fail/untrusted/mutation/model/proof gates remain open.

The actual `35ac5aa9c` successor passes its first 61-command gate: 13 complete
roots, 16 TS/Rust cases, 60 stateful routes plus one pure, exact public transcript
equality, two frontend refusals and the deeper-product scope control. Source
review nevertheless found a further P2 coverage gap: component recursion and
component-local aliases had not been executed. This gate remains development
evidence. Four complete sources/cases now cover a two-dimensional Vector
component, component aliases, Boolean component leaves and unchanged Field
component leaves. The next gate declares 17 roots / 20 cases, derives its route
counts, and drops copied unused assertion/no-mutation paths. At that checkpoint, execution and fresh reviews of the expanded cohort were
pending; no prior result was relabeled as its acceptance.


## Current bounded source/API result

At `cba7de0824d82d013ba61db7a0841abbc185a791`, original compiler180 and the
maintained gate pass **77 commands**: 17 complete unedited generated SDK roots,
20 TS/Rust cases, 76 stateful routes plus one pure route, strict Clippy on every
root, both complete frontend refusals and the deeper-product scope control.
Both recorded routes compare the entire serialized Rust verification-op list
with the actual TS public transcript and replay state/effects; the four Rust
routes agree on state, effects and gas. This is not a TS/replay gas equality claim.

Actual Claude source review v4 reports no P1/P2 (141.73 seconds). A separate
Claude receipt review v3 also reports no P1/P2 (340.86 seconds), independently
reconciling all 1,593 source inputs, 1,124 actual derivation source files, compiler
and tool identities, 17 whole manifests/SDKs, 219 Node runtime files, 20 loaded
external libraries and all command outcomes. It executes no author helper or
proof. The live and frozen trees remain exact at its observation. Host OS bytes,
resolved lock/target environment and transitive proof-runner inputs do not gain
new qualification from this first API gate.

Twelve original178 commands separately demonstrate that all four new component
sources previously passed TS but failed Rust/IR. The current180 pass trace still
contains `talias NarrowPair`/`WidePair` at the component in the `emit-rust-ir`
input; aliases were not already erased at that boundary. These observations
and all prior failures/reviews remain at their literal epochs. The separately
emitted IR lane uses the same packaged Scheme compiler, not an independent
compiler implementation.

Current local receipts: `original-native-v6/receipt.json`,
`whole-sdk-execution-v4/receipt.json`, `local-claude-source-review-v4.json`,
`local-claude-receipt-review-v3.json`, `new-component-baseline-and-trace-v1/receipt.json`
and `consumer-depfile-preservation-v4.json` under the existing evidence root.
The retained consumer depfile lists only `consumer.rs`; it does not establish
transitive generated/runtime or proof-runner completeness. Circuit test names
`proof_keep`, `proof_flip` and `proof_field` are **not proofs**.

Next, promote the eight unrun static misuse/positive-twin drafts into the
maintained gate, add marked diagnostics, malformed-IR and complete producer
mutations, and qualify checked-helper/witness first-error cases. Bind resolved
locks, target/environment and physical proof-runner inputs before proof build.
Observe every accepted export/model before selecting any tiny-admitted proof;
retain intact over-cap refusals. Fresh keys, real proofs and separate funded
default-strict applications with immutable identity and replay controls are
still required. Constructors, unselected/all-input outcomes, full required CI,
Linux and integration stay open. No Forge work, publication or cleanup occurs.

## Original gap and baseline (historical)

Original compiler177 at `438bf21c9f64b133970f8dadb212c24e54a2873d` retains
15 complete sources / 45 commands in `next-vector-product-elements-v1/receipt.json`.
Thirteen sources pass TS. Six valid parameter/helper/Field/nested/Boolean/full
branch-candidate forms fail Rust and independent IR on nested Cell.write.
Local and transparent-alias variants emit IR then fail native source/target
Vector type checking. Two witness variants emit a declared witness but a plain
call in a legacy expression binding; Rust then reports unknown circuit
vector_value. Trace the actual supplied casts and evaluation owner before fixing.

Pure nested coercion, same-type stateful return and the accepted V scalar-product
control emit successfully in all three lanes. Two narrowing/inner-dimension
frontend refusals remain intact. They establish emission/native-checking controls,
not complete current SDK execution or proof acceptance.

A focused whole unedited SDK-root probe (`next-control-roots-v1/receipt.json`)
passes strict Clippy for the pure and accepted V controls. The same-type stateful
control fails eight type_complexity style diagnostics in complete signatures.
**Close this concrete baseline issue first**, using the existing signature AST
owner and narrow macro style allowlist as required. Do not suppress other lints,
change carriers or begin coercion features while that focused baseline is red.

## Owners and intended public API

- `compiler/rust-ir-passes.ss`: supplied aggregate-cast relation, existing ordered
  return/local binding evaluation and typed witness dispatch. Preserve the
  original child type/value before checked target conversion and single evaluation.
- Existing shared native/recording Coerce rules and structured tuple/Vector AST
  emission remain semantic authorities. Reuse their bounded shape, leaf, layout,
  work/resource and both-static-branch checks; add no parallel evaluator.
- `CompactSignatureLint` and the existing witness bridge own generated signature
  metadata. A style repair must remain function scoped, preserve type/body/ABI and
  keep semantic/conditional/broad attributes rejected.

The public carrier remains the actual complete type, e.g.
`FixedVector<(FixedVector<BoundedUint<255>,2>,BoundedUint<65535>),2>` to
`FixedVector<(FixedVector<BoundedUint<65535>,2>,BoundedUint<4294967295>),2>` or
`FixedVector<(FixedVector<Field,2>,Field),2>`. Preserve ordinary, recorded and
consuming functional Program/context APIs. No new compiler flag, dependency,
IR primitive, runtime unchecked conversion or public type erasure.

## Scope and exclusions

Bounded structural widening through product components that are fixed Vectors,
with exact Vector dimensions/product arities, monotone numeric leaves and observed
Boolean identity. Characterize additional tuple nesting, aliases, checked helpers,
stateful helpers, queries and witness/error order before admitting them. Do not
infer arbitrary identity/nominal support from structural shape alone.

No Struct/nominal redesign, narrowing relaxation, general effect framework,
backend refactor, broad alias project or CI optimization. No source extraction,
geometry normalization, proof/verdict cache, cap increase or smaller substitute
contract to make a selected proof pass. Retain all complete probes and failures.

## Acceptance and order

1. After V's actual receipt findings settle, preserve its reviewed signed
   documentation/evidence. Inspect exact checkpoint/refs/worktrees before a new
   isolated successor; use the actual reviewed V head, never a guessed old
   rust-ast checkout. Run focused baseline and strict complete control roots;
   close the observed signature regression before new semantic code.
2. Trace current original frontend cast/source/target/effect owners. Fix the
   observed return/local/witness cases at those owners, retaining original bounds,
   declaration/callback/effect order, checked intermediate casts and first errors.
   Preserve accepted pure and V controls with complete source/IR/Programs and
   correct physical source paths; no JSON/label normalization.
3. Execute independent TS and complete unedited Rust SDK ordinary/recorded/
   functional routes at the real nested types. Use asymmetric outer/inner values,
   both branches and observable helper/witness/private-state effects; preserve
   transcript geometry, public replay and precise Rust-route gas agreement.
   Test checked helper narrowing/guards and both witness positions before claiming
   their support. Run strict full SDK roots, including control cases.
4. Include marked wrong inner bound/type/dimension/product arity/result carrier
   and consuming Program/context misuse, each with an independent correct-type
   twin. Preserve source refusal bytes and untrusted malformed Coerce/binding/
   dead-branch/static-empty validation; regenerate complete producer mutations and
   consume a real typed changed-result oracle.
5. Bind exact original source/compiler/backend/recipe/tools/config/serialization/
   generated physical inputs before build and retain the actual depfile before
   target reuse. Inventory every current export/model before any proof selection.
   No model size or key/proof feasibility follows from this proposal's complete
   two-export branch/Field candidate. Verify locked material before fresh keys;
   use actual tiny-admitted full-source proofs and separate funded default-strict
   applications with immutable verifier/binding/corruption/real same-geometry
   output/replay controls. If a complete selected model exceeds automatic k<=8,
   retain it and the actual refusal; do not trim, raise the cap or redefine success.
   Keep constructor/unselected/all-input obligations explicit.
6. Actual Claude-first source and separate receipt review (authorized Codex fallback
   only on actual limits), signed Conventional DCO/Assisted-by commits, verified
   Obsidian process/CAS. Focused local arm64 iteration retains full required CI,
   Linux/proof/integration gates. Current goal stays local only; no Forge,
   publication, shared install or cleanup. Broad goal remains incomplete.
