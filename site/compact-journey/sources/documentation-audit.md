# Documentation audit and editorial conventions

This audit reconciles the book's 24 chapters, navigation, iteration summaries
and technical reference with the engineering knowledge base, accepted reports
and source identities frozen for this edition. It changes the explanation of
the history; it does not extend the implementation or its qualification.

## Findings and corrections

| Finding | Correction | Basis |
| --- | --- | --- |
| Calendar-based titles assumed knowledge of an internal planning discussion. | Describe the engineering problem and design rationale directly. Keep dates in source records and edition provenance. | Architecture rebuild plan and post-algebra roadmap in the engineering journal; corresponding repository ADRs and implementations. |
| An internal “0.3.0” project label resembled an unexplained public release series. | Name the stage “Consumer qualification” and the chapter “Qualifying generated APIs and applications.” | The accepted report distinguishes project acceptance from compiler, crate, IR and ABI versions. |
| Pilot reference pages described checked-effect integration as currently pending. | Identify the reference as the Ledger 8 typed target and pilot. Link to subsequent checked-effect integration. | Pilot source and the later integrated replacement through PR #641. |
| “386 source availability” conflated source files with exports. | State 217 sources, 196 compiled roots and 755 exports, including 386 proof-required and 369 non-proof exports. | Recorded-target inventory. Availability remains distinct from proof execution. |
| A reference explanation of the ACC exclusion differed from the recorded decision. | State the construct-inventory rationale and retain the exact application composition as unqualified. | ADR 0316 and the consumer qualification handoff. Exclusion is not delivery. |
| The glossary combined frontend proof requirement with readiness of proving artifacts. | Separate the frontend requirement from source-, artifact- and policy-specific qualification. | Capability reporting and proof/application qualification records. |
| “Replay” could mean either replaying a recording or resubmitting a transaction. | Define recording replay and repeated-transaction refusal separately. | Runtime recording tests and strict transaction application controls. |
| An evidence refresh could restore older reference prose over reviewed explanations. | Preserve authored chapters and stable URLs when refreshing frozen evidence. | The book's source and publication workflow. |

## Implementation order

1. Pin the Ledger 8 behavior and package family so differences are reproducible.
2. Introduce typed IR, checked lowering and structured Rust syntax construction.
3. Retain ordered execution effects and bind transaction preparation to observed
   state and the installed operation.
4. Qualify generated consumers and selected application cohorts.
5. Replace whole-circuit recognizers with composable checked operations.
6. Relate native and recorded execution through shared admitted programs and
   close individual source-semantic gaps.
7. Migrate the package, representation and transaction interfaces to Ledger 9.
8. Port upstream capabilities with their own original-source, runtime and
   applicable proof/application coverage.

This sequence explains technical dependencies. It does not assert that every
capability in a stage was implemented in a single commit or that every later
compatibility obligation is complete.

## Source checks

- [Native Rust execution baseline](https://github.com/betsudo/compact/commit/ea4cc006f973632c4c7b1fb4569af2b4c93ff38a)
- [Recorded-target qualification](https://github.com/betsudo/compact/commit/03c03a39a2d39c43c91143b9526ca1c2ffae706c)
- [Consumer qualification and exclusions](https://github.com/betsudo/compact/blob/0d7610b2792d60fee5d3713d4eb0ede3c98cc82a/doc/rust/0.3.0-closure.md)
- [Checked-effect integration](https://github.com/betsudo/compact/commit/0437c894cd069af475bd8d5b254c24cc73615a09)
- [Shared-execution integration](https://github.com/betsudo/compact/commit/c353907d8c323db16eedb35d5a7f0151aab859bb)
- [Ledger 9 migration foundation](https://github.com/betsudo/compact/pull/725)
- [Frozen compatibility plan](ledger9-upstream-compatibility.md)
- [Edition identities and platform scope](edition.json)

Private engineering notes were used to reconcile intent and acceptance scope.
Their workstation paths and complete contents are not published. Original Git
messages and frozen reports retain their historical wording. The chapter
narrative supplies context for shorthand in those records.

## Maintenance rules

Use capability-based titles and explain the problem, decision, result and
validation boundary. Explain protocol and representation versions at the point
where they affect behavior. A planning label is not a release declaration.

Keep implemented, compiled, executed, recorded, proved, verified and applied
as separate claims. Identify the source revision, platform and fixture
population for quantitative results. Preserve failed runs and excluded work.

Keep existing chapter URLs when improving titles, so shared links remain valid.
Reconcile later evidence explicitly instead of presenting a pilot's open items
as the current implementation state.
