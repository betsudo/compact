## The next experiment

With checked effects available, the team explored an immutable Ledger 8 program request interpreted through shared execution logic. Native and recorded target adapters could consume the same admitted request while retaining their different output responsibilities.

This aimed to reduce the amount of contract-specific generated control flow. It also made the programme's operation inventory available for review.

An illustrative consumer shape is shown below. Constructing the immutable request performs no effects. Executing it against a context performs the admitted programme.

~~~rust
let request = replace::Program::new(next);
let recorded = request.execute_recorded(context)?;
~~~

The generated module and parameter types depend on the contract. The example shows the ownership boundary and is not a standalone runnable contract.

## Keep metadata out of the execution body

Review metadata is useful for auditing a generated programme. Large strings and inventories can also dominate an otherwise small Rust file.

The follow-up separated metadata assets while retaining stable public access, exact identities and execution behavior. That made generated code easier to read without erasing the material needed for review.

## Witnesses and effects still need order

An immutable description does not make execution order irrelevant. Witness callbacks can return values and update private state. Ledger operations observe and change public state. A Unit-valued call may still have an essential effect.

Later slices preserved ordered Unit initializers, terminal effects, tuple returns and named bind receivers. Each one corrected a boundary where a superficially small expression could lose behavior during lowering.

## From the prototype to the working branch

The October 18 local prototype checkpoint was documented at ec7262e49 with an explicit experimental flag and Ledger 8.1.3 scope. The historical note records 3,299 workspace passes, seven manual ignores, and generated scenarios across four routes.

The working branch integration c353907d8c323db16eedb35d5a7f0151aab859bb records that transition on October 27. Earlier typed effects, witnesses and collection work from October 18–19 was included in the reviewed baseline. Subsequent PRs added Fold, Map size, projections, DID and SDK consumers.

## What was better

The programme model made native and recording behavior easier to relate. The generated API could express a request whose operations had already passed semantic checks, with review material outside its main execution text.

## What remained

The broader functional scope was a proposal with its own acceptance work. A narrow prototype result could not silently establish the entire roadmap. Source admission and proof coverage continued as independent deliveries.

### Source trail

- [Working-line integration](https://github.com/betsudo/compact/commit/c353907d8c323db16eedb35d5a7f0151aab859bb)
- [Metadata ownership #665](https://github.com/betsudo/compact/issues/665)
- [Ordered Unit initializers #694](https://github.com/betsudo/compact/pull/694)
- [DID lifecycle #693](https://github.com/betsudo/compact/pull/693)
