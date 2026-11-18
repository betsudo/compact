# Original shielded immediate self-send on Ledger 9

The immediate-self implementation was first qualified as trial commit
`895ef4767e0dbae35441d7301224076be2329e79`, based on the earlier
self-send head `c215b30f0b4a5885ef965b1efd0523405f7ce47e`. The trial
receipt is separate evidence. This integrated branch applies that work above
the original user-send head `2c0f7768d815626c3bbd415e4bae049967aee872`;
its fresh signed-head results are recorded in the integrated qualification
receipt, not inherited from the trial.

This local slice qualifies the unchanged pinned `sendImmediateShieldedToSelfTestNoChange`
case from LFDT-Minokawa/compact `672bd0f226f47a7d036c5c074ce71fbe1af1925e`.
The source and generated TypeScript/IR/ZKIR/key bundle are inherited unchanged
from the signed original-token artifact receipt. The released Compact 0.36.0
and Ledger 9.1.0.1 SDK matrix remain the interoperability baseline.

The original circuit takes an unqualified 100-unit shielded coin. Its
`sendImmediateShielded` wrapper upcasts it to singleton Merkle index zero and
records one nullifier, one claimed shielded spend and one matching self
receive, with every other Effect empty. This is checked directly against the
generated Rust result and source-owned call trace.

The source-bound Rust gate uses a real contract-owned transient, not a
historical coin at index zero. An upstream wallet input funds its output;
the original `receiveShieldedTest` call precedes the original immediate-send
call in one guaranteed transaction segment. A narrow runtime adapter checks
both traces against the same observed contract and complete physical offer,
reconciles the ordered output/input/output events, exact transient proof
identity, owner, source singleton index, commitments, wallet input and unchanged
contract state, then prepares both installed operations. Existing single-call
offer preparation remains strict. Focused runtime tests reject reversed
calls, either isolated call, an altered event stream and a changed physical
offer output. The smoke also checks isolated-call refusal and an actual
wrong-singleton-index proving failure with the same public coin identity.

Both original calls receive fresh 5,392-byte standalone proofs and separate
verification, changed-binding and corrupt-proof refusals. The transaction
has separate call proofs, a positive Dust fee, default-strict Ledger
well-formedness and application, matching post-state commitments at the
physical offer indices, a consumed transient nullifier, and byte-identical
replay refusal. The wallet seed is a privileged local fixture; application
uses the Ledger snapshot directly because `TestState`'s separate wallet
mirror does not observe that privileged seed.

The qualification is local Darwin arm64 offline evidence. Native Linux,
live node/indexer/proof server, and broader original immediate Contract/User
circuits remain open. The original TypeScript 19/19 result is carried only
by identical generated JavaScript bytes; it is not rerun here. No push,
hosted CI or publication is claimed from this slice.
