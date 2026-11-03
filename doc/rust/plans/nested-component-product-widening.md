# Nested scalar products below Vector components

Status: corrected original compiler185 and complete24case native API qualification
pass at `32d7eaa79`;9marked static misuse/twin pairs also pass in the development
consumer. Maintained boundary/producer controls, models/proofs, final independent
reviews and full integration remain open. Based on reviewed request-field baseline
`d9e9cb2cb`; the primary Rust AST goal is incomplete.

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
Vector, empty/single tuple and large Field-identity exports. It is an independently declared complete
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


## Characterization and development implementation

At clean `29c1658e4`, twelve original184 emission commands pass their expected
outcomes: the complete original and new twelve-export family are TS-positive
and Rust/independent-IR refused, while separately declared pure and same-type
controls pass all three lanes. Both complete control SDKs pass strict all-target/
all-feature Clippy after root-lock resolution. The first family prototype correctly
failed TS privacy checking because its branch selector was not disclosed before
branch-dependent ledger writes; the successor uses explicit disclose(pick),
retaining that failed epoch and every branch.

The pure control's actual IR contains the expected complete recursive Coerce.
The existing bounded native plan and independent recorded-domain owner already
check tuple subtrees. The development compiler185 source adds only recursive
scalar-product composition at the existing ordered selector; no renderer,
validator, runtime, schema or ABI implementation is changed. Four affected
recorded coercion-domain methods pass, including the new nested bound/shape,
invalid identity and static-empty checks. These are development source results,
not current185 whole-source/API/proof acceptance.

At that development checkpoint, compiler185 documentation generation was pending.
Its maintained ledger.ss owner and literal version-gate successes are recorded
below; current source retains that full generated document unchanged. The compiler Nix development environment itself
builds the changed CLI prerequisite; retain that actual precursor epoch. The
original full compiler package must be built from a separate frozen clean source
checkout, then the current qualification head must resolve the same original
recipe with builds disabled. Do not infer fresh source acceptance from the new
relation or from earlier compiler184 results.


## Current complete native API result

The first frozen original185 package (`c5d8f564e`) and its exact current-head
existing-only recipe pass. All twelve TS/Rust/independent-IR emission commands
pass for the original, first twelve-export family and separate pure/same-type
controls. The initial documentation Nix-shell client hit its 180-second deadline
while the changed CLI precursor was building; that failed attempt remains. The
separate full frozen producer completes; only then does a fresh owner-generator
attempt produce compiler185 docs and pass the literal local version gate. No
build is restarted merely because its observation times out.

The first complete API attempt executes all22Rust methods across four routes,
then fails strict consumer lint on a copied unused pure-case macro. The whole
attempt stays failed. A versioned fixture successor removes that macro and adds
one complete export for Field identity with input2^130+21 beside widened Uints.
All previous twelve source-export bodies and the original regression are unchanged.
Because examples are explicit original Nix inputs, the successor is rebuilt from
frozen `2dfcbb81f` (69.44 seconds) rather than relabelling the earlier package.
The actual current-clean original-recipe recorder passes at `ebd9c2b52` in12.19s.

The maintained `check_nested_component_products.py` gate now passes in22.08s:
two complete sources, fourteen exported stateful APIs,23fresh live-TS cases and
23typed Rust methods covering92route executions (including24error routes) and
34successful recorded VM replays. Every declared export has a case. Both complete
unedited SDK roots pass strict all-target/all-feature Clippy and the consumer's
actual test code passes all-target strict Clippy. Exact results, full public
transcripts, state/effects/gas, private outputs/counters, both branch choices,
helper assertion/cast first errors and before/after witness failure order pass.
Independent whole IR equals each SDK's raw IR; fourteen checked Program assets
and complete schema3 capability rows remain source-bound. These counts are not
language coverage or executed proofs.

