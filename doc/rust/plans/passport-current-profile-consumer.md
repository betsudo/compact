# Current-profile Passport standalone consumer

Status: fresh clean v4 qualification passed at signed `f0c09ff56`, original
compiler 0.31.142/Ledger8.1.3. The untouched raw SDK passes 202 live TS cases and
codec vectors, 16 adoption Rust methods, and the actual unchanged two-test guide
crate (both manifest and source). SDK, adoption consumer and guide crate each
pass direct strict Clippy. The guide uses the documented qualified source-lock
seeding recipe; its original historical P1 lock is separately retained unchanged.

Correct-type twins of both misuse binaries compile; their restored negative
sources each fail with exactly one E0308 at the intended primary source location.
A changed owned oracle triggers a real assertion failure and exact restoration
passes. These controls rerun existing methods, not additional unique methods.
All 389 public Python methods pass with zero failures/errors and 20 existing
manual-only skip events. All 30 focused Passport/workflow controls pass, including
real same-group and detached-pipe timeout controls.

The v4 gate pins 186 source inputs and 117 generated SDK files (including the
explicitly qualified SDK lock). Formatting touches a separate retained copy;
SDK bytes remain raw. It preserves shared Cargo reuse outside the qualification
output and rejects targets in receipt/source directories. Earlier `2bbf4daaf`
passed the ambient-unset cold-target case; the v3/v4 runs reuse that explicitly mutable Cargo target
after an independently retained disk-full dependency-build failure. Every test
and compiler command executed afresh; no verdict or proof was cached. Only
obsolete build-script binaries/static archives from verified merged #676 were
reclaimed after process/open-file/source guards; all failed output and evidence
remain preserved.

Owned POSIX command process groups are stopped and reaped on timeout with partial
diagnostics; post-kill pipe draining is bounded even if a detached helper holds
a pipe. Detached process groups are outside the gate's stop scope, and hard
termination can prevent final receipt writing. Cargo compiler/wrapper/lint
substitutions are rejected; selected ancestor/home configuration file hashes
(including absence) are checked before/after. All 105 expected bundled runtime
files, manifests and support files must match the source owner exactly, with no
extra build script or symlink. The guide local graph excludes the adoption crate.
Source/oracle target directories are rejected; the workspace target and external
dedicated Cargo caches remain supported. Failure uploads have distinct run-attempt names and cover a failed
Passport step while the job/runner remains alive. Hard job timeout/cancellation
or runner loss can prevent final uploads. Selected executable and compact-runtime
package/dist identities are checked; transitive TS/WASM bytes are not independently
content-hashed by this gate.

Actual Claude review through published `668f270d4` found no hard code blockers;
the concrete Cargo-input/liveness and isolation findings are addressed in v4. Earlier `834a41025` tested formatted SDK bytes; `2bbf4daaf`
executed the guide test source through a gate-written manifest. Those receipts
retain their literal narrower scopes. Repeat exact-head PR review and complete
current-head CI remain required before integration.

PR #696 is verified merged at signed canonical `f0e4ba788`; normal reconciliation
`342b9fc46` preserves that accepted history. Passport remains its separate logical
slice and does not complete the broader Rust AST goal.

## Gap and accepted baseline

The delivery roadmap requires current application behavior and independently
usable generated artifacts. The Passport P1 walkthrough explicitly retains a
historical two-test result at `1705fc1a`; its later lock-seeding recipe was
guidance, not a newly executed consumer. Existing pure semantics and original
adoption evidence remain accepted. This slice closes a current application and
distribution qualification gap; it introduces no speculative language feature.

