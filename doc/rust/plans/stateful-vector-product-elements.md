# Stateful Vector products with Vector elements

Status: reviewed bounded local source/API stage, continuing from V documentation head
`0bd77849cae0cf907e54c361ce92a9c04e45f5d3`, on
`codex/rust-ast-vector-product-elements`. The first change repairs the observed
strict signature baseline. Compiler180 implements the bounded component conversion.
The current stage passes 19 complete SDK roots / 27 TS/Rust cases and 141
commands, including marked static, private ingress, complete producer and actual
checked-helper/witness first-error controls. Fresh source and separate receipt
review report no P1/P2. Model/key/proof qualification and integration remain open. Forge is deferred and does not block this backend work.

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


## First bounded source/API result (historical)

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

## Next static and untrusted-input stage (under qualification)

The reviewed `037ca61d5` documentation head is preserved by a separate local
evidence ref before this stage. Eleven marked ordinary caller misuses now have
eleven explicit independent positive twins. Each twin retains a real correct-type
call; ownership twins also execute a Program/context once. The diagnostic gate
requires the declared E0308/E0382 at the exact marked expression, rejecting
unrelated errors. Twins are separate files so Rust formatting cannot leave a
partial invalid statement when a marker spans multiple lines.

A complete empty-component source extends the positive roster to 18 SDK roots /
21 cases (80 stateful routes plus one pure). Twelve private malformed-IR controls
cover outer/inner dimensions, product arity, inner leaf/narrowing, input/binding
types, helper argument source bounds, a dead branch, an empty body, an invalid
inner product and an action smuggled into an expression. Each has a separately
rendered valid private-IR twin. These control the actual native JSON ingress;
they do not pretend to be valid Compact programs or genuine proofs.

The gate now records resolved consumer/SDK lockfiles before build, verifies every
registry/Git version/checksum against the original root lock and admits only the
declared local packages. It also binds the generated consumer manifest/source,
TS result file, marked callers/twins and target/toolchain environment before use,
then checks the recorded bytes again at completion. This strengthens build
provenance without claiming complete proof-runner/transitive input qualification.
Current execution and reviews are pending. Whole-producer mutations, checked
helper/witness error cases and all model/proof/application gates remain open.

The first static/untrusted gate at `08dd37873` passes 127 commands. A successor
at `9dde83467` also passes 127 after making malformed targets agree with declared
results, keeping the dead branch's outer result correct and changing only an
invalid nested conversion, and keeping the empty body's target correct while
forging an incompatible source leaf. The 11 marked misuses and their 11 twins,
12 native malformed controls and their valid private-IR twins, and all 21 normal
cases execute. These are bounded source/API/ingress results, not new proofs.

Actual Claude review v5 reached its session limit (error, not approval); the
authorized Codex CLI fallback v6 (`gpt-6-astra`, high) reports no P1/P2 at
`9dde83467`. Its lower-priority input-reuse suggestion is addressed by checking
every bound input before each subsequent Cargo command. Selected lockfiles stay
bound after authorized resolution and before compilation; manifests/consumers
are bound before resolution, and private IR payloads before their use.

Three complete helper/witness producer mutations and a controlled changed-result
oracle now join the maintained gate after an external prototype exercised them
successfully. Regenerate each whole producer, authenticate its manifest and SDK,
bind its sources/manifest/resolved lock, and execute the unchanged old consumer.
It must reject reversed helper values or changed witness order in its first
native lane. This does not claim that every mutated route executes. The corrupt
expected-result table is an explicit negative oracle, not a genuine new TS
verdict or a proof result. Current expanded execution and fresh review remain
pending; checked-helper/witness first-error and proof gates still remain open.

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

## Reviewed static, ingress and producer result

At `eb4b7cd680afb130ca5e4607a8e833e925094b4d`, the maintained gate passes
137 commands: 106 successful exits and 31 intended refusals/assertion failures.
It executes 18 complete unedited SDK roots, 21 paired TS/Rust cases, 80 stateful
Rust routes plus one pure route and 40 exact public transcript comparisons. All
11 marked caller misuses refuse at the intended diagnostic line, all 11 explicit
positive twins pass, and all 12 private malformed inputs refuse with separately
successful valid twins. Three complete regenerated producer mutations fail the
unchanged selected consumer in its first native lane. A deliberately corrupted
expected-result atom separately fails the actual result assertion.

Actual Codex source review v7 and separate receipt review v4 report no P1/P2.
This fallback follows Claude v5's actual session-limit error, not approval. The
receipt reviewer independently reconciles 8,037 physical paths, 137 command
records/274 raw logs, 21 manifests/243 declared assets, 22 resolved locks and
7,750 package occurrences. These observations are bounded read-only checks;
they do not establish cryptographic or global supply-chain acceptance.

Reporting precision: rechecks before every Cargo invocation cover the 129
explicitly bound build inputs. Broader source/tool/configuration and baseline
SDK inventories are checked at phase boundaries/completion, not before every
Cargo invocation. The retained depfile binds the direct consumer only. The
original-native-v9 receipt establishes cached original compiler180 realization
and paired binary identity, not a new rebuild. Current receipts are
`whole-sdk-execution-v7/receipt.json`, `local-codex-source-review-v7.json`,
`local-codex-receipt-review-v4.json` and `consumer-depfile-preservation-v7.json`.
Earlier 77/127-command passes, producer prototypes and predecessor proofs retain
their original epochs and are not added to this command count.

