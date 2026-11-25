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

//! Standalone generated proof-data facades for bounded semantic families.
//! Other shapes retain their existing native/recorded/transaction APIs.
use crate::ir::{
    CounterAmount, Expr, LedgerField, LedgerFieldKind, LocalBinding, Parameter, PureCircuit,
    StateAction, StateReturn, StatefulCircuit, Type,
};
use crate::{
    RenderError, field_literal_bytes, ident, is_compact_struct_instantiation,
    public_parameter_idents, rust_type, semantic_identifier,
};
use quote::quote;
use std::collections::{HashMap, HashSet};

mod authenticated_cell_clear;

/// A closed Field literal may be bound by the frontend before a Set mutation.
/// Admit only that single typed binding and its exact use, not general lets,
/// calls, coercions or expressions whose evaluation can have effects.
fn literal_set_action(
    action: &StateAction,
    remove: bool,
    fields: &HashMap<&str, &LedgerField>,
) -> bool {
    let (action, binding) = match action {
        StateAction::Let { bindings, action } if bindings.len() == 1 => {
            let binding = &bindings[0];
            if binding.ty != Type::Field
                || !matches!(&binding.value, Expr::FieldLiteral { value } if field_literal_bytes(value).is_ok())
            {
                return false;
            }
            (action.as_ref(), Some(binding))
        }
        _ => (action, None),
    };
    let (field, index, value) = match (remove, action) {
        (
            true,
            StateAction::SetRemove {
                field,
                index,
                value,
            },
        )
        | (
            false,
            StateAction::SetInsert {
                field,
                index,
                value,
            },
        ) => (field, index, value),
        _ => return false,
    };
    // lib.rs has already validated the contract's complete physical paths.
    let Some(declaration) = fields.get(field.as_str()) else {
        return false;
    };
    if declaration.index != *index
        || !matches!(
            declaration.declaration,
            LedgerFieldKind::Set { ty: Type::Field }
        )
    {
        return false;
    }
    match (binding, value) {
        (None, Expr::FieldLiteral { value }) => field_literal_bytes(value).is_ok(),
        (Some(binding), Expr::Parameter { name }) => binding.name == *name,
        _ => false,
    }
}

fn unit_set_mutations(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    circuit.parameters.is_empty()
        && circuit.result == Type::Unit
        && circuit.return_value == StateReturn::Unit
        && matches!(&circuit.actions[..], [remove, insert]
            if literal_set_action(remove, true, fields) && literal_set_action(insert, false, fields))
}

/// A closed unsigned expression retains both its declared type and checked value.
/// Classification never evaluates a ledger read or rewrites runtime arithmetic.
#[derive(Clone, Copy)]
struct CounterUnsigned {
    max: u64,
    value: u64,
}

struct CounterBudget {
    remaining: usize,
}

impl CounterBudget {
    fn visit(&mut self, depth: usize) -> Option<()> {
        if depth >= 64 {
            return None;
        }
        self.remaining = self.remaining.checked_sub(1)?;
        Some(())
    }
}

fn counter_unsigned(text: &str) -> Option<u64> {
    // The frontend emits canonical decimal Uint values, at most UInt64 here.
    if text.is_empty() || text.len() > 20 {
        return None;
    }
    let value = text.parse::<u64>().ok()?;
    (value.to_string() == text).then_some(value)
}

fn counter_bindings(
    bindings: &[LocalBinding],
    env: &HashMap<String, CounterUnsigned>,
    budget: &mut CounterBudget,
    depth: usize,
) -> Option<HashMap<String, CounterUnsigned>> {
    if bindings.is_empty() {
        return None;
    }
    let mut local = env.clone();
    for binding in bindings {
        budget.visit(depth)?;
        if binding.name.is_empty() {
            return None;
        }
        let Type::Unsigned { max } = &binding.ty else {
            return None;
        };
        // Initializers see earlier bindings (or an outer shadowed binding),
        // never themselves or a later binding. The local scope cannot escape.
        let value = counter_closed(&binding.value, &local, budget, depth + 1)?;
        if value.max != counter_unsigned(max)? {
            return None;
        }
        local.insert(binding.name.clone(), value);
    }
    Some(local)
}

