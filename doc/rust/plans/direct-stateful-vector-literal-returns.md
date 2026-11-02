# Next proposal: direct stateful Vector literal returns

## Status and baseline

S is admitted on `codex/rust-ast-direct-stateful-vector-literal-returns` from
reviewed R documentation head `c0b294d3b4dc6372591e38b3d6a28dba8064e6b5`.
Bounded local S qualification is accepted at execution `7ab60b62061a728b3e81f34ae31e9693b1063d41`, with the source/receipt scopes below. The investigation sections retain earlier checkpoint states; the final receipt section records current local acceptance. The primary Rust AST goal remains
active and incomplete, with local work only. Preserve canonical and every
R/Q/P/O/N/M predecessor. Forge, publication and cleanup remain excluded.

## Concrete source gap

`next-direct-stateful-literal-characterization-v1/receipt.json` retains nine
complete sources and 54 commands under original compiler 166 and current 167.
All nine sources compile for TS.

| Cohort | Independent IR | Rust |
| --- | --- | --- |
| Four direct implicit Uint/Field widening forms, using Vector or Struct inputs | Refuses a nested `Cell.write` | Same frontend refusal |
| Four direct identity/explicit-element forms | Emits a Tuple tail | Rejects Tuple against declared Vector |
| Complete helper control with two stateful exports | Accepts | Accepts |

These failures also occur under 166. The complete helper control works in every
lane; its six full IR, raw-origin, Rust, capability and Program artifacts are
byte-identical across 166 and 167. Accepted helper-based returns, fixed-vector
projection, Fold and Map capabilities are not reopened.

Three actual 167 source traces retain the relevant forms before `emit-rust-ir`:

- Implicit widening wraps `seq(Cell.write, tuple(projections))` in
  `safe-cast(target Vector, source Tuple, ...)`.
- Identity retains the same complete stateful sequence and Tuple tail with a
  declared Vector circuit result and no outer cast.

The traces and diagnostics establish a stateful expected-result/carrier ingress
gap. Investigate the precise step/value owners before implementation; this is
not yet a feasibility or implementation claim.

## Intended API and owners

Keep the original complete direct exports: `run` accepts either
`Vector<2, Uint<8>>` or the declared generated `types::Input`, writes the Cell and
returns its literal as `FixedVector<BoundedUint<65535>, 2>`, the identity Uint8
carrier, or `FixedVector<Field, 2>`. Preserve ordinary, recorded and consuming
functional APIs on the current Ledger8 profile.

Extracting the return into a pure helper is an already-working control and does
not implement this gap. Preserve the direct source and its effects.

Reconcile `compiler/rust-ir-passes.ss` stateful value, step and return owners.
Retain statement handling outside ordinary value lowering. Reuse existing typed
literal construction, Q/R actual-element context, native coercion legality,
recording certification and Vector AST owners. An aggregate cast around a whole
stateful body must preserve that body's complete effect sequence.

No arbitrary Tuple-to-Vector conversion relation, IR/runtime ABI, dependency,
executor, generic expression normalization or framework is proposed.

## Exclusions

No constructor result redesign, dynamic indexing, Map/Fold migration, nominal
redesign, general stateful cast normalization, effect lifting, source rewriting
or helper substitution. Preserve primitive cast guards, original parameter and
field bounds, source order, profile/private policy and required integration
gates. No CI optimization or Forge work.

## Acceptance plan

1. Read the current checkpoint, actual refs/worktrees, roadmap, crosswalk and
   checked-recording ownership. Preserve accepted authorities. Admit a separate
   clean local baseline from the reviewed predecessor; record any stack
   dependency explicitly. Run focused baseline checks first.
2. Fix all eight observed direct forms, including both identity construction and
   implicit whole-body cast context. Keep the original `Cell.write` and source
   syntax. Characterize relevant lexical, conditional, alias and nested tails,
   and assertions before/after state changes, before expanding scope. Preserve
   all six accepted helper artifacts.
