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