fn counter_closed(
    value: &Expr,
    env: &HashMap<String, CounterUnsigned>,
    budget: &mut CounterBudget,
    depth: usize,
) -> Option<CounterUnsigned> {
    budget.visit(depth)?;
    let result = match value {
        Expr::UnsignedLiteral { value, max } => CounterUnsigned {
            max: counter_unsigned(max)?,
            value: counter_unsigned(value)?,
        },
        Expr::Parameter { name } => *env.get(name)?,
        Expr::UnsignedCast { max, value } => CounterUnsigned {
            max: counter_unsigned(max)?,
            value: counter_closed(value, env, budget, depth + 1)?.value,
        },
        Expr::UnsignedMultiply { max, left, right }
        | Expr::UnsignedSubtract { max, left, right } => {
            let left = counter_closed(left, env, budget, depth + 1)?;
            let right = counter_closed(right, env, budget, depth + 1)?;
            let result = if matches!(value, Expr::UnsignedMultiply { .. }) {
                left.value.checked_mul(right.value)?
            } else {
                left.value.checked_sub(right.value)?
            };
            CounterUnsigned {
                max: counter_unsigned(max)?,
                value: result,
            }
        }
        Expr::Let { bindings, body } => {
            let local = counter_bindings(bindings, env, budget, depth + 1)?;
            counter_closed(body, &local, budget, depth + 1)?
        }
        // No calls, conditionals, reads, witnesses or other expressions, even
        // when they would be in a branch not taken by the original fixture.
        _ => return None,
    };
    (result.value <= result.max).then_some(result)
}

fn counter_slot(field: &str, index: u8, fields: &HashMap<&str, &LedgerField>) -> bool {
    let Some(declaration) = fields.get(field) else {
        return false;
    };
    // Global physical-path uniqueness is already validated by lib.rs. This
    // optional family additionally admits only the actual root Counter shape.
    declaration.id == field
        && declaration.declaration == LedgerFieldKind::Counter
        && declaration.index == index
        && index < 16
        && declaration.path.len() <= 1
        && declaration.physical_path() == vec![index]
}

fn counter_assertion(
    condition: &Expr,
    env: &HashMap<String, CounterUnsigned>,
    fields: &HashMap<&str, &LedgerField>,
    budget: &mut CounterBudget,
    depth: usize,
) -> Option<()> {
    budget.visit(depth)?;
    let Expr::Equal { left, right } = condition else {
        return None;
    };
    let (read, constant) = match (left.as_ref(), right.as_ref()) {
        (read @ Expr::CounterRead { .. }, constant)
        | (constant, read @ Expr::CounterRead { .. }) => (read, constant),
        _ => return None,
    };
    budget.visit(depth + 1)?;
    let Expr::CounterRead { field, index } = read else {
        return None;
    };
    if !counter_slot(field, *index, fields)
        || counter_closed(constant, env, budget, depth + 1)?.max != u64::MAX
    {
        return None;
    }
    Some(())
}

fn counter_action(
    action: &StateAction,
    env: &HashMap<String, CounterUnsigned>,
    fields: &HashMap<&str, &LedgerField>,
    budget: &mut CounterBudget,
    depth: usize,
) -> Option<(bool, bool)> {
    budget.visit(depth)?;
    match action {
        StateAction::CounterIncrement {
            field,
            index,
            amount,
        }
        | StateAction::CounterDecrement {
            field,
            index,
            amount,
        } => {
            if !counter_slot(field, *index, fields) {
                return None;
            }
            budget.visit(depth + 1)?;
            match amount {
                CounterAmount::Literal { .. } => (), // u16 in the typed IR.
                CounterAmount::Parameter { name } if env.get(name)?.max == u16::MAX as u64 => (),
                _ => return None,
            }
            Some((true, false))
        }
        StateAction::CounterReset { field, index } => {
            counter_slot(field, *index, fields).then_some((true, false))
        }
        StateAction::Assert { condition, .. } => {
            counter_assertion(condition, env, fields, budget, depth + 1)?;
            Some((false, true))
        }
        StateAction::Let { bindings, action } => {
            let local = counter_bindings(bindings, env, budget, depth + 1)?;
            counter_action(action, &local, fields, budget, depth + 1)
        }
        // No other writes, branches, helper calls or witnesses join this family.
        _ => None,
    }
}