3. Compare independent TS alignment/results with unmodified generated ordinary,
   recorded and functional execution. Check typed output, Cell state/effects,
   private output, gas, trace replay and observable first errors. Preserve
   documented ContractLab checkpoint behavior without promising rollback of
   external callbacks or inventing partial traces.
4. Include correct-type positive twins and marked Rust refusals for wrong input,
   result length/element/maximum/Field carrier and consumed Program/context.
   Retain frontend length, kind, narrowing and nominal refusals. Existing full
   native validation must reject malformed carriers, field/index/type errors and
   invalid unselected branches; do not weaken validators to accept wrong IR.
5. Bind complete source/imports, independent/golden IR, all Programs, the current
   original compiler/recipe, serialization, Cargo configuration, tools, runtime
   and profile identities. Require strict unedited SDK roots and actual
   consumer failures after producer/result changes.
6. Inspect full selected models and verify locked material before fresh direct
   keys. Keep automatic k<=8 and every full source/export. Require selected
   genuine Uint/Field proofs with immutable verifiers, corruption/binding/
   same-geometry real-carrier result/wrong-verifier controls, and separate,
   independently proved funded default-strict transactions with exact replay
   refusal. Disclose selected coverage and over-cap obligations; never trim,
   raise the cap or waive original N k=9.
7. Use signed Conventional DCO/Assisted-by commits, actual Claude-first
   independent source/receipt CLI reviews with actual-quota different-model
   Codex fallback, documentation and verified Obsidian process/CAS/handoff.
   Full required arm64/Linux CI and reviewed integration remain open until
   actually executed. Cleanup follows verified GitHub merge of completed owned
   work.


## Admission and owner reconciliation

The current checkpoint and actual refs/worktrees were inspected before creating
the separate branch. Relevant stateful and native/runtime owner blocks are
identical to canonical `1bdf185ea`; the retained accepted helper control depends
on Q's pure projection fix. Use the explicit R stack to retain that behavior
without duplicating owners or inventing an integrated baseline.

The legacy return path emits the literal Tuple without its Vector context. The
existing ordered return plan already preserves actions and conversions as
continuations. The native planner already supports bounded, nonempty
Tuple-to-Vector coercion. Investigate reuse of that relation after source actions;
no new conversion relation is needed. Empty literals and explicit checked casts
require actual characterization before selecting their construction path.


## Baseline and implementation decision

The signed clean S baseline passes 32 coercion, projection, order, historical
Vector and stateful-return guard tests. Expanded original-167 characterization
retains 33 complete sources and 99 commands: 26 TS-valid sources, 25 Rust
mismatches and one accepted helper control, plus seven frontend refusals. The
private conditional remains a privacy refusal; a separate disclosed version is
positive. Whole-vector narrowing remains refused. Empty, nested, heterogeneous,
wide, alias and 13-element cases are preserved. The direct proof source has an
explicit constructor state so both branch outcomes can be qualified.

Compiler 168 selects the existing ordered return plan for Vector-result literal
tails. Actual literal syntax is inspected only for frontend routing; native and
recording admission remain independent. Helper-only roots retain their old route; mixed conditionals use one ordered body.
The supplied source type is carried through aggregate casts as the expected
source value; the existing checked coercion is applied as a continuation after
actions. This retains intermediate casts and evaluates conditions once. Identity
and empty literals use the existing typed Vector builder, with the existing
Vector alias carrier helper. No Unit-to-Vector conversion is introduced.

No native validator, runtime, IR schema, dependency or executor changes. Original
current compiler, complete current source/API/proof and independent review
acceptance are still open.


## Maintained qualification corpus

`tools/compact-rust-backend/check_direct_stateful_vector_literal_returns.py`
binds 31 complete stateful/impure sources, their independent and golden full IR, all
checked Programs and unedited SDKs. The 50 declared cases use an independently
executed TypeScript oracle and ordinary, recorded, functional native and
functional recorded Rust APIs. Success checks include typed alignment/results,
Cell state, private state/output, cross-route effects/gas and recorded replay.
Failures check the actual first diagnostic; no partial trace or external callback
rollback is invented. Six marked Rust misuse controls, seven retained frontend
refusals, malformed native IR including a dead branch and action-as-value, source
producer changes and a changed oracle result protect the boundaries.

