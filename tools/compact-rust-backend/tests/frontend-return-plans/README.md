# Frontend ordered-return regression sources

`frontend_return_plan_gate.py` compiles these sources with two explicit Scheme
frontends and one identical Rust backend. It checks fresh IR, native compilation
and reported recording capabilities. Runtime and proof receipts are separate.

- `storage_mutations`: nested branches and lexical bindings; storage update,
  remove and coin-check classes. These use existing StateAction emitters. Native
  compilation does not imply algebra support for Set or coin operations.
- `value_effects`: assertions, calls, witness and storage reads keep their existing
  expression representation and recording availability.
- `checked_subtraction`: a compiler-inserted subtraction guard remains part of
  one checked arithmetic expression after promotion to an ordered return plan.
- `examples/rust_backend/effect_plan_counter_only.compact`: the source-bound
  Counter runtime/proof fixture has exactly two Counter fields and no extra Cell.

Example (supply actual local tool paths):

```sh
python3 -B tools/compact-rust-backend/frontend_return_plan_gate.py \
  --compiler /path/to/compactc \
  --scheme /path/to/candidate-compactc-scheme \
  --baseline-scheme /path/to/baseline-compactc-scheme \
  --output /tmp/frontend-return-gate \
  --source-manifest /path/to/retained-source-census.json
```

The optional census reads `rows[].source`; it recompiles each source and never
substitutes previously retained IR. Each baseline must succeed. Accepted native
compilation and recorded/observed-call availability must not decrease.

## Accepted wrappers and the next lexical boundary

The scalar `cast_wrapped_mutation.compact` and tuple
`tuple_cast_wrapped_mutation.compact` are maintained positive regressions since
#691/#692. Their nonfailing result conversions preserve the selected storage
actions and original result types. They are accepted work, not remaining gaps.

`unit_mutation_initializer.compact` covers the ordered Unit-valued Set insertion
initializer admitted by the #516 follow-up. Its complete source cohort is
`examples/rust_backend/unit_mutation_initializers.compact`. The maintained API
and separate proof gates carry execution and selected-proof evidence; source
generation alone makes neither claim.

Unit initializers nested in statement-only blocks, Unit-returning circuits with
a later conditional mutation, Cell-specific lexical redesign, non-Unit mutation
initializers and arbitrary
result conversions remain outside this bounded follow-up. Existing checked
lowering, resource limits and capability refusals remain in force.