/// Counter writes plus read/equality assertions from a closed, typed Unit graph.
/// All actions are inspected; no fixture names, amounts or assertion text matter.
fn unit_counter_program(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    if !circuit.parameters.is_empty()
        || circuit.result != Type::Unit
        || circuit.return_value != StateReturn::Unit
    {
        return false;
    }
    let mut budget = CounterBudget { remaining: 4096 };
    let env = HashMap::new();
    let (mut mutation, mut assertion) = (false, false);
    for action in &circuit.actions {
        let Some((writes, reads)) = counter_action(action, &env, fields, &mut budget, 0) else {
            return false;
        };
        mutation |= writes;
        assertion |= reads;
    }
    mutation && assertion
}

/// The concrete standard-library Maybe<Field> carrier, including renamed
/// compiler instantiations. This does not admit arbitrary structured outputs.
fn maybe_field(ty: &Type) -> bool {
    matches!(ty, Type::Struct { name, fields }
        if is_compact_struct_instantiation(name, "Maybe")
            && fields.len() == 2
            && fields[0].name == "is_some" && fields[0].ty == Type::Boolean
            && fields[1].name == "value" && fields[1].ty == Type::Field)
}

/// A closed read-only graph from the genuine Tiny frontend family. The graph
/// may cross stateful enum/Cell predicates and pure some/none constructors.
/// Every reachable branch is checked; witness calls, actions and unknown
/// expressions refuse even when they occur in a branch not taken at runtime.
/// The optional classifier refuses beyond 64 active callees or 4096 visits,
/// bounding repeated acyclic DAG expansion without changing the old APIs.
fn maybe_field_getter(
    circuit: &StatefulCircuit,
    circuits: &[StatefulCircuit],
    pure: &[PureCircuit],
    ledger_fields: &HashMap<&str, &LedgerField>,
) -> bool {
    // Immutable declarations travel together; recursion retains the same
    // mutable active-set/fuel references and exact traversal order.
    struct ReadOnlyCallees<'a> {
        stateful: &'a [StatefulCircuit],
        pure: &'a [PureCircuit],
    }
    fn scalar(ty: &Type) -> bool {
        matches!(ty, Type::Boolean | Type::Field | Type::Enum { .. })
    }
    fn stateful(
        circuit: &StatefulCircuit,
        callees: &ReadOnlyCallees<'_>,
        fields: &HashMap<&str, &LedgerField>,
        active: &mut HashSet<String>,
        fuel: &mut usize,
    ) -> Option<(Type, bool)> {
        *fuel = (*fuel).checked_sub(1)?;
        let key = format!("stateful:{}", circuit.name);
        if !circuit.actions.is_empty()
            || !circuit.parameters.iter().all(|p| scalar(&p.ty))
            || !(scalar(&circuit.result) || maybe_field(&circuit.result))
            || active.len() >= 64
            || !active.insert(key.clone())
        {
            return None;
        }
        let result = match &circuit.return_value {
            StateReturn::Expression { value } => expression(
                value,
                &circuit.parameters,
                true,
                callees,
                fields,
                active,
                fuel,
            ),
            _ => None,
        };
        active.remove(&key);
        result.filter(|(ty, _)| *ty == circuit.result)
    }
    fn expression(
        value: &Expr,
        parameters: &[Parameter],
        allow_reads: bool,
        callees: &ReadOnlyCallees<'_>,
        fields: &HashMap<&str, &LedgerField>,
        active: &mut HashSet<String>,
        fuel: &mut usize,
    ) -> Option<(Type, bool)> {
        *fuel = (*fuel).checked_sub(1)?;
        let mut recurse = |value| {
            expression(
                value,
                parameters,
                allow_reads,
                callees,
                fields,
                active,
                fuel,
            )
        };
        match value {
            Expr::Boolean { .. } => Some((Type::Boolean, false)),
            Expr::FieldLiteral { value } if field_literal_bytes(value).is_ok() => {
                Some((Type::Field, false))
            }
            Expr::Parameter { name } => parameters
                .iter()
                .find(|p| p.name == *name)
                .map(|p| (p.ty.clone(), false)),
            Expr::EnumVariant { ty, variant } => match ty {
                Type::Enum { variants, .. } if variants.contains(variant) => {
                    Some((ty.clone(), false))
                }
                _ => None,
            },
            Expr::CellRead { field, index } if allow_reads => {
                let declaration = fields.get(field.as_str())?;
                let LedgerFieldKind::Cell { ty } = &declaration.declaration else {
                    return None;
                };
                (declaration.index == *index && scalar(ty)).then(|| (ty.clone(), true))
            }
            Expr::Coerce { value, ty } => {
                let (actual, read) = recurse(value)?;
                (actual == *ty).then_some((actual, read)) // identity coercions only
            }
            Expr::Equal { left, right } => {
                let (left_ty, left_read) = recurse(left)?;
                let (right_ty, right_read) = recurse(right)?;
                (left_ty == right_ty && scalar(&left_ty))
                    .then_some((Type::Boolean, left_read | right_read))
            }
            Expr::If {
                condition,
                then,
                otherwise,
            } => {
                let (condition_ty, condition_read) = recurse(condition)?;
                let (then_ty, then_read) = recurse(then)?;
                let (otherwise_ty, otherwise_read) = recurse(otherwise)?;
                (condition_ty == Type::Boolean && then_ty == otherwise_ty)
                    .then_some((then_ty, condition_read | then_read | otherwise_read))
            }
            Expr::StructLiteral { ty, fields: values } if maybe_field(ty) && values.len() == 2 => {
                let (flag, flag_read) = recurse(&values[0])?;
                let (value, value_read) = recurse(&values[1])?;
                (flag == Type::Boolean && value == Type::Field)
                    .then(|| (ty.clone(), flag_read | value_read))
            }
            Expr::Call { name, arguments } => {
                let mut argument_types = Vec::new();
                let mut argument_read = false;
                for argument in arguments {
                    let (ty, read) = recurse(argument)?;
                    argument_types.push(ty);
                    argument_read |= read;
                }
                // Release the argument walk's mutable borrow before visiting a callee.
                drop(recurse);
                let stateful_targets = callees
                    .stateful
                    .iter()
                    .filter(|c| c.name == *name)
                    .collect::<Vec<_>>();
                let pure_targets = callees
                    .pure
                    .iter()
                    .filter(|c| c.name == *name)
                    .collect::<Vec<_>>();
                let check_arguments = |parameters: &[Parameter]| {
                    parameters.len() == argument_types.len()
                        && parameters
                            .iter()
                            .zip(&argument_types)
                            .all(|(p, ty)| scalar(&p.ty) && p.ty == *ty)
                };
                let (ty, read) = match (&stateful_targets[..], &pure_targets[..]) {
                    ([callee], []) if allow_reads && check_arguments(&callee.parameters) => {
                        stateful(callee, callees, fields, active, fuel)?
                    }
                    ([], [callee])
                        if check_arguments(&callee.parameters)
                            && (scalar(&callee.result) || maybe_field(&callee.result)) =>
                    {
                        let key = format!("pure:{}", callee.name);
                        if active.len() >= 64 || !active.insert(key.clone()) {
                            return None;
                        }
                        let result = expression(
                            &callee.body,
                            &callee.parameters,
                            false,
                            callees,
                            fields,
                            active,
                            fuel,
                        );
                        active.remove(&key);
                        let (ty, read) = result?;
                        if ty != callee.result {
                            return None;
                        }
                        (ty, read)
                    }
                    _ => return None, // unresolved or ambiguous target; no namespace fallback
                };
                Some((ty, read | argument_read))
            }
            _ => None,
        }
    }
    circuit.parameters.is_empty()
        && maybe_field(&circuit.result)
        && stateful(
            circuit,
            &ReadOnlyCallees {
                stateful: circuits,
                pure,
            },
            ledger_fields,
            &mut HashSet::new(),
            &mut 4096,
        )
        .is_some_and(|(_, read)| read)
}

