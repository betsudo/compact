# Constant-valued Vector indices

Status: reconciled local version 0.31.144 qualification passes at `3fa7cec1d`.
Own PR review, full CI and integration remain pending. Delivery target: `codex/rust-backend-ast`.
The isolated witnessed SDK #698 is verified merged at `d3a32d9e7`; the local
Vector branch reconciles that accepted canonical source without rewriting history.

## Gap and accepted baseline

The Compact reference permits constant-valued Vector indices, including local
constant aliases and addition, subtraction and multiplication. The ordinary
compiler validates these indices, but Rust starts from the earlier analyzed
program and currently lowers only literal `tuple-ref` indices. Valid sources
therefore compile to TypeScript and refuse Rust emission.

Accepted literal projection, owned Vector reuse, typed recording and public
replay remain the baseline. Their eight focused backend projection tests pass
on the clean accepted parent. The historical 0.3.0 acceptance crosswalk and
recording retirement ledger remain accepted evidence; this slice does not reopen
their retired recognizers or claim complete language parity.

## Owners and intended API

`compiler/circuit-passes.ss` retains authority for constant evaluation and bounds.
A small per-invocation bridge will observe its successful Vector index results.
`compiler/passes.ss` owns that bridge's lifetime. `compiler/rust-ir-passes.ss`
consumes a result only for the corresponding original analyzed expression.

The bridge must distinguish original expression identity and lexical scope from
source coordinates. An absent, duplicate or conflicting occurrence must refuse;
inlined or unrolled copies cannot overwrite a different resolved value. Initial
admission is restricted to the reference-defined local constant grammar. It does
not infer a constant from a call, a runtime formal parameter or a conditional.
Generic and loop forms must be characterized explicitly before admission.

Rust retains the Vector operand once and selects the validated position through
existing `tuple_index` IR and structured AST emission. General CTV knowledge is
insufficient to erase index evaluation. This bridge additionally requires a
closed, effect-free constant grammar with no runtime formal, call or conditional.
Its unsigned subtraction guard must match the exact subtracted operands. The
existing circuit evaluator must successfully resolve the result and bounds.
Within this narrow grammar there is no remaining index effect or possible
failure to erase; original local bindings retain their evaluation order. Forms
outside it refuse even when the general CTV owner infers a number. No runtime ABI,
schema or public indexing helper is intended.

Untyped small unsigned arithmetic operands retain the exact literal maximum
assigned by `infer-types`, including cases without an inserted widening cast.
This is needed for retained local constant bindings such as `const i = 1 * 2`.
Stateful typed arithmetic also retains the declared result maximum, matching the
pure owner, rather than widening it to the rounded bit carrier. This preserves
valid bindings such as `const i = 0 + 0`, whose declared maximum is zero.

Public arguments remain `FixedVector<T, N>` and selected values retain `T`.
Functional Programs and contexts remain consuming. Owned elements and repeated
selection preserve the accepted cloning/ownership rules.

## Scope exclusions

No Bytes indexing, slices, updates, dynamic indexing, new constant evaluator,
whole-program normalization, new semantic graph, broad refactor or CI optimization.
Do not pass the transformed circuit program into the Rust emitter. Forge remains
deliberately deferred, incomplete and independent of backend progress.

## Acceptance

1. Preserve literal first/last/singleton, tuple projection, owned reuse and existing
   independent Vector plan-validator controls.
2. Compile complete alias/arithmetic sources with the original updated compiler.
   Cover chains, first/last positions, unsigned subtraction, lexical shadowing,
   outer captures and owned repeated/temporary selections. Characterize generic
   naturals and loop indices with honest refusals for excluded forms.
3. Compare live TypeScript and applicable ordinary/functional native/recorded
   routes: full output, state, private output/state, effects, query-summed gas,
   witness order, first failure and successful public replay.
4. Preserve frontend runtime-index, out-of-bounds, empty and wrong-type refusals.
   Retain wrong ordinary Rust Vector length/type and consumed Program/context
   compile-fail controls, with positive twins and intended diagnostic spans.
