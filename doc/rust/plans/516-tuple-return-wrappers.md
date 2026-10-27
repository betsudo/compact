# #516 slice: typed scalar pairs around terminal effects

## Gap and ownership

The scalar wrapper slice is PR #691. Its frozen source accepts inferred scalar
returns but still rejects a declared `[Field, Field]` result whose inferred
integer tuple wraps a terminal Set mutation conditional. TypeScript accepts the
source; explicitly casting both components is a positive control. The retained
frontend trace contains same-arity `ttuple` safe casts around the conditional
and inferred Uint branch joins.

`compiler/rust-ir-passes.ss` owns wrapper recognition and terminal ReturnPlan
decomposition. Extend its non-failing relation only to pairs of identity Field,
monotone Uint and Uint-to-Field components. Existing `Expr::Coerce`, checked
`CoercionPlan` and aggregate Rust AST rendering own conversion validation and
bind the reached aggregate exactly once before ordered component conversion.
Existing native/recorded/functional lowering keeps its authority. No new IR,
runtime ABI, generated builder or source option is required.

The ordinary and functional Field-pair APIs return `(runtime::Field, runtime::Field)`.
Mixed results retain `(runtime::BoundedUint<255>, runtime::Field)` for
`[Uint<8>, Field]`; the Fold input remains `FixedVector<Field, 2>`.
Applications should not need manual casts for valid inferred Compact branches.
Typed arguments and consuming `Program` execution remain unchanged.

## Scope

Fixed scalar pairs (Field and non-failing Uint widenings), terminal selected effects, inferred joins, lexical capture and
component witness/checked-arithmetic ordering, including their composition in
existing inline Fold reducers. No new Fold operation or general vector-return
admission is added. No arbitrary vectors, structs,
nested products, arity changes, general narrowing, effect-bearing initializers
outside this wrapper, Cell lexical redesign, retired recording paths, Forge,
SDK/distribution work, broad refactor or CI optimization project. This does not
close all of #516 or all Rust language coverage.

## Acceptance

- Establish the actual clean merged canonical baseline before publication.
  Keep the frozen parent source intact and stop dependent work on a real
  parent source failure. Local uncommitted staging is not integration acceptance.
- Regenerate complete source-bound Rust/TS assets, original IR, origins,
  capabilities and all eight functional programs. Preserve historical declaration
  and source authorities; add only this reviewed source. Preserve existing scalar
  and expression-only output, recording capabilities and required gates.
- Execute both branches, replacement/insertion, nested joins, lexical capture,
  component witnesses and checked subtraction across four APIs against live
  pinned TS and independent expectations. Check both canonical result atoms,
  all fixture state, neutral effects, private state/output and query-summed cost.
  Replay every successful recorded case. Compare failure cause and callback
  prefixes; make no failed-context rollback claim.
- Compile-fail incorrect ordinary argument and result-tuple types with E0308,
  and consumed Program reuse with E0382. Both targets must refuse tuple arity,
  undeclared disclosure and pure mutations at their genuine source boundaries.
  Retain malformed, inactive-arm, conversion-domain and resource controls.
  The actual consumer must reject an altered TS result.
- Generate fresh selected keys/proofs, verify with immutable verifier identity,
  reject changed public binding and each output component at unchanged geometry,
  apply funded default-strict transactions and reject replay without mutation.
  Constructor, unselected branch and other exports are explicitly separate.
- Focused local checks during iteration; original compiler build, complete
  existing native Linux/Darwin/compiler/package/wallet/proof CI and independent
  Claude CLI review before signed Conventional/DCO/Assisted-by PR delivery.
  Codex CLI with a different suitable model is fallback only on actual limits.
  Preserve concurrent knowledge edits and checkpoint CAS. Clean only unused own
  worktrees/targets after verified GitHub merge.

## Historical local prototype checkpoint, before review — 2026-10-27

