// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use decode::runtime as r;
use midnight_base_crypto::{
    data_provider::{FetchMode, MidnightDataProvider, OutputMode},
    fab::{Value as FabValue, ValueAtom},
};
use midnight_ledger::{
    construct::ContractCallExt,
    dust::{DUST_EXPECTED_FILES, DustResolver},
    prove::Resolver as LedgerResolver,
    semantics::TransactionResult,
    structure::{
        ContractDeploy, Intent, ProofPreimageMarker, ProofPreimageVersioned, ProofVersioned,
        Signature, Transaction,
    },
    test_utilities::{TestState, tx_prove},
    verify::WellFormedStrictness,
};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf, IrBuf,
};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_storage::{arena::Sp, storage::HashMap};
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    curve::Fr,
    proofs::{Proof, ProofPreimage, ProvingKeyMaterial, VerifierKey, Zkir},
};
use midnight_zkir::IrSource;
use midnight_zswap::prove::ZswapResolver;
use r::{
    CompactError, Field, FixedBytes, Secp256r1Base, Secp256r1Point, Secp256r1Scalar,
    context::{CircuitResult, ConstructorContext},
    fab::AlignedValue,
    ledger::{ContractAddress, DefaultDB, StateValue},
    public_state::PublicStateSource,
    recording::RecordedCircuitResult,
    transaction::{Observation, ObservedCallError, ObservedContractState, RecordedCall},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json::{Value, json};
use std::{env, error::Error, fmt::Debug, fs, io, path::Path, sync::Arc};

type TestResult<T> = Result<T, Box<dyn Error>>;

fn check_recording<Output: PartialEq + Debug>(
    native: &CircuitResult<(), Output>,
    recorded: RecordedCircuitResult<(), Output>,
    witnesses: usize,
) -> TestResult<()> {
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.public_state(), recorded.public_state());
    assert_eq!(
        native.private_transcript_outputs,
        recorded.execution.private_transcript_outputs
    );
    assert_eq!(native.private_transcript_outputs.len(), witnesses);
    let (initial, program) = recorded.public.into_parts();
    let replay = initial.query(&program, None, &recorded.execution.context.cost_model)?;
    assert_eq!(replay.context.state.get_ref(), native.public_state());
    Ok(())
}

struct Case<'a> {
    id: &'a str,
    circuit: &'a str,
    fixture: &'a str,
    k: u8,
}