5. Check ambiguous/missing/conflicting index facts, source occurrence identity,
   scope and same-type operand/index mutations independently. Well-typed private
   IR alone does not establish source equivalence.
6. Generate fresh selected Field keys and genuine proofs with immutable verifier,
   binding/input/output and proof-byte negatives. Separately prove funded strict
   transactions, exact final state and unchanged replay refusal. Label pure and
   non-proof cases by actual applicability.
7. Iterate with focused arm64 checks, then retain original compiler compatibility
   and complete required Linux/native/proof/package/archive/E2E CI gates. Actual
   Claude CLI PR review, signed Conventional/DCO/Assisted-by commits, normal merge,
   GitHub verification and guarded cleanup of completed owned work remain required.

## Evidence boundaries

The baseline source characterization observed a finite gap; it is neither an
absence proof nor a proof qualification. A local bridge prototype is not accepted
until genuine generated programs, consumers, negative controls and proof gates
pass. Source metadata identifies an occurrence; it is not compiler attestation.

## Local development checkpoint

The original compiler characterization covers fifteen complete sources (both
targets). Literal generic indices already work. The development bridge also
emits alias, addition, subtraction, multiplication, chained and shadowed indices,
plus an unambiguous generic arithmetic instantiation. Loop indices retain the
existing named-Fold refusal. Multiple instantiated occurrences sharing one
original source object are conservatively ambiguous and require a separate
transport decision; do not advertise unrestricted generic or loop coverage.

Seventeen independent internal controls cover missing facts, exact original-node
lookup, repeated observations, identical coordinates on distinct source objects,
equal-looking distinct nodes, conflicts, invocation isolation, duplicate source
occurrences, runtime formals, an unrelated false subtraction guard and inferred
zero from runtime data. They do not claim hostile-host attestation.

`examples/rust_backend/constant_valued_vector_indices.compact` keeps the complete
maintained projection consumer API and changes its indices to genuine aliases
and arithmetic. The existing gate's explicit `--source` option executes this
source without editing generated code. Its independent seven Rust consumer
tests exercise all four applicable routes, owned reuse, witness order/failure,
state, private outputs, gas and replay. The live TypeScript capture contains five
pure and seven stateful cases and three refusals; a changed TS result must fail
the Rust comparison. Wrong length/type and repeated execution retain E0308,
E0308 and E0382 with ordinary positive execution alongside them.

The first development consumer build hit ENOSPC. After three obsolete test
executables from verified merged predecessor slices were reclaimed under source,
identity and open-file guards, the remaining consumer and negative commands
passed against independently reauthenticated retained fresh generation and TS
observations. The failed attempt remains failed evidence. This is development
frontend execution and authenticated generation reuse, not a clean original
compiler-package, fresh proof or full integration acceptance claim.

The subsequent clean signed implementation `a1f0fa8b1` built successfully as an
original native compiler package, with no Scheme override. Its fresh complete
new-source gate also passes: all seven generated-consumer tests, live TS
comparison, changed-result refusal and three intended compile-fail controls.
These are literal a1 execution receipts. That execution used local compiler version 0.31.142 before SDK reconciliation
and version advancement; it does
not publish or relabel a historical compiler release. Development failures and
authenticated-reuse receipts remain separate historical evidence.

The later signed proof-binding source `9a9d1b8b6` completed the separate
new-source and retained literal proof gates described below. The ownership
controls and accepted SDK reconciliation are now complete. Next: qualify the
final version 0.31.144 source, then complete PR review and full CI before integration.
Any changed compiler/source identity receives its required qualification. Existing
literal proof coverage remains required and does not substitute for new-source
proofs. The broader backend goal and this slice's integration remain incomplete.

## Separate new-source proof qualification

