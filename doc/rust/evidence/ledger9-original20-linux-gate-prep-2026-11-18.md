# Original-token 20-circuit Linux gate preparation

This source-only slice extends the manual integration gate tracked by
[issue #773](https://github.com/betsudo/compact/issues/773) to every circuit
in the pinned original `unshielded-tokens` suite. It is based on signed
integrated head `c50363769f8be4e16350be4f7da186ad2e847826`, which
includes the read-only runtime offer accessor used by the immediate Contract
qualification. The gate binds the compiler, Rust runtime/backend, Cargo lock
and Nix inputs to that exact signed source head.

The selector requires all twenty unique original circuits and nineteen
TypeScript cases. It names nine shielded proof batches, thirty fresh
standalone/transaction proof pairs, twenty-two byte-identical replay pairs,
fifteen refusal artifacts, and the recipient self-color control proof. The
three immediate cases require the recorded original receive/send pair, with
the Contract case also requiring the recipient's receive call in the same
guaranteed transaction. Their wrong singleton-index proof refusals are part
of the Linux inventory.

The released Compact 0.36.0 / Runtime 0.21.0 and onchain 4.0.1 input pins,
Ledger 9.1.0.1 semantics, physical ZKIR v3 keys and independent original
TypeScript run remain in the full gate. A Darwin `--preflight-only` result is
`prepared` with `native_linux=false`; full qualification refuses any host
other than native Linux x86_64 before producing output. This slice does not
claim a Linux proof or funded application. The native Linux run and shared
node gate remain open.