`tools/compact-rust-backend/check_direct_stateful_vector_literal_returns_proof.py`
runs that whole protocol before proof admission. Its opt-in proof runner includes
the complete `direct_branch_proofs` SDK and two Programs. It observes every corpus
export model, then separately enforces k<=8 for selected exports and verifies
locked material before fresh direct keys. The three selected scenarios are the
Uint return with constructor state 0 and 1, and the Field return. Each requires a
standalone proof plus a separately proved funded default-strict application,
immutable verifier/corruption/binding/same-geometry typed-result controls and
exact replay refusal. Constructor execution and unselected exports are unproved;
the complete required integration gates and original N k=9 obligation stay open.

The initial 168 development epoch passed all 41 then-declared storage cases and seven malformed-IR refusals. These
are development observations, not clean-head qualification or proof acceptance.
Run the maintained gates using a new output directory and a default current-head
original compiler receipt. No generated SDK rewriting or source trimming is
permitted. Full source/receipt reviews and actual proof results remain required.


### Review and strict-root follow-up

The first clean protocol caught the retained characterization Program source paths
rather than the maintained repository-relative paths. Actual full regeneration
fixed those goldens without normalization; exact byte checks remain unchanged.
The next protocol passed current execution but caught strict Clippy on the
13-element intermediate Tuple type. Uniform source literals now materialize a
Vector at their original checked element bound before the existing aggregate
conversion. Heterogeneous source carriers retain the Tuple path. No lint allow,
validator/runtime/ABI change or wider source bound is introduced.

Claude's actual source review found a coverage gap for the same routing without
storage mutation and with witness-call elements. Add separate non-mutating and
sequential witness sources. The latter must check TS-aligned two calls in source
order, evolving private state and both private transcript outputs exactly once
across all four APIs. These are API coverage, not a witness proof claim. Current
compiler 169, expanded protocol and final source/receipt review remain pending.


The witness source exposed an ordinary native emission gap: Vector constructors
were delegated entirely to pure expression lowering. The same stateful aggregate
owner now handles the pure owner's explicit effect refusal by evaluating Vector
children once in order and checking each against the declared element carrier.
Successful pure Vector AST emission stays byte-identical, with other errors
retaining their observed diagnostic owner. Recording certification, validators,
runtime and ABI are unchanged. Two focused native tests check witness child
ordering and element mismatch; a third preserves pure Vector AST output.
Compiler metadata is 170. All expanded qualification remains pending.

The first non-mutating fixture was inferred pure by the compiler and correctly
refused functional-stateful emission. Its complete failed source is retained.
The actual impure positive reads the public Cell in a condition without writing
it; it has a genuine existing stateful API obligation.


Current170 development passes43cases/172routes, including the actual two-call
witness transcript and private-state evolution. All1833native unit tests pass
with7existingignored; strictnativeClippy passes. All283retained compiler fixtures
are freshly identical, with no rewriting.

The expanded clean gate correctly exposed an overly broad proof assumption in
the harness: the witness-only export is recorded/observed but has explicit
`proof_required:false`, `recording_status:not_applicable` and no ZKIR. Preserve
that established distinction. Definitions now pin proof applicability explicitly;
all34exports are inventoried, with33actual models and one explicit non-applicable
row. The selected direct proofsource still has2requiredexports/3scenarios, and
31requiredexports remain unproved. No witness-only proof obligation is invented.


Fresh Claude v2 source review confirmed the native/frontend repair and identified
finite missing controls. The corpus now includes literal/helper and
literal/parameter mixed returns on both branches, public-read Vector children,
and constructor-controlled else outcomes for the no-write and state-condition
sources. Only helper-only roots preserve their legacy route; mixed bodies use a
single ordered owner. Complete current tests/proofs and final review are pending.
The failed inferred-pure source is external retained evidence at
`/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-vector-literal-returns-20261102/development-api-v2/nonmutating-failed-source.compact`.