`tests-rust-backend/constant-valued-vector-indices-source` retains the complete
original-compiler IR, formatted-only Rust and all five checked programs. It has
its own source registration and Cargo package. The additive declaration inventory
adds exactly this source's twelve identities; historical authorities remain
unchanged. Fresh comparison passes for both old literal and new source fixtures.

The direct proof runner keeps `--constant-vector-projection` and adds
`--constant-valued-vector-indices`. Thin modules bind each source's separate
fixture and whole IR to one shared proof protocol. The maintained proof gate's
`--constant-valued-indices` option selects the new source explicitly; default
behavior retains the literal source. Full proof orchestration and Linux CI
require both receipts, with their source labels checked independently.

Before keys or state, the gate requires complete linked Rust/IR/program equality
and source-bound refusal of a changed full IR and a genuinely in-range same-type
index change. Each selected branch requires a fresh proof, verification,
changed binding/input/result and proof-byte refusals, plus a separately proved
funded default-strict transaction, exact final state and unchanged replay refusal.
Receipt pass flags cannot replace these fields. Constructor execution remains
unproved. Existing official material is verified before the new runner build;
the gate separately verifies its actual selected parameter size.

Source/receipt/orchestration controls and strict selected proof-profile Clippy
pass for the new fixture and shared runner. At clean signed `9a9d1b8b6`, both separately selected gates passed with fresh
k=6/54-row keys and two 2912-byte standalone proofs per source. Each branch
also completed a separately proved funded default-strict ledger application,
exact expected state and unchanged replay refusal, with the stated crypto and
whole-IR/index negatives. These are bounded arm64 receipts; they do not claim
constructor execution, Linux execution or full integration acceptance.

The full ordinary compiler `./compiler/go` passed at the same head in the
standard default Nix development shell: all Scheme groups and both JavaScript
modes. An earlier lean-shell attempt could not find Vitest; its failure and
overwritten-input preservation remain separate. No source fix was required.

The current API gate adds a reused-context E0382 refusal alongside reused Program,
wrong length and wrong element type. Every primary error must identify the
marked misuse in its own external binary. A positive twin executes independent
Programs with fresh contexts and checks separate cells against 17 and 41,
bringing the consumer roster to eight. The historical a1 receipt above remains
seven tests/three controls and is not relabelled by these additions. Compiler,
consumer, proof, application, final source review and integration receipts retain
their separate identities and scope.

## Ownership-control qualification

At clean signed `e2b12807c`, fresh complete native generation and the live
TypeScript comparison pass for both selected sources with all eight external
Rust consumer tests. All four compile-fail binaries fail at their intended
primary source spans: E0308 for length/type and E0382 for Program/context reuse.
The complete public Python gate runs 399 selected methods, with 380 fully
exercised parent methods, 20 expected manual-only skip events and no failures
or unexpected skips.

The compiler, backend emitter, both runtimes, complete source fixtures and proof
runner/protocol trees are unchanged from the literal `9a9d1b8b6` proof epoch.
This is a checked owner bridge, not new proof execution. Final reconciled source
qualification, external PR review and complete CI remain required.

## Packaged compiler shape matrix

At clean signed `8aaa30b42`, the original native package also compiled the
complete fifteen-source matrix for both targets without a Scheme override.
Nine positive Rust sources produced unedited crates and passed nine external
Rust tests, covering ten independently expected observations also executed in
TypeScript. These include alias chains, unsigned arithmetic, both lexical
shadowing/outer-capture branches and the characterized unambiguous generic
literal/arithmetic cases.

Runtime indices, out-of-bounds arithmetic, underflow, empty vectors and Boolean
indices refused both targets at their intended frontend diagnostics before
output publication. The loop case still compiles to TypeScript and retains
the named-Fold Rust refusal; it is not admitted by this slice. These are pure
source cases, without a ledger/proof applicability claim. The first matrix
harness incorrectly supplied the Rust-only runtime-root option to TypeScript;
that retained failed attempt is not a compiler regression or a passing result.

