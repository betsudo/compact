// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Review metadata, never execution authority or a replacement program format.
use super::functional::{
    Capability, ConsumedSourceBytes, EffectiveGuard, FunctionalProgram, OriginEvidence, Privacy,
    SnapshotCorrespondence,
};
use serde_json::{Value, json};

pub(super) fn metadata(program: &FunctionalProgram<'_, '_>) -> String {
    let guards = program
        .guards()
        .iter()
        .enumerate()
        .map(|(id, guard)| match guard {
            EffectiveGuard::Root => json!({"id": id, "kind": "root"}),
            EffectiveGuard::Branch {
                parent,
                condition,
                taken,
            } => json!({
                "id": id, "kind": "branch", "parent": parent.0,
                "condition_value_id": condition, "taken": taken,
            }),
        })
        .collect::<Vec<_>>();
    let inputs = program
        .inputs()
        .iter()
        .map(|input| {
            json!({
                "value_id": input.value().id,
                "type": input.value().ty,
                "privacy": privacy(input.privacy()),
                "origin": origin(input.origin()),
            })
        })
        .collect::<Vec<_>>();
    let effects = program
        .effects()
        .iter()
        .map(|effect| {
            let binding = effect.binding();
            json!({
                "value_id": binding.value.id,
                "type": binding.value.ty,
                "diagnostic_path": binding.path,
                "effective_guard_id": effect.guard().0,
                "required_capability": capability(effect.capability()),
                "privacy": privacy(effect.privacy()),
                "origin": origin(effect.origin()),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": 1,
        "purpose": "review_only_not_execution_authority",
        "ledger_version": program.target().ledger_version(),
        "root": program.checked().declarations().root().map(|root| root.name.as_str()),
        "root_origin": origin(program.root_origin()),
        "required_capabilities": program.requirements().iter().copied().map(capability).collect::<Vec<_>>(),
        "inventory_semantics": "guards_and_occurrences_not_a_linear_execution_schedule",
        "map_body_semantics": "lexical_static_occurrences_with_branch_guards; a body may execute per selected element and zero times for an empty vector; not a linear execution trace",
        "guards": guards,
        "inputs": inputs,
        "effects": effects,
    }).to_string()
}

fn capability(value: Capability) -> &'static str {
    match value {
        Capability::Pure => "pure",
        Capability::CellRead => "cell_read",
        Capability::CellWrite => "cell_write",
        Capability::CounterRead => "counter_read",
        Capability::CounterIncrement => "counter_increment",
        Capability::CounterDecrement => "counter_decrement",
        Capability::CounterReset => "counter_reset",
        Capability::CounterCompare => "counter_compare",
        Capability::BooleanWitness => "boolean_witness",
        Capability::TypedWitness => "typed_witness",
        Capability::ExactWitness => "exact_witness",
        Capability::Require => "require",
        Capability::Select => "select",
        Capability::Call => "call",
        Capability::VectorMap => "vector_map",
        Capability::MapRead => "map_read",
        Capability::MapLookup => "map_lookup",
        Capability::MapRemove => "map_remove",
        Capability::MapWrite => "map_write",
        Capability::SetRead => "set_read",
        Capability::SetWrite => "set_write",
        Capability::ListRead => "list_read",
        Capability::ListHead => "list_head",
        Capability::ListWrite => "list_write",
        Capability::MerkleInspect => "merkle_inspect",
        Capability::MerkleCheckRoot => "merkle_check_root",
        Capability::MerkleInsert => "merkle_insert",
        Capability::MerkleInsertHash => "merkle_insert_hash",
        Capability::MerkleReset => "merkle_reset",
        Capability::MerkleResetHistory => "merkle_reset_history",
    }
}

fn privacy(value: Privacy) -> &'static str {
    match value {
        Privacy::Unknown => "unknown",
    }
}

fn origin(value: OriginEvidence<'_>) -> Value {
    let (kind, source) = match value {
        OriginEvidence::Unknown => ("unknown", None),
        OriginEvidence::BoundOriginalNode(source) => ("bound_original_node", Some(source)),
        OriginEvidence::BoundOriginalRootFallback(source) => {
            ("bound_original_root_fallback", Some(source))
        }
    };
    let consumed = match value.consumed_source_bytes() {
        ConsumedSourceBytes::Unknown => "unknown",
    };
    let snapshot = match value.snapshot_correspondence() {
        SnapshotCorrespondence::Unknown => "unknown",
    };
    json!({
        "kind": kind, "source": source,
        "consumed_source_bytes": consumed,
        "snapshot_correspondence": snapshot,
    })
}

#[cfg(test)]
#[path = "functional_metadata_map_tests.rs"]
mod map_tests;
