# Unit-valued mutation initializers

Compact's empty tuple `[]` has one value. Evaluating an expression that returns
it can still write ledger state, call witnesses or fail. The Rust backend must
preserve that evaluation before introducing the lexical binding.

```compact
import CompactStandardLibrary;
export ledger values: Set<Field>;
export circuit conditional(selected: Boolean, value: Field): Field {
  const inserted = disclose(selected) ? values.insert(disclose(value)) : [];
  return inserted == [] ? 1 : 0;
}
```

The selected insertion happens exactly once. The unselected arm performs no
insertion. An unused binding still executes its initializer. A later witness
reads the updated state, and a failing earlier witness prevents later callbacks.
Bindings retain lexical scope and shadowing. Users need no helper circuit to
express this supported source form. Initializers also work in statement-only blocks and before later conditional
or unconditional mutations in Unit-returning circuits. These scopes use the
same ordered lexical binding owner as return-producing scopes. Source scopes
still apply: a binding cannot refer to itself in its initializer.

```compact
export circuit statement_field(selected: Boolean, value: Field): Field {
  if (disclose(selected)) {
    const inserted = disclose(selected) ? values.insert(disclose(value)) : [];
  }
  return 1;
}
```

The unused `inserted` still performs the selected insertion. Returning `1` does
not erase the effect, and an unselected block does not execute its witnesses.

## Ordinary typed APIs

The generated ordinary method keeps the source argument/result types:

```rust,ignore
use generated::{ledger_contract as c, runtime::Field};
let output = c::conditional(context, true, Field::from(5_u64))?;
assert_eq!(output.result, Field::from(1_u64));
```

With `--rust-functional-ledger8`, the same supported circuit also has a typed,
consuming Program:

```rust,ignore
let program = c::functional::conditional::Program::new(true, Field::from(5_u64));
let output = program.execute_recorded(context)?;
```

Wrong ordinary argument/result types fail Rust compilation. Executing the same
Program twice remains a moved-value error. See [ownership](ownership.md) for the
existing consuming API, and [checked recording](checked-recording.md) for
validation at private IR and runtime boundaries.

## Comparisons do not skip evaluation

For actual checked Unit operands, `left == right` is true and `left != right`
is false **after both operands have evaluated successfully**. The generated
structured Rust AST evaluates left once, then right once, using typed Unit
discards and a Boolean result. It does not discard witness calls or assertions.
A left failure prevents the right operand from executing. Both operands still
receive static type, scope and resource checking, including inactive branches.
This is specific to checked Unit, not a rule for arbitrary empty carriers or
unknown values.

## Evidence and boundaries

The complete source cohort is
`examples/rust_backend/unit_mutation_initializers.compact`. The maintained
`check_unit_mutation_initializers.py` gate generates unedited Rust/TS, original
IR/origins/capabilities and all fifteen checked programs. It compares independent
expectations and live TypeScript with native, recorded and both functional
routes, including full state, effects, private outputs, cost and recording
replay. Pure comparisons use the existing pure API. Rust compile-fail,
both-target source refusal and changed-oracle controls are separate checks.
Both maintained gates require a clean checkout before and after execution,
retain literal source/tool identities and write failed receipts on exceptions.

The separate `check_unit_mutation_initializers_proof.py` gate qualifies only the
selected `conditional` and `statement_field` calls for insertion and
reinsertion. Standalone proofs and
independently proved funded transactions are distinct retained artifacts. The
constructor, prestate setup, unselected branch and other exports are not proved
by those selected calls. Unit-witness-only exports remain nonproof according to
the frontend; recordability is a separate dimension. See
[capabilities](capabilities.md).

This bounded slice does not admit arbitrary non-Unit effectful initializers,
general result casts, new runtime/IR types or a Cell lexical redesign. Existing
resource limits and capability refusals remain in force. The
[statement-action plan](../plans/516-unit-action-bindings.md) records development
results separately from final clean compiler, review and CI acceptance.
