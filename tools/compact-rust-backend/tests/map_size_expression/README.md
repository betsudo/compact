# Map size expressions (#610)

The genuine source is `examples/rust_backend/map_size_expression.compact`.
Its complete generated Rust crate, private IR and eight unmodified checked
programs are retained in `tests-rust-backend/map-size-expression-source`.
Formatting Rust is the only generated-source transformation.

`check_map_size_expression.py` compiles that source afresh and executes an
external consumer of the unedited crate. Eight tests cover independent expected
results, ordinary native/recorded and functional native/recorded execution,
public query replay, state/effects/private outputs, actual summed cost, Map
replacement versus insertion, whole nested-Map metadata, lexical reads,
query/witness order, inactive branches and first failures. Wrong ordinary
arguments and consuming a Program twice must fail Rust compilation. Pure
ledger queries and an incorrectly declared return type must fail at the source
boundary before output publication.

With `--ts-runtime`, the gate executes the actual generated TS wrapper against
that exact runtime package. It retains the runtime dependency closure, Node
binary and actual external libraries, verifies them before and after, and checks
19 successful cases, four refusals and two inactive malformed-state controls.
The source has only Map effects: Rust's typed effects must equal their neutral
value and the TS effect collections must each be empty. Other values, query
costs, private outputs and complete known Map entries are compared directly.
An altered TS result must fail the genuine Rust consumer.

Host-supplied Arrays in declared Map slots are invalid state. The underlying TS
VM can nevertheless take their size. Rust deliberately rejects this shape at
its existing Map metadata boundary before a later witness. An inactive size
query performs neither validation nor a query. This stronger malformed-state
rejection is separate from valid-state TS parity. Nested Map values are not
decoded as scalar cells.

`check_map_size_expression_proof.py` is the separate proof gate. It binds the
complete generated IR and all eight checked programs, selects fresh direct
`put` keys, verifies locked official proof material and proves both replacement
and new insertion. It independently verifies each proof, rejects changed public
binding and same-geometry result, and requires a fresh funded transaction with
default strict verification, successful application and unchanged replay
refusal. Constructor execution and nested-ADT proofs are not claimed. No proof,
verification verdict or transaction is reused.

These focused gates supplement the complete native, compiler, refusal, package,
wallet and CI integration gates. Local passes do not establish hosted or Linux
qualification. Forge is deferred and is not a prerequisite.