fn funded<Output, F>(
    root: &Path,
    out: &Path,
    case: Case<'_>,
    initial_state: StateValue<DefaultDB>,
    native: &CircuitResult<(), Output>,
    call: F,
) -> TestResult<()>
where
    Output: Into<AlignedValue> + PartialEq + Debug,
    F: for<'a> Fn(&'a ObservedContractState) -> Result<RecordedCall<'a, (), Output>, CompactError>,
{
    let name = case.circuit.to_owned();
    let expected_state = native.public_state();
    let key_dir = root.join("slow-proof").join(case.id);
    let prover_bytes = fs::read(key_dir.join("statement.prover"))?;
    let verifier_bytes = fs::read(key_dir.join("statement.verifier"))?;
    let verifier: VerifierKey = tagged_deserialize(verifier_bytes.as_slice())?;
    let wrong_id = match (case.fixture, case.id) {
        ("upstream-roundtrip", "upstream-roundtrip-test0-0") => "upstream-ledger-storePoint-0",
        (name, _) if accessor_fixture(name) => "upstream-roundtrip-test0-0",
        ("upstream-equality", "pointsEqual-0") => "pointsNotEqual-0",
        ("upstream-equality" | "upstream-identity", _) => "pointsEqual-0",
        ("upstream-base", _) => "upstream-scalar-test-3",
        ("upstream-scalar" | "unary-controls", _) => "upstream-base-test-3",
        ("arithmetic", "mulb-6") => "muls-6",
        ("arithmetic", _) => "mulb-6",
        (_, "decode-14") => "encode-14",
        _ => "decode-14",
    };
    let wrong: VerifierKey = tagged_deserialize(fs::File::open(
        root.join("slow-proof")
            .join(wrong_id)
            .join("statement.verifier"),
    )?)?;
    assert_ne!(verifier, wrong);
    let standalone: Proof = tagged_deserialize(fs::File::open(key_dir.join("statement.proof"))?)?;
    let family = root.join("family-first");
    let fixture = match case.fixture {
        "arithmetic" => family.join(case.fixture),
        name if unary_fixture(name) => family.join("unary").join(name),
        name if point_fixture(name) || name == "upstream-roundtrip" => {
            family.join("point").join(name)
        }
        name if accessor_fixture(name) => family.join("accessors").join(name),
        name => family.join("casts").join(name),
    };
    let raw = fixture
        .join("ts-rust/.build/zkir")
        .join(format!("{name}.zkir"));
    let ir = IrSource::load(fs::File::open(&raw)?)?;
    assert_eq!(ir.k(), case.k);
    let mut binary = Vec::new();
    tagged_serialize(&ir, &mut binary)?;
    assert_eq!(binary, fs::read(raw.with_extension("bzkir"))?);
    let op = ContractOperation::new(
        Some(verifier.clone()),
        Some(Sp::new(IrBuf::from(binary.clone()))),
    );
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let initial_contract = ContractState::new(
        initial_state,
        HashMap::new().insert(entry.clone(), op.clone()),
        ContractMaintenanceAuthority::default(),
    );
    let mut rng = StdRng::from_rng(rand::rngs::OsRng)?;
    let deploy = ContractDeploy::new(&mut rng, initial_contract);
    let address = deploy.address();
    env::var("MIDNIGHT_LEDGER_TEST_STATIC_DIR")?;
    let mut state: TestState<DefaultDB> = TestState::new(&mut rng);
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(state.give_fee_token(&mut rng, 1));
    state.ledger.contract = state
        .ledger
        .contract
        .insert(address, deploy.initial_state.clone());
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state,
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let randomness: Fr = rand::rngs::OsRng.r#gen();
    assert!(matches!(call(&observed)?.prepare(wrong, randomness),
        Err(ObservedCallError::VerifierMismatch(ref entry)) if entry == &name));
    let recorded = call(&observed)?;
    assert_eq!(recorded.recorded().execution.result, native.result);
    assert_eq!(
        recorded.recorded().execution.private_transcript_outputs,
        native.private_transcript_outputs
    );
    assert_eq!(
        recorded
            .recorded()
            .execution
            .private_transcript_outputs
            .len(),
        0
    );
    assert_eq!(recorded.recorded().execution.public_state(), expected_state);
    let prepared = recorded.prepare(verifier, randomness)?;
    assert_eq!(prepared.address, address);
    assert_eq!(prepared.entry_point, entry);
    assert_eq!(prepared.op, op);
    let typescript: ProofPreimage = tagged_deserialize(fs::File::open(
        root.join("preimages-first")
            .join(format!("{}.preimage", case.id)),
    )?)?;
    let commitment = typescript
        .communications_commitment
        .ok_or("missing commitment")?
        .0;
    let ProofPreimageVersioned::V2(rust) = ProofPreimage::construct_proof(&prepared, commitment)
    else {
        return Err("expected current ledger preimage".into());
    };
    assert_eq!(rust.inputs, typescript.inputs);
    assert_eq!(rust.private_transcript, typescript.private_transcript);
    assert_eq!(
        rust.public_transcript_inputs,
        typescript.public_transcript_inputs
    );
    assert_eq!(
        rust.public_transcript_outputs,
        typescript.public_transcript_outputs
    );
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, state.time + state.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1u16, intent));
    let materials = Arc::new((prover_bytes, verifier_bytes, binary));
    let zswap = ZswapResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        midnight_zswap::ZSWAP_EXPECTED_FILES.to_owned(),
    )?);
    let dust = DustResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        DUST_EXPECTED_FILES.to_owned(),
    )?);
    let name_for_resolver = name.clone();
    let resolver = LedgerResolver::new(
        zswap,
        dust,
        Box::new(move |location| {
            let materials = materials.clone();
            let name = name_for_resolver.clone();
            Box::pin(async move {
                Ok::<_, io::Error>((location.0.as_ref() == name).then(|| ProvingKeyMaterial {
                    prover_key: materials.0.clone(),
                    verifier_key: materials.1.clone(),
                    ir_source: materials.2.clone(),
                }))
            })
        }),
    );
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("P-256 funded transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, applied) = calls.next().ok_or("missing funded call")?;
    let ProofVersioned::V3(transaction_proof) = &applied.proof else {
        return Err("expected v3 proof".into());
    };
    assert!(calls.next().is_none());
    assert_ne!(transaction_proof, &standalone);
    let dust_spends = sealed
        .intents()
        .filter_map(|(_, intent)| intent.dust_actions)
        .map(|actions| {
            actions
                .spends
                .iter()
                .filter(|spend| !spend.proof.0.is_empty() && spend.v_fee > 0)
                .count()
        })
        .sum::<usize>();
    assert_eq!(dust_spends, 1);
    let verified =
        sealed.well_formed(&state.ledger, WellFormedStrictness::default(), state.time)?;
    let mut ctx = state.context();
    ctx.block_context.last_block_time = state.time;
    let (updated, result) = state.ledger.apply(&verified, &ctx);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("missing applied state")?;
    assert_eq!(actual.data.get_ref(), expected_state);
    assert_eq!(actual.operations.get(&entry).as_deref(), Some(&op));
    ctx.ref_state = updated.clone();
    let (replayed, result) = updated.apply(&verified, &ctx);
    assert!(
        matches!(
            result,
            TransactionResult::Failure(
                midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                    midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
                )
            )
        ),
        "{result:?}"
    );
    assert_eq!(replayed, updated);
    fs::create_dir(out)?;
    let (mut after, mut replay, mut tx_bytes) = (Vec::new(), Vec::new(), Vec::new());
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    tagged_serialize(&sealed, &mut tx_bytes)?;
    assert_eq!(after, replay);
    fs::write(out.join("applied-transaction.bin"), &tx_bytes)?;
    fs::write(out.join("transaction-bound.proof"), &transaction_proof.0)?;
    fs::write(out.join("ledger-after.bin"), &after)?;
    fs::write(out.join("ledger-replay.bin"), &replay)?;
    println!(
        "{}",
        json!({"status":"passed","circuit":name,
        "ledger_matches_native":true,"case":case.id,"fixture":case.fixture,"k":ir.k(),
        "transaction_proof_bytes":transaction_proof.0.len(),"standalone_proof_bytes":standalone.0.len(),
        "transaction_bytes":tx_bytes.len(),"positive_fee_dust_spends":dust_spends,
        "default_strict":true,"typescript_preimage_fields_match":true,"wrong_verifier_refused":true,
        "replay_refusal":"IntentAlreadyExists","replay_unchanged":true})
    );
    Ok(())
}

