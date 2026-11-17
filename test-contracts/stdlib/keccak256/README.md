# Ledger 9 Keccak parity

The nine fixture families and their test declarations retain the source bytes
from LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The adapter runs the original runtime assertions against generated TypeScript
and a consumer of the generated Rust APIs. It also checks the original
compile outcomes, including the ADT diagnostic, and checks the stateful
fixture emits a `keccak256` ZKIR instruction.

In the Compact development shell, with Rust 1.99 and installed JavaScript
test dependencies, point the adapter at a local `compactc`, the built
TypeScript runtime from this checkout and the installed `@noble/curves`
`utils.js`. The output directory must be absent. Cargo uses the checkout's
locked registry dependencies in offline mode.

```sh
python3 tools/compact-rust-backend/check_upstream_ledger9_keccak.py \
  --compiler "$COMPACTC" \
  --ts-runtime "$COMPACT_RUNTIME" \
  --curves-utils "$NOBLE_CURVES_UTILS" \
  --output "$KECCAK_OUTPUT"
```

The v3 profile reports language 0.28.0 and checks all nine intact upstream
sources with their `>= 0.25.0` pragmas: eight compile and the ADT case refuses.
`--allow-pragma-probe` also checks that the legacy v2 profile at language
0.23.105 refuses each original pragma without
publishing output. It does not alter a fixture. This suite skips key generation.
The stateful k=14 proof requires a separate slow lane.
