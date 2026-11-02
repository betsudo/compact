# Declared Vector results after named pure Fold

Candidate O is isolated on `codex/rust-ast-fold-vector-result-widening` from the
actual clean canonical `codex/rust-backend-ast` at
`1bdf185ea35929292a79d35946d4a98fb344d11c`. Candidate N and all earlier branches
remain preserved; this branch has no implementation dependency on them.
Previous goal turn was PROGRESS: source/API and selected genuine Map proofs,
independent source/receipt reviews, signed documentation and verified KB/CAS
were completed within their stated local scopes. Forge remains deferred.

Focused canonical baseline passes 20 tests: ten shared coercion laws, three
compiled Vector coercion controls, two call/Fold signature/order controls and
five historical Vector-widen consumer tests. No baseline regression was found.
This is focused local evidence, not a full CI or proof qualification.

## Concrete gap and accepted controls

The prior candidate's `next-fold-result-characterization-v2/receipt.json` contains six complete sources and 36 original compiler commands: TypeScript, Rust and independent Scheme IR for each source under original compiler 0.31.144 and current 0.31.163. All six sources are accepted by TS. Named scalar Fold results already widen to Uint and Field, including zero iterations and local/conditional controls. Do not reopen those accepted capabilities or historical Fold input widening.

The named Fold whose accumulator/result is `Vector<2, Uint<8>>` and enclosing declared result is `Vector<2, Uint<16>>` produces valid Scheme IR, but Rust rejects the actual Vector element bound 255 against expected bound 65535 under BOTH compilers. This is a distinct result-carrier gap. The earlier v1 inline probes stopped at the separate named-only pure Fold ingress; they do not establish a result-widening defect and must not expand this slice into general #661 inline/stateful Fold work.

## Owners and intended API

- Frontend typed expected-result ingress in `compiler/rust-ir-passes.ss`: preserve the complete Fold and its original accumulator/reducer types, then use existing Coerce for the declared Vector result where necessary.
- Existing shared aggregate, pure callee certification and coercion owners: keep independent signature, conversion, fallibility, dimension and static-body validation. No new conversion relation, IR variant, runtime ABI or executor.
- Generated consumers: retain `FixedVector<BoundedUint<255>, 2>` source/seed carriers and expose the declared `FixedVector<BoundedUint<65535>, 2>` output.

## Scope and acceptance

1. First reconcile this gap against the current backend roadmap/crosswalk and held source branches. Use the actual canonical branch; do not reset it or invent a baseline.
2. Characterize nonempty/empty Fold, narrow original accumulator guards, nested numeric products, aliases and Uint/Field/wide result carriers. Preserve already accepted scalar/local/conditional and historical Fold-input controls. Reject wrong dimensions, unrelated element kinds and illegal implicit narrowing at their existing boundaries.
3. Run independent TS expectations, unedited generated Rust, callback/source failure order and source-once evaluation. Positive typed twins must compile; wrong lengths/carriers/result bounds and consuming Program reuse must refuse at the intended ordinary Rust expression where applicable.
4. Bind full source, independent/golden IR, every Program, tools and original compiler recipe. Use the corrected compiler/wrapper preflight. Check the generated SDK as an actual strict root. Changed producers and expected outputs must fail actual consumers; malformed IR controls retain native authority.
5. For admitted stateful callers through the pure helper, inspect actual models before choosing selected fresh proofs. Keep the full source and all Programs. Require verified material before keys and separate proof/strict application identities. Preserve automatic k<=8; disclose any larger unproved circuit explicitly. Do not trim or raise limits to manufacture a passing cohort.
6. Keep all work local with signed Conventional DCO/Assisted-by commits, independent CLI source and receipt review, verified knowledge-base updates and checkpoint CAS. Full required arm64/Linux CI and reviewed integration remain required later.

Exclusions: general inline/multi-input/effectful Fold, new operation families, general #516 stateful whole-body cast normalization, broad refactoring, CI optimization and Forge. This proposal does not close the earlier Map slice's original k9 branch-proof criterion, full CI or integration.


## Reconciliation and original order boundary

The current capability/retirement book and closure crosswalk distinguish native
pure Fold, certified pure calls and direct stateful Fold. The integrated #661
plan preserves legacy named `VectorFoldCall` source-first native initialization;
new lexical stateful Fold is a distinct seed-first owner. Its documented legacy
compatibility is not TS pure-helper order parity. This result-carrier slice must
not silently alter that accepted boundary. Retain independent source/seed
failure controls, document any inherited TS/native difference explicitly and
verify coercion occurs after the complete Fold. A result fix does not close
that separate semantics obligation.

The held aggregate-return candidate changes stateful whole-body return context,
while this gap is the pure expected-result ingress before it. Existing ADR0330
owns fallible Vector conversion rendering, source-once evaluation, first failure
and bounded output size; reuse those owners and historical tests unchanged.
Any transparent Vector alias cast-dispatch repair must be scoped and tested at
this same ingress; it must not broaden unrelated casts.

## Execution plan

1. Preserve the original expanded source characterization and all failed
   attempts. Cover aliases, empty/nonempty/nested/product/wide/Field results,
   original accumulator guards and accepted local/conditional/scalar controls.
2. Lower the original complete named Fold first; apply existing Coerce only when
   its supplied Vector result differs from the declared Vector. Preserve already
   supplied conditional coercions and all scalar ingress behavior.
3. Build the default original native compiler at a clean signed HEAD, then run
   independent current TS and unedited Rust source/API/strict-root/refusal gates.
4. Inspect actual applicable model costs, verify material before keys, and run
   selected genuine proofs and separate default-strict funded applications with
   identity-bound negatives. Never trim full source or Programs to fit a bound.
5. Obtain authentic independent CLI source and receipt reviews, then update the
   existing KB and checkpoint with literal epochs and open integration gates.

Evidence root:
`/Users/ysh/.codex/local-evidence/rust-ast-fold-vector-result-widening-20261102`.
Implementation, current compiler/API/proof qualification and complete required CI
are not yet accepted. The broader goal remains active and incomplete.


## Implementation and first current-source checks

Compiler0.31.164 at `770a7c56f` builds with the default original native recipe.
The complete20-source/60-command characterization passes:17TS-valid sources
emit Rust and independent equal IR, and three frontend refusals remain. The
local, conditional, scalar and original order-control sources retain all16
original IR/raw-origin/Rust/capability assets byte-for-byte. Vector alias target
normalization also resolves the reducer's previously invalid Vector literal
carrier; unrelated aliases retain their owner.

A development corpus now contains17complete positive sources, including the
unchanged accepted controls, plus the existing narrow-accumulator guard family
through a second stateful helper export. Thirty-two independent TS/Rust cases
pass:27pure calls and20stateful route executions. Both-operand-failure controls
retain the explicit original144 native/TS difference, bound to the complete
original control source and actual six-case observations. They are not labeled
parity or used to bypass source/IR/Program identity checks. Current fixed-source
acceptance, strict SDK roots, ordinary/untrusted refusals, independent review and
applicable real proofs remain pending.

The maintained gate is `tools/compact-rust-backend/check_fold_vector_result_widening.py`.
It requires a clean original compiler receipt at its exact HEAD, complete source
IR/Program goldens, an explicit TS runtime and pinned Rust tools with inherited
compiler/wrapper substitutions refused. Consumers and strict root SDK copies
are external to the unedited generated bundle. Changed outputs/producers must
fail actual consumers; the zero-length reducer retains static signature checks.
