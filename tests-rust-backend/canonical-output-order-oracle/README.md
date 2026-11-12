# Current-generated distribute compatibility fixture

`build.rs` renders the complete fresh `canonical-output-order-schema20-ir.json` with the current backend. `RETAINED_IR` is the proof adapter's whole-source pairing input. The existing independent TypeScript rows and runtime assertions are preserved.

This fixture contains two public exports, `distribute` and `read_coin`. Body-owner attribution tests qualify the new recording algebra separately from generated-crate compilation. Proof readiness requires full source/IR/current Rust pairing and the original funded proof adapter; rendering alone does not establish funding or proof validity.

The `distribute` adapter checks the complete IR, generated Rust, both raw v3 circuits, and the selected binary IR and verifier before deployment or proving. Each commitment order receives a funded default-strict deployment and a distinct funded call proof. The adapter verifies the standalone proof separately and checks exact transaction replay refusal with unchanged ledger bytes. The explicit offline seed supplies the initial contract-owned coin.
