# Functional Ledger8 programs (experimental)

This opt-in prototype projects the compiler's checked effect graph into one
immutable Rust request and one execution body. Native execution and recording
select different leaf adapters for that body. Compact remains the source language.

The executable target is Ledger **8.1.3**. The target-neutral effect vocabulary
keeps logical identity separate from adapter paths; a future ledger target still
needs its own adapters, capability checks and qualification. This does not implement Ledger9,
ZKIR3, a new contract language, or complete language coverage. Existing generated
APIs remain available and the default compiler output is unchanged.

## Generate a crate

Use the current compiler and its matching source runtime:

```sh
compactc --target rust --rust-functional-ledger8 --skip-zk \
  --rust-runtime-root /absolute/path/to/compact \
  examples/rust_backend/let_return_oracle.compact ./generated
```

`--skip-zk` skips key generation. The compiler still emits the circuit inputs;
proof qualification is a separate operation. The generated Cargo dependency
enables `experimental-functional`. Registry runtime mode is refused because
this prototype is not a published runtime release.

Functional mode requires an exported stateful circuit and retained frontend
origin bindings. It refuses the whole experimental publication when an export
uses an operation outside the supported subset. Originless IR is not an opt-in
publication path.

## Use the request

The example contract reads the old `stored` Field, writes `next`, and returns
the old Field. Its generated module provides this API:

```rust
use compact_contract_let_return_oracle::{
    ledger_contract::functional::replace,
    runtime,
};

pub fn record_replace(
    context: runtime::context::CircuitContext<()>,
    next: runtime::Field,
) -> Result<
    runtime::recording::RecordedCircuitResult<(), runtime::Field>,
    runtime::CompactError,
> {
    replace::Program::new(next).execute_recorded(context)
}
```

Use `execute_native` for a `CircuitResult`. Both convenience methods select
the program's compiler-issued slot bindings. Witness-bearing entries also take
the existing `TryWitnesses` implementation by reference.

Constructing a request performs no ledger or witness work. Execution consumes
the request and context. Construct another request for another execution;
there is no hidden witness cache, implicit request cloning, or failed-frame
recovery.

The lower-level `run` method accepts an executor satisfying the generated leaf
capabilities. Custom executors and handwritten callbacks are ordinary Rust;
their effects and privacy are not certified by this interface.

## Domain and component model

```text
Compact + retained invocation origins
                  |
            validated CheckedPlan
                  |
       immutable Program arguments
                  |
          one generic run body
             /           \
      NativeLedger8    RecordedLedger8
             \           /
       existing frames, slots and codecs
                  |
          official Ledger8 domains
```

- The checked graph owns value types, ordering and both branch bodies.
- Target capabilities select a bounded projection; no whole-contract shape
  recognizers are added.
- `Cell<T>`, `Counter` and witness identities carry logical identity in the
  core. Physical paths and official database/value bounds belong to adapters.
- `Circuit<E, T>` is a consuming success/error carrier, not another stored graph
  or a runtime typechecker. Continuations are executed immediately during `run`.
- Generated continuations defer operands until previous steps succeed. Rust
  still evaluates ordinary method arguments eagerly in handwritten code.
- Both branches are checked during compilation; only the selected branch
  executes at runtime.

## Current bounded operations

The initial projection includes checked literals/defaults, enums, equality,
Cell reads/writes, Counter reads/increments, assertions, conditionals, and
Boolean witnesses with one Boolean argument and a Boolean result. Existing
ordinary APIs can support operations outside this experimental subset.

Calls, conversions, other Counter operations, collections, hashes and other
witness signatures are refused by this projection. These refusals are a
capability boundary, not a claim that the ordinary Rust backend lacks them.

The generated witness adapter currently uses `DefaultDB`, matching the existing
generated `LedgerView`. Other database adapters require separate work.

## Source evidence and privacy

Each entry exposes `CHECKED_PROGRAM`, a review-only JSON inventory of typed
inputs, effects, capabilities and branch guards. It is not an executable
serialization, complete graph identity, proof certificate, or admission token.

Production origins currently use a clearly labeled coarse original-root
fallback. They do not pretend to be exact failed-node spans. A binding to an
original IR occurrence plus a later source snapshot does not authenticate which
source bytes the configured frontend consumed. Privacy and consumed-source
bytes remain explicitly **unknown** in this inventory.

