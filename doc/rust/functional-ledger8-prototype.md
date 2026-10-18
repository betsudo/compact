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

`--skip-zk` skips key generation. Proof qualification is a separate operation;
inspect the emitted circuit inputs rather than assuming every native entry is
proof-bearing. The generated Cargo dependency
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

The current projection includes checked literals/defaults, enums, aggregates,
projections, Field and unsigned arithmetic, comparisons, conversions, Cell
reads/writes, Counter reads/increments, assertions, conditionals, and typed witnesses.
Pure value syntax shares the existing checked ordinary renderer. Expressions
remain inside successful binds, so later producers do not run after a failure.
Zero, single and multiple arguments, plus Unit, Field, fixed Bytes and product
results, have generated consumer coverage. Existing Boolean witness APIs remain
compatible. Arguments and result types are checked against retained declarations;
the existing metered frames own gas, private state and private outputs.

Equal emitted Rust signatures share one handler implementation. Checked witness
IDs still distinguish callbacks, including zero arguments versus one Unit and a
single tuple argument versus multiple arguments. These are mechanical adapters
to the existing `TryWitnesses` trait, not a new callback language.

Opaque witness results select the explicit Ledger8 `ExactWitnessOutput` strategy,
which delegates existing exact metered frame encoding. Opaque arguments alone do
not select exact output encoding. The default witness constructor and two-type
parameter call remain compatible. Exact in-memory transport does not promise
that opaque values can pass later proof serialization checks.

