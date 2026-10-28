# Current-profile Passport standalone consumer

Status: fresh clean qualification passes at signed `834a41025` with original
compiler 0.31.142: 202 live TypeScript cases, 18 Rust test methods including the
complete documented two-test consumer, direct generated-SDK and consumer strict
Clippy, and two independent E0308 refusals. The SDK is selected separately for
linting; its additional seeded Cargo.lock is explicitly qualified and hash-bound.
All original generated source bytes remain unchanged during execution.

Seven adversarial controls pass at this source. The combined public Python suite
passed 377 selected methods at `7e1f9d363`, with 20 existing manual skip events.
That earlier run executed consumer Clippy only: it did not independently lint its
generated SDK dependency. Earlier wording claiming generated-source Clippy was
too broad; the fresh direct SDK invocation above closes that gap without changing
generated code or lint allowances. Historical receipts retain their literal scope.

PR #696 is verified merged at signed canonical `f0e4ba788` after all eleven
checks, actual Claude review and retained-artifact verification. Normal signed
reconciliation `342b9fc46` includes that accepted history without changing the
qualified Passport source tree. This slice still needs its own exact-head PR
review, full CI and verified merge.

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
`342b9fc46` with identical whole source tree and all 179 gate inputs unchanged.
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
