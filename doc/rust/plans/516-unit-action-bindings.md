# Unit mutation initializers in statement actions (#516)

Status: local development on a single child of reviewed PR694 head `c0eb15915`.
Accepted canonical remains `373fe452c`; the parent has clean local qualification,
accepted exact-head review, green Rust CI and original Apple Silicon consumers.
Its complete Linux integration CI is still open. Do not publish this child before
verified parent integration and actual canonical reconciliation. Stop dependent
work before commit if parent CI fails, its head changes or evidence is unknown.
Keep at most two active engineering branches and preserve all unique work.

## Demonstrated gap and owner

The original compiler0.31.139 admits these sources for TS and refuses Rust at
Set.insert before output: a Unit-valued initializer inside a statement-only block
in a Field-returning circuit; the corresponding Unit-returning block; and a
Unit-returning circuit with a later conditional or unconditional mutation.
Original compiler pass traces show non-tail StateAction let* bindings inside a
sequence. Reading those normalized inputs with the source identifies the common
binding boundary; this is not an instrumented runtime call stack.

`compiler/rust-ir-passes.ss` owns the repair. The existing ordered return-plan
Unit initializer walk already sequences a supported storage action before a
scoped Unit binding. Share that exact lexical/provenance walk with StateAction
let*, using the existing let and sequence constructors. Preserve previous-scope
initializer lookup, fresh shadowing scopes, source origins, first failure and
once-only evaluation. Current return-plan support stays accepted.

## Public API and exclusions

Users keep ordinary typed circuit methods with Boolean/Field arguments and
Field or Unit results, and consuming functional `Program::new(...)` APIs. No
helper circuit, new builder, runtime type, IR enum, ABI change or source-name
exception is introduced. Current typed lowering and independent checked-plan
validation remain mandatory; unsupported operations and resource limits retain
explicit refusals. Historical migration/retirement reference authorities remain
unchanged.

The slice covers Unit-valued existing mutation initializers in statement scopes,
including unused bindings, selected/unselected branches, nesting, shadowing,
sequential mutations and witness/error order. Characterize neighboring operation
families before claiming them. Exclude non-Unit mutation results, arbitrary
aggregate conversion, Cell lexical redesign, general effect frameworks, target
expansion, CI optimization and Forge. Proofs remain selected call coverage;
constructors and prestate setup are not proved by those calls.

## Acceptance

1. Preserve the reviewed/local-qualified parent and its original source refusal
   evidence. The child is not an accepted canonical baseline. Reconcile the
   actual verified canonical merge before final versioning and qualification.
2. Generate complete unedited Rust/TS, original IR/origins/capabilities and all
   checked Programs for a source cohort. Preserve all old accepted registered
   sources and authority rows; any changed output requires concrete explanation,
   never normalization into a legacy oracle or blanket repinning.
3. Compare independent expected outcomes and live TS with native, recorded and
   both functional routes from identical prestates. Cover Field/Unit returns,
   selected/unselected paths, insertion/reinsertion, unused effects, shadowing,
   sequencing, witness reads of updated state and early failures. Compare complete
   state/effects/private outputs/transcripts and summed query costs; replay records.
   No failed-context or external-witness rollback claim.
4. Ordinary argument/result mismatch must reject with E0308; consumed Program
   reuse with E0382. Pure/disclosure, malformed/inactive-arm IR, scope, conversion
   and resource controls remain effective. Strict complete generated/consumer
   Clippy passes. Promote actually admitted old boundary sources into honest
   source admission controls and retain their historical refusal receipts.
5. Fresh selected applicable keys/proofs verify under immutable verifier identity
   with binding and applicable same-geometry input/result negatives. Independently
   proved funded default-strict transactions reach exact state and reject replay
   without mutation. Retain standalone and applied-transaction files separately;
   state unproved constructor/prestate/unselected/all-export scope explicitly.
6. Focused arm64 local checks, then a separate logical PR, exact-head Claude CLI
   review (Codex fallback only on actual limit), complete required Linux/Apple
   Silicon/compiler/native/proof/package/archive/wallet/E2E CI, signed DCO/AI
   delivery and verified canonical merge. Record literal epochs in the midnight
   knowledge base and CAS checkpoint. Clean only own completed unused worktrees
   and targets after verified merge, preserving unpublished/ignored artifacts.


## Development results — 28 October 2026

The shared frontend walk now admits the four source shapes refused by the
original compiler 0.31.139. A separate prototype passes 80 Rust executions and
40 recording replays, then strict consumer Clippy. Initial consumer accessor
and Unit-discard failures remain retained; generated code was not edited.

The maintained cohort appends six exports and 37 independently specified cases.
Its development API gate passes **95 live TS cases, 344 Rust executions and 126
recording replays**, including full state/effects/private outputs, witness reads
of updated state, summed query costs and first-failure ordering. Three genuine
compile-fail controls, pure/disclosure refusals, changed TS output rejection and
strict complete generated/consumer Clippy pass. The clean-source requirement in
the maintained gate remains intact; an external development harness explicitly
records dirty source and the packaged 139 CLI with the child Scheme override.
This is not clean original compiler or integration acceptance.

All previous 58 case rows and nine checked Programs are unchanged. Appending the
six exports changes three allocated temporary names and their matching lexical
references in two old circuit IR bodies; no other old stateful circuit IR field
changes. The exact six differences are retained and actual complete generated
output is copied without normalization. The explicit fixture now binds all
fifteen Programs. Only six new declaration rows extend the additive registry;
all 482 prior rows and the separate historical authority remain unchanged.

The source self-reference attempt in an inner shadowing initializer was rejected
by the Compact frontend before Rust generation. The maintained positive uses a
valid shadowed binding and nested lexical use; it does not claim self-reference
admission. Earlier generation and test failures are retained.

The proof gate is being extended to retain the existing two return-form calls
and add two statement-form calls, each with fresh keys, standalone verification,
negative controls and independently proved default-strict transactions. Its first
build caught the test resolver's required static circuit-name lifetime; that
failed attempt is retained. No new proof success is claimed yet. Parent #694
remains unchanged, with its full Linux gate still pending.


The separate development proof gate v2 now passes all terminal guards. It binds
the complete IR and all fifteen Programs, freshly generates keys for both
`conditional` and `statement_field` (each k=7, 76 rows), and verifies four distinct
2,912-byte standalone proofs. Changed public binding and same-geometry result
commitments are rejected. Independently proved funded transactions are
6,589/6,591 bytes for return-form reinsertion/insertion and 6,593/6,595 bytes for
the statement form at this epoch. Default-strict application reaches exact
state; replay rejects with `IntentAlreadyExists` without mutation. All 2,899
source premises, material, complete generated assets and copied tool identities
remain stable. This development result does not relabel the required later
clean original compiler, review or CI epoch.


After the proof run, the ordinary invalid-usage controls were extended to include
the newly generated statement APIs while retaining the prior return-form calls.
A fresh development API run again passes 95/344/126, and rejects both old and new
wrong-argument/result uses only with E0308 and both consumed Program reuses only
with E0382. The public Python runner discovers and runs 369 methods with zero
failures/errors and its 20 expected genuine-artifact skip events; these skips
are explicit, not fresh proof execution. The earlier 60 focused protocol,
fixture and declaration controls also pass. Full required CI is retained.
