# Next proposal: direct stateful Vector literal returns

## Status and baseline

S is admitted on `codex/rust-ast-direct-stateful-vector-literal-returns` from
reviewed R documentation head `c0b294d3b4dc6372591e38b3d6a28dba8064e6b5`.
No S implementation is accepted yet. The primary Rust AST goal remains
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
recording admission remain independent. Helper calls retain their old route.
The supplied source type is carried through aggregate casts as the expected
source value; the existing checked coercion is applied as a continuation after
actions. This retains intermediate casts and evaluates conditions once. Identity
and empty literals use the existing typed Vector builder, with the existing
Vector alias carrier helper. No Unit-to-Vector conversion is introduced.

No native validator, runtime, IR schema, dependency or executor changes. Original
current compiler, complete current source/API/proof and independent review
acceptance are still open.
