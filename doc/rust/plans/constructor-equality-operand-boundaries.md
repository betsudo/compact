<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Constructor equality operand boundaries — #720

## Baseline and gap

This independent local slice starts at the actual canonical merge of #719,
`264a5a40476cf5ccb044af87700c939efd4e9a92`. Its tree is byte-identical to the
reviewed publication head `97bbb44a8`. A merge is not a hosted proof receipt.
The older local canonical ref and all unpublished scalar-triple branches,
qualified binaries, failures, targets and receipts remain preserved.

#719 delivered typed equality guards inside supported constructor Vector loops.
Its existing cohort uses same-width scalar operands. The admission relation also
visits aggregates and coercions; that syntax alone does not establish genuine
source acceptance, checked typing, generated ownership or observable behavior
for mixed-width Uint, tuple, Vector and struct operands. #720 owns this concrete
coverage gap and the narrower CHANGELOG wording. It does not reopen #674, the
recording algebra, existing ownership normalization or the delivered equality
feature. #707 and #706 retain their separate CI and coverage-policy ownership.

The scalar-triple proof cohort is independently refused before case execution:
three intact checked exports automatically select k9, above its unchanged cap8.
That frontier stays open on `codex/rust-ast-scalar-triple-model-closeout`, with
execution frozen at `15cffbdc1`. This slice neither depends on that experiment
nor replaces its outstanding full proof cohort with easier cases. Forge stays
deferred on `experiment/zkir-forge`.

## Owners and public API

- Scheme frontend and `compiler/rust-ir-passes.ss`: characterize actual source
  typing/coercion and retained raw IR without changing accepted Compact rules.
- `tools/compact-rust-backend/src/lib.rs` constructor-condition admission and
  `constructor_vm` Vector-loop binding: retain their existing responsibilities.
- Shared pure equality checking/rendering: preserve type/comparability and
  left-before-right refusal rules; no new equality engine or generic framework.
- Existing constructor equality source/API, model and proof qualifiers: extend
  the finite cohort only after source characterization establishes applicability.
- Existing compiler195 CHANGELOG entry and this plan: narrow published scope
  and distinguish local execution, merge and hosted qualification.

The public API remains generated `initial_state(ConstructorContext, ...)` with
existing typed Uint, tuple, FixedVector and struct arguments. No runtime ABI,
IR schema, new public methods, traits or compiler options are planned. A real
backend defect, if found, receives a bounded owner fix and a red regression;
valid frontend refusal is recorded rather than widened to make a test pass.

## Work sequence

1. Run focused constructor unit/render tests and existing equality qualifier
   controls at a clean source baseline. Close a concrete baseline regression
   before changing implementation or adding dependent features.
2. Characterize complete genuine Compact sources for mixed Uint<8>/Uint<16>,
   tuple, Vector and struct operands with `==` and `!=` in constructor Vector
   loops. Record accepted or intended typed-refused outcomes. Inspect raw IR
   to distinguish original input widths from frontend-inserted coercions.
3. For each supported source, retain the complete current ordinary/functional
   SDK, compiler/recipe/runtime/tool identities and all generated interfaces.
   Never edit generated Rust or normalize source/proof binding.
4. Compare actual `initial_state` with an independent live TypeScript oracle.
   Exercise true/false outcomes with first/last loop elements distinguishing
   final-iteration behavior. Include an owned Bytes-containing composite and
   reuse caller inputs after constructor invocation where the API permits it.
5. Add marked wrong-width/type/shape ordinary Rust compile failures with valid
   twins, checking the intended call and primary diagnostic. Preserve existing
   witness/helper/ledger/arithmetic operand refusals, invalid inactive arms,
   representation/resource limits, error order and top-level `if` exclusion.
6. Where changed behavior needs proof coverage, preflight complete selected
   contracts under the existing automatic tiny policy before keys. Use fresh
   standalone proofs and independent immutable verification, identity-bound
   negatives, and separate funded DEFAULT-strict applications/replay refusal.
   Later-entrypoint proofs do not prove constructor initialization/deployment.
7. Obtain fresh Claude CLI source and actual-result review (Codex CLI fallback
   only on actual Claude quota failure), sign coherent local commits, and update
   the existing knowledge base and CAS checkpoint. Preserve all failed attempts.

## Acceptance and exclusions

Acceptance is an explicit supported/refused matrix for all four operand shapes,
actual complete generated-API behavior and ownership checks, relevant marked
compile failures and original refusal boundaries. Applicable genuine proof and
strict application evidence must retain exact source/tool/material identities;
model classification is not proof acceptance. Focused local checks guide
iteration; complete required CI and proof gates remain integration requirements.

No broad refactor, new constructor language, arithmetic admission, general
composite comparison expansion, CI optimization, workflow activation, push,
merge, release, shared installation or Forge work. Preserve historical tests
and accepted capability/retirement decisions. Clean only owned inactive unneeded
reproducible intermediates after process/open-file/lock guards; keep current
executables, dependencies, results, failures, parameters and keys.

Status: plan only. No current baseline checks, source characterization, new
SDK execution or proofs have run for this slice. Stop initiating new slices at
2026-11-08 09:05 UTC and leave an exact handoff; the broad goal stays incomplete.

## Baseline and genuine source characterization

At clean plan head `69f18db43`, the focused baseline passes 64 constructor unit
and 14 renderer tests, plus 30 existing Python controls. The original compiler
0.31.195 realizes from that same source and reports its expected version.
These are focused baseline checks, not whole-CI or generated-API acceptance.

Eight complete three-export sources characterize four shapes with both
operators. All eight independently generate TypeScript. Tuple, Vector and
Bytes-containing struct sources also generate Rust. Both mixed-width sources
fail Rust generation at constructor `if`; no generated SDK has yet been compiled
or executed for this matrix.

The original Scheme producer independently emits both mixed-width IRs. Input
and loop element maxima remain 255, and the right parameter maximum is 65535.
The comparison's left child is `unsigned_cast` to 65535. The constructor's
infallibility predicate excludes all such general casts before typed equality.
The frontend accepted an implicit, guaranteed monotone conversion, so this is a
transport gap with a reusable checked owner, rather than an invalid Compact
comparison. The initial raw-producer caller incorrectly combined legacy wrapper
flags; that failure is preserved. Corrected calls use the exact original Scheme
component and the unchanged sources.

## Bounded implementation decision

Use an emitter-local scoped constructor-condition context in
`compiler/rust-ir-passes.ss`. Inside that context, emit the existing `coerce`
node for a nonliteral safe unsigned conversion only when both original bounds
are known and the source bound is at most the target bound. Existing literal
inference stays with its current owner. Keep source-origin callbacks on the
original expression producer; never post-process generated IR or SDK output.

The existing Rust coercion relation independently checks source/target bounds,
constructs checked facts and renders the conversion. Reuse it without adding a
constructor whitelist entry for arbitrary `UnsignedCast`, a second type checker,
a new public API or a replacement AST path. Parameter scope restores after each
condition, so initializer and ordinary circuit transports retain their owners.
Require source controls for that scope and malformed/narrowing private-IR
coercions, as well as both real mixed-width operators and all original refusals.

Compiler version declarations will use the next unused number in the current
0.31 series. Actual local branch inventory reserves through 0.31.208 in separate
unpublished work, so the proposed new source identifier is 0.31.209. The unrelated
`chore/version-scheme` experiment is not this branch's baseline or numbering
policy. No prior receipt is relabeled and no release is authorized.

Status: the semantic fix, current native/TS comparisons, marked misuse checks
and applicable proofs remain unimplemented/unexecuted at this planning epoch.
