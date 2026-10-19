# Optional local ZKIR artifact reuse

The experimental shared compiler boundary can reuse Ledger8 prover and verifier
keys through the Forge CLI. It is **off by default** and serves both TypeScript
and Rust targets. It does not cache proofs, test outcomes or verification results.

```sh
compactc --target ts --target rust \
  --zkir-cache local --zkir-cache-policy qualified-policy.json \
  --zkir-cache-dir ./job-cache contract.compact ./generated
```

`qualified-policy.json` is an operator-owned qualification document. Its complete
schema pins the exact Forge and official ZKIR executable paths and SHA256 values,
Ledger8 profile and revision, expected build fingerprint, serialization and
key-generation recipe, and actual parameter-file digests. Matching a version
string or copying an advertised profile is insufficient qualification. A tested
Darwin build does not admit a Linux executable.

## Tiny circuits

The following optional fields belong inside the complete policy:

```json
{"selection": "auto", "tiny_max_k": 8}
```

Automatic selection uses the current official ZKIR model. At `k <= 8`, the
compiler generates fresh keys without invoking Forge or reading/writing cache
contents. Larger circuits are eligible for reuse; this is not a guarantee that
every larger circuit benefits. The cutoff is provisional and configurable from
0 to 255. `selection: "force"` ignores the cutoff so qualification can exercise
tiny cache hits, restored snapshots and corruption controls.

`k` is the proof-domain size parameter reported by the official cost model,
not source line count or total test duration. Selection belongs to the consumer;
Forge can still cache a tiny circuit when explicitly requested.

Fresh model generation and authentication still have costs. Complete test time,
tool provisioning and snapshot transfer must be measured before claiming a CI
speedup. The initial two tiny fixtures had essentially equal warm-cache and
ordinary key-generation test times.
Qualification reports identify each circuit's actual acquisition path as a hit,
miss, automatic bypass or explicit unavailable-provider fallback. The
`compiler-local` mode alone does not establish that Forge was used.

## Current inputs and failure behavior

Every preparation reads current circuit bytes, computes an actual official model
and checks parameter identity. Cache hits must agree with the current binary IR,
recipe, manifest, descriptor and artifact bytes. Artifacts are copied privately;
the compiler builds its normal output integrity manifest after key preparation.
All circuits must prepare successfully before the public native command replaces
its output directory.

Malformed responses, changed inputs, incompatible profiles, corrupt artifacts
and failed subprocesses are errors. The policy may explicitly allow ordinary
key generation when the configured Forge executable is absent. An available but
invalid executable or corrupt cache never triggers a silent stale fallback.

`--skip-zk`, help/version queries and empty circuit sets perform no provider
preparation or policy-content reads. Ordinary output replacement semantics still
apply. Keep policy, tools, parameters and caches outside generated output trees.
Path metadata checks protect referenced inputs and stores from destructive
publication; these checks are distinct from opening a cache or invoking Forge.

Without an explicit directory, the effective store is
`MIDNIGHT_ZKIR_FORGE_DIR`, then `~/.midnight-zkir-forge`. Use a private store for
tests involving corruption, eviction or deletion. The shared default store must
not be mutated by negative fixtures.

This first boundary supports local CLI transport and Ledger8 ZKIR v2. It does not
admit v3, HTTP or a different backend through matching source syntax. Retain the
separate direct official key-generation qualification path when adopting it.

## Explicit generation requests in tests

Some tests generate current ZKIR with `--skip-zk`, then invoke `zkir compile`
themselves. The optional test wrapper can route those requests to the same
native provider. Supply all four pins; ordinary imports and the existing Python
force-qualification route retain their previous behavior.

```sh
python3 tools/compact-rust-backend/forge_circuits.py \
  --native-helper /qualified/bin/compact-zkir-cache \
  --native-helper-sha256 "$HELPER_SHA256" \
  --native-policy qualified-policy.json \
  --native-policy-sha256 "$POLICY_SHA256" \
  --cache-dir ./job-cache --params-dir /qualified/parameters \
  -- test-command
```

The wrapper prepares only the requested circuit in an owned temporary directory.
Its native receipt contains the already-computed fresh model, parameter identity,
actual acquisition path and authenticated artifact hashes. It performs no second
model compilation. Copies remain private; every proof and assertion still runs.
Source, policy, tools, parameters and path bindings are checked during the request.
Child output uses disk spools with bounded memory reads, not a disk quota.

`forge_ci.py plan` accepts the optional group `--native-script`, `--native-policy`
and `--native-helper`. It binds the selected script, tools, policy and declared
source inventory to the existing snapshot lifecycle. Native snapshots default to
256 MiB; this per-snapshot limit does not cap total repository cache storage.

The Linux compiler job calls `check_native_ci.sh` once. That script retains the
full native CI command list and stops on a failed proof or assertion command.
This shared entry point runs uncached unless explicitly wrapped by a qualified
provider; extracting it does not enable Forge in Linux CI.
`check_native_ci_smoke.sh` exercises two real proof fixtures for lifecycle
qualification and is not a substitute for that full gate. Exact Linux enrollment,
immutable published Forge provisioning and hosted workflow adoption remain
separate requirements. No hosted acceleration is claimed by these local tools.

## Prepare public proof material explicitly

An empty runner also needs the official built-in Zswap/Dust keys and SRS.
The existing preparation command accepts additional parameter sizes:

```sh
cargo run --profile proof-checks --locked --offline \
  -p compact-rust-proof-smoke -- \
  --prepare-proof-material --parameter-k 7 --parameter-k 17
```

Set `MIDNIGHT_PP` to an owned parameter directory. This explicit command may
download missing material through the official provider, which checks the
locked upstream hashes. Requested sizes are deduplicated and combined with the
sizes required by authenticated built-in circuits. Unknown sizes are refused
by the provider. Cargo's `--offline` flag concerns dependencies; it does not
disable this explicit material download.

Add `--verify-only` to check the same set without downloading or repairing it.
Missing or corrupt material is an error. With no `--parameter-k` options, the
existing default receipt and built-in preparation behavior are unchanged.
Preparing material neither runs tests nor qualifies a cache or backend profile.

For the explicit larger Schnorr/Merkle comparison, ordinary public-compiler
baseline, retained proof controls and bounded results, see
[local loop measurement](forge-local-loop-measurement.md).
