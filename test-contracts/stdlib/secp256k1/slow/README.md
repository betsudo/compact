# Ledger 9 secp256k1 slow fixtures

The two Compact sources and compile declarations match
LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
`storeEthereumAddress` exercises the original #612 path. `two` and
`three` verify independent signatures in source order and write the
conjunction to the ledger.

In the development shell, provide the built compiler and TypeScript
runtime and the Ledger SDK ZKIR 3.2 CLI and WASM package. Choose an
absent output directory under a fresh test root.

```sh
SLOW=test-contracts/stdlib/secp256k1/slow
python3 "$SLOW/check-compile.py" --compiler "$COMPACTC" --zkir "$ZKIR" \
  --ts-runtime "$TS_RUNTIME" --output "$OUTPUT/emitted"
node "$SLOW/check-preimages.mjs" "$OUTPUT/emitted" "$ZKIR_WASM" "$OUTPUT/preimages"
```

The current language 0.23.105 compiler refuses both intact sources.
Add `--allow-pragma-probe` to record those refusals and remove only
the 0.25.0 pragma in separate output copies. Keep intact-source
qualification open until both original fixtures compile.

The preimage check validates the original address write and return,
the ordered witness calls, and ten independent key or signature
substitutions. Those substitutions produce zero folds with fresh
execution. The SDK refuses the same substitutions on the honest
one-fold statements. It also refuses changed public writes and the
changed address return.

The Rust consumer compares native execution, recorded execution and
VM replay. Copy `consumer.Cargo.toml` and `consumer.rs` to
`$OUTPUT/funded/consumer/Cargo.toml` and `main.rs` and seed its lock
from the workspace lock. Build it with Rust 1.99 and the workspace's
pinned offline dependencies. The `parity` mode takes the evidence
root and an unused output argument and requires no proving keys.

The funded modes take one of the three export names, the evidence
root, and a fresh output directory. Supply the matching freshly
generated and independently verified tagged key and proof files
under `$OUTPUT/slow-proof/<export>/statement.{prover,verifier,proof}`.
Set `MIDNIGHT_PP` and `MIDNIGHT_LEDGER_TEST_STATIC_DIR` to the
verified Ledger 9 parameter and Dust material directory. These modes
compare the Rust and TypeScript input and transcript fields, prove
a separate funded transaction, apply it with default strictness, and
check replay refusal with unchanged ledger bytes. All three circuits
require the explicit slow lane.
