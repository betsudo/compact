# #612 local typed-IR provenance audit

## PR publication history

Before pushing, the 25-commit local stack was rebased to add the required
DCO sign-off trailers. Every corresponding commit retained the same Git tree,
and all 25 published-history commits have valid GPG signatures and DCO
trailers. Historical hashes in the dated per-slice evidence notes refer to
the pre-publication local commits and retained build evidence. Key old-to-PR
commit mappings are `fe7c5dd84` → `64bc5a973` (first slice),
`bfa7c06c2` → `b69d91b9f` (standalone CLI),
`fa429b909` → `782e4c988` (cumulative parity),
`6a05c3120` → `a72a1d7fd` (schema guard), and
`688f0e6e9` → `6e91db217` (compiler warning transport).

Date: 2026-11-10. Base: `origin/codex/rust-backend-ast` at
`09c75e2891283fed65f2db385925a5a4b9409af6`. Current local worktree:
`codex/rust-ast-transient-provenance-compiler-warnings`, stacked on the
GPG-signed schema-guard head `6a05c3120`. Earlier local stack commits
also have verified GPG signatures. No remote publication.

## Verified bounded deliverable

`tools/compact-rust-backend/src/transient_provenance.rs` exports the explicit
`analyze(&Contract, Limits) -> Report` and `advise` entry points over typed
schema 20 IR. A separate `compact-transient-provenance` binary consumes
already produced typed IR and emits advisory JSON; rendering, capability
selection and recording admission do not invoke it. Findings
carry entry and sink circuit, source location where schema 20 provides one,
IR path, sink input, producer and ordered dependency routes, possible-path
control routes, symbolic root-parameter dependencies for data/control, and
unknown-input flags. Unsupported forms and exhausted
budgets are reported as unavailable; a missing known producer is never a
stability certificate. An exact, bounded, caller-supplied acknowledgement
changes advisory warning status without removing evidence. Eligible pure
helper return summaries, including supported nested pure-call chains and
action-free stateful query returns, are materialized with formal routes and
instantiated at call sites; sinkful and actionful stateful helpers retain
bounded body traversal.

The schema-guard evidence head ran every `transient_provenance*.rs`
integration test target: 21 targets, 71 tests, zero failures, under
`/Users/ysh/.codex/local-evidence/transient-provenance-schema-guard-20261110/target`
with `CARGO_BUILD_JOBS=2`. The source-produced fixtures and exact-parent/
current generated-output comparisons are documented in the adjacent
per-slice plans; the full focused run is retained as `provenance-tests.log`
in that evidence directory. The selected corpora's generated `contract/lib.rs`
and `contract/rust-capabilities.json` files compared byte-identical. A direct
canonical-base/current comparison also found all 1,855 generated files
byte-identical across all 16 source-backed provenance corpora; the standalone
cumulative-parity note and external `comparison.json` retain the details.
No shared fixture was regenerated. Cumulative independent read-only review
found no blocker for
the preceding bounded stack; the symbolic-input review found two initial
issues and no blocker after their fixes. The state-collections review found
an indexed-position test gap; four synthetic analyzer regressions closed it,
and re-review found no remaining bounded-slice blocker. Helper-summary review
identified route-order, aggregate-projection and unknown-field wording gaps;
all were addressed. Nested-summary review found no code blocker and requested
a stateful-call exclusion regression, which is included. State-query-summary
read-only review and final re-review found no blocker. CLI review identified
schema, input-budget and duplicate-key gaps; after fixes, final re-review
found no remaining blocker.

The standalone advisory was also run over all 16 retained typed-IR outputs:
80 warnings, eight explicit assertion-control unavailable records and no
truncation. Its structured reports and summary are retained alongside the
cumulative parity evidence.

## Acceptance trace

