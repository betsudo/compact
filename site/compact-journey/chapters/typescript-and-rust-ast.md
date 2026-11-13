## Why add a Rust AST backend?

The Rust AST backend lets a Rust application call generated Compact code through types that preserve more of the contract’s meaning. A bounded integer carries its limit. A fixed Vector carries its element type and length. A circuit call consumes a context and returns the next one. These choices make several host-side mistakes visible while compiling the application.

The compiler also gains a structured way to build Rust. It checks a typed internal program, lowers admitted operations and constructs Rust syntax trees. That gives contributors explicit places to implement a language feature and diagnose unsupported cases.

The TypeScript backend supplies generated contract interfaces, witness callbacks, ledger access and proof data for JavaScript applications. It has typed declarations and runtime checks of its own. The comparison below follows the implementations and tests retained in the November 13, 2026 book edition.

## How the two paths work

<div class="backend-comparison" markdown="1">
<section markdown="1">

### TypeScript backend

~~~text
Compact source
  → shared frontend checks
  → TypeScript-oriented lowering
  → JavaScript + TypeScript declarations
  → Compact TypeScript runtime
  → ledger operations and proof inputs
~~~

The application imports a generated Contract class and calls a method such as `contract.circuits.increment_by(context, 7n)`. The result contains the successor context, return value, gas and proof data.

This path fits applications already using Node.js, TypeScript and the existing JavaScript SDK integration.

</section>
<section markdown="1">

### Rust AST backend

~~~text
Compact source
  → shared frontend checks
  → versioned typed IR
  → checked semantic lowering
  → structured Rust syntax
  → generated crate + typed runtime
  → ledger operations and proof inputs
~~~

The application imports generated Rust functions or contract wrappers. A call such as `increment_by(context, amount)` returns a `Result` containing the execution result and successor context.

This path fits Rust services and tools that need generated Compact APIs in the same language as their application.

</section>
</div>

Both paths begin with Compact’s frontend checks. Source typing and disclosure rules apply before host code is emitted. Rust’s generated interface adds checks at the application boundary, where developers supply arguments and connect calls.

[Read the checked Rust pipeline](a-closed-rust-boundary.html) · [Read the TypeScript reference workflow](typescript-and-fixed-ground.html)

## Benefits for application developers

### Keep contract meaning in the argument types

Consider the same Counter contract used in the walkthrough. Its amount parameter is Compact `Uint<16>`, which admits values from 0 through 65,535.

| Surface | Generated argument | What the caller must supply |
| --- | --- | --- |
| TypeScript | `bigint` | A `bigint`. The generated runtime checks the value’s range. |
| Rust AST | `BoundedUint<65535>` | A value constructed through the bounded carrier’s checked API. The function signature preserves that bound. |

~~~rust
let amount = BoundedUint::<65_535>::new(7)?;
let increment = increment_by(context, amount)?;
~~~

The Rust compiler refuses a plain `u16` in this position. It also distinguishes carriers with different declared bounds. The integer’s value is checked when the bounded value is constructed. For example, `new(65_536)` returns an error.

TypeScript already rejects a `number` where the declaration expects `bigint`. Rust makes the contract’s bound part of the parameter type as well.

### Make the state handoff explicit

A sequence of calls needs to continue from the returned context. The Rust call takes the context by value, so the compiler tracks which binding has been consumed.

~~~rust
let increment = increment_by(context, amount)?;
let reset = reset_round(increment.context)?;
~~~

Trying `reset_round(context)` after the increment produces `E0382`, “use of moved value.” In the TypeScript interface, the original and returned contexts have the same type and the caller selects which one to pass. Rust’s ownership rule catches reuse of that consumed binding before the application runs.

[Open the complete Counter example and the locally exercised diagnostics](one-counter-two-backends.html)

### Preserve values with useful Rust types

| Compact concept | Rust interface benefit |
| --- | --- |
| `Field` | A field carrier gives field arithmetic and representation a named owner. |
| Uint with a bounded range | `BoundedUint` carries the admitted maximum in its type and checks construction. |
| Vector with a fixed length | `FixedVector` carries the element type and length. Consumers can use ordinary Rust type checking for both. |
| Ledger declaration | A generated typed slot connects a declaration to its ledger path and read/write operations. |
| Fallible circuit or witness work | `Result` and the `?` operator make error propagation explicit in the generated call path. |

