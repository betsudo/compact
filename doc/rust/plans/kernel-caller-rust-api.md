# Ledger 9 Kernel.caller Rust API qualification

The independently reviewed local slice is signed source `23af5a8e78a55c3a0d9aa51b672c34989fea8cd4`, based on `7cece56c5`. It adds genuine generated Rust support for the original Caller Inner contract. [#778](https://github.com/betsudo/compact/issues/778) retains the broader Caller and transaction work. This slice remains separate from the accepted Events v4 integration because it changes the signed runtime closure.

## Implemented behavior

The frontend emits a typed caller query only in the existing Ledger 9 released-runtime profile. The backend checks the complete canonical `Maybe<PublicAddress>` structure, including the nested Either and 32-byte address members. Native and recorded code execute the pinned 14-operation slot-6 VM program. They consume the constructed aligned read value, preserve its gas, and replay the exact read in Verify mode. A forged observed read refuses. A constructor Cell assignment containing the query takes the VM path; the functional target explicitly refuses this unsupported operation.

The unchanged original `Caller/Inner.compact` has SHA-256 `db593aeba03d3e5f738128f9444ece313b4f81494677a2e094031e46966bd466`. The retained exact-source compiler emits Rust and TypeScript successfully, and the generated crate passes an offline all-features Cargo check. Its constructor/root test checks `None`. A separate sealed A-to-B snapshot test derives B's `Some(Contract(A))` from the observed parent and checks native/recorded execution and replay gas; it does not supply caller identity as a transaction authority.

## Evidence and limits

The signed delivery index `delivery-23af.json` has SHA-256 `77c4209a33e29d76feafd821e846abb6597eb7d3f35886eede739a9f8fa1d9a8`. It binds the signed qualification `99d2618fc02df4632ac6e2b3ea9ed23c5cad799056f6ede7885f5e1a4a290d03`, the original source, 31 committed changed blobs, retained Inner output, commands and logs. The retained compiler binary has SHA-256 `9db533a34899c53af6d92496c34ba1d0c8dcec500ca4caec805a630b73de9953`.

Five focused tests pass: two runtime tests, one sealed-child test, one backend type guard and one generated constructor/root test. The retained Inner tree has 135 files; its scratch test and Cargo lock were added to the evidence copy after CLI emission and are disclosed separately. The signed Events frontend regression `37785eb9c3c1655ddc2843b46caeb31c90ef793a27f26c8cf888a35e235792b6` checks two byte-exact compiles and eight refusals.

These are local Darwin arm64 API/VM results. They do not qualify original Outer `viaInner`, nested/multiple calls, fresh Caller proofs, funded strict application/replay, `Some(User)` or native Linux. Constructor `None` and the tested root context are distinct: an actual top-level transaction with unanimously owned unshielded inputs can derive `Some(User)`, so root `None` is not unconditional.

The signed v4 package remains valid for its original runtime. Four signed runtime files differ from this slice, and a new caller test file is present. The current-runtime admission comparator must reject that old closure. Signed boundary note SHA-256 `e80b2d7e95b2573cb2ea856af5c6d68a140057eee470a86d3bd4482afdb71a75` records this explicitly. The separately reviewed [v5 continuation](original-events-v5-status.md) now provides the physical package and current-runtime CLI admission. Fresh v5 proof/funded qualification remains pending; the archived v4 package and frozen native Linux handoff remain intact. All owned Cargo targets and the finished source worktree were removed; signed source, compiler binary and evidence are retained.
