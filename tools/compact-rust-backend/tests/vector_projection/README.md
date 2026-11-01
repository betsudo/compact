# Constant vector projection (#674)

The source owner is `examples/rust_backend/constant_vector_projection.compact`.
`check_constant_vector_projection.py` generates Rust and TypeScript using the
selected compiler/frontend, then builds an independent consumer of the unedited
generated crate. Its metadata and lock checks retain repository dependency
identities. Outputs and reports must be absent before a run; failures are kept.

```sh
COMPACTC_SCHEME=/path/to/compactc-scheme \
python3 tools/compact-rust-backend/check_constant_vector_projection.py \
  --compiler /path/to/compactc \
  --output /fresh/vector-projection \
  --report /fresh/vector-projection.json \
  --ts-runtime /existing/package/@midnight-ntwrk/compact-runtime
```

The eight consumer tests exercise independent first/last/singleton values,
non-Copy text and temporary results; ordinary native/recorded and functional
native/recorded stateful execution; owned vector reuse; one witness evaluation;
first witness failure; inactive branches; ordered private outputs; state/effects,
query-summed gas agreement and actual public replay. Two independent Programs
with fresh contexts execute and update separate cells to 17 and 41. Four external
compile-fail binaries require E0308 for wrong length/type and E0382 for reused
Program/context ownership. Each error must have a primary span at the marked
misuse in its own binary; unrelated failures cannot satisfy the gate.
The two `.compact` negatives must fail at frontend bounds before publication.

Backend unit tests independently cover typed projection, original operand and
index binding, bounds/type/scope forgery and preserving tuple-only operations.
The private occurrence registry binds this operation family; it does not claim
whole-plan equivalence or compiler-source authentication.

With `--ts-runtime`, the gate executes five pure and seven stateful TS cases,
plus three refusal controls, against that exact existing production dependency
closure. It compares actual canonical result/Cell/private-output encodings and
private transitions to the Rust consumer (and its four execution routes). The
TS query hook retains each real query, including witness ledger-view reads;
their summed gas agrees with Rust. The TS wrapper's raw last-query cost is also
retained and is not interpreted as a total. Source/runtime/tool/library hashes
are checked; macOS system-cache libraries are identified by OS path only.
A changed TS result must fail the actual Rust consumer. The complete compiler
workflow requests this live comparison using its Nix runtime package. Without
`--ts-runtime`, the report explicitly makes no executed TS claim.

Kernel virtual-library report names (`linux-vdso.so.1`, `linux-gate.so.1`) are
retained explicitly without disk-byte claims. The gate injects those names into
the actual capture to check this boundary and requires an unknown missing
library to fail before capture publication. These injected controls are host
tests, not execution or qualification on Linux.

This gate does not generate keys, prove, verify, prepare/apply a funded transaction, establish
performance, or replace the complete required CI/proof gates. Those acceptance
items remain separately tracked in the bounded #674 plan. The full compiler
workflow retains its existing gates and adds this finite generated API check;
no hosted run has been dispatched by the local implementation.


## Direct proof qualification

`check_constant_vector_projection_proof.py --compiler /path/to/compactc
--zkir /path/to/official/zkir --output /fresh/vector-proof` builds and snapshots
the maintained native proof runner. Set `MIDNIGHT_PP` to the existing official
parameter directory and optionally `CARGO_TARGET_DIR` for Cargo reuse. It
resolves the exact locked ledger static fixtures, verifies built-in material and
the selected SRS, and generates a fresh native key pair for `witnessed`.

Before loading those keys, fresh Rust (formatted only), full IR and all five
complete checked programs must equal the linked source fixture. Relative source
labels are supplied to the compiler; no generated metadata is normalized. Both
private-vector selections execute four routes and compare ordinary preparation.
The functional recording reaches the actual prover. Each fresh proof is verified
independently; a changed binding and a changed result commitment must fail
verification, and changed-result preimage checking must also fail. Each branch
then creates a fresh funded transaction, applies with default strictness, and
requires exact `IntentAlreadyExists` replay rejection without ledger mutation.
Constructor data is supplied by the source constructor, whose execution is not
proved. The original full integration gates remain required.

The scheduler for the deferred Forge experiment remains deleted. Task-owned
worktrees and reproducible build targets are eligible for cleanup after this
work is complete and merged in GitHub, while preserving unpublished changes,
needed inputs and evidence. This condition does not authorize publishing/merging
or deleting shared stores, caches, containers or qualification artifacts.


The live API gate also generates the separate pure `numeric_aliases.compact`
control. Three actual Rust tests compare live TypeScript behavior for nested
unsigned aliases and multiplication by zero at inputs 0, 1, 5 and 255. Explicit
checked source casts must still reject nonzero values, and their failure must
precede the later assertion. This pure control generates no ZKIR or proof.
Its complete generated-file inventory is retained independently of the selected
Vector source. Capture unit controls use an explicitly synthetic module to
reject wrong values or missing failures before publishing a capture; they do
not claim compiler execution.