The independent baseline is accepted canonical `709a86230` (PR #695), with
original compiler 0.31.140 and Ledger8.1.3. Statement-action PR #696 has its own
review and full CI; the pure-library behavior does not depend on its frontend
fix. Local normal merge `91b412f2e` includes that exact reviewed source for
sequential compiler142 preparation; that earlier merge was not an accepted
canonical baseline. Actual canonical `f0e4ba788` was subsequently reconciled by
`342b9fc46` with identical whole source tree and all 179 then-qualified gate inputs unchanged (the earlier reconciliation epoch).
Keep at most two active engineering branches and retain normal canonical history.

Focused fresh baseline checks pass on the clean accepted source:

- The complete original 17-file import closure generates Rust and TypeScript.
- Its 89 internal pure IR circuits produce 75 public pure functions, with no
  ledger fields, witnesses, stateful circuits or proof-applicable exports.
- All 202 live TypeScript cases match the independently retained behavior and
  vectors. The current profile label remains distinct from historical profiles;
  original captures, source pins and their provenance are unchanged.
- Sixteen Rust test methods from the ten existing behavior modules pass against
  a freshly generated standalone crate and its matching bundled runtime. All
  local Cargo manifests resolve inside the external consumer closure; registry,
  Git and checksum identities match the qualified dependency cohort.

These baseline results are separate from the maintained gate, current P1
execution, compile-fail controls, strict Clippy, own PR/review and complete CI
required below. The existing historical `provenance.rs` intentionally checks its
original generated-library hash. It stays unchanged in its historical owner;
the fresh consumer separately checks all 17 source, 39 oracle and two codec-source
pins rather than rewriting that historical library identity.

## Owners and public API

- `examples/rust_backend/vc_passport_adoption/source-manifest.json` and its
  unchanged source/import closure: Passport `2e13b029f` and credential sources
  `1a65558b9`. A later upstream pin is a new adoption decision.
- `tests-rust-backend/vc-passport-adoption`: reuse its actual behavior tests,
  coverage inventory, independent expectations and twelve capture scripts.
- `doc/rust/guides/examples/passport/consumer` and the existing runtime bundle
  owner: preserve the complete documented P1 consumer and generated source.
- A narrow maintained gate under `tools/compact-rust-backend`: generation,
  dependency/identity guards, isolated Rust execution, live TS comparisons and
  explicit positive/refusal receipts. Use the existing cohort policy; no common
  gate framework or new public helper is required.
- Passport guide, delivery roadmap, midnight knowledge base and main checkpoint:
  record new results separately from the historical epochs.

Consumers retain the original 75 `pure_circuits` functions, generated types,
matching `runtime` re-export, `FixedBytes`, `BoundedUint` and fallible results.
No compiler option, builder, macro, runtime type or ABI change is intended.

## Acceptance

1. Bind the clean selected source, compiler/runtime/profile, dependency lock and
   complete original import closure before generation and at completion. Retain
   all generated bytes and metadata without handwritten edits or normalization.
2. Generate into a fresh standalone directory with bundled matching runtime and
   macros. Seed and validate the selected dependency cohort. Reject unexpected
   local manifests or registry/Git/checksum drift; no path may lead back into a
   development checkout. Compile and execute the complete documented two-test
   consumer as well as the unchanged adoption behavior modules.
3. Execute the twelve actual TypeScript captures and compare complete behavior
   fields and vectors to the independent original expectations. Preserve profile
   identity separately and validate the 202-case/75-export inventory. Retain
   genuine signature primitives, exact assertion/error precedence and both
   positive and tampered credential/protocol/request cases.
4. Reject wrong ordinary argument shapes and fixed-byte widths in independently
   compiled consumer controls. Preserve dynamic failures for well-typed but
   invalid schema/input values. Run strict consumer/generated-source Clippy and
   ordinary formatting; a typed value is not proof of a valid credential.
5. Add false-success controls for changed TS results, missing captures/source
   files and dependency/identity drift. A pass flag, generation alone or historical
   hash metadata cannot substitute for actual consumer execution.
6. A separate logical draft PR, exact-head Claude CLI review and all complete
   required CI/proof gates before signed Conventional/DCO/Assisted-by integration.
   Proof, recording replay and ledger application are explicitly N/A for this
   original pure library; do not invent stateful exports. Clean own completed
   worktree and obsolete reproducible targets only after verified GitHub merge,
   preserving all unpublished and ignored evidence.

## Exclusions and failure handling

No Forge, new CI optimization project, registry/release publication, source-pin
upgrade, network/wallet rollout, Ledger9 expansion, general refactor or historical
authority rewrite. This slice does not establish exhaustive branch coverage or
protocol-wide compatibility. A real current baseline failure must be retained,
minimized and repaired at its existing owner before accepting dependent results.
Harness setup failures retain their own receipts and are not compiler defects.
The broader Rust AST goal remains incomplete after this bounded slice.