Checked local calls inline their retained bodies on the same carrier. Certified
pure calls reuse the existing pure Rust functions, including supported pure Fold.
Both nested branches remain statically checked; only selected calls execute.
This does not implement the separate direct stateful Fold gap (#661).

Scalar Map membership, lookup, insertion, removal, default insertion, reset,
size and emptiness use the existing typed slots on both interpreters. Logical
bindings preserve each declaration's full physical path. Nested Map declarations
are currently refused by this projection and have a separate interface refinement.

Other Counter operations, remaining collections and hashes are still refused by
this projection. Ordinary-backend conversion restrictions also remain authoritative.
Vector construction is supported; vector indexing currently reaches tuple
projection and is tracked separately in #674. These boundaries must not be
interpreted as a claim of full language or proof coverage.

The generated witness adapter currently uses `DefaultDB`, matching the existing
generated `LedgerView`. Other database adapters require separate work.

## Source evidence and privacy

Each entry exposes `CHECKED_PROGRAM`, a review-only JSON inventory of typed
inputs, effects, capabilities and branch guards. It is not an executable
serialization, complete graph identity, proof certificate, or admission token.

The inventory is stored beside `lib.rs`, in
`checked-programs/program-<ordinal>.json`. Each constant uses a relative
`include_str!`, preserving its public `&str` type and exact JSON bytes. The
compiler assigns filenames from retained exported circuit order; source names
remain JSON data. Keep these files when copying, packaging or building the
crate. The publication manifest records their hashes alongside the Rust source and
detects edits that do not also replace that manifest. It is not a signature or
an external source of trust.

Library callers of `render_functional_ledger8` must write every entry returned by
`FrontendRender::functional_assets()` unchanged, relative to `lib.rs`. The CLI
writes these assets inside its existing atomic publication stage. Default
rendering returns no functional assets.

In a matched local comparison of seven contracts (13 exported entries), this
moves 9.9–22.1% of each `lib.rs` out of Rust source. Total output sizes change
little: the JSON remains present and the manifest gains asset entries. These
measurements concern readability and packaging, not execution performance.

This separates review material from execution source. It does not remove the
additional functional execution module, change recording authority, or establish
a compile-time, binary-size or proof-performance improvement.

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

The current finite consumer run covers **16 tests**, **56 behavioral scenarios**
and **four execution routes per scenario**: ordinary native, ordinary recorded,
functional native and functional recorded. It checks successful values, state,
effects, gas, private outputs and replay, plus expected failures and observable
witness prefixes. Failed-call state and gas cannot be compared because the
existing consuming error API does not return that context. These numbers are
fixture coverage, not language conformance or proof coverage. The seventh source
fixture adds both branches for byte, small/wide unsigned and enum literals or
defaults, with equality checks through the generated execution body.
The eighth source exercises typed witnesses, distinct IDs sharing Rust signatures,
zero/single/multiple arguments, inactive and denied branches, metered ledger reads,
and independent expected callback arguments, private outputs and state.
The ninth source exercises exact opaque witness outputs, nested products,
returned-value use and failure after an exact result. The tenth adds shared pure
arithmetic, aggregates, non-Copy projection and conversion boundary controls.
These preserve opaque witness bytes without relaxing opaque-cell storage rules.
The eleventh source covers repeated local calls, fresh formals, metered witness
reads, failure inside a callee and certified pure Fold.
The twelfth source exercises all eight scalar Map operations through ordinary
private helpers and checked calls, with owned values and chunked paths. Direct
nested-source normalization remains limited in the ordinary frontend (#516 and
#610); helper composition does not establish direct-syntax parity.

A separate measurement test performs **4,096 executions** across 16 alternating
route samples; these are not additional behavioral coverage. In one local debug
run, median time per read/write call was approximately 6.28 ms ordinary native,
6.37 ms ordinary recorded, 6.36 ms functional native and 6.40 ms functional
recorded. Context setup and result checks were outside each measured interval;
call execution and retaining complete results were inside. Noise and this tiny
single-contract sample preclude a speedup claim. Allocations were not measured.

Fresh default/opt-in compilation also records generated bytes and lines. In the
initial embedded-metadata prototype, all seven measured crates grew because
opt-in publication **adds** the functional module
and retains ordinary APIs: for the read/write example, 7,737 → 15,349 bytes and
194 → 312 lines. The experiment currently improves execution structure, not total
generated-file size. Removing duplicate public surfaces requires a separate API
compatibility decision.

To qualify the metadata extraction against an embedded-metadata predecessor,
retain that baseline compiler separately and select it explicitly:

```sh
python3 tools/compact-rust-backend/check_functional_metadata.py \
  --baseline /path/to/embedded-metadata/compactc \
  --candidate target/debug/compactc \
  --output /tmp/compact-functional-metadata-run \
  --report ./functional-metadata-run.json
```

Both compilers must use the same Scheme frontend. The output must be absent
and outside the checkout, so Cargo does not associate relocated crates with the
repository workspace. The helper checks exact metadata bytes and constant
mapping, whole default-output equivalence, and functional-output equivalence
except the metadata declarations, sidecar files and updated manifest. It builds
relocated crates in both explicit source-runtime and bundled-source modes,
checks Cargo's package file list, exercises missing/tampered/undeclared asset
refusals, and restores the private mutation fixture. This tests local source
runtime packaging; it does not establish registry distribution or a packaged
tarball build. Keep the regular behavior and proof helpers as separate gates.

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

## Typed Field-witness proof qualification

The additional local helper checks one complete stateful Field-witness fixture:

```sh
python3 tools/compact-rust-backend/check_functional_witness_proof.py --help
```

Provide the explicit matching compiler, parameters, target directory, qualified
Forge binary, uncached ZKIR binary and a new private cache directory. Each of its
three laps freshly generates the source, executes functional recording, compares
ordinary observed preparation, and sends the **functional-derived** preparation
to the existing prover and ledger-application checks. Independent assertions check
the Field result, private state, callback event, exact private output and applied
ledger value. First-store, repeat-store and uncached key generation are separate.

The negative fixture changes only a private verifier copy to malformed bytes.
Its refusal establishes corrupt-key handling; it does not establish rejection of
a well-formed verifier for a different circuit. No proof or verdict is cached.

A witness-only entry can be native-only: the existing `witness_argument.compact`
source is marked `proof_required: false` and emits no ZKIR. The qualification uses
its own complete stateful fixture rather than relaxing manifest checks or
removing exports from a larger contract. One proved Field witness is not proof
coverage for every supported argument/result signature or opaque transport.