The gate pins selected source/artifact/configuration/tool identities and locks;
it is not complete transitive/physical build attestation or a hostile-host trace.
It reuses the existing bounded-command and Cargo-target guards, resolves host
tools without machine-specific paths, and preserves inherited Cargo reuse.
Marked compile-fail twins, native malformed/dead/empty IR controls, consumed
producer/result mutations, all-export models, fresh applicable proofs with
independent verification and separate strict applications, own source/receipt
reviews and full CI/Linux/integration are still required before slice acceptance.
The old current scope table still expects the original refusal; archive its
compiler184 epoch and reclassify only that row after complete positive controls,
before claiming current parent-gate or integration readiness.

Evidence includes `native-api-controller-v1.json`,
`current-existing-recipe-v1/receipt.json`, `native-api-v1/receipt.json`,
`original-build-v1.json`, `original-build-v2.json`, `family-successor-v1.json`,
`api-prototype-v1/receipt.json` and both raw owner-generator attempts. Source
review is a separate stage; no review or proof result is inferred from these
native executions. Forge remains deferred and every predecessor is retained.


## Independent source review corrections in progress

Actual Claude source review at `ebd9c2b52` finds no compiler-correctness defect in
the bounded relation, but identifies a real P1 inventory failure: both locally
added root-level examples lacked registrations. Preserve earlier local receipts,
not a claim that full CI was green. Each example is now in its owned cohort
directory, with full bytes and basename unchanged. The parent field repair is
separately corrected on `be4ba31d9` without rewriting or merging branches; actual
source_registry fails before and passes after, and its fresh parent184 compiler
recipe/consumer qualification passes. The child existing fixture-inventory suite
passes all31methods. Older source/receipt paths and parent refs stay retained.

The new coercion test now mutates actual inner Vector length, nested tuple arity
and nested tuple-versus-scalar in addition to existing bound/outer/static-empty
checks; all four affected domain methods pass. A third-cast error case is added
with simultaneous later outer failure to make first-error order observable.
Oracle formatv2 adds explicitly typed source argument FAB values, and each Rust
route compares its actual supplied arguments before execution, including errors.
The clone-only held-state assertion is replaced by an actual pre-call state
check; no failed-call rollback claim is made. The complete native API must rerun
with24cases before these corrections are accepted.

The full current compiler184 scope table is archived byte-for-byte and literal
hash-pinned in both older consumers. Only the supported nested-product row is
reclassified in place; all3 historical rows/files stay. The own unsupported
top-level nested-tuple source preserves the intended boundary outside component
Vectors. Actual source-refusal/static/ingress/producer/model/proof and review
gates remain open. Parent/child ancestry reconciliation is a future integration
step; current local authority allows no merge, publication or hosted run.

## Maintained boundary stage

The review-corrected current recipe resolves its exact frozen original185 package
in7.03s; the full fresh native API gate passes in17.71s:24TS cases,24Rust methods,
96routes including28errors and34successful recorded VM replays. Every Rust lane
binds its actual supplied source-typed argument FAB values before execution.
This supersedes the23case result for current behavior without relabelling its
earlier receipt or frozen source.

Development static controls execute9marked failures with exact error codes and
primary misuse spans, plus9matching positive commands. Five input-shape/bound
controls share the same genuine Narrow signature in their positive files; those
are not five distinct positive semantic behaviors. A separate whole13export IR
diagnostic probe executes12valid/malformed pairs, including actual nested tuple
shape, bounds, source/helper binding, dead-branch and static-empty child checks.
All valid twins render; each invalid copy exits before any Rust output. These
private copies are untrusted ingress controls, not genuine new compiler output.

The maintained API gate now includes those controls and three complete producer
mutations (helper result and both witness orders), plus one controlled corrupt
expected-result table. SDK source trees remain unedited, and changed producers
have separate whole-source SDK/consumer roots. The unchanged typed consumer must
fail through a real executed assertion, not an unrelated compiler error. The
new maintained stage must execute before acceptance; no model/proof verdict or
whole-CI result is inferred from the earlier development commands.