Current170 expanded development executes all50cases/200routes and real TS outcomes, including all fresh review controls. The complete31sources have34exports:33required models and one explicitly non-applicable witness-only export. Selected proof coverage remains two direct exports with three scenarios;31requiredexports are unproved. Clean gate/proofs/current review are pending.


The exact242 clean source/API gate passes all155commands,50tests/200routes and
31strict unedited roots, including every refusal and mutation control. Fresh
Claude v3 reports no correctness/routing/coercion/effect/binding defect; its two
low clarity findings are fixed (changelog subsection and proof applicability
naming). The full proof gate then correctly stopped before models/keys because
Cargo records the embedded source roster with lexical `src/../../` components.
Parse the actual Make dependency paths and resolve them before membership checks;
retain raw depfile hashes and require every physical SDK/Program/roster include.
This changes no IR/Program/source normalization or proof binding. Renew the exact
clean gate before any fresh proof acceptance. All failed bytes remain retained.


## Reviewed bounded local qualification — 2026-11-02

Execution `7ab60b62061a728b3e81f34ae31e9693b1063d41` / compiler170 passes the
default original native recipe and the complete maintained source/API/proof gate.
The current source gate passes155commands:31complete sources,50cases/200Rust
routes, actualTS,31strict unedited SDK roots,6marked Rust compile-fail controls,
7frontend refusals,7native malformed-IR refusals, forged-origin refusal and
actual failures after two complete producer changes and one oracle-result change.
Complete independent/golden IR and all Programs remain byte-bound. Current native
1833unit tests pass with7existingignored; all283retained compiler fixtures are
fresh with zero rewrites. A separately captured strict native Clippy run at7ab
pins its actual `-D warnings` command, toolchain and clean source identity.

The34export inventory retains33proof-required models and one explicit witness
not-applicable row. Selected `direct_branch_proofs.run` is k8/130rows;
`run_field` is k7/85rows. Locked material is verified before two fresh direct key
pairs. The three genuine2912B standalone proofs cover Uint initialSeen0 and1,
and Field initialSeen0. Three separately proved funded default-strict
transactions are6604/6604/6561B. Corruption, changed public binding, real typed
same-geometry changed output, opposite immutable verifier and exact
`IntentAlreadyExists` replay controls pass. Each replay leaves the ledger intact.

Actual Claude source reviews v1–v3 and a separate current-delta/receipt review
are retained. The final review reports no blocking P1/P2 issue and independently
rehashes4736pin records, source/frozen Git identities, complete source/tool/key/
material/output maps and31strict roots. Its low audit duplicate/missing-v5 pin
is corrected in an author successor packet4751records/4347distinct paths; the
original reviewed packet remains untouched. Raw Cargo depfile bytes are retained
in private evidence with the exact admitted digest. This is local engineering
review and finite qualification, with no broader host-trust or security-audit
claim. All failed epochs, complete original sources and preceding goldens remain.

Evidence root:
`/Users/ysh/.codex/local-evidence/rust-ast-direct-stateful-vector-literal-returns-20261102`.
Current receipts: `original-native-v9/receipt.json`, `proof-v6/receipt.json`,
`proof-v6/protocol/receipt.json`, `receipt-audit-v1.json`, its corrected successor
`receipt-audit-v2.json`, `strict-native-clippy-v2/receipt.json`, actual CLI review
results and the signed documentation bridge. Log/retention hashes are observed
at closeout; they are not retrospective execution-time attestations.

Constructor execution,31unselected required exports and unenumerated inputs are
unproved. The full `wide_struct` and `wide_vector` models are k9 and remain
unproved under the unchanged automatic k<=8 rule. Original N k9 qualification,
complete required arm64/Linux CI, reviewed integration and the broader backend
goal remain open. This does not complete deferred Forge work. Canonical1bdf and
all predecessor stores/results/worktrees remain preserved; cleanup follows
verified GitHub merge of completed owned work.

The next [stateful Vector value-widening proposal](stateful-vector-value-widening.md)
is based on a fresh source-only probe. No next-slice implementation is admitted
by this closeout.
