# Field wrappers around terminal effects (#516)

Generated from the complete `examples/rust_backend/field_return_wrappers.compact`
source with `--rust-functional-ledger8`. Only `lib.rs` formatting and the fixture
Cargo package name/dependency paths differ from compiler output. Complete IR,
origins, capabilities and all five checked programs are retained; they are not
hand-edited authorities.

The source covers inferred versus explicit Uint-to-Field conversion, nested
Uint branch joins, a lexical capture, selected Set/Counter effects, witness-view
reads after mutation, witness failures before checked subtraction, and a pure
expression control. Source constructor seeds Set key 5, Field 2 and Counter 0.

`check_field_return_wrappers.py` regenerates both targets into a new directory,
runs an external ordinary consumer against live pinned TypeScript and independent
expectations, rejects a changed TS result, and checks E0308 wrong-type/E0382
consumed-Program controls. 33 stateful cases exercise four APIs; three pure
controls use the pure API: **36 cases, 135 executions, 56 successful recorded VM
replays**. Five stateful failures compare callback prefixes and exact Rust errors;
no failed-context rollback claim is made. Successful state, neutral effects,
private outputs and query-summed cost are compared. Pure-effect and disclosure
source refusals run for both targets; strict consumer Clippy is required.

`check_field_return_wrappers_proof.py` independently regenerates and matches the
complete linked IR/programs, rejects changed IR before keys/state, and qualifies
two selected `inferred(true, key)` calls: replacement key 5 and new key 6. It uses
fresh native keys and proofs, immutable verification, binding/result negatives,
funded default-strict application and replay refusal without mutation.
Constructor execution, unselected branch and other exports are not proved by
this gate. Full CI remains required before integration.
