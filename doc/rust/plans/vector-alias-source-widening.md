# Next bounded Rust AST slice: widening a Vector alias source

Candidate P is admitted on `codex/rust-ast-vector-alias-source-widening` from
clean O docs head `03a1141124404f119fcbcfb33aa788663840d4af`. This is an explicit
local shallow-stack dependency on O's Vector carrier helper, not integrated
canonical work. No implementation is accepted yet. Primary goal remains
ACTIVE / INCOMPLETE and LOCAL ONLY. Forge remains deferred.

## Established gap

The complete four-source,24-command original144/current164 comparison is
`alias-boundary-characterization-v1/receipt.json`. Both independent TS and
Scheme IR accept a pure parameter typed through `Input = Vector<2,Uint<8>>`
returned as `Vector<2,Uint<16>>`; Rust rejects actual255 versus expected65535
under both compilers. An explicit Vector result cast also retains the same
Rust failure. This is the actual source carrier remaining aliased at pure
expected-type ingress, distinct from candidate O's complete Fold result and
expected target carrier fixes. It predates O and is not a regression caused
by the result repair.

Original144 and current164 both reject implicit conversion into a nominal
`new type Output` at the shared frontend. Explicit nominal Vector casts are
TS-valid; original144 Rust/IR refuses, current164 emits both. This confirms
that O's carrier erasure includes frontend-approved nominal explicit casts;
its earlier wording as only transparent was incomplete. This characterization
is emission/refusal evidence only, not native/API/proof qualification of that
additional nominal population. Retain nominal frontend authority in the next
slice's positive and refusal controls.

## Baseline and owners

Read the actual current main checkpoint before admission. Canonical remains
`codex/rust-backend-ast` at1bdf185ea; do not reset or move it. O is isolated at
f75de428d with source/API and selected proof/application qualification; its
receipt review and docs closeout are complete within their local stated scope. A separate local branch
may explicitly shallow-stack on O's exact clean docs head to reuse the carrier
helper. Record that dependency, rather than representing it as integrated
canonical work. Retain N and all earlier candidates/evidence.

Reviewed context: ledger8-delivery-and-completion.md, checked-recording.md,
ownership.md, capability-and-retirement-book.md and closure crosswalk. Existing
Coerce/ADR0330 conversion relations, fallibility, bounded work, origin and
signature owners remain authoritative. Pure expected-type variable ingress in
compiler/rust-ir-passes.ss is the proposed changed owner. Do not replace shared
Rust AST, native or checked-recording owners.

## Intended API and finite scope

A typed alias input should retain the source FixedVector<BoundedUint<255>,2>
carrier while a valid wider declared result uses
FixedVector<BoundedUint<65535>,2>. Generate ordinary typed methods with existing
Coerce after the original source expression and existing bound/type validation.
Do not add public adapters, runtime traits, unchecked constructors, ABI/IR
variants or a new conversion relation. First characterize alias chains,
unchanged equal types, supported nested/product elements, empty vectors,
explicit casts and relevant lexical/call-argument boundaries. Reduce the scope
if independent frontend typing rejects a proposed implicit conversion.

Exclusions: general nonVector alias redesign, nominal type semantic erasure,
arbitrary tuple/stateful return/call normalization, new Fold/Map semantics,
new proof framework, refactoring, CI optimization, Forge and publication.

## Acceptance

1. Establish focused current clean baseline and accepted controls, independently
   characterize complete source and current generated output before edits.
2. Preserve exact frontend rejection for wrong dimensions, unrelated kinds,
   prohibited narrowing and implicit nominal conversion; preserve existing
   explicit nominal-cast behavior. Keep old compatibility fixtures unchanged.
3. Independent TS expectations and unedited generated Rust must execute with
   exact values/alignment; zero/nonzero and original bounds remain checked.
   Invalid ordinary arguments/result assignments refuse at the intended Rust
   expression, with positive twins; consuming Program reuse refuses.
4. At untrusted IR boundaries retain native type/signature/layout validation,
   including empty/static paths. Source-once and first-error controls apply
   where the actual source expression can fail. Test changed expectations and
   regenerated producers against actual consumers without normalizing output.
5. Bind full source, independent/golden IR, all Programs, current original
   compiler recipe, tools and complete SDK. Strict Clippy applies to actual
   roots. Inspect full helper models; verified material precedes fresh direct
   keys. Where admitted stateful callers use the repaired pure helper, require
   fresh proof/immutable verification plus separate funded default-strict
   application and result/binding/verifier/replay controls under unchanged
   automatic k<=8. Do not trim full source or raise limits.
6. Signed Conventional DCO/Assisted-by commits; authentic independent CLI source
   and receipt review; verified Obsidian process updates and checkpoint CAS.
   Focused local validation during iteration; all required arm64/Linux CI and
   reviewed integration stay required for later integration.


## Baseline, characterization and implementation boundary

At signed plan cf388f8b2 the focused20 coercion/Fold/historical Vector checks
pass. Actual pure variable ingress is byte-identical across canonical, held
scalar/aggregate, M, N and O owners: it has no actual-source Vector alias case.
The corrected22-source/66-command original164 characterization has18TS-valid
sources:13Rust mismatches and5 accepted controls, plus4frontend refusals in all
three lanes. The initial run applied functional flags to pure-only sources;
its extra boundary refusals remain diagnostic evidence, not semantic failures.

The change adds only the missing alias case under the existing pure expected
Vector variable owner. It resolves the source's Vector carrier and delegates
conversion to existing Coerce. If alias-erased actual and target types already
match, it returns the original expression so accepted identity outputs remain
unchanged. NonVector aliases retain their old owner. The shared frontend still
checks nominal legality before this phase; explicit nominal casts are covered,
illegal implicit nominal conversion remains a required refusal. Scalar, tuple
and stateful expected ingress are unchanged. Compiler165 metadata identifies
this source; no runtime ABI, IR schema, dependency or executor is added.
