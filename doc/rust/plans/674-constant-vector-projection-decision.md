# Decision — checked constant vector projection (#674)

Date: 2026-10-25. Status: implementation in progress, local only.
Parent: [bounded plan](forge-deferral-and-vector-projection.md).

## Observed gap

The accepted Compact frontend treats an in-range constant Vector projection as
`tuple-ref`, and the private IR transports `tuple_index`. The ordinary compiler
at separation commit `cfb28592d` accepts genuine `Vector<2,Field>` first/last
and `Vector<1,Field>` source for TypeScript; its Rust backend rejects it with
`ExpectedTuple(Vector { ... })`. Index N and indexing `Vector<0,Field>` already
fail in source analysis with an out-of-bounds diagnostic. These are source
observations, not fabricated private IR fixtures or fresh proof qualification.

## Decision

Keep private IR schema20 and its current type-directed `tuple_index` spelling.
The shared semantic projection owner checks the child once, then selects tuple
arity or vector bounds. Preserve existing tuple errors and precedence. Emit a
distinct checked vector projection fact and recording `VectorProject`; never
weaken the invariant that `TupleProject` requires a tuple.

Native emission uses structured `syn` indexing of the existing
`FixedVector<T,N>` carrier. A known binding provides an addressable native place;
ordinary owned-value reads retain their original semantics. Select from the
place, then retain only a non-Copy selected element. Temporaries execute once.
No public indexing trait, new runtime ABI or Copy/Default bound is introduced.
The stateful emitter reuses the same checked index facts and output formatter;
recorded and functional emission share checked-plan formatting.

The independent recording validator must recheck vector kind, exact input and
result types, bounds and scope. The planned same-type operand-substitution
control also needs an independent identity binding: plain type/scope validation
alone is not a claim of source-operation equivalence. Its acceptance remains
open until that control actually rejects the substitution.

## Ownership and boundaries

Semantic owner: `pure_semantics/aggregate/projection.rs`. Native output owners:
`pure_native/{fragment,sink,value,aggregate/projection}.rs` and
`stateful/expression.rs`. Checked recording owners: `effect_plan/{model,lower,
validate,render}` and exhaustive context-free/public-effect/functional consumers.
Original Expr traversal, source-origin identity and resource budgets stay intact;
no new source variant or node-count exception is required.

Generated inputs remain `FixedVector<T,N>`, results retain element type T, and
functional `Program::new/run/execute_native/execute_recorded` stays consuming.
Invalid ordinary length/type and double execution must fail Rust compilation.
Untrusted private IR/plans continue to need explicit validation. Source bounds
checks do not justify omitting independent plan checks.

## Acceptance and remaining work

Use the finite acceptance in the parent plan: genuine first/last/singleton,
owned reuse/temporary, witness and error order, four execution routes, selected
TS parity, independent plan mutations and external compile-fail controls. Bind
actual source/full IR/ZKIR/key/runtime/parameter identities for genuine proof,
verification and changed-binding negatives. Funded strict application and replay
refusals remain necessary for an application claim. Do not report emitted APIs,
key generation or passing type tests as executed proof acceptance.

The new `constant_vector_projection.compact` fixture is the real source owner.
Do not alter the frozen old tuple oracle to make it accept a new capability;
new vector laws and actual generated consumers establish the widened boundary.
Existing tuple-as-vector forgery refusals and all required integration gates
remain. No Forge work, dynamic indexing, slicing/update, broad refactor, new
runtime framework, CI optimization or public ABI expansion is included.

## Implementation checkpoint — 2026-10-25

The first eight vector tests pass. A small private occurrence registry now
retains source-lowered vector result/operand IDs and the selected constant
outside the candidate plan. Independent validation rechecks type/scope/bounds
and that binding. The control rejects both another genuinely in-scope same-type
vector and a different in-range index; an unbound candidate cannot mint its own
occurrence. This does not establish whole-plan equivalence or serialized source
attestation. The ordinary source/runtime ABI and private IR transport are
unchanged. Genuine generated-consumer, compile-fail and proof acceptance remain
open at this checkpoint.

## Generated behavior checkpoint — 2026-10-25

The updated compiler generated the genuine fixture, including source-bound
functional programs. The full local backend gate passed 1,805 unit tests and
154 renderer tests, with seven inherited unit ignores unchanged. Strict backend
all-target/all-feature Clippy passed. Seven independent external consumer tests
passed against the unedited generated crate, including four stateful execution
routes, owned vector reuse, one witness evaluation, inactive/failing witness
behavior, state/effects/gas agreement and actual public replay. External wrong
length/type controls produced E0308; double execution produced E0382.

