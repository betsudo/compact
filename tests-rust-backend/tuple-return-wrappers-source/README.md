# Typed Field pairs around terminal effects (#516)

Generated from the complete `examples/rust_backend/tuple_return_wrappers.compact`
source with `--rust-functional-ledger8`. Only Rust formatting and fixture Cargo
package/dependency placement differ from compiler output. Original complete IR,
origins, capabilities and all eight checked programs are retained unedited.

The source covers inferred versus explicit pair conversion, nested Uint joins,
a mixed Field/Uint pair with lexical capture, Set replacement/insertion,
Counter effects and ordered component witnesses/subtraction. The constructor
seeds Set key 5, Field 2 and Counter 0. `[Field, Field]` remains the ordinary
Rust result `(runtime::Field, runtime::Field)`; no new caller-side cast is needed.

`check_tuple_return_wrappers.py` regenerates both targets and runs an external
consumer against live pinned TypeScript and independent expected pairs/state/
witness events. **42 cases / 159 Rust executions / 68 successful recorded VM
replays**: 39 stateful cases use four routes, and three pure controls use one.
Five failures compare callback prefixes and the actual Rust error; no failed
context rollback is claimed. Both result atoms, all fixture state, neutral
effects, private state/output and query-summed costs are checked. Altered TS
results must fail the real consumer. E0308 argument/result-tuple/mixed-domain/vector-length and E0382
Program-ownership controls, both-target arity/narrowing/pure-effect/disclosure refusals,
and strict consumer Clippy are required.

`check_tuple_return_wrappers_proof.py` independently regenerates and matches
complete linked IR/programs, refuses altered full IR before keys/state and
qualifies selected replacement/new-key calls using fresh native keys/proofs,
immutable verification, changed binding and both same-geometry tuple components,
funded default-strict application and unchanged replay refusal. Constructor,
unselected branch and other exports are not proved by that gate.

This candidate is based on the accepted canonical merge of PR #691. Fresh
clean-source compiler qualification, proof acceptance, full required CI,
independent review and canonical merge remain open. See the
[bounded plan](../../doc/rust/plans/516-tuple-return-wrappers.md). Forge remains
deferred and is not a dependency of backend development.

## Historical selected development proof checkpoint, before review

The uncommitted source passes the fresh direct proof gate at **k=7, 100 rows**:
two **2,912-byte proofs**, for Set replacement key 5 and new key 6. The gate
compares all four generated APIs, complete linked IR and all six programs,
refuses altered full IR before keys/state, verifies with the immutable verifier,
rejects changed binding and each same-geometry tuple component, applies funded
default-strict transactions and refuses replay without changing state.
Constructor and unselected-branch proofs are not claimed. The receipt explicitly
records dirty source at parent e568; it does not replace the later clean-source
original compiler, fresh proof and full CI integration gates.

The earlier Python run discovered **347** controls: **327 passed**, with
**20** explicit existing skips.
The initial development run exposed incomplete new registration accounting;
its failed receipt remains failed. The corrected run preserves all historical
rows and adds only eleven declarations for this source.

## Review-driven coverage

Mixed `[Uint<8>, Field]` results retain a `BoundedUint<255>` first component;
Field-pair results retain two Fields. The Fold consumer exercises vectors
`[0,1]`, `[1,0]`, `[1,2]`, `[0,0]`, checking final pairs and selected Set effects.
The test-only observation projection preserves the exact aligned result type.
Each TS atom is tampered separately. Inaccessible failed-context state is null,
not a rollback expectation. Complete fixture programs/declarations were freshly
regenerated after this source extension. Selected mixed/Fold development proofs pass below;
clean-source integration qualification remains open. Earlier receipts keep their
original source epoch. See the bounded plan for the exact continuation.

## Expanded selected development proof checkpoint

Protocol v2 passes four fresh 2,912-byte standalone call proofs: inferred key5
replacement/key6 insertion, mixed key6 insertion and Fold `[1,0]`/key0 insertion.
Fresh keys use k7/100 rows for inferred/mixed and k8/130 rows for Fold. Each call
checks ordinary/recorded/both functional APIs and observed-call preparation,
immutable verifier identity, changed binding and either output component at
unchanged geometry, funded default-strict application and unchanged replay
refusal. Full IR and all eight programs match fresh generation; altered IR
refuses before keys/state. Parameters are verified from locked upstream tables.

Constructor execution, inferred/mixed false-branch proofs and other exports
remain outside this finite gate. The dirty-source status is retained; these are
not final clean-compiler/current-CI receipts. The current Python run reports
328 passed and 20 existing skipped cases out of 348 discovered, with no errors.

## Accepted baseline and clean-source qualification

PR #691 is verified merged at canonical `c3a529cb5` after complete current CI,
with a valid GitHub GPG signature and a tree identical to reviewed `e568b6064`
and tested synthetic merge `a84d08158`. The preserved prototype was advanced
onto that actual baseline with all 49 source files and 16 private review files unchanged.
The private reviews were then preserved outside this checkout to keep clean
qualification source separate from unpublished review data. Historical local
receipts above retain their dirty-source epochs; they are not relabelled.

This source selects compiler 0.31.137, unchanged language/runtime versions and
runtime ABI 53. Maintained document regeneration/full compiler tests, a clean
original compiler build, fresh final-source API/proof gates, full required CI,
actual PR review and merge remain required before delivery. Earlier local
preflights do not replace the published-head review. This does not close all of
#516, qualify all applications or complete the primary Rust AST goal.

## Maintained compiler generation — 0.31.137

The default development shell's `./compiler/go` completed successfully on
2026-10-27. It regenerated the ledger reference at compiler **0.31.137** and
passed the full ordinary compiler suite, including **636/636** manifest tests
and **1/1** JavaScript execution tests in each ZKIR mode. This staged-source
run is distinct from a clean original-package build and the final generated
API/proof qualification. Those checks and the full published-head CI/review
gates remain required before merge.

## Clean original-compiler qualification — 2026-10-27

Clean source `9fcb4d7f1` built original compiler **0.31.137** and passed the
fresh API/live TS gate: **42 cases / 159 Rust executions / 68 successful
recorded replays**, all compile-fail/source-refusal/altered-TS controls and
strict consumer Clippy. The complete **280-fixture** generated-source comparison
passed with zero stale outputs or failures. Public Python discovery passed
**328** tests, with **20** existing explicit skips out of **348** discovered.
The package build itself has `doCheck = false`; maintained compiler tests are
the separate successful `./compiler/go` run above.

Fresh selected native keys and four **2,912-byte** proofs passed protocol v2:
inferred/mixed at **k7 / 100 rows**, Fold at **k8 / 130 rows**. Each call
verified with immutable verifier identity, rejected changed binding and both
same-geometry output components, applied funded default-strict transactions
and rejected replay without mutation. Full linked IR and all eight programs
matched fresh generation; altered IR refused before keys or state. Constructor,
inferred/mixed false-branch and other-export proofs remain outside this scope.

Implementation commit `ee08ef845` has the identical complete tree
`eadde362316d9b7d121e9b3d2cce5024d9fbded0`; only the unpublished commit message
was corrected to use proper Assisted-by/DCO trailers. The original qualified
commit and all receipts are retained with their actual identities. Subsequent
qualification-document updates do not relabel those compiler/proof artifacts.
Full published-head CI, actual Claude CLI PR review and verified signed merge
remain required; the two earlier local preflights do not substitute for review.
