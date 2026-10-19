# Measure the local contract and proof loop

Forge reuses key artifacts. It does not reuse a proof, ledger application, test
assertion or passing verdict. Measure the whole developer command before deciding
that a faster key-preparation phase improves the loop that matters.

## Explicit larger workload

`check_forge_circuit_cache.py --workload-set ledger8-large` selects two complete,
unchanged source files. Its default remains the original two-fixture `smoke` set.

| Source | Compiled exports | Actual proof/application scope |
| --- | ---: | --- |
| `examples/rust_backend/schnorr_attest_oracle.compact` | 2 | Both strict Schnorr entry points, funded Ledger8 application and changed-binding refusal; constructor is supplied, not proved |
| `examples/rust_backend/effect_plan_merkle.compact` | 7 | `run` with selected and skipped insertion branches, proofs, application and original state assertions; other exports are compiled, not all proved |

Every case also runs its original consumer against a privately corrupted verifier
that the selected proof actually consumes. The expected official header refusal
and exit code 1 are required; live generated artifacts remain unchanged.
This larger set qualifies bounded Rust scenarios. It does not extend the separate
actual-JS Counter qualification to Schnorr or Merkle.

Use an already qualified compiler, Forge/ZKIR pair, proof runner and complete
operator-owned policy as described in [local artifact reuse](local-zkir-cache.md).
Prepare official built-in proof material and the declared SRS sizes 5 through 17
in a private parameter directory before timing. Preserve upstream parameter
verification. Record any first-use download and compiler/Cargo build cost
separately; missing proof material must fail rather than weaken funded application.

## Ordinary compiler versus cache-enabled compiler

Use the same fixed source, tools, proof runner, parameters and process environment
for both commands. Set `RESULTS` to a new owned output directory and `CACHE_DIR`
to a new private store outside compiler output trees. Keep the official ZKIR tool
on `PATH`, `MIDNIGHT_PP` bound to `PARAMS_DIR`, and the required locked-ledger static
fixture directory available to the proof runner. Remove any unrelated inherited
compiler/cache wrapper configuration first.

```sh
python3 -B tools/compact-rust-backend/check_forge_circuit_cache.py \
  --workload-set ledger8-large \
  --compiler "$COMPACTC" --proof-runner "$PROOF_RUNNER" \
  --forge "$FORGE" --params-dir "$PARAMS_DIR" --cache-dir "$CACHE_DIR" \
  --compiler-uncached --report "$RESULTS/ordinary.json"

python3 -B tools/compact-rust-backend/check_forge_circuit_cache.py \
  --workload-set ledger8-large \
  --compiler "$COMPACTC" --proof-runner "$PROOF_RUNNER" \
  --forge "$FORGE" --params-dir "$PARAMS_DIR" --cache-dir "$CACHE_DIR" \
  --compiler-cache-policy "$QUALIFIED_POLICY" --report "$RESULTS/cold.json"
```

For a warm observation, repeat the second command with the same private store and
a new report path. Confirm actual per-circuit acquisition routes; a mode label
does not prove a hit. Do not delete a shared store to manufacture a cold case.

`--compiler-uncached` invokes the ordinary complete public compiler **without
`--skip-zk` or a cache policy**. The compiler owns key generation and its output
manifest. The older `--uncached-zkir` mode instead compiles with `--skip-zk`, then
invokes the official ZKIR tool directly. Retain that independent qualification
path, but do not label it an ordinary public-compiler baseline.

Reports retain all nine source/binary-IR/prover/verifier artifact identities,
the raw compiler contract-manifest hash, generated contract file hashes, actual
acquisition routes and separate compile/proof/negative timings. Source, tools,
policy and declared parameter identities are checked around execution. A failed
compile still produces its failed receipt with incomplete acquisition accounting;
it is not a successful timing observation.

## Bounded local result — 2026-10-19

One fixed-tool Darwin sequence ran ordinary, cold, warm, warm, ordinary after an
untimed successful settling lap. All selected genuine proofs, funded applications,
assertions and negative controls passed on every timed lap. All nine artifact
sets, raw manifests and generated contract source matched across the sequence.

| Complete command | Observations, seconds | Median, seconds |
| --- | --- | ---: |
| Ordinary public compiler plus proofs/controls | 10.300, 10.085 | 10.193 |
| Cold Forge plus proofs/controls | 13.972 | 13.972 |
| Warm Forge plus proofs/controls | 8.679, 8.717 | 8.698 |

The warm saving was **1.495 seconds / 14.666%**. Warm laps had six hits and three
automatic tiny-circuit bypasses; cold had six misses and the same three bypasses.
The existing automatic cutoff of 8 and default-off cache behavior were unchanged.
Schnorr's complete case was slightly slower warm; Merkle supplied the measured
saving. The predeclared engineering screen of at least 15% **and** 5 seconds failed.
That screen was not an owner-agreed product requirement or a global cutoff rule.

Private official proof-material setup took 480.5 seconds outside these timings.
An initial missing-Dust-material failure was retained, then the original strict
consumer passed after supported material preparation; no proof was relaxed.
These are small-sample, settled-executable local observations. They do not measure
fresh compiler/Cargo builds, edited-contract rebuilds, the full suite, Linux,
snapshot transfer or hosted CI acceleration. Keep Forge optional and investigate
other build costs separately when those dominate turnaround.

## Cargo freshness is a separate cost

A separate bounded Darwin probe used Rust 1.99.0 and one current, compiler-generated
Counter consumer. It retained the original four Counter integration tests from
`check_compactc_target.py`: execution, typed slots, recorded replay and slot replay.
Generation used `--skip-zk` because this probe measured Cargo compilation and test
execution, **not cryptographic proof generation or key reuse**.

The first two invocations used the same source paths, new private target,
dependency lock, features, tool binaries and effective Cargo configuration. Both
actually ran all four tests. A third invocation relocated the complete private
generated/consumer root, preserving generated bytes and file modification times;
only the consumer manifest's required absolute dependency path changed. It reused
the same owned target. The baseline input files remained untouched.

| Actual Counter test command | Wall seconds | `Compiling` package lines | `Fresh` package lines | Tests passed |
| --- | ---: | ---: | ---: | ---: |
| First, new private target | 63.683 | 244 | 0 | 4 |
| Unchanged repeat, same paths | 0.512 | 0 | 244 | 4 |
| Same bytes, relocated temporary root | 5.602 | 4 | 240 | 4 |

Relocation changed exactly four local package IDs: consumer, generated contract,
bundled runtime and bundled runtime macros. Those were exactly the four packages
that rebuilt. External package identities, lock bytes and dependency-feature graph
were unchanged, and no external package rebuilt. All four tests ran again with
zero failures or ignores. These package-line counts are not proof counts or a
complete count of internal compiler invocations.

This confirms useful Cargo reuse with stable paths and identifies a local
temporary-path cost in this consumer. The target harness creates a fresh temporary
root per invocation and deliberately tests the runtime bundled with its emitted
contract. Do not bypass that packaging check or replace it with a cached passing
test result to save the observed rebuild. No production path, profile or packaging
behavior was changed by this probe.

It does **not** attribute every earlier full-suite compilation line to temporary
paths, establish a whole-suite saving, or measure profile/feature changes or
edited-source rebuilds. Do not add these Counter timings to the separate
Schnorr/Merkle key-cache measurements: their workloads and scopes differ. The
compiler-producer candidate addresses repeated compiler realization separately;
any later consumer-path optimization must preserve current generation, complete
package integrity, standalone packaging coverage and every original assertion.