| #612 criterion | Local evidence | Status |
| --- | --- | --- |
| ADR-0463 / CoPS-029 design, before/after examples, compatibility limits | Existing midnight vault ADR-0463 and CoPS-029; standalone local iteration note | Recorded design; implementation status remains separate |
| Opt-in typed-IR prototype independent of recording, deterministic and bounded | `transient_provenance.rs`, standalone CLI and focused tests | Verified for selected IR/corpus; direct API reports unavailable for unsupported schemas, while CLI rejects them and oversized/malformed JSON |
| Direct producers, aliases, arithmetic, products/projections, conditional paths | First-slice and aggregate source fixtures/tests | Verified for supported forms |
| Helper-produced and forwarded origins, including action and value calls | Helper, action-call, value-call, effectful-return, and pure-action fixtures/tests | Verified for supported forms and bounded call depths |
| Field-to-bytes and `upgradeFromTransient` do not launder provenance | Distinct `FieldToBytes32` source fixture/test; first-slice and standard-library `evolveNonce` source tests for `upgradeFromTransient` | Verified for selected conversions, including commitment opening |
| Ordinary Field argument/witness acceptance; known route survives unknown input | First-slice, helper, membership and map-lookup source controls | Verified; unknown origin is not certified stable |
| Symbolic formal dependencies and external/unknown contributions | Helper, aggregate, state-sink, map-lookup and membership source fixtures; symbolic-input and summary tests with explicit parameter/component/summary/call limits | Caller-root parameter dependencies reported separately for data/control; eligible pure and action-free stateful query return summaries materialized and instantiated |
| Intentional standard-library use and acknowledgement | Value-call source fixture, exact-acknowledgement tests and CLI JSON input/output test | Verified as caller-side advisory policy |
| Persistent values/openings, direct state writes, control dependence, summaries | Sink, state-collections, aggregate, helper-summary, nested-summary, state-query-summary, action-call, assertion, membership and lookup fixtures/tests; policy matrix below | Supported paths verified; eligible pure and action-free stateful query returns cached, sinkful/actionful helpers traversed per call |
| Warning-only mode preserves native/recorded results, public API and ledger encoding | Public `compactc` opt-in warning flag; canonical-base/current generated tree (1,855 files across 16 corpora); warning/default Rust and capability files match for all 16; four selected native/recorded calls and tagged ledger-state bytes match base/current | Verified for selected corpora; nonempty result FAB bytes compared for two calls; private FAB arrays empty |
| Compact compiler warning transport and exact acknowledgement | Public `compactc` opt-in flag, focused 34-test target, 16 genuine source runs with 84 route warnings and eight unavailable notes, one genuine exact acknowledgement | Verified for the public Rust wrapper; frontend-owned transport remains outside this slice |
| Optional wrapper feasibility separated from compiler lint | ADR-0463 nominal wrapper and trait discussion; no generated wrapper | Design separation recorded; no wrapper claimed |

## Policy matrix and explicit boundaries

| IR path | Local policy |
| --- | --- |
| `persistent_hash` and `persistent_commit` value; commitment opening | Finding for each known transient route reaching that input. |
| Selected cell/set/map/list/merkle write forms | Finding for known transient data or branch-control route; key, value, hash and position operands remain distinct. Cell/set/map/list/merkle have direct source cases; transient-derived Merkle positions have synthetic schema-20 IR analyzer regressions but lack a source case because the attempted Field-to-Uint downcast is rejected (retained compiler diagnostic in the state-collections evidence directory). Other write forms remain unsupported. |
| Conditional expression/action/return path | Possible-path control evidence; no reachability proof. Identical primitive literal result arms avoid a false result-control route. |
| Set/map membership and map lookup | Known key/operand route survives with unknown ledger state. Lookup selection is a possible dependency, not a claim about stored-value origin. |
| Assertion condition | Nested sinks remain visible. Known-transient or unknown/symbolic continuation after the assertion is explicitly unavailable. |
| External argument, witness result, preexisting ledger read | Unknown origin remains usable; known routes combined with unknown inputs remain visible where supported. |
| Unsupported fold, dynamic/opaque projection, constructor flow, other unsupported action/expression | Explicit unavailable or unknown boundary; no stability claim. |
| Parameter or aggregate component budget exhaustion | Explicit unavailable/truncated record; retained symbolic and component evidence has independent caps. |

## Remaining limits

The public Rust `compactc` wrapper now transports opt-in warnings, but no
frontend-owned diagnostic transport, default lint policy or strict failure
mode is selected. The focused native/recorded comparison verifies four
map-lookup calls and two nonempty tagged result FAB returns; private FAB
arrays are empty. It does not verify proof execution or universal Compact
language coverage. Sinkful and actionful stateful helper summaries remain
unmaterialized. These limits are explicit unavailable/unknown boundaries
or deferred integrations, not stability claims. The new compiler and runtime
evidence note records the completed local iteration and independent review.