Isolated uncommitted development from frozen parent `e568b6064`; PR #691 remains
unmerged. The live TS/generated Rust consumer gate passes **34 cases / 127 Rust
executions / 52 successful recorded VM replays**, including independent ordered
witness events, first/second-component failure controls, E0308 argument/result
shape and E0382 ownership refusals, both-target source refusals, changed-TS
rejection and strict consumer Clippy. The three pure controls use one route;
31 stateful cases use four routes, including five failure cases. This is
provisional development evidence, not a clean-source compiler/proof/CI receipt.

Original compiler qualification, selected fresh proofs, complete latest-head
CI, independent review and canonical merge remain **OPEN**. Forge remains
intentionally deferred and incomplete.

## Selected development proof checkpoint

The uncommitted source passes the fresh direct proof gate at **k=7, 100 rows**:
two **2,912-byte proofs**, for Set replacement key 5 and new key 6. The gate
compares all four generated APIs, complete linked IR and all six programs,
refuses altered full IR before keys/state, verifies with the immutable verifier,
rejects changed binding and each same-geometry tuple component, applies funded
default-strict transactions and refuses replay without changing state.
Constructor and unselected-branch proofs are not claimed. The receipt explicitly
records dirty source at parent e568; it does not replace the later clean-source
original compiler, fresh proof and full CI integration gates.

The prior public Python run discovered **347** controls: **327 passed** and
**20** explicit existing cases were skipped.
The initial development run exposed incomplete new registration accounting;
its failed receipt remains failed. The corrected run preserves all historical
rows and adds only eleven declarations for this source.

## Independent local review and coverage extension

Actual Claude CLI read-only preflight found no implementation bug and identified
two introduced routes lacking direct tests: declared mixed Uint/Field pairs and
pair returns inside inline Fold reducers. Baseline Rust rejects both examples;
the candidate and TypeScript accept them. Keep the compositional admission and
cover it through existing owners rather than adding route-specific exclusions.
The general triple-return boundary and invalid tuple narrowing retain their
original source refusals; no triple/narrowing support is claimed.

The extended live TS/four-route gate passes **42 cases / 159 Rust executions /
68 successful recorded VM replays**. Four E0308 controls cover ordinary argument,
result arity, the exact mixed result domain and fixed-vector input length;
E0382 checks consumed Programs. Eight both-target source refusals include invalid
tuple narrowing. Each TS output component is changed independently through the
real consumer. Failed cases set inaccessible result/final-state expectations to
null, preserving only actual error and callback-prefix obligations.

The complete fixture now retains eight checked programs and thirteen source
declarations, without rewriting historical authorities. The selected proof gate
is being extended to four calls: inferred replacement/new key, mixed insertion,
and Fold input `[1, 0]`. Each must verify, reject binding and both component
changes at identical geometry, apply default-strict funding and refuse replay
without mutation. The earlier two-proof receipt above retains its old source
scope; it does not qualify this new source epoch. Clean original compiler,
final-source proofs, full current CI and eventual PR review/merge remain open.

## Expanded selected development proof checkpoint

The new uncommitted source passes protocol v2: fresh native keys for `inferred`
and `mixed` at **k7 / 100 rows**, and `folded` at **k8 / 130 rows**. Four fresh
**2,912-byte** standalone call proofs cover inferred key5 replacement, inferred
key6 insertion, mixed key6 insertion and Fold input `[1, 0]` inserting key0.
Every call checks all four typed APIs, full aligned output/state/private/gas,
ordinary observed-call preparation, immutable verification, altered binding,
each output component at unchanged geometry, funded default-strict application
and unchanged replay refusal. Complete linked IR/all eight checked programs
match fresh generation; altered full IR refuses before keys/state. Locked
built-ins and selected k7/k8 parameters are verified and remain unchanged.

Constructor execution, inferred/mixed false-branch proofs and the other exports
are not claimed. Fold's one proved input executes its actual first false and
second true reducer branches. These are dirty-source development receipts;
final clean original-compiler/proof/current-CI acceptance remains open.
The current Python run discovered **348** controls: **328 passed**, **20 existing
cases skipped**, no failures/errors. The focused pair/scalar goldens both match;
the earlier 280-fixture census retains its earlier source epoch, with the old
scalar source and compiler behavior unchanged by the added test coverage.

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
