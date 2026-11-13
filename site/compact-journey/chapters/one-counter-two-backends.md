## One source file

Start with a public counter at zero. Call `increment_by` with 7, read the counter, then reset it. The source also exports `decrement_by` so a caller can subtract from the counter. The file is byte-for-byte identical at the two source revisions linked below.

~~~compact
import CompactStandardLibrary;

export ledger round: Counter;

export circuit increment_by(amount: Uint<16>): [] {
  round.increment(disclose(amount));
}

export circuit decrement_by(amount: Uint<16>): [] {
  round.decrement(disclose(amount));
}

export circuit reset_round(): [] {
  round.resetToDefault();
}
~~~

`Uint<16>` admits 0 through 65,535. The `[]` return is Compact’s Unit value. The calls change ledger state, so the examples read the new value from the returned context.

## Build the two interfaces

Use each compiler with its own ledger runtime. The Ledger 8 run used Compact 0.31.116 with the 0.16.102 TypeScript runtime and Onchain Runtime 3.1.2. The Ledger 9 run used Compact 0.31.211 to generate Rust for ABI 54 and Rust 1.99.0 to build the caller and Onchain Runtime 4.0.1.

~~~sh
compactc-ledger8 --target ts --skip-zk counter_parameter.compact ledger8
compactc-ledger9 --target rust --skip-zk counter_parameter.compact ledger9
~~~

Here `compactc-ledger8` and `compactc-ledger9` are local names for the corresponding compiler executables. Put the TypeScript example beside the generated contract directory and resolve the matching `@midnight-ntwrk/compact-runtime`. In Rust, add the generated crate as a path dependency. The caller below uses the same dependency name as the repository fixture. Both examples create a local context. They exercise the generated APIs without deploying a contract or producing a proof.

From a Rust consumer manifest alongside the output directories, the dependency is

~~~toml
[dependencies]
compact_rust_counter_parameter_fixture = { package = "compact-contract-counter-parameter", path = "ledger9/contract" }
~~~

## Call the same circuit

Both callers pass the context returned by the increment into the reset. They see the same public counter values, 7 and then 0.

<div class="backend-comparison" markdown="1">
<section markdown="1">

### Ledger 8 · TypeScript

The generated method takes a `bigint`. The ledger helper projects the counter from the new context.

~~~typescript
import {
  createCircuitContext, dummyContractAddress,
  emptyZswapLocalState,
} from "@midnight-ntwrk/compact-runtime";
import { Contract, ledger } from "./contract/index.js";

const contract = new Contract<null>({});
const coinKey = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: emptyZswapLocalState(coinKey),
});
const context = createCircuitContext(
  dummyContractAddress(),
  coinKey,
  initial.currentContractState.data,
  initial.currentPrivateState,
);

const increment =
  contract.circuits.increment_by(context, 7n);
const after = ledger(
  increment.context.currentQueryContext.state,
);
console.log(after.round); // 7n

const reset =
  contract.circuits.reset_round(increment.context);
const cleared = ledger(
  reset.context.currentQueryContext.state,
);
console.log(cleared.round); // 0n
~~~

</section>
<section markdown="1">

### Ledger 9 · Rust AST

The generated function takes a `BoundedUint<65535>`. The generated `round` slot reads the counter from the new context.

~~~rust
use compact_rust_counter_parameter_fixture::{
    ledger_contract::{
        initial_state, increment_by, reset_round,
    },
    ledger_slots::round,
    runtime::{
        BoundedUint, CompactError,
        context::ConstructorContext,
    },
};

fn main() -> Result<(), CompactError> {
    let initial =
        initial_state(ConstructorContext::new(()))?;
    let context = initial
        .into_circuit_context(Default::default());
    let amount = BoundedUint::<65_535>::new(7)?;

    let increment = increment_by(context, amount)?;
    let state = increment.context.query.state.get_ref();
    assert_eq!(round.inspect(state)?, 7);

    let reset = reset_round(increment.context)?;
    let state = reset.context.query.state.get_ref();
    assert_eq!(round.inspect(state)?, 0);
    Ok(())
}
~~~

</section>
</div>

## What the host compiler catches

| Mistake | Ledger 8 TypeScript | Ledger 9 Rust AST |
| --- | --- | --- |
| Pass the host’s ordinary integer | TypeScript rejects 7 because it is a `number`. The generated parameter expects `bigint`, so the valid call uses `7n`. | Rust rejects `7_u16`. The generated parameter requires `BoundedUint<65535>`, even though `u16` has the same numeric range. |
| Pass the original context to reset after increment | Both variables still have type `CircuitContext`, so the call type-checks. The caller chooses which state to pass. | The increment takes ownership of the original context. Using that binding again fails to compile. Pass `increment.context` to continue the same sequence. |
| Try to increment by 65,536 | `65_536n` type-checks. The generated circuit rejects it when called. | `BoundedUint::<65_535>::new(65_536)` returns an error when it checks the value. |

Two intentionally invalid Rust calls show the actual diagnostics, shortened here to the relevant lines.

~~~rust
increment_by(context, 7_u16)?;
// E0308: expected BoundedUint<65535>, found u16
~~~

~~~rust
let increment = increment_by(context, amount)?;
reset_round(context)?;
// E0382: use of moved value: context
~~~

Rust’s ownership check applies to the context binding passed by value. The range check applies when an integer becomes a Compact argument. A caller can handle an invalid amount before entering the circuit.

### Source trail

- [The exact Ledger 8 Compact fixture](https://github.com/betsudo/compact/blob/9dc8d22b67eda0a90a8e99950556ef64b4e6f986/examples/rust_backend/counter_parameter.compact)
- [The Ledger 8 TypeScript capture and context setup](https://github.com/betsudo/compact/blob/9dc8d22b67eda0a90a8e99950556ef64b4e6f986/runtime-rs/tests/fixtures/capture-counter-parameter-reset.mjs)
- [The same Compact fixture at the Ledger 9 revision](https://github.com/betsudo/compact/blob/9409b7da1cc652940469903a8380000b0b707239/examples/rust_backend/counter_parameter.compact)
- [The Ledger 9 generated Rust library](https://github.com/betsudo/compact/blob/9409b7da1cc652940469903a8380000b0b707239/tests-rust-backend/counter-parameter/lib.rs)
- [The Ledger 9 generated API and replay tests](https://github.com/betsudo/compact/blob/9409b7da1cc652940469903a8380000b0b707239/tests-rust-backend/counter-parameter/tests/counter_parameter.rs)
