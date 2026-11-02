# Next proposal: stateful Vector value widening

## Status and baseline

T is admitted on `codex/rust-ast-stateful-vector-value-widening` from reviewed S docs `cf02d5693b1680823a25cb719c23c836db1ce5bd`, after inspected current refs/worktrees and clean canonical/predecessor states. Implementation and current qualification remain pending. Prior reviewed local S execution
`7ab60b62061a728b3e81f34ae31e9693b1063d41` (compiler170). Canonical
`codex/rust-backend-ast` remains clean at1bdf185ea. The explicit local candidate
stack is not integrated canonical. Inspect the exact checkpoint/refs/worktrees
and admit a clean successor separately before implementation. Preserve every
predecessor and all evidence; no Forge, publication or cleanup work.

## Concrete missing capability

`next-stateful-vector-value-characterization-v1/receipt.json` under S evidence
retains eight complete sources and24current default170 commands. Every source
passes TS. Five stateful sources fail both Rust and independent IR on a nested
`__compact_Cell.write`:

- A narrow Vector-returning pure helper widened to Uint16 or Field after a write.
- A narrow Vector parameter widened to Uint16 after a write.
- Transparent input/output Vector aliases in the same stateful flow.
- A Struct field holding the narrow Vector widened after a write.

The identity-helper, pure-parameter widening and direct-literal controls pass in
all three lanes. This is an actual source ingress gap. S's mixed helper branch
already works because its sibling literal selects one ordered return plan. A
helper-only or parameter-only root has no literal to select that owner.
Accepted literal, pure alias/member, Map/Fold and projection work is not reopened.

## Owners and intended public API

Reconcile `compiler/rust-ir-passes.ss` aggregate cast classification, storage
mutation/return selection and `stateful-return-plan-ir-value`. Reuse S's ordered
source-value conversion, typed contextual expression owner and existing native
Vector coercion. Preserve the frontend's actual source type and intermediate
checks before converting the reached value. Native and recording validators
remain independent, with no source-shape admission shortcut.

Keep the original complete exports and helper calls. Ordinary/recorded APIs
accept their existing `FixedVector<BoundedUint<255>,2>` or declared generated
Struct input and return `FixedVector<BoundedUint<65535>,2>` or
`FixedVector<Field,2>`. Functional Programs retain consuming context/program
ownership. No new flag, dependency, IR/runtime ABI or public helper API is planned.

## Scope exclusions

No source extraction/substitution, generated-code editing, arbitrary aggregate
conversion, narrowing relaxation, dynamic index, nominal redesign, effect lifting
framework, backend refactor, CI optimization or new primitive. Existing checked
bound/layout/resource/private/profile rules and all full integration gates stay.
Automatic proof k<=8 stays; preserve full sources and record every over-cap model.

## Acceptance criteria

1. Preserve refs/worktrees and reconcile current roadmap/crosswalk/checked-recording
   and retirement owners. Admit a separate clean baseline; focused existing
   source/API/coercion/return checks must pass before implementation.
2. Fix all five complete source forms at their existing owner. Characterize local,
   conditional, guarded and alias tails before extending beyond the observed
   Vector-to-Vector conversions. Preserve original Cell writes, helper calls,
   supplied bounds, source order and single condition evaluation. Keep complete
   identity/pure/literal controls unchanged where their route is unaffected.
3. Execute actual TS and unedited generated ordinary/recorded/functional APIs at
   original narrow input and widened result types. Check alignment, state/effects,
   private outputs, witness/call order, query gas and recorded replay. Include
   observable first-error controls and both meaningful state-dependent branches.
4. Correct-type twins accompany marked compile refusals for wrong input/result
   element/bound/length and reused Program/context. Keep frontend narrowing,
   length/kind/nominal refusals and independent malformed-IR/dead-branch checks.
   Changed real producer and result-oracle data must fail actual consumers.
5. Bind full source/import/IR/Program/SDK/compiler/default-recipe/tool/config/
   runtime/serialization identities and strict unedited SDK roots. Inventory all
   exports/models with explicit nonproof applicability before selecting proofs.
   Verify locked material before fresh keys. Require selected genuine Uint/Field
   proof/verification plus separately proved funded default-strict application,
   immutable wrong verifier, corrupted/binding/same-geometry typed-result controls
   and replay refusal. Cover both branches where the selected source has them.
   Do not trim/raise caps or imply constructor/all-input/unselected proofs.
6. Use focused arm64 checks during iteration, signed Conventional DCO/Assisted-by
   commits, actual Claude-first source/receipt reviews (actual-quota Codex fallback),
   documentation and verified Obsidian process/CAS. Retain complete required Linux
   CI/proof/integration gates. Bounded local acceptance cannot finish the broad goal.

No implementation, API execution, model feasibility or proof acceptance follows
from the source-only characterization above.
