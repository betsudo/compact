# #612 local typed-IR provenance audit

Date: 2026-11-10. Base: `origin/codex/rust-backend-ast` at
`09c75e2891283fed65f2db385925a5a4b9409af6`. Reviewed local head:
`591502730` on `codex/rust-ast-transient-provenance-map-lookup` in an
isolated worktree. All 13 stacked commits have verified GPG signatures and
remain local.

## Verified bounded deliverable

`tools/compact-rust-backend/src/transient_provenance.rs` exports the explicit
`analyze(&Contract, Limits) -> Report` and `advise` entry points over typed
schema 20 IR. The module is only exported from `lib.rs`; rendering,
capability selection and recording admission do not invoke it. Findings
carry entry and sink circuit, source location where schema 20 provides one,
IR path, sink input, producer and ordered dependency routes, possible-path
control routes, and unknown-input flags. Unsupported forms and exhausted
budgets are reported as unavailable; a missing known producer is never a
stability certificate. An exact, bounded, caller-supplied acknowledgement
changes advisory warning status without removing evidence.

The latest head ran every `transient_provenance*.rs` integration test target:
13 targets, 38 tests, zero failures, under
`/Users/ysh/.codex/local-evidence/transient-provenance-map-lookup-20261110/target`
with `CARGO_BUILD_JOBS=2`. The source-produced fixtures and exact-parent/
current generated-output comparisons are documented in the adjacent
per-slice plans. The selected corpora's generated `contract/lib.rs` and
`contract/rust-capabilities.json` files compared byte-identical. The latest
map-lookup corpus hashes are in its own plan. No shared fixture was
regenerated. A fresh cumulative independent read-only review at this head
found no blocker for this bounded local prototype.

## Acceptance trace

| #612 criterion | Local evidence | Status |
| --- | --- | --- |
| ADR-0463 / CoPS-029 design, before/after examples, compatibility limits | Existing midnight vault ADR-0463 and CoPS-029; standalone local iteration note | Recorded design; implementation status remains separate |
| Opt-in typed-IR prototype independent of recording, deterministic and bounded | `transient_provenance.rs`; `transient_provenance.rs` tests | Verified for the selected IR/corpus |
| Direct producers, aliases, arithmetic, products/projections, conditional paths | First-slice and aggregate source fixtures/tests | Verified for supported forms |
| Helper-produced and forwarded origins, including action and value calls | Helper, action-call, value-call, effectful-return, and pure-action fixtures/tests | Verified for supported forms and bounded call depths |
| Field-to-bytes and `upgradeFromTransient` do not launder provenance | `FieldToBytes32` is handled by the unary conversion visitor; first-slice and standard-library `evolveNonce` source tests exercise `upgradeFromTransient` | Upgrade conversion source-verified; direct Field-to-bytes source fixture remains to add |
| Ordinary Field argument/witness acceptance; known route survives unknown input | First-slice, helper, membership and map-lookup source controls | Verified; unknown origin is not certified stable |
| Intentional standard-library use and acknowledgement | Value-call source fixture and exact-acknowledgement tests | Verified as caller-side advisory policy |
| Persistent values/openings, direct state writes, control dependence, summaries | Sink, aggregate, action-call, assertion, membership and lookup fixtures/tests; policy matrix below | Supported paths verified via bounded call-site traversal; no separate cached summary representation |
| Warning-only mode preserves native/recorded results, public API and ledger encoding | Pass has no emission hook; per-corpus generated Rust/capability byte comparisons | Rust/capability parity verified for selected corpus; native/recorded result and FAB-byte equivalence unverified |
| Optional wrapper feasibility separated from compiler lint | ADR-0463 nominal wrapper and trait discussion; no generated wrapper | Design separation recorded; no wrapper claimed |

## Policy matrix and explicit boundaries

| IR path | Local policy |
| --- | --- |
| `persistent_hash` and `persistent_commit` value; commitment opening | Finding for each known transient route reaching that input. |
| Selected cell/set/map/list/merkle write forms | Finding for known transient data or branch-control route; key, value and static position operands remain distinct. Cell/set/map have direct source cases; list/merkle matches lack source cases in this stack. Other write forms remain unsupported. |
| Conditional expression/action/return path | Possible-path control evidence; no reachability proof. Identical primitive literal result arms avoid a false result-control route. |
| Set/map membership and map lookup | Known key/operand route survives with unknown ledger state. Lookup selection is a possible dependency, not a claim about stored-value origin. |
| Assertion condition | Nested sinks remain visible. Transient-dependent continuation after the assertion is explicitly unavailable. |
| External argument, witness result, preexisting ledger read | Unknown origin remains usable; known routes combined with unknown inputs remain visible where supported. |
| Unsupported fold, dynamic/opaque projection, constructor flow, other unsupported action/expression | Explicit unavailable or unknown boundary; no stability claim. |

## Remaining issue-level work

The local prototype does not transport diagnostics through the frontend or
compiler CLI, and no default or strict compiler policy is selected. It does
not verify native/recorded runtime result parity, FAB bytes, proof behavior,
or universal Compact language coverage. Direct Field-to-bytes source evidence
and materialized helper summaries are also pending. The integration and
runtime tasks were expressly excluded
from this local iteration. The evidence therefore supports a reviewable
bounded typed-IR deliverable, not closing the full compiler-integrated
issue #612. The goal stays active until the remaining acceptance requirements
are authorized and verified or the issue scope is revised explicitly.
