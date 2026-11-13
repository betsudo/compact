## The new pipeline

After shared Compact analysis, Chez Scheme lowered supported constructs into a versioned JSON model. Rust deserialized and checked that model, selected typed runtime operations, and built output through syn, quote, proc-macro2 and prettyplease.

The crucial word is supported. A closed internal representation gives each admitted node an explicit meaning. Unsupported constructs should produce a diagnostic at a known boundary. Silently manufacturing a placeholder would conceal a missing semantic case.

## A vertical slice before a framework

The first slice used schema 1 for pure Field, Boolean, Unit, tuple and vector circuits, including Field addition. It explicitly refused ledger and witness constructs. Ledger-backed primitive types, Cell and Counter state changes, typed values and typed reads followed. Set, Map and List operations expanded that native surface. Pure conditionals, local bindings and circuit calls expanded the usable source subset.

Each slice joined several owners. Scheme supplied the source node and type. The Rust model represented it. Checked lowering established a legal operation. Syntax emission built the Rust expression. The runtime delegated ledger behavior to the VM. Generated consumers exercised the ordinary API.

## Why structured syntax helped

A Rust syntax tree gives the emitter a representation for expressions, statements, blocks, paths and types. The formatter can render those nodes without encoding semantics in whitespace or string fragments.

That benefit has a boundary. Constructing an AST does not prove that the correct field operation or source conversion was chosen. The semantic checks still need to occur before rendering. The long-term architecture therefore pairs structured syntax with checked lowering.

## The Milestone 1 result

The broad local native baseline closed at ea4cc006f973632c4c7b1fb4569af2b4c93ff38a. Publication followed at a5ee30f171322f07c06e3846117f20c9e075ef95 on October 2. Historical notes record 128 generated sources and 410 workspace test/doc-test groups.

The milestone used private IR schema 6 and runtime ABI 3. Its wrapper forced skip-zk. That is an essential scope boundary when reading the early success report.

## What was solved

Compact could generate useful typed Rust that executed a supported contract subset natively through the ledger runtime. The compiler boundary became inspectable, and new source cases had named owners.

## What was still open

Recording a transcript, preparing a call, generating a proof and applying a funded transaction were later deliveries. Native execution established an important foundation for them.

### Source trail

- [Start typed Rust from Ledger 8](https://github.com/betsudo/compact/commit/67064813977a9af901792e5cfb62e295e49bff43)
- [Milestone 1 local baseline](https://github.com/betsudo/compact/commit/ea4cc006f973632c4c7b1fb4569af2b4c93ff38a)
- [Published native baseline](https://github.com/betsudo/compact/commit/a5ee30f171322f07c06e3846117f20c9e075ef95)
- [Native baseline issue #102](https://github.com/betsudo/compact/issues/102)
