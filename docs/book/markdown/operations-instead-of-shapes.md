## How success created a bottleneck

Recording support expanded by recognizing particular program shapes. A known contract pattern could select a recorder that understood its reads, writes and returns. This approach delivered useful capabilities, but each new combination risked needing another recognizer.

A contract with a supported Map operation inside an unfamiliar expression might fail even though its primitive behavior was already understood. The implementation had learned a catalogue of complete shapes faster than it had learned their composition.

## The checked effect graph

The replacement treats an operation as part of a typed effect program. Reads, writes, witness observations, calls, returns, branches and sequencing carry explicit semantic information. Checked lowering determines whether the composed program is legal.

This gives the compiler a place to reason about ordering and ownership before choosing a renderer. A shared semantic fold can interpret the same admitted program through native and recorded adapters.

## Why a versioned IR matters here

Every new node has consequences for traversal, validation, lowering, capabilities and diagnostics. Exhaustive handling makes those obligations visible. A version change can state that the boundary has changed instead of letting a consumer guess from generated text.

Source spans matter for the same reason. An unsupported child expression should point back to Compact source. A Rust panic or a generic target failure leaves users debugging the implementation rather than their program.

## Retire a recognizer with evidence

A recognizer can be removed after the new composition covers its accepted family. That requires inclusion checks across the old family's behavior, diagnostics and resource refusals. Deleting a special case after proving one happy example would lose useful support.

The retirement programme therefore worked through bounded operation families and explicit refusal controls. A capability book and shared registry made admission easier to inspect.

## Local prototype to integrated change

The October 14 storybook froze f3cef799 while final effect qualification was still open. The later integrated commit 0437c894cd069af475bd8d5b254c24cc73615a09 on October 16 records the checked-effect replacement through PR #641.

This is a good example of why a living book needs dated chapters. The old snapshot accurately described a prototype. The continuation records its later integration.

## What was solved

Operations became reusable components of larger contract behavior. Native and recorded execution could share semantic ownership, and retirement acquired a measurable obligation.

## What remained

A compositional foundation still needs specific source features, result conversions, public interfaces and proof gates. The following iterations show that work in concrete terms.

### Source trail

- [Checked effect epic #508](https://github.com/betsudo/compact/issues/508)
- [IR boundary #482](https://github.com/betsudo/compact/issues/482)
- [Checked effects integration](https://github.com/betsudo/compact/commit/0437c894cd069af475bd8d5b254c24cc73615a09)
- [Shared capability book #486](https://github.com/betsudo/compact/issues/486)
