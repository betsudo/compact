# Constant-valued Vector indices

Status: local implementation and qualification in progress. Delivery target:
`codex/rust-backend-ast`. This slice follows the isolated witnessed SDK consumer;
its publication waits for that slice's acceptance and canonical reconciliation.

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
These are literal a1 execution receipts. The local compiler version remains
0.31.142 pending normal SDK reconciliation and version advancement; this does
not publish or relabel a historical compiler release. Development failures and
authenticated-reuse receipts remain separate historical evidence.

Next: bind complete new-source IR and checked programs into the selected proof
consumer, run genuine proofs and funded applications, reconcile against the
accepted SDK branch, and finish final-source review/full CI before integration.
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
pass for the new fixture and shared runner. Actual new-source proof and ledger
execution are still open until their retained run succeeds. Compiler, generated
consumer, proof, application, final source review and integration receipts retain
their separate identities and scope.
