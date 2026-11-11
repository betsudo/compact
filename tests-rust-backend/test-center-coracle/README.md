# Current-generated original Coracle

`build.rs` generates this fixture from the complete retained
`coracle-source-schema20-ir.json` with the current default backend. That IR is
fresh compiler output from unchanged `test-center/test-contracts/coracle.compact`.
All existing types, pure functions, four public recorded APIs and tests remain
available. Historical per-export IR files remain separate dependency projections;
they are not substituted for the complete source artifact.

For the unit-action family retirement, the focused runtime gate is
`--test guess`: its 26 existing TypeScript/native/recorded scenarios include both
colors, selected witness order, identity/board/turn/position failures, gas limits
and empty-Maybe behavior. No additional whole-game qualification is implied.

The `--coracle-guess` adapter binds the complete retained IR, generated Rust,
all raw circuits, and the selected binary IR and verifier key. It generates
fresh Ledger 9 v3 keys for `guess.zkir`. Both colors execute from explicit
prior states, verify fresh standalone proofs separately, fund deployment
and a distinct strict call, and check exact unchanged replay refusal.
Each applied state also refuses the repeated same-color move. The prior
state represents game setup for the fixture.

The `start`, `concede` and `withdraw` proof adapters use selected Ledger 9 v3
artifacts bound to the same complete original source. Each color has a funded
deployment, a separately verified standalone call proof and a distinct strict
transaction proof. The seeded cases install the explicit prior game and coin
state after deployment. Blue start and both withdrawal cases check that a
changed phase rolls back the selected offer while guaranteed Dust and replay
processing persist. Withdrawal also checks refusal of misplaced proved coins.
Each successful case checks exact transaction replay refusal with unchanged
ledger bytes.