Next qualify a complete checked helper with valid narrow-then-widen output,
assertion-before-cast, inner-component-before-scalar cast errors and actual
fallible witnesses before/after the ledger write across all four routes. No
returned error context implies transaction rollback; external witness journals
remain observable. Then inventory every accepted model before tiny selection,
bind the actual proof runner and locked material, and execute fresh proofs,
verifier negatives and separate funded default-strict applications. Constructor,
unselected/all-input, full required CI, Linux and integration gates remain open.

## Checked-helper and fallible-witness error stage (under qualification)

A complete checked-helper source takes the original wide nested input, checks
an assertion, narrows each component in source order and returns a narrow value
that its stateful caller widens. The declared cases independently require valid
reversed output, assertion failure before both invalid casts, inner 300/255
failure before scalar 100000/65535, and the scalar failure after valid inner
casts. Actual fallible witnesses separately refuse before/after the ledger
write, retaining one external callback observation at seen=0/1 and private=7.
All cases must execute ordinary/functional native/recorded routes and compare
actual typed Rust errors against independent expectations and actual TS errors.
No returned execution/proof data is accepted on error. Holding an immutable
initial-state reference is not a transaction rollback or witness-journal rollback
claim. This stage adds no production operation, public ABI or dependency.

The maintained roster becomes 19 complete roots and 27 cases: 21 successful
stateful cases across four routes, five intended error cases across four routes,
and one pure case. Exact public transcripts apply to the 42 successful recorded
routes; error cases have no returned public trace. Fresh execution/review is
pending; the reviewed 137-command predecessor remains a separate epoch.

## Reviewed ordered-error result

At signed `e293f2fe1500783b84d456f679767dfa5315b621`, original compiler180
recipe v10 builds successfully and the maintained whole-SDK v8 gate passes
**141 commands** (110 zero exits, 31 intended nonzero controls). All 27 cases
execute: 84 successful stateful Rust routes, 20 intended error attempts and one
pure route, with 42 successful recorded-route public transcript comparisons.
All 19 complete unedited SDK roots pass strict compilation. Assertions beat
invalid casts, inner 300/255 beats scalar 100000/65535, the scalar fails after
valid inner casts, and each refusing witness records one actual callback at
private=7 / seen=0 or 1 through all four APIs. The valid checked helper returns
its reversed, narrowed-then-widened value. Earlier static/ingress/producer controls
retain their successful intended outcomes.

Actual Codex source review v8 reports no P1/P2 (137.68 seconds); the separate
receipt review v5 reports no P1/P2 (550.22 seconds). The latter independently
checks before/after inventories covering 15,271 paths, 6,878 tracked Git blobs,
1,618 source pins, 131 explicitly bound build inputs, 23 locks and 282 raw
command logs. Claude's recorded session-limit error justifies these fallback
reviews; it is not an approval. Reporting remains bounded: TS assertion/cast
checks require error class and message fragments, not full diagnostic bytes;
producer mutations fail in their first native lane; Programs are authenticated
review inventories, with consumed-source correspondence unresolved by this API
gate. Pre-Cargo checks cover the explicit bound inputs; broader inventories are
phase/end checks. Held immutable state does not establish transaction rollback,
and external witness journals remain observable.

A separate current original180 freshness check passes all **283 retained
fixtures**, with zero stale output and zero failures (154.65 seconds). It compares
the normal formatted libraries and complete functional Program assets without
`--update`; no retained fixture is changed. It is separate from the 141-command
API population and from the receipt review's execution scope. Earlier178 fixture
results remain historical. Receipts: `original-native-v10/receipt.json`,
`whole-sdk-execution-v8/receipt.json`, `local-codex-source-review-v8.json`,
`local-codex-receipt-review-v5.json`, `review-adjudication-v3.json`,
`consumer-depfile-preservation-v8.json` and `fixture-freshness-v2/receipt.json`.

### Next: model inventory and genuine proof/application qualification

The remaining gap is current-source model/proof/application evidence, not a new
production coercion or API framework. Extend the existing opt-in proof-smoke
owner with a separate component-Vector cohort and exact nested carriers; ordinary
default builds must remain independent of generated SDKs. Inventory all 19
complete sources / 20 exports (19 required and one pure) before selecting proofs.
Retain intact over-cap models under automatic k<=8, with no source trim/cap raise.
Only actual observed models may justify the selected run branch states and Field
export; this plan does not assert that both fit.

Bind complete physical generated SDK/IR/Program inputs, actual runner depfile,
resolved dependencies/configuration and locked parameters before build/key use.
Require changed-IR/changed-Program/missing-Program refusal at the intended binding
guard before keys/state. Qualify actual typed ordinary/functional native/recorded
outputs and preparation, fresh keys/proofs, immutable verifier corruption/wrong
verifier/changed binding/same-geometry typed output negatives, then separately
proved funded default-strict transactions and unchanged-state replay refusal.
The external `proof-runner-proposal-v1` draft and copied complete fixtures are
**unrun source proposals**, not an adopted runner, model result or proof receipt.
Constructor, unselected/all-input, full required CI/Linux, integration, application
closure, distribution and finite capability handoff remain separate obligations.
