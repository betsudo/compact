# Original DID: functional deactivation lifecycle

Complete functional Rust output for the byte-identical midnight-did v0.7.0
source view in `examples/rust_backend/third_party/did_functional_lifecycle`. Only ordinary
Rust formatting and Cargo package/dependency placement differ from generation.
Full original IR, raw origins, capabilities and all 12 checked programs remain
unedited. The existing ordinary DID fixture and historical authorities remain
unchanged.

Handwritten integration tests use a normal crate alias and genuine tutorial
witnesses. Selected valid deactivation and late-witness failure compare
ordinary/functional native and recorded routes, full state/effects/private
outputs, witness order, gas and sealed VM replay against fresh TS observations.
The data file retains every field of all 12 DID capture rows unchanged; it
projects unrelated crypto cases and host-specific private identity metadata
out of the public fixture. Full identity-bound capture receipts are separate.
Five Rust test methods cover the original valid lifecycle, late witness failure
and all ten captured invalid inputs. The invalid inputs run through four routes
(40 refusal executions), preserving witness order and the exact first error.
They include stale/signature/inactive precedence and arithmetic limits. The
separate timestamp-zero success row remains TS-only. Seeded invalid prestates
are explicit test inputs; failed consumed contexts are not claimed to roll back.

This candidate starts from verified canonical merge `acf529eff` of reviewed,
full-CI PR #692. Development API/proof gates pass at their recorded epochs;
final clean compiler138 qualification, full integration CI and exact-PR
independent review for this slice remain open. No constructor
execution proof or whole-application completion is claimed. See the bounded
`doc/rust/plans/did-functional-lifecycle-current-profile.md` plan.

The generated functional PureCall owner treats checked Unit results as fallible
statements rather than Unit locals. All six affected generated sites come from
fresh complete generation; IR/origins/capability and program assets are unchanged.

Maintained local gates (use a fresh absent output directory for each run):

```sh
python3 tools/compact-rust-backend/check_did_functional_lifecycle.py \
  --compiler /absolute/path/to/compactc --ts-runtime /absolute/path/to/compact-runtime \
  --output /absolute/path/to/new-did-api-result
MIDNIGHT_PP=/absolute/path/to/verified-parameters \
python3 tools/compact-rust-backend/check_did_functional_lifecycle_proof.py \
  --compiler /absolute/path/to/compactc --output /absolute/path/to/new-did-proof-result
```

The proof gate selects deactivation only. It retains all twelve generated
programs, checks changed IR/program refusal before proof setup, generates a fresh
key and proof, and verifies changed binding/version controls against the original
verifier. A Unit result has no atoms to mutate. Funded default-strict application
and exact replay rejection are distinct from proof success; no network claim.