The maintained focused gate and consumer fixtures live in
`tools/compact-rust-backend/check_constant_vector_projection.py` and
`tools/compact-rust-backend/tests/vector_projection/`. Its local receipt is
unpublished. The full compiler workflow adds this check while retaining existing
functional, proof, wallet, fixture, workspace and packaging gates; no hosted run
was initiated. Executed pinned TS oracle parity, genuine source-bound proof and
verification/changed-binding negatives and the remaining complete integration
qualification are still open. Do not describe this checkpoint as #674 acceptance
or complete language parity.


## Local TS and proof qualification — 2026-10-26

The current named `OwnedSelection` fixture preserves repeated non-Copy selection
and returning the original vector. It gives the generated result named fields.
Full workspace Clippy exposed redundant `Ok(...?)` functional closures; the
structured emitter now evaluates the native expression into one local before
returning it, preserving its original error conversion and single evaluation.
Strict all-target/all-feature workspace Clippy passes without new lint allowances.

Actual local qualification now includes:

- Required Rust workspace command (`--workspace --exclude compact`): 3,326 tests passed across 959 result groups;
  7 existing ignores retained. Backend 1,805 unit and 154 render tests pass.
- The maintained full generated functional-consumer gate also passes after the
  shared closure correction, retaining its complete existing scenario set.
- Maintained Python suite: 329 tests, 20 expected skips. Fresh golden output and
  all five complete checked programs match; historical ledger-8.1.3 registry
  bytes remain frozen separately and the current extension preserves every
  predecessor row.
- Pinned TS runtime 0.16.102 executes five pure and seven stateful cases plus
  three refusal controls. Canonical result/Cell/private-output encodings,
  private transitions and measured query-summed cost agree with the Rust
  consumer and its four execution routes. The TS wrapper's last-query cost is
  retained separately; no costs are subtracted or estimated. A changed TS
  result fails the actual Rust consumer. Node/runtime production bytes are
  pinned; macOS system-cache library names are retained without byte claims.
- The direct native proof runner checks exact source/full IR and complete
  generated functional output, verifies locked official built-ins and selected
  SRS, and generates a fresh `witnessed` key pair: k=6, 54 rows. Both selections
  yield an actual 2,912-byte proof and independent verification. Changed public
  binding and changed-result commitment fail verification; changed-result
  preimage checking also refuses. Mutated full IR refuses before absent keys
  or state.
- Each branch then proves a fresh funded transaction, applies with default
  strictness and rejects replay with `IntentAlreadyExists` without mutation.
  Constructor state is supplied; constructor execution is not proved. This is
  local ledger-fixture evidence, with no live-chain or other-circuit proof claim.

The matching gates are `check_constant_vector_projection.py` and
`check_constant_vector_projection_proof.py`. Full integration keeps its existing
compiler/native proof/wallet/rejection/archive/package gates and adds the
bounded vector qualifications. No hosted trial was run. Complete integration
and remaining review remain OPEN; neither #674 integration nor the broad
backend goal is declared complete. The ordinary Nix compiler package must still
be checked at the final new head, separately from the development CLI cohort.

Every failed and earlier passing iteration stays retained under
`/Users/ysh/.codex/local-evidence/rust-ast-vector-projection-20261025/`. In particular,
the earlier tuple-output proof cohort is distinct from this named-result cohort.
The initially selected global parameter directory lacked built-ins. The current
run uses isolated copies of retained official material, independently reverified
against the locked provider hashes; prior stores/evidence were not repaired.

Forge remains deferred at `experiment/zkir-forge`; its scheduler remains deleted.
Per the owner, clean up task-owned worktrees and reproducible build targets after
completion and verified GitHub merge, preserving unpublished changes, needed
inputs/binaries and evidence. No worktree/target cleanup is due yet, and this
condition grants no push or merge authority.


## Integration gate correction — 2026-10-26

The ordinary offline Nix compiler package at signed `fa8e8ab17` built successfully.
Runtime archive rehearsal and exact manifest verification also passed in a new
task-owned target, preserving previous archives. Those receipts remain specific
to that source head.

The PR header-regression gate against the preserved pre-feature baseline
`860923e249f9139fb0786d9bd481830a07394ffd` found 39 new/modified files with
incomplete headers. Complete project headers now cover those files, while the
original contributor notice remains verbatim. The gate passes with no new
violations and 1,318 unchanged inherited missing-header files; this is not a
claim of full-tree header compliance. Source bodies after the header/separator
are preserved byte-for-byte.

