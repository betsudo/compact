## The first useful proof of direction

The retained original Rust backend showed that Compact could produce useful Rust. Its Scheme passes operated on TypeScript-oriented compiler representations and assembled Rust source text. The original oracle's ancestry introduces the eight-file Ltypescript emitter in June 2026.

This was a practical implementation strategy. A compiler can reuse an already lowered representation and get a working target quickly. Generated crates and examples then make the missing operations visible.

## Where text emission becomes expensive

Consider a nested expression containing a ledger read, a fallible conversion and a function call. A textual emitter must preserve the source type, track the updated context, arrange error propagation, and output legal Rust syntax. If those decisions share one string-building layer, a local feature can affect many unrelated cases.

A missing clone may become an ownership error. An extra conversion can change a value's admitted range. Parentheses can make syntactically legal code with the wrong grouping. Testing generated text catches some problems after emission, but the compiler still lacks a single closed place to decide whether a construct is valid for the target.

## Why retain the original

The old backend supplied a valuable reference for the replacement. It contained behavior and generated-crate conventions that the new architecture had to preserve deliberately. Removing it would have discarded a comparative baseline precisely when the rebuild most needed one.

The project therefore used the original implementation as an oracle. The replacement could adopt a different internal design while comparing the external behavior of selected contracts.

## The decision

Keep Scheme as the shared Compact frontend. Move target-specific semantic admission into a versioned internal model. Let a Rust backend construct Rust syntax through Rust syntax-tree libraries.

This change was about ownership of decisions. It did not require a new Compact language, a replacement ledger VM, or a parallel cryptographic specification.

## A contributor's test

When investigating an old generated example, identify the source revision, backend line and runtime family first. Two files both called generated Rust can belong to different architectures and ABI generations. A bug fixed in the oracle is a clue for the AST line, with a separate port and test still required.

### Source trail

- [Original Ltypescript emitter introduction](https://github.com/betsudo/compact/commit/67f7cbc8b2ea3164d9fb225c7f9fd67a9aa3d552)
- [Original Rust oracle comparison](https://github.com/betsudo/compact/commit/589c92ef961ce89ac730140e81b76d4c08222a70)
- [Architecture decision research #95](https://github.com/betsudo/compact/issues/95)
