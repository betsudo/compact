# Original Ledger 9 block-time TestChain case

The pinned upstream test at `test-center/ts/block-time.ts` is retained byte-for-byte at
`test-center/ts/ledger9-original/block-time.ts`. Its Compact source remains at
`test-center/compact/block-time.compact`, also byte-for-byte. The existing
`test-center/ts/block-time.ts` is the synchronous Runtime 0.18.0 adapter used
by `compiler/test.ss`; this port leaves that Ledger 8 coverage intact.

| Owner | Coverage |
| --- | --- |
| Compiler standard library and generated TypeScript | Four public `blockTimeLt/Lte/Gt/Gte` calls, each at 0, 1, and 5 seconds against an actual TestChain circuit context whose block time is 1 second. The original 12 assertions must pass under released Compact 0.36.0 and the source compiler's released 0.21 profile. |
| Runtime 0.21 | `startContract` creates a circuit context; the original test changes `context.callContext.currentQueryContext.block.secondsSinceEpoch` to `1n` and `secondsSinceEpochErr` to `0`. The runtime produces the boolean results. |
| Rust AST and runtime | Comparisons read QueryContext slot 2 (`tblock`). Slot 7 (`last_block_time`) is separately carried into child contexts and is not the comparison operand. An exact-source `--emit-rust-ir` probe of the original fixture currently refuses `Kernel operation blockTimeGreaterThan`; `compiler/rust-ir-passes.ss` has no `blockTimeLessThan` or `blockTimeGreaterThan` lowering. Add those typed kernel operations, then qualify generated Rust against the same explicit one-second context and all 12 original boundary results. |

The source-owned gate `tools/compact-rust-backend/check_original_block_time_testchain.py`
uses the pinned Runtime 0.21.0 fixture, exact original test bytes and source,
released Compact 0.36.0, and a source-built ZKIR v3 compiler. It records the
compiler commands, generated files, test logs, source and tool hashes. Its
`--skip-zk` run checks TestChain behavior, not fresh proofs or funded Ledger 9
transactions. Native Linux remains a separate qualification.

The Rust IR probe is recorded separately from the passing TypeScript gate.
It is a compiler refusal, not generated Rust behavior. It identifies the
first unsupported operation encountered in the intact fixture; both block
time kernel operations need implementation before Rust parity can be claimed.

Independent local review passed at signed source `2d2132f0b`: the original
asynchronous runtime test passes 1/1 with all twelve assertions on released
and source TypeScript. Signed receipt SHA-256:
`bab51d341f7c07ed164737d246274a7fd34b79b8a11308bec7fab677d360bd84`.
This is generated TypeScript execution in a circuit context, without a funded
transaction or proof. [#783](https://github.com/betsudo/compact/issues/783)
tracks Rust lowering; the older generic Ledger 8 gap remains [#442](https://github.com/betsudo/compact/issues/442).
The separate Rust slice at signed `b577d8d8462bd5c109292460f48ee4fb3cb5173b`
now passes independent local arm64 review. It adds exact slot-2 VM Lt/Gt,
typed Uint64/Boolean IR, native/recorded metered queries and context-aware
functional support. Source profiles pass ten controls; runtime passes three
and backend two. The generated original crate typechecks, and all twelve
original values pass in both native and recorded execution (24 observations)
with `tblock=1` and distinct `last_block_time=900`. The Ledger 8
no-default-features check establishes compilation only; its existing source
refusals and synchronous adapter remain intact. Signed receipt SHA-256:
`ba0629197aa9b5cab6dda1d6bfe5ceaa62258fc766e2c4882b1e3581fa516bea`.
The initial generated-fixture functional failure is retained as historical
failure evidence. Fresh proof/funded application, repository-wide compiler
gates and native Linux remain open; this runtime identity is separate from
the signed Events v4/v5 packages.
