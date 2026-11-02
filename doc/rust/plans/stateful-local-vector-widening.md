# Typed Vector widening in stateful local bindings

## Concrete gap and accepted boundaries

Current compiler 172 at T execution `eaefb375285b2f7b2185ac63111887bd45a75e41`
accepts a narrow Vector parameter widened directly at a stateful return. It
also accepts the pure local-binding control. A declared widened local in a
stateful body instead emits narrow initializer IR against its wider declared
binding type, and the independent native checker correctly rejects it.

T evidence `next-stateful-local-vector-characterization-v1/receipt.json`
retains 12 complete sources / 36 commands. Eight valid stateful forms pass TS and
independent IR but fail Rust with a concrete narrow/wide Vector type mismatch:
local binding before or after a Cell write, Uint-to-Field, pure helper source,
transparent aliases, nested and empty Vectors, and a checked helper retaining
its original narrowing/assertion. Both pure-local and direct-stateful-return
controls pass all three lanes. Implicit local narrowing remains a frontend
refusal. The initial candidate proof constructor omitted required `disclose`
and was correctly refused; that source and diagnostic remain retained.
`next-stateful-local-vector-proof-source-v2/receipt.json` retains the corrected
complete two-export helper/branch source and three commands: TS/IR pass, Rust
fails at the same local initializer boundary. No model/proof feasibility or
implementation acceptance follows from this characterization.

This is separate from the accepted return-value T slice. Pure coercion, typed
native rejection, S literal returns, checked recording, Fold/Map and prior alias
work keep their recorded acceptance. Other probes found stateful products still
missing, but arbitrary Tuple/Struct/product conversion is excluded here.

## Owners and intended API

Reconcile `compiler/rust-ir-passes.ss` `stateful-bindings-ir`, its supplied local
type and `stateful-typed-expression-ir` safe-cast/context handling. Follow the
actual frontend cast and binding value before selecting a fix. Reuse the
existing checked Coerce and Vector carrier owners; preserve original initializer
source bounds before the declared local target conversion. Initializers use
only the preceding lexical scope, and preceding/following effects stay in place.
Native and recording validators remain independent and continue rejecting
malformed/mismatched annotations. No validator bypass or duplicated renderer.

Public `run` APIs keep the original
`FixedVector<BoundedUint<255>,2>` input and declared
`FixedVector<BoundedUint<65535>,2>` or `FixedVector<Field,2>` result. Ordinary,
recorded and consuming functional Program/context ownership remains ergonomic
and unchanged. This is source-semantic support, with no new caller flag or helper.

## Bounded acceptance

1. Admit a separate clean branch from the reviewed T documentation head only
   after inspecting checkpoint/refs/worktrees. Run focused current baseline and
   reconcile roadmap/crosswalk/checked-recording/capability retirement owners.
2. Fix the eight complete local-binding forms at the existing owner. Retain the
   declared local target and original source/intermediate casts, lexical scope,
   action/helper/witness ordering and first refusal. Preserve accepted controls
   byte-for-byte where their route is unaffected. Characterize both conditional
   helper branches and any multiple-binding/witness extensions before admission.
3. Execute independent TS and whole unedited generated Rust ordinary/recorded/
   functional SDKs. Check actual source/output geometry, state/effects/private
   transcript and callback order, Rust-route gas and independent public replay.
   Verify before/after-write binding positions and checked-helper first errors.
4. Include correct-type twins and marked input/result element/bound/length and
   Program/context reuse compile refusals; retain source narrowing/kind/length/
   nominal refusals and malformed binding/Coerce/dead-branch native checks.
   Real producer operand/order mutations and wrong result oracles must fail
   actual consumers. Preserve historical compatibility tests independently.
5. Bind exact original compiler/default recipe, complete source/IR/Programs/SDK,
   tools/config/serialization/runtime and physical generated/golden includes.
   Inventory all exports/models/applicability before selecting proof lanes.
   Verify locked material before fresh keys, then genuine selected Uint/Field
   proofs and separate funded default-strict applications with immutable wrong
   verifier/corruption/binding/same-geometry result/replay controls. Cover both
   selected helper branches. Keep every over-cap source intact under k<=8.
6. Use focused local arm64 checks, signed Conventional DCO/Assisted-by commits,
   actual Claude-first source and receipt review, docs and verified Obsidian/CAS.
   Full required CI/proof/integration gates remain required for integration.

## Exclusions and limits

No arbitrary aggregate/product widening, nominal redesign, narrowing relaxation,
Fold reducer expansion, effect framework, broad refactor or CI optimization.
No IR/runtime ABI, resource cap, flag, dependency or ownership redesign. No
source extraction, generated-code editing or proof/verdict caching. Constructor,
all-input and unselected proof coverage remains explicit. No Forge work,
publication or cleanup is authorized by this local proposal. The broad Rust AST
goal remains incomplete; no implementation has started on this next slice.


## Admission

U is admitted separately on `codex/rust-ast-stateful-local-vector-widening`
from reviewed T documentation `573ce6d2edfea6e59e7f7d816e2f97c7ba148a7e`.
The exact current checkpoint, refs/worktrees, clean canonical and clean T
predecessor were inspected first. All predecessor branches and evidence remain
untouched. Focused clean baseline and actual binding/cast owner tracing are next;
no U implementation, current compiler, API, model or proof acceptance is claimed.


## Baseline and owner trace

Clean signed admission `6359c1f52` passes all 35 focused coercion, Vector, Fold,
projection, historical fixture, native Vector-child and return-order tests.
Original default172 Nix recipe passes at that head. Expanded original evidence
retains 28 complete sources / 84 commands: 21 pass TS/IR, 17 fail Rust at the
missing local conversion, and four controls already pass. All seven frontend
refusals remain. Two additional complete scalar/Unit result sources pass TS/IR
and fail Rust at the same local boundary; the conversion belongs to the binding,
not just a Vector-valued return.

Three complete original frontend traces retain the actual supplied safe-casts,
local target/source types and witness/binding order. `stateful-bindings-ir`
already passes the declared local type to `stateful-typed-expression-ir`; its
Vector safe-cast fallback loses the source/target conversion because stateful
roots do not use the pure variable-type table. The legacy action/return split
can also repeat an effectful initializer. Therefore a cast-only patch is
insufficient.

Classify supplied nonidentity monotone Vector casts using the existing T type
rules. At typed stateful casts, lower the original value at the supplied source
type, then emit the existing Coerce to the supplied target. Bodies containing
such local initializers select the existing ordered return owner so each binding
and its surrounding actions execute once in source order. Identity casts and
unrelated expressions keep their owners. This is routing and source preservation,
not recording admission; native/recording validators, resource rules, AST owners
and ABI remain unchanged. Compiler173 identifies this implementation. Current
compiler/source/API/proof/review qualification is pending.
