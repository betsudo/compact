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


The first clean165 source characterization fixed17/18 TS-valid emissions but
retained one failure: nominal-source explicit casts contain an inner frontend
alias-unwrapping cast that discarded the wider expected context. Preserve
that complete cast at its original type, then apply existing Coerce in its
Vector context. Conditional arms already supply their own outer Coerce; keep
raw alias variables and casts there, preserving original accepted conditional
IR/origins rather than adding redundant wrappers. Four other control sources
retained all16 original IR/raw/Rust/capability bytes in that partial run; all
five controls must be freshly compared after this correction. Failed/partial
receipts remain evidence; the nominal case is not removed from acceptance.


## Corrected current-source result and maintained corpus

Default original165 at clean signed9d42ba6da builds without substitutions.
The full22-source/66-command characterization now passes:18TS-valid sources
emit Rust and independently equal IR, and4frontend refusals retain their
original causes. All20 original control assets across5sources (fullIR, raw
origins, unedited Rust and capability bytes) are byte-identical to original164,
including the previously affected conditional. The failed Nix compile at638
was an undeclared nanopass metavariable, corrected at9d42; its logs and the
partial predecessor characterization remain retained.

The maintained gate is
`tools/compact-rust-backend/check_vector_alias_source_widening.py`. Its complete
source corpus includes20sources and26 independently specified cases:24pure
calls and8stateful route executions. This includes alias chains, empty/nested/
product/wide/Field carriers, explicit nominal source and target controls,
identity and conditional controls, call-argument/literal-element/local contexts,
and ordered input/later-assertion failures. Every result has an explicit TS
descriptor, and route counts derive from the declared case roster. No native
error substitution is permitted in this population.

Four Rust refusal controls cover wrong output length, wrong input element,
wrong output maximum and consuming Program reuse;4source refusals retain
frontend dimension/kind/narrowing/nominal authority. Changed full producers and
same-geometry expectations must fail actual consumers. Three malformed IR
controls include validating impossible element conversion even for zero
length. ActualSDKroots remain strict, unedited compile inputs. Full current
clean-head source/API execution, fresh selected proofs/applications and
independent review remain required; generation alone is not acceptance.


The first maintained clean-head API run at192 executed all26 consumer cases,
20 strict roots,4 ordinary Rust and4 source refusals, and the changed-result
control successfully. It then stopped because the changed small producer
placed indexed narrow scalars directly in a wider Vector, exercising a distinct
indexed-scalar expected-context gap instead of this source-alias conversion.
The repaired full producer reverses into its original Input type and then
returns that alias to the wider result, matching the actually repaired owner.
Its emission succeeds; the new full run must still demonstrate actual consumer
failure. The earlier failed gate remains failed evidence; no case was deleted.

Literal192 regression passes all283 historical fixture freshness checks,
1830 backend tests with7 existing ignored, and strict backend Clippy. Claude
review was actually unavailable with an explicit session-limit response;
independent review uses the authorized Codex CLI fallback and a complexity-
appropriate different model. No successful review result is inferred.


## Source/API acceptance and proof preparation

At clean signeda6cfd854c, original native165 and the complete maintained gate
pass:102 commands,20 complete sources,26 TS/Rust cases (24pure and8stateful
routes),20 strict SDK roots,4typed/4source/3nativeIR refusals and the actual
changed expectation/two complete changed-producer consumer failures. No source
or negative control was omitted to obtain this result. Historical regression
retains its literal192 epoch; compiler/native/runtime owners are unchanged.

Authentic Codex CLI fallback review (gpt-6-astra, high) reports no introduced
P1/P2 in the complete diff and actual shared coercion/certification owners.
Claude's explicit session-limit result is retained as unavailable, not review
acceptance. The suggested matching nominal-source implicit-unwrapping refusal
is now added: current165 independently refuses it in TS, Rust and Scheme IR.
The next full gate must execute all5 source refusals.

The proof gate is
`tools/compact-rust-backend/check_vector_alias_source_widening_proof.py` with
explicit `--compiler-receipt`, `--ts-runtime`, `--zkir` and a new `--output`.
Rust1.99.0 and absolute `MIDNIGHT_PP` with locked upstream material are required.
`--prepare-only` performs no key generation, proofs or applications. The opt-in
Cargo feature includes the complete current unedited helper SDK and both full
Programs. Its host module is public so all generated exported alias types
remain reachable under strict lint; generated files are not patched. Actual
development opt-in compilation and strict Clippy pass.

The gate redoes the full current source/API check, checks default/all-features
without a generated SDK and actual included dependencies, refuses changed IR
and Program bytes/membership before keys or state, and inspects both helper
models under unchanged automatic k<=8. A separate complete retained Fold
source supplies a real larger diagnostic only for the pre-key cap refusal;
it does not replace, trim or admit the alias helper source. Compiler/tool hashes
are rechecked before that diagnostic and after proof execution. Verified locked
material must precede fresh direct keys. Acceptance requires both selected
fresh proofs with immutable verification and binding/result/corruption/different-
verifier controls, plus independently proved funded default-strict transactions
and unchanged-ledger replay refusal. Constructor proof, full required CI and
integration are not claimed by compilation or selected-helper qualification.