This completes the packaged-compiler characterization gap without changing
compiler, runtime, fixture or proof owners. Final reconciled-source qualification
and full review/integration gates remain open.

## Accepted SDK reconciliation

SDK #698 passed all eleven required GitHub checks and actual Claude review.
The exact same-run runtime archive passed GitHub digest, safe ZIP/membership,
source and protocol verification before the normal signed canonical merge at
`d3a32d9e7`. The completed SDK worktree is recoverably archived after preserving
all ignored review evidence. Its E2E summary reports 480 passed and six skipped
unique tests, with no failures; skips are not counted as passed.

The Vector branch merged that accepted canonical commit normally at
`e24e5eea0`, producing the previously previewed conflict-free tree. Version144
advances the compiler, Nix package, current ledger reference and changelog
together. All prior compiler/API/crypto receipts retain their literal source
epochs; final-source qualification below must precede this slice's publication.

## Current reconciled version144 qualification

At clean signed `3fa7cec1d`, original native compiler 0.31.144 realization,
both complete fresh generated-consumer gates and exact whole-fixture comparison
pass without a Scheme override or normalization. Each source passes eight Rust
consumer tests, live TS observations and all four intended-primary-span type/
ownership refusals. The packaged fifteen-source matrix also passes its expected
both-target outcomes and nine actual Rust tests/ten TS-matched observations.

The complete public Python gate runs 411 selected methods, 392 fully exercised
parent methods and twenty expected manual-only skip events, with no failures,
errors or unexpected skips. Both separately selected proof gates generate fresh
k=6/54-row keys, two 2,912-byte standalone proofs each and separately proved funded
default-strict transactions with exact expected state and unchanged replay
refusal. Whole-IR and same-type in-range index changes refuse before keys/state;
binding/input/result/proof-byte negatives pass. Constructors remain unproved.

A separate root audit rehashes 2,861 source, 20 material, 19 generated and seven
result files per proof gate, all four snapshotted components and their source
binaries, command logs and seven upstream fixture inputs. Complete original IR
and all five checked programs match each distinct linked fixture. That audit is
not an external review or a new cryptographic execution. Final descriptive
Markdown changes retain an explicit source bridge; own Claude PR review and the
complete Linux/native/proof/package/archive/E2E CI gates remain mandatory.

The Vector protocol retains standalone proofs, preimages, final-state bytes and
runner logs/results. Sealed transaction bytes are not separately retained;
strict application and replay are exercised in the recorded runner execution.


## Review repairs — 2026-10-29

The actual PR #699 Claude review exposed valid constant arithmetic forms missing
from the occurrence grammar. Compound subtraction uses analyzer-created scoped
temporaries. Its comparison width is the larger original operand bound, while
its subtraction width follows the left operand. The bridge now checks both
widths, exact operand identities, cast bounds and the canonical assertion message.
Independent forged-width, message, operand and checked-downcast controls refuse.

Unsigned safe-cast annotations are erased by the authoritative circuit pipeline;
they can narrow for multiplication by zero. They are distinct from checked
source downcasts. Arithmetic emission retains the original carrier under a
narrowing arithmetic annotation and uses existing checked runtime arithmetic,
preserving operand evaluation and failures rather than prematurely rejecting a
valid value. Runtime-dependent indices remain excluded, even for multiplication
by zero. The maintained complete source exercises these cases in pure and
stateful routes, including the witnessed circuit selected for real proofs.

The same-type in-range IR mutation now changes the selected witnessed projection.
It remains an exact whole-IR binding refusal before keys or state, not an
independent semantic verifier. Linux CI also found that the test helper name
`collect` conflicts with the newer Scheme import; it has a specific local name.

Historical qualification receipts above remain evidence for their exact source
epochs. These source repairs require a new original packaged compiler, complete
fixture comparison, native/TS and API controls, fresh keys/proofs and funded
transaction checks, full PR CI, and another actual Claude CLI review before
integration. Development override execution is labelled separately throughout.