The runtime delegates recording to existing frames so the existing read FAB,
gas accounting and private-output handling remain in use. Replay, proof,
verification and ledger application are still separate correctness gates.

## Local qualification

From the repository root, select a freshly built compiler and its matching
Scheme frontend. `COMPACTC_SCHEME` selects the frontend actually invoked by the
compiler. The consumer helper records both executable identities and checks
them again after execution.

```sh
cargo build --locked --package compact-rust-backend --bin compactc
export COMPACTC_SCHEME="$(command -v compactc-scheme)"
export CARGO_BUILD_JOBS=4
python3 tools/compact-rust-backend/check_functional_effects.py \
  --compiler target/debug/compactc \
  --output ./functional-consumer-run \
  --report ./functional-consumer-run.json
python3 tools/compact-rust-backend/check_functional_origins.py \
  --compiler target/debug/compactc --scheme "$COMPACTC_SCHEME" \
  --report ./functional-origins-run.json
```

If `CARGO_TARGET_DIR` is set, use its `debug/compactc` instead. The consumer
output directory must be absent: each invocation compiles current sources and
runs fresh assertions, rather than accepting a previous result. Rust dependencies
must already be available locally because consumer Cargo commands use `--offline`.
The helper also runs generated-package Clippy with warnings denied.

The accepted finite consumer run covers **8 tests**, **25 behavioral scenarios**
and **four execution routes per scenario**: ordinary native, ordinary recorded,
functional native and functional recorded. It checks successful values, state,
effects, gas, private outputs and replay, plus expected failures and observable
witness prefixes. Failed-call state and gas cannot be compared because the
existing consuming error API does not return that context. These numbers are
fixture coverage, not language conformance or proof coverage. The seventh source
fixture adds both branches for byte, small/wide unsigned and enum literals or
defaults, with equality checks through the generated execution body.

A separate measurement test performs **4,096 executions** across 16 alternating
route samples; these are not additional behavioral coverage. In one local debug
run, median time per read/write call was approximately 6.28 ms ordinary native,
6.37 ms ordinary recorded, 6.36 ms functional native and 6.40 ms functional
recorded. Context setup and result checks were outside each measured interval;
call execution and retaining complete results were inside. Noise and this tiny
single-contract sample preclude a speedup claim. Allocations were not measured.

Fresh default/opt-in compilation also records generated bytes and lines. All seven
measured crates grew because opt-in publication **adds** the functional module
and retains ordinary APIs: for the read/write example, 7,737 → 15,349 bytes and
194 → 312 lines. The experiment currently improves execution structure, not total
generated-file size. Removing duplicate public surfaces requires a separate API
compatibility decision.

Proof qualification additionally requires matching parameters, an explicitly
qualified Forge executable, and the uncached `zkir` binary. Inspect the helper's
arguments before supplying these local installations:

```sh
python3 tools/compact-rust-backend/check_functional_proof.py --help
```

The proof helper preserves the original proof/verification/application and
negative assertions. Its functional substitution covers direct Let recording
and sends that functional-derived prepared call to the actual prover. Counter
and the typed observed/prepared comparison remain ordinary APIs. It records
that scope explicitly.

Forge is optional artifact tooling. It can avoid redundant compatible key
generation; it does not cache proofs or test results. Use actual before/after
statistics and complete test wall time to assess savings. A matching ledger
version alone does not qualify an installed Forge build.

## Prototype decision

Keep the surface opt-in while extending coverage and reducing repeated output.
Local qualification passed the Rust backend workspace (3,299 tests, seven
existing manual ignores), workspace Clippy, formatting, the default target
consumer, the readability policy and 14 focused runtime tests on Rust 1.89.
These checks do not replace the separate manual historical qualification.

The three proof laps executed nine successful proof/application scenarios and
six corrupt-verifier refusals. Forge recorded two cold misses then two warm
hits, with zero errors. Complete two-case harness time was 3.613s cold, 3.124s
warm and 3.055s uncached. These tiny circuits establish compatible cache reuse;
they do not establish overall acceleration. Private runner compilation is a
separate cost.

Next improvements should address wider operation/witness coverage, output
duplication and clearer refusal/error text. Privacy facts, exact node origins,
wire-level conformance, allocation counts and future target qualification each
need their own evidence.

Tracking: [issue #663](https://github.com/betsudo/compact/issues/663).
Architecture history: ADR-0542 in the `midnight` Obsidian vault.
