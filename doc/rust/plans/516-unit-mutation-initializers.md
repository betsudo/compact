# Proposed next slice: ordered Unit mutation initializers (#516)

Status: independent local development from canonical `acf529eff`. The original
compiler137 realization and all six maintained frontend return regressions pass
at that clean baseline; the genuine TS/Rust initializer comparison reproduces
there. PR #693 has exact-head review acceptance but full CI remains pending.
Keep this branch local until that delivery settles; stop dependent publication
on a real baseline failure. Two active engineering branches is the hard limit.
Reconcile the actual canonical merge before this slice is published.

## Demonstrated gap and accepted neighbor

The complete minimal source below is accepted by TypeScript and refused by Rust
at `Set.insert` using the original compiler137 at canonical `acf529eff`:

```compact
import CompactStandardLibrary;
export ledger values: Set<Field>;
export circuit insert(selected: Boolean, value: Field): Field {
  const inserted = disclose(selected) ? values.insert(disclose(value)) : [];
  return inserted == [] ? 1 : 0;
}
```

A neighboring initializer `const inserted = insertIf(...)` with a local
mutation-bearing circuit already compiles for both targets. Do not reopen that
accepted case, or the scalar/pair terminal-return slices (#691/#692). Source
emission does not establish execution, recording or proof behavior.

## Owners and intended public API

- `compiler/rust-ir-passes.ss`: typed lexical binding/action decomposition at
  `stateful-bindings-ir`, `let-scope-needs-action?` and existing terminal/action
  lowering. Lift supported Unit initializer effects in exact source order using
  existing StateAction/Let/Sequence/If forms and ledger metadata.
- Preserve original occurrence/provenance, previous-scope initializer lookup,
  fresh shadowing scopes, exhaustive branches and current resource refusals.
- Existing IR20, checked effect lowering/validation, native/recorded/functional
  Rust AST owners retain their authority. No production body recognizer or
  source-name exception; no new IR variant or runtime ABI.
- Public APIs remain ordinary typed circuit methods and consuming functional
  `Program::new(selected: bool, value: Field)`. The example still returns Field
  and uses existing witnesses, slots, ContractLab and preparation APIs. Users
  should not need to turn a valid initializer into a helper circuit manually.
- Maintained frontend/consumer/proof gate owners and delivery/ownership docs
  record genuinely new acceptance, preserving all historical authorities.

## Bounded scope and exclusions

Start with conditional Unit-valued existing Set insertion initialization and
lexical uses, shadowing, sequencing and selected/unselected branches. Add only
operation-family controls required to establish metadata-directed lowering;
characterize each before expanding. Do not promise every mutating initializer.
Exclude non-Unit result initializers, arbitrary aggregates/tuple arity expansion,
general casts/narrowing, Cell lexical redesign, new runtime/public builder,
recording retirement reopening, new targets, proof caching, distribution work,
CI optimization and Forge. Correct stale frontend-return README descriptions of
already merged scalar/pair wrappers without requalifying them as new features.

## Acceptance

1. Preserve the clean canonical `acf529eff` baseline and its six passing frontend
   regressions before feature changes. Reproduce TS acceptance/Rust refusal with
   full source and original diagnostics; preserve all failed artifacts. Reconcile
   actual canonical after #693 before publication and renew affected checks.
2. A minimal owner fix emits complete unedited Rust/TS, original IR/origins,
   finalized capabilities and full functional programs. Existing source census,
   local-call initializer, scalar and pair wrappers remain accepted; no repinning.
3. Independent expected outcomes and live TS compare native, recorded, functional
   native/recorded from identical prestates. Selected/unselected conditions,
   insertion/replacement, lexical use/shadowing, multiple initializer ordering,
   witness/early-error order and checked failure controls must agree in canonical
   result/state/effects/private outputs and query cost. Replay successful records.
   Do not invent rollback of inaccessible failed contexts or external witnesses.
4. Wrong ordinary arguments/results must fail with intended Rust diagnostics;
   consumed Program reuse E0382. Pure mutation and missing disclosure retain
   genuine both-target source refusals. Invalid/inactive-arm checked IR, scope,
   conversion, layout and resource controls remain effective. Strict complete
   generated-consumer Clippy and changed-oracle refusal pass.
5. Where proof-applicable, fresh direct selected keys and genuine proofs verify
   with immutable verifier identity; binding and applicable same-geometry result
   negatives refuse. Separately funded default-strict transaction application
   reaches exact expected state and rejects replay without mutation. Keep
   standalone-proof and independently proved transaction artifacts distinct.
6. Focused local arm64 iteration, then full existing Linux/GitHub/native/proof/
   package/wallet/E2E gates and exact-head Claude review (Codex fallback only on
   actual Claude limit). Signed Conventional/DCO/AI commits, normal PR push and
   reviewed merge into canonical. Knowledge base and main checkpoint use
   concurrent-edit-safe process/CAS. Cleanup own completed unused targets and
   worktree only after verified GitHub merge; retain evidence and shared inputs.

## First prototype and concrete rendering follow-up

The first owner prototype emits the original initializer as an ordered storage
conditional followed by a typed Unit binding. The minimal source now has full
IR, recorded/observed availability and one checked Program. Eight Rust route
executions and four recorded replays pass independent result/Set expectations.
This is a development prototype, not TS execution or proof acceptance.

Strict generated-consumer Clippy exposes Unit temporary/equality syntax in this
newly admitted source. The ordinary expression owner must annotate the Unit
temporary; the existing neutral arithmetic fact and AST owners must retain both
checked/evaluated Unit operands in source order and produce the known equality
result without a Rust Unit comparison. Unit/Boolean mismatch and two-child
visitation controls remain required. Preserve other arithmetic rendering and
historical oracle authorities; this is not general constant folding or new
aggregate/carrier admission. Add actual source-level pure guard comparisons to
check left/right failure order, plus live TS/four-route initializer controls.

## Development E2E and separate selected proof — 2026-10-27

At the uncommitted canonical-based development epoch, the maintained source/API
consumer gate passes **58 live TypeScript cases / 196 Rust executions / 62
recording replays**, with three genuine compile-fail controls (argument and
result E0308, consumed Program E0382), both-target pure/disclosure refusals,
changed-TS-output rejection, exact recorded journal/replay-cost agreement and
strict generated/consumer Clippy. Witness reads observe the actual updated Set;
all successful output FAB/state/effects/private state/transcripts and summed
query costs agree. Pure Unit comparisons retain left/right assertion failures;
Unit witness comparisons retain both callbacks, empty aligned private outputs
and first failure. Initializer binding use, unused effects, nesting, shadowing,
sequencing and insertion/reinsertion have independent expected controls.

The seven maintained frontend regressions pass with the same development Rust
CLI and explicit original137/candidate Scheme frontends. Five focused arithmetic
and nine existing inequality controls pass. The full backend library passes
1,846 methods with seven existing ignores. The earlier one-test inequality
syntax expectation and test-harness label parsing failures remain retained.

The separate direct proof gate freshly generates `conditional` keys at **k=7,
76 rows** and two 2,912-byte standalone proofs. Immutable-verifier checks reject
changed binding and changed Field result at unchanged geometry. Independently
proved funded transactions (6,589 / 6,591 bytes at this development epoch) pass
default-strict verification/application, exact poststate and `IntentAlreadyExists`
replay rejection without mutation. Complete original IR and all nine checked
programs are bound to the linked fixture; changed IR/programs refuse before keys
or state. Transaction files are written only after all application/replay checks
pass. The constructor, seeded prestate, unselected branch and other exports are
not proved. The two Unit-witness-only stateful exports are nonproof according
to actual frontend metadata; recording availability does not change that.

An initial proof attempt executed cryptographic/application checks but failed
its final frozen-source guard because the gate itself was edited during that
run. It remains failed evidence. The successor froze all 2,869 source premises,
verified material and tool identities, generated new proofs/transactions and
passed the complete gate. No proof or verdict was reused.

These are development results, not clean final compiler or integration
acceptance. The canonical reconciliation after #693, final original compiler,
complete fresh fixture/source census, required CI, exact-head independent
review and signed reviewed merge remain open. Historical accepted work and
recording-retirement authorities remain unchanged.