fn signature(circuit: &StatefulCircuit) -> bool {
    circuit.actions.is_empty()
        && circuit.result == Type::Boolean
        && circuit.parameters.len() == 1
        && circuit.parameters[0].ty
            == (Type::Unsigned {
                max: u64::MAX.to_string(),
            })
}

/// Select by typed semantics, never a fixture or exported circuit spelling.
/// Boolean calls may reach the standard-library comparison/negation wrappers.
/// Unit calls admit the closed-literal Set mutations or bounded Counter graph
/// above, or the separately checked authenticated Cell-clear graph.
/// Maybe<Field> getters remain read-only. Other witness/action shapes refuse.
pub(crate) fn supported(
    circuit: &StatefulCircuit,
    circuits: &[StatefulCircuit],
    pure: &[PureCircuit],
    ledger_fields: &HashMap<&str, &LedgerField>,
    witnesses: &[crate::ir::WitnessDeclaration],
) -> bool {
    fn visit(
        circuit: &StatefulCircuit,
        circuits: &[StatefulCircuit],
        active: &mut HashSet<String>,
    ) -> Option<bool> {
        let helper_signature = circuit.actions.is_empty()
            && circuit.result == Type::Boolean
            && circuit.parameters.iter().all(|p| {
                p.ty == Type::Boolean
                    || p.ty
                        == (Type::Unsigned {
                            max: u64::MAX.to_string(),
                        })
            });
        if !helper_signature || !active.insert(circuit.name.clone()) {
            return None;
        }
        fn expression(
            value: &Expr,
            circuits: &[StatefulCircuit],
            active: &mut HashSet<String>,
        ) -> Option<bool> {
            match value {
                Expr::KernelBlockTimeLessThan { bound }
                | Expr::KernelBlockTimeGreaterThan { bound } => {
                    expression(bound, circuits, active)?;
                    Some(true)
                }
                Expr::Parameter { .. } | Expr::Boolean { .. } => Some(false),
                Expr::Coerce { value, .. } => expression(value, circuits, active),
                Expr::If {
                    condition,
                    then,
                    otherwise,
                } => Some(
                    expression(condition, circuits, active)?
                        | expression(then, circuits, active)?
                        | expression(otherwise, circuits, active)?,
                ),
                Expr::Call { name, arguments } => {
                    let mut in_arguments = false;
                    for argument in arguments {
                        in_arguments |= expression(argument, circuits, active)?;
                    }
                    let target = circuits.iter().find(|c| c.name == *name)?;
                    Some(in_arguments | visit(target, circuits, active)?)
                }
                _ => None,
            }
        }
        let result = match &circuit.return_value {
            crate::ir::StateReturn::Expression { value } => expression(value, circuits, active),
            _ => None,
        };
        active.remove(&circuit.name);
        result
    }
    !circuit.internal
        && (unit_set_mutations(circuit, ledger_fields)
            || unit_counter_program(circuit, ledger_fields)
            || maybe_field_getter(circuit, circuits, pure, ledger_fields)
            || authenticated_cell_clear::supported(
                circuit,
                circuits,
                pure,
                ledger_fields,
                witnesses,
            )
            || (signature(circuit) && visit(circuit, circuits, &mut HashSet::new()) == Some(true)))
}