The Compact source's line offsets changed, so its full IR and all five complete
checked-program records were regenerated by the ordinary compiler. Fresh golden
output and the 329-test Python suite (20 expected skips) pass. Runtime manifest
comments now describe the independent official ledger dependency cohort; actual
dependencies and compatibility identities are unchanged. Final compiler and
source-bound qualification receipts must be renewed for this header revision.
Forge stays deferred; complete integration/review and the broad goal stay open.

## Local integration checkpoint — 2026-10-26

The ordinary packaged compiler at `0fed4fe90` passes the complete maintained
`check_compactc_target.py --consumer --proof` command (1,469.829 seconds),
followed by the existing JavaScript ledger-v8 wallet handoff. All 256 generated
fixtures are current, all 37 pinned oracle sources pass, and the complete source
rejection gate passes. The original compiler/source-map test phase also passes
in an isolated test-enabled Nix build with the normal test dependencies and a
writable copied test cache. Production and test-enabled binary receipts remain
separate; identical whole compiler bundles are not claimed.

Together with the actual packaged-compiler vector TS/API/proof checks, runtime
archive rehearsal/verification and archive-only consumer checks, these close
the outstanding local execution gates. Earlier full Rust workspace, strict
Clippy and functional-consumer results retain their original source cohort;
the intervening header correction preserved implementation bodies and renewed
source-position metadata. The wider native suite retains each case's existing
application policy, including explicitly unbalanced smoke cases. It does not
turn every historical case into a funded default-strict application claim.

Final source self-review found a portability defect in the new TS capture.
[Node's loaded-library report](https://github.com/nodejs/node/blob/v24.14.0/src/debug_utils.cc#L370-L382)
can retain the [kernel's virtual-library SONAMEs](https://github.com/torvalds/linux/blob/v6.12/arch/x86/entry/vdso/Makefile#L32).
The old capture tried to open those names as disk files. An injected-name run
of the actual capture reproduced ENOENT before publication. Signed `548ab5801`
retains `linux-vdso.so.1` and `linux-gate.so.1` explicitly without disk-byte
claims; all other non-system report entries still require readable hashed
bytes. Fresh live TS/Rust checks pass with both known-name and unknown-missing
library controls. The injected controls are host tests, not Linux execution.
Compiler, native runtime and proof source are unchanged by this four-file
test-gate correction; their original successful receipts are not relabeled.

Current evidence is indexed under
`/Users/ysh/.codex/local-evidence/rust-ast-vector-projection-20261025/`, with
`full-native-v1/run-result.json`, `wallet-handoff-v1/run-result.json`,
`full-source-gates-v1/`, `ordinary-compiler-tests-v3/build-result.json`,
`packaged-vector-consumers-v2-report.json` and `source-self-review-v2.json`.
The self-review is by the implementing root agent, not an independent external
review. Independent review and intended Linux/hosted integration remain open;
no push, merge or publication occurred. The broader backend goal remains active
and incomplete. This slice establishes no blanket language-parity claim.

Forge remains deferred at `experiment/zkir-forge`, with its scheduler deleted
and its work/evidence preserved. Cleanup is due only after completion and
verified GitHub merge, for task-owned inactive unneeded worktrees and targets,
preserving unpublished work, needed inputs/binaries and qualification evidence.

## Publication review: fixture refresh

The complete functional JSON asset comparison remains strict in check mode.
`check_fixture_outputs.py --update` now refreshes the complete checked-program
roster as well as `lib.rs`, including first registration and removed JSON assets.
Unrelated retained files are untouched. Empty fresh program output refuses
before changing retained files. Entrypoint regression tests exercise stale
refusal, exact refresh, first registration and empty-output refusal without
running a compiler or claiming fresh proof qualification.

## Current PR delivery authority — 2026-10-27

The owner explicitly superseded the earlier local-only publication restriction.
Each meaningful Rust AST slice receives focused local validation, a documented
PR, complete required hosted CI/proof gates, independent CLI review and then
merge into `codex/rust-backend-ast`. No repeated push/merge approval is needed.
Local development uses arm64; Linux qualification remains in the full PR
Compiler Build lane. PR triggers provide that coverage without repeating the
same expensive full job for the working-branch merge push. Manual and scheduled
qualification remain available. No required test/proof gate is removed.
Forge is still deliberately deferred; no release or shared installation is
authorized. Cleanup remains after completion and verified GitHub merge.
