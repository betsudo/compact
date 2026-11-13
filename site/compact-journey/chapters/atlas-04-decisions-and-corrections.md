## Reference scope

This reference describes the Ledger 8 typed target and checked-effect pilot. It is an architectural reference, not a status report for the later Ledger 9 implementation. The [compositional recording chapter](operations-instead-of-shapes.html) explains the pilot's subsequent integration; the [migration chapter](ledger9-is-a-protocol-migration.html) explains the protocol changes.

The following decisions connect observed limitations to architectural changes.
Each entry separates the problem, chosen approach, evidence, benefit and limit.

## 1. Preserve the old backend instead of deleting it

**Problem.** A clean rewrite could regress behavior while making comparison
impossible.

**Decision.** Keep `codegen-rust` as an oracle and compare exact sources,
results, state, transcripts and refusals.

**Evidence.** The native Rust qualification ran all 37 exact top-level oracle
sources through the new path.

**Benefit.** The project could change the architecture without rewriting its
own history.

**Limit.** An oracle can contain defects; agreement is evidence, not truth.

## 2. Use a closed IR, not a Rust-source escape hatch

**Problem.** Text fragments let unsupported source shapes leak into output and
made exhaustive review difficult.

**Decision.** Model Compact types, expressions, actions and returns as strict
serde variants and reject unknown input.

**Evidence.** Compiler-backed refusal probes, fixture regeneration and external
Cargo consumers accompanied schema changes.

**Benefit.** New constructs require an explicit compiler/backend decision.

**Limit.** The private schema still couples the Chez producer and Rust consumer;
its version is not a public language specification.

## 3. Generated syntax is necessary, not sufficient

**Problem.** `syn` guarantees parseable Rust construction, but semantic bugs
can still live inside valid syntax.

**Decision.** Keep AST construction and add execution, independent expected
values, TypeScript observations, negative controls, replay and proof gates.

**Evidence.** Field modular arithmetic, Boolean witness order, source-defined
returns and output-recovery tests caught stale or overly textual assumptions.

**Benefit.** Generated code is checked as a program, not a string.

**Limit.** Hundreds of substring assertions remain useful smoke tests but are
not semantic proof.

## 4. The point type was too “safe”

**Problem.** Rust represented Compact `JubjubPoint` as a validated subgroup
point. Compact's source value is a coordinate pair, and the upstream constructor
could panic on some invalid/subgroup-external coordinates.

**Decision.** Separate source representation from checked operation admission;
validate only where an operation requires a real point.

**Evidence.** Cross-language parity tests reproduced the divergent values and
panic boundary.

**Benefit.** Rust no longer changes source representation under the banner of
type safety.

**Limit.** A checked backend may still deliberately refuse behavior that the
reference path accepts; that difference must be documented, not hidden.

## 5. Shape recognizers reached their ceiling

**Problem.** Every useful application family added another whole-program
admission profile. Supported operations did not compose automatically.

**Decision.** Introduce a typed `RecordedPlan`, migrate one family at a time,
and retain the old paths as differential oracles until retirement evidence
exists.

**Evidence.** Per-family default/feature tests, generated-source inventories,
source-bound proof/application cases and an explicit retirement ledger.

**Benefit.** The unit of reuse becomes an operation plus its invariants.

**Limit.** A compositional IR can centralize bugs too; exhaustive variants,
resource limits and old-first laws remain necessary.

## 6. Separate compiler qualification from full application adoption

**Problem.** Full application adoption adds a distinct integration obligation.
A construct inventory was needed to determine whether ACC introduced a unique
witness, struct or ledger mechanism beyond the retained compiler cohorts.

**Decision.** After the inventory found no unique mechanism requiring another
qualification family, remove full ACC adoption from the accepted scope. Retain
generic Jubjub work and separately qualified Passport and DID cohorts.

**Evidence.** ADR 0316 and the consumer qualification report record the removed outcome and the exact retained
application cases.

**Benefit.** Useful compiler work survived without a misleading product claim.

**Limit.** The bounded passport cohort is not the full ACC application.

## 7. A faster test gate may not skip authentication

**Problem.** Historical validation took about 171 minutes, more than 99% in
historical checks. Naive memoization would have hidden live mutation.

**Decision.** Reject path/result caches and shard whole modules into isolated
processes, preserving fixture boundaries and independent temporary state.

**Evidence.** All 974 IDs and assertions remained; corruption and source
mutation controls passed; measured wall time fell to about 55 minutes.

**Benefit.** Faster feedback without turning a historical assurance suite into
a snapshot replay.

**Limit.** The result is machine/source specific and still needs the full gate
for final qualification.

## 8. Source provenance must be honest

**Problem.** Hand-built IR can make a backend look more source-compatible than
the frontend actually is, while absolute paths and private artifacts are unsafe
to publish.

**Decision.** Authenticate source, IR and artifact sidecars; distinguish
original occurrences, owned snapshots and missing origins; carry safe logical
source identity and structural paths.

**Evidence.** Authenticated input views, original-source guards and publication
manifests preserve exact hashes and failure cases.

**Benefit.** A passing fixture can be tied back to a real Compact source.

**Limit.** Nested expression ranges and transient-value provenance are not
established by the pilot reference.

## Independent review boundary

The project used independent coding-agent reviews for architecture, dependency,
correctness and final-slice checks. They found actionable issues and provided a
useful second model. They are not a human security audit, formal verification
or production certification. This guide uses the phrase “independent
coding-agent review” for that reason.

### Source provenance

Adapted from the [retained reference source](https://github.com/betsudo/compact/blob/51a079bd8598b9decb3df7de92ec10768ec9b8dc/doc/rust/journey/04-decisions-and-corrections.md). That documentation records implementation f3cef799; the documentation and implementation revisions are distinct. Later results are identified in the main chapters rather than inferred from this reference.