fn from_hex(value: &Value) -> [u8; 32] {
    hex::decode(value.as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap()
}

fn field_bytes(value: &Value) -> [u8; 32] {
    let mut raw = from_hex(value);
    raw.reverse();
    raw
}

fn decode_base(value: &Value) -> Secp256r1Base {
    Secp256r1Base::from_le_bytes(field_bytes(value)).unwrap()
}

fn decode_scalar(value: &Value) -> Secp256r1Scalar {
    Secp256r1Scalar::from_le_bytes(field_bytes(value)).unwrap()
}

fn decode_point(value: &Value) -> Secp256r1Point {
    let x = decode_base(&value["x"]);
    let y = decode_base(&value["y"]);
    let mut fab = FabValue::from(x);
    fab.0.extend(FabValue::from(y).0);
    fab.0
        .push(ValueAtom::from(value["identity"].as_bool().unwrap()));
    Secp256r1Point::try_from(&*fab).unwrap()
}

fn accessor_fixture(name: &str) -> bool {
    matches!(
        name,
        "upstream-roundtrip" | "upstream-ledger" | "upstream-asserted" | "guarded-writes"
    )
}

fn point_fixture(name: &str) -> bool {
    matches!(name, "upstream-equality" | "upstream-identity")
}

fn unary_fixture(name: &str) -> bool {
    matches!(name, "upstream-base" | "upstream-scalar" | "unary-controls")
}

fn check_bool_cell(state: &StateValue<DefaultDB>, expected: bool) {
    let StateValue::Array(fields) = state else {
        panic!("expected a single Boolean Cell")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(
        fields.get(0).unwrap(),
        &r::ledger::constructor_cell(expected)
    );
}

fn run_case(root: &Path, id: &str, row: &Value, funded_output: Option<&Path>) -> TestResult<()> {
    let fixture = row["fixture"].as_str().unwrap();
    let circuit = row["circuit"].as_str().unwrap();
    let inputs = row["inputs"].as_array().unwrap();
    let expected = &row["expected"];
    let bytes = |i: usize| FixedBytes::<32>::new(from_hex(&inputs[i]["bytes"]));
    let base = |i: usize| decode_base(&inputs[i]["field"]);
    let scalar = |i: usize| decode_scalar(&inputs[i]["field"]);
    let guard = || inputs[0].as_bool().unwrap();
    let point = |i: usize| decode_point(&inputs[i]);
    macro_rules! exercise {
        ($package:ident, $method:ident, $call:ident, ($($arg:expr),*), |$view:ident $(, $result:ident)?| $verify:block $(, seed = $seed:expr)?) => {{
            use $package::ledger_contract as contract;
            let init = || contract::initial_state(ConstructorContext::new(()));
            let context = || {
                let ctx = init()?.into_circuit_context(ContractAddress::default());
                $(let ctx = ($seed)(ctx)?.context;)?
                Ok::<_, CompactError>(ctx)
            };
            let native = contract::$method(context()?, $($arg),*)?;
            if unary_fixture(fixture) {
                check_bool_cell(native.public_state(), true);
            } else if fixture == "upstream-equality" {
                check_bool_cell(native.public_state(), expected.as_bool().unwrap());
            }
            let $view = contract::PublicStateView::from(&native);
            $(let $result = native.result;)?
            $verify
            check_recording(&native, contract::recorded::$method(context()?, $($arg),*)?, 0)?;
            if let Some(out) = funded_output {
                let models: Value = serde_json::from_slice(&fs::read(root.join("slow-proof/receipt.json"))?)?;
                let model = &models[id];
                assert_eq!(model["fixture"], fixture);
                assert_eq!(model["circuit"], circuit);
                let case = Case { id, fixture, circuit, k: model["k"].as_u64().unwrap().try_into()? };
                let contract = contract::Contract::default();
                funded(root, out, case, context()?.query.state.get_ref().clone(), &native,
                    |observed| contract.recording().$call(observed, (), $($arg),*))?;
            }
        }};
    }
    macro_rules! base_arithmetic {
        ($method:ident, $call:ident) => {
            exercise!(
                arithmetic,
                $method,
                $call,
                (base(0), base(1)),
                |view, result| {
                    assert_eq!(result, decode_base(&expected["base"]));
                    assert_eq!(view.base()?, result);
                    assert_eq!(view.scalar()?, Secp256r1Scalar::default());
                }
            )
        };
    }
    macro_rules! scalar_arithmetic {
        ($method:ident, $call:ident) => {
            exercise!(
                arithmetic,
                $method,
                $call,
                (scalar(0), scalar(1)),
                |view, result| {
                    assert_eq!(result, decode_scalar(&expected["scalar"]));
                    assert_eq!(view.base()?, Secp256r1Base::default());
                    assert_eq!(view.scalar()?, result);
                }
            )
        };
    }
    macro_rules! unary_control {
        ($method:ident, $call:ident, ($($arg:expr),*), $expected:expr) => {
            exercise!(unary_controls, $method, $call, ($($arg),*), |_view, result| {
                assert_eq!(result, $expected);
            })
        };
    }
    macro_rules! point_equality {
        ($method:ident, $call:ident) => {
            exercise!(
                point_equality,
                $method,
                $call,
                (point(0), point(1)),
                |_view, result| {
                    assert_eq!(result, expected.as_bool().unwrap());
                }
            )
        };
    }
    let seed = || decode_point(&row["seed"]);
    match (fixture, circuit) {
        ("upstream-roundtrip", "test0") => exercise!(
            accessor_roundtrip,
            test0,
            test0_call,
            (point(0)),
            |view, result| {
                assert_eq!(result, decode_point(expected));
                assert_eq!(view.point()?, result);
            }
        ),
        ("upstream-roundtrip", "test1") => exercise!(
            accessor_roundtrip,
            test1,
            test1_call,
            (),
            |view, result| {
                assert_eq!(
                    result,
                    (decode_base(&expected[0]), decode_base(&expected[1]))
                );
                assert_eq!(view.point()?, seed());
            },
            seed = |ctx| contract::test0(ctx, seed())
        ),
        ("upstream-ledger", "storePoint") => exercise!(
            accessor_ledger,
            storePoint,
            storePoint_call,
            (point(0)),
            |view| {
                assert_eq!(view.pt()?, point(0));
            }
        ),
        ("upstream-ledger", "storeX") => exercise!(
            accessor_ledger,
            storeX,
            storeX_call,
            (),
            |view| {
                assert_eq!(view.pt()?, seed());
                assert_eq!(view.x()?, decode_base(&row["written"]));
                assert_eq!(view.y()?, Secp256r1Base::default());
            },
            seed = |ctx| contract::storePoint(ctx, seed())
        ),
        ("upstream-ledger", "storeY") => exercise!(
            accessor_ledger,
            storeY,
            storeY_call,
            (),
            |view| {
                assert_eq!(view.pt()?, seed());
                assert_eq!(view.x()?, Secp256r1Base::default());
                assert_eq!(view.y()?, decode_base(&row["written"]));
            },
            seed = |ctx| contract::storePoint(ctx, seed())
        ),
        ("upstream-asserted", "storePoint") => exercise!(
            accessor_asserted,
            storePoint,
            storePoint_call,
            (point(0)),
            |view| {
                assert_eq!(view.pt()?, point(0));
            }
        ),
        ("upstream-asserted", "storeXChecked") => exercise!(
            accessor_asserted,
            storeXChecked,
            storeXChecked_call,
            (),
            |view| {
                assert_eq!(view.pt()?, seed());
                assert_eq!(view.x()?, decode_base(&row["written"]));
            },
            seed = |ctx| contract::storePoint(ctx, seed())
        ),
        ("guarded-writes", "guardedX") => exercise!(
            accessor_guarded,
            guardedX,
            guardedX_call,
            (guard(), point(1)),
            |view, result| {
                assert_eq!(result, decode_base(expected));
                assert_eq!(view.x()?, result);
                assert_eq!(view.y()?, Secp256r1Base::default());
            }
        ),
        ("guarded-writes", "guardedY") => exercise!(
            accessor_guarded,
            guardedY,
            guardedY_call,
            (guard(), point(1)),
            |view, result| {
                assert_eq!(result, decode_base(expected));
                assert_eq!(view.x()?, Secp256r1Base::default());
                assert_eq!(view.y()?, result);
            }
        ),
        ("upstream-equality", "pointsEqual") => point_equality!(pointsEqual, pointsEqual_call),
        ("upstream-equality", "pointsNotEqual") => {
            point_equality!(pointsNotEqual, pointsNotEqual_call)
        }
        ("upstream-identity", "storePoint") => exercise!(
            point_identity,
            storePoint,
            storePoint_call,
            (point(0)),
            |view, result| {
                assert_eq!(result, decode_point(expected));
                assert_eq!(view.point()?, result);
            }
        ),
        ("upstream-identity", "hashPoint") => exercise!(
            point_identity,
            hashPoint,
            hashPoint_call,
            (point(0)),
            |view, result| {
                assert_eq!(result, FixedBytes::<32>::new(from_hex(expected)));
                assert_eq!(view.hash()?, result);
            }
        ),
        ("upstream-base", "test") => exercise!(
            unary_base,
            test,
            test_call,
            (decode_base(&inputs[0])),
            |_view, result| {
                assert_eq!(
                    result,
                    (decode_base(&expected[0]), decode_base(&expected[1]))
                );
            }
        ),
        ("upstream-scalar", "test") => exercise!(
            unary_scalar,
            test,
            test_call,
            (decode_scalar(&inputs[0])),
            |_view, result| {
                assert_eq!(
                    result,
                    (decode_scalar(&expected[0]), decode_scalar(&expected[1]))
                );
            }
        ),
        ("unary-controls", "negBase") => unary_control!(
            negBase,
            negBase_call,
            (decode_base(&inputs[0])),
            decode_base(expected)
        ),
        ("unary-controls", "negScalar") => unary_control!(
            negScalar,
            negScalar_call,
            (decode_scalar(&inputs[0])),
            decode_scalar(expected)
        ),
        ("unary-controls", "guardedInvBase") => unary_control!(
            guardedInvBase,
            guardedInvBase_call,
            (guard(), decode_base(&inputs[1])),
            decode_base(expected)
        ),
        ("unary-controls", "guardedInvScalar") => unary_control!(
            guardedInvScalar,
            guardedInvScalar_call,
            (guard(), decode_scalar(&inputs[1])),
            decode_scalar(expected)
        ),
        ("unary-controls", "discardInvBase") => unary_control!(
            discardInvBase,
            discardInvBase_call,
            (decode_base(&inputs[0])),
            expected.as_bool().unwrap()
        ),
        ("unary-controls", "discardInvScalar") => unary_control!(
            discardInvScalar,
            discardInvScalar_call,
            (decode_scalar(&inputs[0])),
            expected.as_bool().unwrap()
        ),
        ("unary-controls", "localOverloads") => unary_control!(
            localOverloads,
            localOverloads_call,
            (guard()),
            (
                expected[0].as_bool().unwrap(),
                expected[1].as_bool().unwrap()
            )
        ),
        ("arithmetic", "addb") => base_arithmetic!(addb, addb_call),
        ("arithmetic", "subb") => base_arithmetic!(subb, subb_call),
        ("arithmetic", "mulb") => base_arithmetic!(mulb, mulb_call),
        ("arithmetic", "adds") => scalar_arithmetic!(adds, adds_call),
        ("arithmetic", "subs") => scalar_arithmetic!(subs, subs_call),
        ("arithmetic", "muls") => scalar_arithmetic!(muls, muls_call),
        ("decode", "test") => exercise!(decode, test, test_call, (bytes(0), bytes(1)), |view| {
            assert_eq!(view.base()?, decode_base(&expected["base"]));
            assert_eq!(view.scalar()?, decode_scalar(&expected["scalar"]));
        }),
        ("encode", "test") => exercise!(encode, test, test_call, (base(0), scalar(1)), |view| {
            assert_eq!(view.base()?.into_array(), from_hex(&expected["base"]));
            assert_eq!(view.scalar()?.into_array(), from_hex(&expected["scalar"]));
        }),
        ("guarded-save", "save") => {
            exercise!(guarded_save, save, save_call, (guard(), base(1)), |view| {
                assert_eq!(view.out()?.into_array(), from_hex(&expected["out"]));
            })
        }
        ("guarded-decode", "test") => exercise!(
            guarded_decode,
            test,
            test_call,
            (guard(), bytes(1), bytes(2)),
            |view| {
                assert_eq!(view.base()?, decode_base(&expected["base"]));
                assert_eq!(view.scalar()?, decode_scalar(&expected["scalar"]));
            }
        ),
        ("guarded-encode", "test") => exercise!(
            guarded_encode,
            test,
            test_call,
            (guard(), base(1), scalar(2)),
            |view| {
                assert_eq!(view.base()?.into_array(), from_hex(&expected["base"]));
                assert_eq!(view.scalar()?.into_array(), from_hex(&expected["scalar"]));
            }
        ),
        ("constant", "castR1") => exercise!(
            constant,
            castR1,
            castR1_call,
            (guard(), base(1), scalar(2)),
            |view| {
                assert_eq!(view.r1b()?, decode_base(&expected["r1b"]));
                assert_eq!(view.r1s()?, decode_scalar(&expected["r1s"]));
            }
        ),
        ("constant", "castZero") => exercise!(constant, castZero, castZero_call, (), |view| {
            assert_eq!(view.r1b()?, decode_base(&expected["r1b"]));
            assert_eq!(view.r1s()?, decode_scalar(&expected["r1s"]));
        }),
        ("unused", "unusedR1") => exercise!(unused, unusedR1, unusedR1_call, (bytes(0)), |view| {
            assert_eq!(
                view.n()?,
                Field::from_le_bytes(&field_bytes(&expected["n"])).unwrap()
            );
        }),
        ("unused", "unusedFieldR1") => {
            let field = Field::from_le_bytes(&field_bytes(&inputs[1]["field"])).unwrap();
            exercise!(
                unused,
                unusedFieldR1,
                unusedFieldR1_call,
                (bytes(0), field),
                |view| {
                    assert_eq!(
                        view.n()?,
                        Field::from_le_bytes(&field_bytes(&expected["n"])).unwrap()
                    );
                }
            )
        }
        _ => return Err("unrecognized P-256 fixture or export".into()),
    }
    Ok(())
}

fn check_zero_inverse() -> TestResult<()> {
    let base = Secp256r1Base::default();
    let scalar = Secp256r1Scalar::default();
    macro_rules! refuse {
        ($package:ident, $method:ident, ($($arg:expr),*), $error:ident) => {{
            use $package::ledger_contract as contract;
            let initial = contract::initial_state(ConstructorContext::new(()))?;
            let state = initial.ledger_state.clone();
            check_bool_cell(state.get_ref(), false);
            let context = initial.into_circuit_context(ContractAddress::default());
            assert!(matches!(contract::$method(context, $($arg),*), Err(CompactError::$error)));
            let recorded = contract::initial_state(ConstructorContext::new(()))?
                .into_circuit_context(ContractAddress::default());
            assert!(matches!(contract::recorded::$method(recorded, $($arg),*), Err(CompactError::$error)));
            check_bool_cell(state.get_ref(), false);
        }};
    }
    refuse!(unary_base, test, (base), Secp256r1BaseZeroInverse);
    refuse!(unary_scalar, test, (scalar), Secp256r1ScalarZeroInverse);
    refuse!(
        unary_controls,
        guardedInvBase,
        (true, base),
        Secp256r1BaseZeroInverse
    );
    refuse!(
        unary_controls,
        guardedInvScalar,
        (true, scalar),
        Secp256r1ScalarZeroInverse
    );
    refuse!(
        unary_controls,
        discardInvBase,
        (base),
        Secp256r1BaseZeroInverse
    );
    refuse!(
        unary_controls,
        discardInvScalar,
        (scalar),
        Secp256r1ScalarZeroInverse
    );

    use unary_controls::pure_circuits::pureDiscardInvScalar;
    let mut two = [0; 32];
    two[0] = 2;
    assert!(pureDiscardInvScalar(
        Secp256r1Scalar::from_le_bytes(two).unwrap()
    )?);
    assert_eq!(
        pureDiscardInvScalar(scalar),
        Err(CompactError::Secp256r1ScalarZeroInverse)
    );
    Ok(())
}

fn check_point_defaults(cases: &serde_json::Map<String, Value>) -> TestResult<()> {
    use point_equality::pure_circuits::getDefault;
    use point_identity::pure_circuits::identityEqual;
    let default = getDefault()?;
    assert_eq!(default, Secp256r1Point::default());
    assert!(default.is_identity());
    let values = ["storePoint-0", "storePoint-1", "storePoint-2"]
        .map(|case| decode_point(&cases[case]["inputs"][0]));
    assert_eq!(values, [default; 3]);
    identityEqual(values[0], values[1])?;
    identityEqual(values[1], values[2])?;
    Ok(())
}

fn check_accessor_refusals(cases: &serde_json::Map<String, Value>) -> TestResult<()> {
    use accessor_direct::pure_circuits::{test0, test1, test2};
    let identity = Secp256r1Point::default();
    let zero = Secp256r1Base::default();
    let x_zero = decode_point(&cases["upstream-roundtrip-test0-0"]["inputs"][0]);
    let generator = decode_point(&cases["upstream-roundtrip-test0-1"]["inputs"][0]);
    let ignored = decode_point(&cases["guarded-writes-guardedX-1"]["inputs"][1]);
    for point in [identity, ignored] {
        assert_eq!(test0(point), Err(CompactError::Secp256r1IdentityX));
        assert_eq!(test1(point), Err(CompactError::Secp256r1IdentityY));
        assert_eq!(test2(point)?, (zero, zero));
    }
    for point in [generator, x_zero] {
        let (x, y) = point.coordinates().unwrap();
        assert_eq!(test0(point)?, x);
        assert_eq!(test1(point)?, y);
        assert_eq!(test2(point)?, (x, y));
    }
    macro_rules! refuse {
        ($package:ident, $method:ident, ($($arg:expr),*), $error:expr) => {{
            use $package::ledger_contract as contract;
            let init = || contract::initial_state(ConstructorContext::new(()));
            let expected = init()?.ledger_state;
            let native = init()?.into_circuit_context(ContractAddress::default());
            let recorded = init()?.into_circuit_context(ContractAddress::default());
            let before = native.query.state.clone();
            let recorded_before = recorded.query.state.clone();
            assert_eq!(contract::$method(native $(, $arg)*).err(), Some($error));
            assert_eq!(contract::recorded::$method(recorded $(, $arg)*).err(), Some($error));
            assert_eq!(before.get_ref(), expected.get_ref());
            assert_eq!(recorded_before.get_ref(), expected.get_ref());
        }};
    }
    refuse!(
        accessor_roundtrip,
        test1,
        (),
        CompactError::Secp256r1IdentityX
    );
    refuse!(
        accessor_ledger,
        storeX,
        (),
        CompactError::Secp256r1IdentityX
    );
    refuse!(
        accessor_ledger,
        storeY,
        (),
        CompactError::Secp256r1IdentityY
    );
    refuse!(
        accessor_guarded,
        guardedX,
        (true, ignored),
        CompactError::Secp256r1IdentityX
    );
    refuse!(
        accessor_guarded,
        guardedY,
        (true, ignored),
        CompactError::Secp256r1IdentityY
    );
    refuse!(
        accessor_asserted,
        storeXChecked,
        (),
        CompactError::AssertionFailed("the identity has no coordinates".to_owned())
    );
    Ok(())
}

fn main() -> TestResult<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [mode, root, out] = args.as_slice() else {
        return Err(
            "expected parity or funded case ID, evidence root, and output directory".into(),
        );
    };
    let root = Path::new(root);
    let vectors: Value =
        serde_json::from_slice(&fs::read(root.join("preimages-first/vectors.json"))?)?;
    let cases = vectors.as_object().ok_or("expected case map")?;
    if let Some(expected) = match mode.to_str() {
        Some("parity") => Some(41),
        Some("arithmetic-parity") => Some(34),
        Some("unary-parity") => Some(30),
        Some("point-parity") => Some(17),
        Some("accessor-parity") => Some(18),
        _ => None,
    } {
        assert_eq!(cases.len(), expected);
        for (id, row) in cases {
            assert_eq!(row["fixture"] == "arithmetic", mode == "arithmetic-parity");
            assert_eq!(
                unary_fixture(row["fixture"].as_str().unwrap()),
                mode == "unary-parity"
            );
            assert_eq!(
                point_fixture(row["fixture"].as_str().unwrap()),
                mode == "point-parity"
            );
            assert_eq!(
                accessor_fixture(row["fixture"].as_str().unwrap()),
                mode == "accessor-parity"
            );
            run_case(root, id, row, None)?;
        }
        if mode == "unary-parity" {
            check_zero_inverse()?;
            println!("six native and recorded active-zero refusals, plus pure discarded inverse");
        } else if mode == "point-parity" {
            check_point_defaults(cases)?;
            println!("original default and both identity equality and hash assertions passed");
        } else if mode == "accessor-parity" {
            check_accessor_refusals(cases)?;
            println!("pure accessors and six native and recorded identity refusals passed");
        }
        println!(
            "{expected} generated Rust cases matched the TypeScript ledger values and public VM replay"
        );
        return Ok(());
    }
    let id = mode.to_str().ok_or("invalid case ID")?;
    let row = cases.get(id).ok_or("unknown case ID")?;
    assert_eq!(row["selected"].as_bool(), Some(true));
    run_case(root, id, row, Some(Path::new(out)))
}
