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

// Default/all-features checks need no generated SDK. The explicit gate selects
// an absolute current bundle; Cargo tracks this env and every actual include.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(compact_unshielded_original_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_UNSHIELDED_ORIGINAL_GENERATED");
    if std::env::var_os("CARGO_FEATURE_UNSHIELDED_ORIGINAL_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_UNSHIELDED_ORIGINAL_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute original unshielded root required"
        );
        for member in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "zkir/mintUnshieldedToSelfTest.zkir",
            "zkir/getUnshieldedBalanceTest.zkir",
            "zkir/getUnshieldedBalanceGtTest.zkir",
            "zkir/getUnshieldedBalanceLtTest.zkir",
            "zkir/sendUnshieldedToContractTest.zkir",
            "zkir/receiveUnshieldedTest.zkir",
            "zkir/sendUnshieldedToSelfTest.zkir",
            "zkir/mintUnshieldedToContractTest.zkir",
            "zkir/mintUnshieldedToUserTest.zkir",
            "zkir/sendUnshieldedToUserTest.zkir",
            "zkir/mintShieldedToSelfTest.zkir",
            "zkir/mintShieldedToContractTest.zkir",
            "zkir/mintShieldedToUserTest.zkir",
            "zkir/mintUnshieldedToSelfTest.bzkir",
            "zkir/getUnshieldedBalanceTest.bzkir",
            "zkir/getUnshieldedBalanceGtTest.bzkir",
            "zkir/getUnshieldedBalanceLtTest.bzkir",
            "zkir/sendUnshieldedToContractTest.bzkir",
            "zkir/receiveUnshieldedTest.bzkir",
            "zkir/sendUnshieldedToSelfTest.bzkir",
            "zkir/mintUnshieldedToContractTest.bzkir",
            "zkir/mintUnshieldedToUserTest.bzkir",
            "zkir/sendUnshieldedToUserTest.bzkir",
            "zkir/mintShieldedToSelfTest.bzkir",
            "zkir/mintShieldedToContractTest.bzkir",
            "zkir/mintShieldedToUserTest.bzkir",
            "keys/mintUnshieldedToSelfTest.prover",
            "keys/mintUnshieldedToSelfTest.verifier",
            "keys/getUnshieldedBalanceTest.prover",
            "keys/getUnshieldedBalanceTest.verifier",
            "keys/getUnshieldedBalanceGtTest.prover",
            "keys/getUnshieldedBalanceGtTest.verifier",
            "keys/getUnshieldedBalanceLtTest.prover",
            "keys/getUnshieldedBalanceLtTest.verifier",
            "keys/sendUnshieldedToContractTest.prover",
            "keys/sendUnshieldedToContractTest.verifier",
            "keys/receiveUnshieldedTest.prover",
            "keys/receiveUnshieldedTest.verifier",
            "keys/sendUnshieldedToSelfTest.prover",
            "keys/sendUnshieldedToSelfTest.verifier",
            "keys/mintUnshieldedToContractTest.prover",
            "keys/mintUnshieldedToContractTest.verifier",
            "keys/mintUnshieldedToUserTest.prover",
            "keys/mintUnshieldedToUserTest.verifier",
            "keys/sendUnshieldedToUserTest.prover",
            "keys/sendUnshieldedToUserTest.verifier",
            "keys/mintShieldedToSelfTest.prover",
            "keys/mintShieldedToSelfTest.verifier",
            "keys/mintShieldedToContractTest.prover",
            "keys/mintShieldedToContractTest.verifier",
            "keys/mintShieldedToUserTest.prover",
            "keys/mintShieldedToUserTest.verifier",
        ] {
            let path = root.join(member);
            assert!(
                path.is_file(),
                "original unshielded member missing: {}",
                path.display()
            );
            println!("cargo::rerun-if-changed={}", path.display());
        }
        println!("cargo::rustc-cfg=compact_unshielded_original_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_outer_vector_product_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_OUTER_VECTOR_PRODUCT_GENERATED");
    if std::env::var_os("CARGO_FEATURE_OUTER_VECTOR_PRODUCT_PROOFS").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_OUTER_VECTOR_PRODUCT_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(
            pool.is_absolute(),
            "absolute complete outer Vector SDK pool required"
        );
        for (source, count) in [("family", 15), ("original", 1)] {
            let root = pool.join(source).join("contract");
            for name in ["lib.rs", "compact-rust-ir.json", "rust-capabilities.json"] {
                assert!(
                    root.join(name).is_file(),
                    "complete generated SDK member required"
                );
                println!("cargo::rerun-if-changed={}", root.join(name).display());
            }
            for index in 0..count {
                let path = root.join(format!("checked-programs/program-{index}.json"));
                assert!(path.is_file(), "complete Program inventory required");
                println!("cargo::rerun-if-changed={}", path.display());
            }
        }
        println!("cargo::rustc-cfg=compact_outer_vector_product_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_component_product_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_COMPONENT_PRODUCT_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_COMPONENT_PRODUCT_VECTOR_WIDENING").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_COMPONENT_PRODUCT_VECTOR_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(
            pool.is_absolute(),
            "absolute complete component Vector SDK pool required"
        );
        for (source, count) in [("family", 14), ("original_nested", 1)] {
            let root = pool.join(source).join("contract");
            for name in ["lib.rs", "compact-rust-ir.json", "rust-capabilities.json"] {
                assert!(
                    root.join(name).is_file(),
                    "complete generated SDK member required"
                );
                println!("cargo::rerun-if-changed={}", root.join(name).display());
            }
            for index in 0..count {
                let path = root.join(format!("checked-programs/program-{index}.json"));
                assert!(path.is_file(), "complete Program inventory required");
                println!("cargo::rerun-if-changed={}", path.display());
            }
        }
        println!("cargo::rustc-cfg=compact_component_product_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_fold_vector_result_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_FOLD_VECTOR_RESULT_GENERATED");
    if std::env::var_os("CARGO_FEATURE_FOLD_VECTOR_RESULT_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_FOLD_VECTOR_RESULT_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_FOLD_VECTOR_RESULT_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_fold_vector_result_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_vector_alias_source_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_ALIAS_SOURCE_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_ALIAS_SOURCE_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_ALIAS_SOURCE_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_ALIAS_SOURCE_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_vector_alias_source_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_aggregate_literal_projection_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED");
    if std::env::var_os("CARGO_FEATURE_AGGREGATE_LITERAL_PROJECTION_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_aggregate_literal_projection_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_struct_member_literal_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STRUCT_MEMBER_LITERAL_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STRUCT_MEMBER_LITERAL_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STRUCT_MEMBER_LITERAL_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STRUCT_MEMBER_LITERAL_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_struct_member_literal_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_direct_stateful_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_DIRECT_STATEFUL_VECTOR_LITERAL_RETURNS").is_some()
        && let Some(root) = std::env::var_os("COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_direct_stateful_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_vector_value_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_VECTOR_VALUE_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_VECTOR_VALUE_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_VECTOR_VALUE_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_VECTOR_VALUE_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_vector_value_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_local_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_LOCAL_VECTOR_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_local_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_bytes_remaining_proof_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_REMAINING_GENERATED");
    let equality = std::env::var_os("CARGO_FEATURE_BYTES_EQUALITY_PROOF").is_some();
    let slice = std::env::var_os("CARGO_FEATURE_BYTES_SLICE_PROOF").is_some();
    // Workspace --all-features checks select both feature flags without a
    // generated crate. A proof run selects exactly one and supplies its root.
    if equality && slice {
        assert!(
            std::env::var_os("COMPACT_BYTES_REMAINING_GENERATED").is_none(),
            "one generated Bytes proof crate per runner"
        );
    }
    if equality ^ slice {
        let root = std::path::PathBuf::from(
            std::env::var_os("COMPACT_BYTES_REMAINING_GENERATED")
                .expect("complete Bytes generated root required"),
        );
        assert!(root.is_absolute(), "absolute Bytes generated root required");
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
        ] {
            let member = root.join(relative);
            assert!(member.is_file(), "complete Bytes generated member required");
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_bytes_remaining_proof_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_bytes_next_proof_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_NEXT_GENERATED");
    let fold = std::env::var_os("CARGO_FEATURE_BYTES_FOLD_BOOLEAN_PROOF").is_some();
    let empty_loop = std::env::var_os("CARGO_FEATURE_BYTES_LOOP_EMPTY_PROOF").is_some();
    let next_root = std::env::var_os("COMPACT_BYTES_NEXT_GENERATED");
    // Workspace --all-features checks select both flags without a generated
    // crate. A proof run selects one flag and supplies its exact root.
    if fold && empty_loop {
        assert!(next_root.is_none(), "one next Bytes proof crate per runner");
    }
    if fold ^ empty_loop {
        let root = std::path::PathBuf::from(
            next_root.expect("complete next Bytes generated root required"),
        );
        assert!(
            root.is_absolute(),
            "absolute next Bytes generated root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete next Bytes generated member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_bytes_next_proof_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_bytes_next_remaining_proof_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_NEXT_REMAINING_GENERATED");
    let remaining = [
        "CARGO_FEATURE_BYTES_FOLD_CONDITIONAL_PROOF",
        "CARGO_FEATURE_BYTES_FOLD_SUMMATION_PROOF",
        "CARGO_FEATURE_BYTES_LOOP_SPREAD_PROOF",
        "CARGO_FEATURE_BYTES_LOOP_ITERATION_PROOF",
        "CARGO_FEATURE_BYTES_MANUAL_UPDATE_PROOF",
    ]
    .iter()
    .filter(|name| std::env::var_os(name).is_some())
    .count();
    let remaining_root = std::env::var_os("COMPACT_BYTES_NEXT_REMAINING_GENERATED");
    assert!(
        remaining <= 1 || remaining_root.is_none(),
        "one remaining Bytes proof crate per generated runner"
    );
    if remaining == 1 {
        let root = std::path::PathBuf::from(
            remaining_root.expect("complete remaining Bytes generated root required"),
        );
        assert!(
            root.is_absolute(),
            "absolute remaining Bytes generated root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete remaining Bytes generated member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_bytes_next_remaining_proof_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_for_generic_bound_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_import_ledger_field_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_bytes_indexed_access_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_bytes_large_reconstruction_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_LARGE_RECONSTRUCTION_GENERATED");
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_LARGE_RECONSTRUCTION_GATE");
    if std::env::var_os("CARGO_FEATURE_BYTES_LARGE_RECONSTRUCTION_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_BYTES_LARGE_RECONSTRUCTION_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute slow Bytes generated root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/bytes_manual_update_large_reconstruction.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete slow Bytes generated member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_bytes_large_reconstruction_generated");
    }
    if std::env::var_os("CARGO_FEATURE_BYTES_LARGE_RECONSTRUCTION_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_BYTES_LARGE_RECONSTRUCTION_GATE")
    {
        let root = std::path::PathBuf::from(root);
        let transcript = root.join("typescript-transcript.json");
        assert!(
            root.is_absolute() && transcript.is_file(),
            "exact semantic transcript required"
        );
        println!("cargo::rerun-if-changed={}", transcript.display());
    }
    println!("cargo::rerun-if-env-changed=COMPACT_BYTES_INDEXED_ACCESS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_BYTES_INDEXED_ACCESS_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_BYTES_INDEXED_ACCESS_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute Bytes indexed access root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/bytes_loop_indexed_access.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete Bytes indexed access member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_bytes_indexed_access_generated");
    }
    println!("cargo::rerun-if-env-changed=COMPACT_IMPORT_LEDGER_FIELD_GENERATED");
    if std::env::var_os("CARGO_FEATURE_IMPORT_LEDGER_FIELD_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_IMPORT_LEDGER_FIELD_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute import/ledger_field root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/import_ledger_field.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete import/ledger_field member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_import_ledger_field_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_keyword_constructor_parameters_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_if_empty_else_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_if_branches_set_insert_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_IF_BRANCHES_SET_INSERT_GENERATED");
    if std::env::var_os("CARGO_FEATURE_IF_BRANCHES_SET_INSERT_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_IF_BRANCHES_SET_INSERT_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute if/branches_write_ledger root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/if_branches_write_ledger.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete if/branches_write_ledger member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_if_branches_set_insert_generated");
    }
    println!("cargo::rerun-if-env-changed=COMPACT_IF_EMPTY_ELSE_GENERATED");
    if std::env::var_os("CARGO_FEATURE_IF_EMPTY_ELSE_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_IF_EMPTY_ELSE_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute if/empty_else_branch root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/if_empty_else_branch.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete if/empty_else_branch member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_if_empty_else_generated");
    }
    println!("cargo::rerun-if-env-changed=COMPACT_FOR_GENERIC_BOUND_GENERATED");
    if std::env::var_os("CARGO_FEATURE_FOR_GENERIC_BOUND_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_FOR_GENERIC_BOUND_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute for/generic_bound root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/for_generic_bound.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete for/generic_bound member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_for_generic_bound_generated");
    }
    println!("cargo::rerun-if-env-changed=COMPACT_KEYWORD_CONSTRUCTOR_PARAMETERS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_KEYWORD_CONSTRUCTOR_PARAMETERS_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_KEYWORD_CONSTRUCTOR_PARAMETERS_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute(),
            "absolute constructor/parameters root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
            "zkir/constructor_parameters.zkir",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete constructor/parameters member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        println!("cargo::rustc-cfg=compact_keyword_constructor_parameters_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_keyword_disclose_generated)");
    println!("cargo::rustc-check-cfg=cfg(compact_keyword_disclose_branch)");
    println!("cargo::rustc-check-cfg=cfg(compact_keyword_disclose_constructor)");
    println!("cargo::rerun-if-env-changed=COMPACT_KEYWORD_DISCLOSE_GENERATED");
    println!("cargo::rerun-if-env-changed=COMPACT_KEYWORD_DISCLOSE_CASE");
    println!("cargo::rerun-if-env-changed=COMPACT_KEYWORD_DISCLOSE_CIRCUIT");
    println!("cargo::rerun-if-env-changed=COMPACT_KEYWORD_DISCLOSE_WRONG");
    if std::env::var_os("CARGO_FEATURE_KEYWORD_DISCLOSE_PROOF").is_some()
        && let Some(root) = std::env::var_os("COMPACT_KEYWORD_DISCLOSE_GENERATED")
    {
        let root = std::path::PathBuf::from(root);
        let case =
            std::env::var("COMPACT_KEYWORD_DISCLOSE_CASE").expect("pinned disclose case required");
        let cfg = match case.as_str() {
            "branch_conditions" => "compact_keyword_disclose_branch",
            "branch_conditions_in_constructor" => "compact_keyword_disclose_constructor",
            _ => panic!("unsupported pinned disclose proof case"),
        };
        assert_eq!(
            std::env::var("COMPACT_KEYWORD_DISCLOSE_CIRCUIT").as_deref(),
            Ok(format!("disclose_{case}").as_str()),
            "disclose circuit must match the selected case"
        );
        let wrong = match case.as_str() {
            "branch_conditions" => "if_empty_else_branch",
            "branch_conditions_in_constructor" => "type_new_type_of_adt",
            _ => unreachable!(),
        };
        assert_eq!(
            std::env::var("COMPACT_KEYWORD_DISCLOSE_WRONG").as_deref(),
            Ok(wrong),
            "disclose independent wrong-verifier circuit changed"
        );
        assert!(
            root.is_absolute(),
            "absolute disclose generated root required"
        );
        for relative in [
            "contract/lib.rs",
            "contract/compact-rust-ir.json",
            "contract/rust-capabilities.json",
        ] {
            let member = root.join(relative);
            assert!(
                member.is_file(),
                "complete disclose generated member required"
            );
            println!("cargo::rerun-if-changed={}", member.display());
        }
        let circuit = format!("zkir/disclose_{case}.zkir");
        let member = root.join(circuit);
        assert!(member.is_file(), "complete disclose raw ZKIR required");
        println!("cargo::rerun-if-changed={}", member.display());
        println!("cargo::rustc-cfg=compact_keyword_disclose_generated");
        println!("cargo::rustc-cfg={cfg}");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_product_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_PRODUCT_VECTOR_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_product_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_vector_product_elements_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_PRODUCT_ELEMENTS").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED must name the complete generated SDK"
        );
        let helper = std::path::PathBuf::from(
            std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED")
                .expect("paired complete helper SDK is required"),
        );
        assert!(
            helper.is_absolute() && helper.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED must name the complete helper SDK"
        );
        let expected_helper = root
            .parent()
            .expect("candidate SDK pool required")
            .join("helper");
        assert_eq!(
            helper.canonicalize().expect("helper SDK must exist"),
            expected_helper
                .canonicalize()
                .expect("paired sibling helper SDK must exist"),
            "helper SDK must be the candidate pool's complete helper sibling"
        );
        let checked = std::path::PathBuf::from(
            std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED")
                .expect("complete checked-helper SDK is required"),
        );
        assert!(
            checked.is_absolute() && checked.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED must name the complete checked-helper SDK"
        );
        let expected_checked = root
            .parent()
            .expect("candidate SDK pool required")
            .join("checked_helper");
        assert_eq!(
            checked
                .canonicalize()
                .expect("checked-helper SDK must exist"),
            expected_checked
                .canonicalize()
                .expect("checked-helper sibling SDK must exist"),
            "checked-helper SDK must be the candidate pool's complete checked_helper sibling"
        );
        println!("cargo::rustc-cfg=compact_vector_product_elements_generated");
    }

    println!("cargo::rustc-check-cfg=cfg(compact_vector_component_products_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_COMPONENT_PRODUCTS").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(pool.is_absolute(), "absolute complete SDK pool required");
        for name in ["witness_before", "witness_after", "inner_product_single"] {
            for relative in [
                "lib.rs",
                "compact-rust-ir.json",
                "rust-capabilities.json",
                "checked-programs/program-0.json",
            ] {
                let path = pool.join(name).join("contract").join(relative);
                assert!(path.is_file(), "complete selected SDK member required");
                println!("cargo::rerun-if-changed={}", path.display());
            }
        }
        println!("cargo::rustc-cfg=compact_vector_component_products_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_nested_component_products_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_NESTED_COMPONENT_PRODUCTS").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(pool.is_absolute(), "absolute complete SDK pool required");
        let root = pool.join("family/contract");
        for relative in ["lib.rs", "compact-rust-ir.json", "rust-capabilities.json"] {
            let path = root.join(relative);
            assert!(path.is_file(), "complete family SDK member required");
            println!("cargo::rerun-if-changed={}", path.display());
        }
        for index in 0..13 {
            let path = root.join(format!("checked-programs/program-{index}.json"));
            assert!(path.is_file(), "complete family Program member required");
            println!("cargo::rerun-if-changed={}", path.display());
        }
        println!("cargo::rustc-cfg=compact_nested_component_products_generated");
    }
}