TypeScript declarations can also express fixed tuples, products and typed method names. The Rust benefit is the combination of domain carriers, Rust ownership and ordinary compiler checking throughout a Rust consumer.

The Ledger 9 cryptography work extends the same approach to distinct base/scalar and point carriers. Runtime decoding validates external representations before they enter typed operations.

[Read the generated types and typed slots](atlas-06-generated-rust-examples.html) · [Read the cryptographic carriers](porting-the-cryptographic-surface.html)

### Integrate with a Rust application directly

The backend emits a Cargo crate, matching runtime material and generated entry points. A Rust service can call the contract, inspect typed state and handle errors through its existing Rust code.

Native execution and recorded execution expose the outputs needed for different steps in the application. Recording retains the ordered public operations for replay and proof preparation. The Counter walkthrough shows the smallest local call, and the recording chapter follows the later transition to an installed operation.

[Read recorded execution and application handoff](recording-is-an-execution.html)

## Benefits for compiler contributors

### Build syntax as a tree

The Rust renderer constructs expressions, statements, blocks, paths and types using Rust syntax-tree libraries. The formatter renders those structures into readable source.

For example, a multiplication whose left operand is an addition is represented as a nested expression. Parentheses follow from that structure. A contributor can inspect the checked operation and the syntax node that represents it.

[See the nested calculation before and after](the-first-rust-emitter.html#before-and-after-emitting-a-nested-calculation)

### Give each semantic decision an owner

A feature crosses the source frontend, typed IR, validator, lowerer, renderer and runtime. Explicit boundaries make the responsibilities reviewable. Unsupported input can produce a diagnostic while its source and operation are still identifiable.

The checked operation model also supports composition. A read, assertion or write can appear inside a branch or sequence. A shared semantic fold handles admitted behavior through native and recorded adapters, reducing duplicated decisions about ordering and results.

[Explore the shapes and operations domain models](operations-instead-of-shapes.html)

### Test ordinary use and misuse

Generated-crate tests exercise the same entry points an application imports. Compile-fail cases check wrong element types, wrong Vector lengths and consumed contexts. Behavioral comparisons check return values, ledger state, witness order and gas against an independent reference. Applicable proof lanes then check the statement, verifier, funded application and replay.

These checks address different failure modes. A type-correct Rust expression can still select the wrong ledger slot or operand, so semantic and proof tests remain part of delivery.

[Follow the verification stages](a-proof-is-one-stage.html)

## What the benefits mean for this migration

Rust AST is useful when the application is written in Rust and benefits from domain types, ownership, native library integration and explicit errors. TypeScript remains a direct fit for JavaScript applications and their SDK workflows.

Ledger 9 is a protocol migration that changes package families, ABI, serialization and transaction behavior. The typed Rust interface existed on Ledger 8 too. The Counter comparison uses Ledger 8 TypeScript and Ledger 9 Rust to show the journey while retaining each runtime’s own protocol pairing.

The current edition records a merged Ledger 9 foundation and selected local arm64 compatibility results. Full-corpus qualification, native Linux results for later local ports and generated joint two-contract execution have the open scopes described in their chapters. The benefits described here come from the generated APIs, compiler architecture and exercised controls. Performance depends on the workload and requires its own measurements.

[Read the Ledger 9 migration](ledger9-is-a-protocol-migration.html) · [Read the current compatibility boundary](the-next-reader-inherits-a-map.html)

### Source trail

- [Generated Ledger 9 Counter interface](https://github.com/betsudo/compact/blob/05912594fa3c483cf4e7031de83c5f9a103ef95c/tests-rust-backend/counter-parameter/lib.rs)
- [Bounded integer carrier and checked construction](https://github.com/betsudo/compact/blob/05912594fa3c483cf4e7031de83c5f9a103ef95c/runtime-rs/src/primitives.rs)
- [Generated Vector wrong-type control](https://github.com/betsudo/compact/blob/05912594fa3c483cf4e7031de83c5f9a103ef95c/tools/compact-rust-backend/tests/vector_projection/wrong_type.rs)
- [Generated Vector wrong-length control](https://github.com/betsudo/compact/blob/05912594fa3c483cf4e7031de83c5f9a103ef95c/tools/compact-rust-backend/tests/vector_projection/wrong_length.rs)
- [Generated context ownership control](https://github.com/betsudo/compact/blob/05912594fa3c483cf4e7031de83c5f9a103ef95c/tools/compact-rust-backend/tests/vector_projection/double_context.rs)
