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

//! Empty maps retain static review evidence without claiming runtime execution.
use super::super::{
    declarations::Declarations,
    functional::{FunctionalProgram, Ledger8Target, OriginEvidence, SourceAuthority, SourceSite},
    model::{Binding, Block, CheckedPlan, Operation, Plan, Value},
};
use super::*;
use crate::ir::{Parameter, SourceLocation, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;
struct Authority(SourceLocation);
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, _: SourceSite<'_>) -> OriginEvidence<'a> {
        OriginEvidence::BoundOriginalNode(&self.0)
    }
}
#[test]
fn empty_map_metadata_keeps_inventory_token_and_explains_static_body_occurrences() {
    let ty = Type::Vector {
        element: Box::new(Type::Unit),
        length: 0,
    };
    let source = Value {
        id: 0,
        ty: ty.clone(),
    };
    let formal = Value {
        id: 1,
        ty: Type::Unit,
    };
    let result = Value {
        id: 2,
        ty: ty.clone(),
    };
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "source".into(),
            ty: ty.clone(),
        }],
        actions: vec![],
        result: ty.clone(),
        return_value: StateReturn::Unit,
    };
    let plan = Plan {
        parameters: vec![source.clone()],
        result_type: ty,
        body: Block {
            bindings: vec![Binding {
                value: result.clone(),
                path: "map".into(),
                operation: Operation::VectorMap {
                    source,
                    parameter: formal.clone(),
                    body: Block {
                        bindings: vec![],
                        result: formal,
                    },
                },
            }],
            result,
        },
    };
    let checked = CheckedPlan::validate(
        plan,
        Declarations::new(&HashMap::new(), &HashMap::new()).with_root(&root),
    )
    .unwrap();
    let authority = Authority(SourceLocation {
        file: "map.compact".into(),
        line: 1,
        column: 1,
    });
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let value: serde_json::Value = serde_json::from_str(&metadata(&program)).unwrap();
    assert_eq!(
        value["inventory_semantics"],
        "guards_and_occurrences_not_a_linear_execution_schedule"
    );
    let semantics = value["map_body_semantics"].as_str().unwrap();
    assert!(semantics.contains("lexical_static_occurrences"));
    assert!(semantics.contains("zero times for an empty vector"));
    assert!(semantics.contains("not a linear execution trace"));
}