pub(crate) struct Facade {
    pub method: Option<syn::ImplItemFn>,
    pub unavailable: Option<crate::StandaloneProofDataUnavailable>,
}

pub(crate) fn render_method(
    circuit: &StatefulCircuit,
    occupied_names: &HashSet<String>,
    uses_witness: bool,
) -> Result<Facade, RenderError> {
    let helper = format!("{}_proof_data", circuit.name);
    if occupied_names.contains(&semantic_identifier(&helper)?) {
        return Ok(Facade {
            method: None,
            unavailable: Some(crate::StandaloneProofDataUnavailable {
                code: "name_collision",
                method: ident(&helper)?.to_string(),
                detail: format!(
                    "standalone proof-data helper {helper:?} collides with a declared stateful circuit"
                ),
            }),
        });
    }
    let method = ident(&helper)?;
    let name = ident(&circuit.name)?;
    let circuit_name = syn::LitStr::new(&circuit.name, proc_macro2::Span::call_site());
    let output = rust_type(&circuit.result)?;
    let (parameter, input, invocation) = if circuit.parameters.is_empty() {
        (quote!(), quote!(()), quote!(self.#name(context)?))
    } else {
        let argument = public_parameter_idents(&circuit.parameters).remove(0);
        let ty = rust_type(&circuit.parameters[0].ty)?;
        (
            quote!(#argument: #ty,),
            quote!(#argument),
            quote!(self.#name(context, #argument)?),
        )
    };
    let witness_bound = uses_witness.then(|| quote!(where W: super::TryWitnesses<Private>));
    let method = syn::parse_quote! {
        /// Capture standalone proof data from one actual recorded execution.
        /// Conversion requires explicit binding and communication randomness.
        pub fn #method<Private>(
            &self,
            context: runtime::context::CircuitContext<Private>,
            #parameter
        ) -> Result<runtime::proof_data::RecordedProofData<Private, #output>, runtime::proof_data::ProofDataError>
        #witness_bound {
            const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize];
            let input = runtime::fab::AlignedValue::from(#input);
            let recorded = #invocation;
            runtime::proof_data::RecordedProofData::from_recorded(#circuit_name, input, recorded)
        }
    };
    Ok(Facade {
        method: Some(method),
        unavailable: None,
    })
}

#[cfg(test)]
mod maybe_classifier_tests {
    use super::*;
    use crate::ir::Contract;
    use serde_json::{Value, json};

    fn repeated_pure_dag(levels: usize) -> Contract {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../fixtures/upstream-672/tiny-get/compact-rust-ir.json"
        ))
        .unwrap();
        value["circuits"].as_array_mut().unwrap().push(json!({
            "name":"fan_base", "internal":true,
            "parameters":[{"name":"x","ty":{"kind":"field"}}],
            "result":{"kind":"field"}, "body":{"kind":"parameter","name":"x"}
        }));
        let mut previous = "fan_base".to_owned();
        for level in 0..levels {
            let name = format!("fan_{level}");
            let call = json!({"kind":"call","name":previous,
                "arguments":[{"kind":"parameter","name":"x"}]});
            value["circuits"].as_array_mut().unwrap().push(json!({
                "name":name, "internal":true,
                "parameters":[{"name":"x","ty":{"kind":"field"}}],
                "result":{"kind":"field"},
                "body":{"kind":"if","condition":{"kind":"boolean","value":true},
                    "then":call,"otherwise":call}
            }));
            previous = name;
        }
        let get = value["stateful_circuits"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["name"] == "get")
            .unwrap();
        let read = get["return_value"]["value"]["then"]["arguments"][0].clone();
        get["return_value"]["value"]["then"]["arguments"][0] =
            json!({"kind":"call","name":previous,"arguments":[read]});
        serde_json::from_value(value).unwrap()
    }

    fn classified(contract: &Contract) -> bool {
        let get = contract
            .stateful_circuits
            .iter()
            .find(|c| c.name == "get")
            .unwrap();
        let fields = contract
            .ledger_fields
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        // Directly exercise the private optional classifier. No renderer runs;
        // integration admission has its own earlier resource refusal test.
        maybe_field_getter(
            get,
            &contract.stateful_circuits,
            &contract.circuits,
            &fields,
        )
    }

    #[test]
    fn small_typed_read_only_dag_is_admitted_by_private_classifier() {
        assert!(classified(&repeated_pure_dag(2)));
    }

    #[test]
    fn repeated_acyclic_dag_exhausts_private_classifier_total_fuel() {
        // Same valid typed/read-only construction, depth below 64, but repeated
        // expansion exceeds 4096 visits. Neither renderer nor runtime is invoked.
        assert!(!classified(&repeated_pure_dag(13)));
    }
}

#[cfg(test)]
mod counter_unit_classifier_tests {
    use super::*;
    use crate::ir::Contract;
    use serde_json::{Value, json};

    fn original() -> Value {
        serde_json::from_str(include_str!(
            "../../fixtures/upstream-672/adt-counter/compact-rust-ir.json"
        ))
        .unwrap()
    }

    fn classified(value: &Value, index: usize) -> bool {
        let contract: Contract = serde_json::from_value(value.clone()).unwrap();
        let fields = contract
            .ledger_fields
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        // Direct private admission: earlier renderer/resource errors cannot
        // masquerade as testing this classifier's scope/arithmetic/budget gate.
        unit_counter_program(&contract.stateful_circuits[index], &fields)
    }

    fn uint(value: &str, max: &str) -> Value {
        json!({"kind":"unsigned_literal","value":value,"max":max})
    }

    fn closed(value: Value) -> Option<(u64, u64)> {
        let expr: Expr = serde_json::from_value(value).unwrap();
        counter_closed(
            &expr,
            &HashMap::new(),
            &mut CounterBudget { remaining: 4096 },
            0,
        )
        .map(|v| (v.max, v.value))
    }

    #[test]
    fn genuine_counter_graph_is_admitted_without_renderer() {
        let value = original();
        assert!(classified(&value, 0));
        assert!(classified(&value, 1));
        assert_eq!(
            closed(value["stateful_circuits"][1]["actions"][1]["bindings"][0]["value"].clone()),
            Some((65535, 4096))
        );
        assert_eq!(
            closed(value["stateful_circuits"][1]["actions"][3]["condition"]["right"].clone()),
            Some((u64::MAX, 4092))
        );
    }

    #[test]
    fn closed_unsigned_checks_actual_values_types_and_lexical_scope() {
        // Operand type maxima256*256 do not fit511, but the closed values do.
        assert_eq!(
            closed(json!({"kind":"unsigned_multiply","max":"511",
            "left":uint("16","256"),"right":uint("16","256")})),
            Some((511, 256))
        );
        let parameter = json!({"kind":"parameter","name":"x"});
        let shadowed = json!({"kind":"let","bindings":[
            {"name":"x","ty":{"kind":"unsigned","max":"65535"},"value":uint("5","65535")},
            {"name":"x","ty":{"kind":"unsigned","max":"65535"},"value":{
                "kind":"unsigned_subtract","max":"65535","left":parameter,"right":uint("1","65535")}}
        ],"body":parameter});
        assert_eq!(closed(shadowed), Some((65535, 4)));
        for invalid in [
            uint("65536", "65535"),
            uint("01", "65535"),
            json!({"kind":"parameter","name":"free"}),
            json!({"kind":"unsigned_cast","max":"1","value":uint("2","2")}),
            json!({"kind":"unsigned_multiply","max":"1","left":uint("2","2"),"right":uint("2","2")}),
            json!({"kind":"unsigned_multiply","max":u64::MAX.to_string(),
                "left":uint(&u64::MAX.to_string(), &u64::MAX.to_string()),"right":uint("2","2")}),
            json!({"kind":"unsigned_subtract","max":"65535","left":uint("0","65535"),"right":uint("1","65535")}),
            json!({"kind":"let","bindings":[{"name":"x","ty":{"kind":"unsigned","max":"65535"},"value":parameter}],"body":parameter}),
            json!({"kind":"let","bindings":[
                {"name":"x","ty":{"kind":"unsigned","max":"65535"},"value":{"kind":"parameter","name":"later"}},
                {"name":"later","ty":{"kind":"unsigned","max":"65535"},"value":uint("1","65535")}
            ],"body":parameter}),
            json!({"kind":"let","bindings":[{"name":"x","ty":{"kind":"unsigned","max":"4"},"value":uint("4","65535")}],"body":parameter}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":uint("4","65535"),"otherwise":{"kind":"parameter","name":"witness"}}),
            json!({"kind":"call","name":"helper","arguments":[]}),
        ] {
            assert!(closed(invalid).is_none());
        }
        // A binding in an inner expression cannot escape into its sibling.
        assert!(closed(json!({"kind":"unsigned_multiply","max":"65535",
            "left":{"kind":"let","bindings":[{"name":"x","ty":{"kind":"unsigned","max":"65535"},"value":uint("2","65535")}],"body":parameter},
            "right":parameter})).is_none());
    }

    #[test]
    fn counter_slots_unknowns_and_scope_are_refused_directly() {
        for kind in [
            "index",
            "path",
            "declaration",
            "free",
            "type",
            "amount",
            "branch",
            "write",
            "call",
        ] {
            let mut value = original();
            match kind {
                "index" => {
                    value["stateful_circuits"][0]["actions"][0]["condition"]["left"]["index"] =
                        json!(1)
                }
                "path" => value["ledger_fields"][0]["path"] = json!([1]),
                "declaration" => {
                    value["ledger_fields"][0]["declaration"] =
                        json!({"kind":"cell","ty":{"kind":"field"}})
                }
                "free" => {
                    value["stateful_circuits"][0]["actions"][3]["action"]["amount"]["name"] =
                        json!("free")
                }
                "type" => {
                    value["stateful_circuits"][0]["actions"][3]["bindings"][0]["ty"] =
                        json!({"kind":"unsigned","max":"5"})
                }
                "amount" => {
                    value["stateful_circuits"][0]["actions"][3]["bindings"][0]["ty"]["max"] =
                        json!("5");
                    value["stateful_circuits"][0]["actions"][3]["bindings"][0]["value"]["max"] =
                        json!("5");
                }
                "branch" => {
                    value["stateful_circuits"][0]["actions"][1] = json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"counter_reset","field":"c","index":0},"otherwise":{"kind":"counter_reset","field":"c","index":0}})
                }
                "write" => {
                    value["stateful_circuits"][0]["actions"][1] = json!({"kind":"cell_write","field":"c","index":0,"value":{"kind":"field_literal","value":"0"}})
                }
                "call" => {
                    value["stateful_circuits"][0]["actions"][1] =
                        json!({"kind":"circuit_call","name":"test_big_increment","arguments":[]})
                }
                _ => unreachable!(),
            }
            assert!(!classified(&value, 0), "{kind}");
        }
        let mut escaped = original();
        escaped["stateful_circuits"][0]["actions"][5]["action"]["amount"]["name"] = json!("tmp_18");
        assert!(!classified(&escaped, 0));
        for action_kind in ["counter_reset", "assert"] {
            let mut incomplete = original();
            incomplete["stateful_circuits"][0]["actions"] = if action_kind == "assert" {
                json!([incomplete["stateful_circuits"][0]["actions"][0].clone()])
            } else {
                json!([{"kind":"counter_reset","field":"c","index":0}])
            };
            assert!(!classified(&incomplete, 0));
        }
    }

    #[test]
    fn total_fuel_and_depth_are_private_admission_gates() {
        let mut fuel = original();
        let assertion = fuel["stateful_circuits"][0]["actions"][0].clone();
        let mut actions = vec![assertion; 2048];
        actions.push(json!({"kind":"counter_reset","field":"c","index":0}));
        fuel["stateful_circuits"][0]["actions"] = json!(actions);
        assert!(!classified(&fuel, 0));
        let mut value = uint("0", &u64::MAX.to_string());
        for _ in 0..65 {
            value = json!({"kind":"unsigned_cast","max":u64::MAX.to_string(),"value":value});
        }
        assert!(closed(value).is_none());
    }
}
