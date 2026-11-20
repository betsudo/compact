// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared Map operations preserve declaration authority and key-before-value order.
use super::*;

pub(super) enum MapQuery<'a> {
    Member(&'a str, u8, &'a Expr),
    Lookup(&'a str, u8, &'a Expr),
    Size(&'a str, u8),
    IsEmpty(&'a str, u8),
}
impl Lowerer<'_> {
    fn map_slot(
        &self,
        name: &str,
        index: u8,
        scalar: bool,
        path: &str,
    ) -> Result<(MapSlot, Type, Option<Type>), RecordingGap> {
        let (declaration, _) = self.field(name, index, path)?;
        let slot = MapSlot { declaration, index };
        let (key, value) =
            super::super::validate::maps::types(self.declarations, slot, scalar, path)?;
        Ok((slot, key.clone(), scalar.then(|| value.clone())))
    }
    fn map_operand(
        &mut self,
        expression: &Expr,
        ty: &Type,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<Value, RecordingGap> {
        let value = self.expression(expression, scope, bindings, path)?;
        if value.ty != *ty {
            return Err(invalid(path, "Map operand type differs from declaration"));
        }
        Ok(value)
    }
    /// Share one recursive expression dispatcher temporary across Map queries.
    pub(super) fn map_expression(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        if let Expr::NestedMapLookup {
            field,
            index,
            outer,
            inner,
        } = expression
        {
            let (slot, _, _) = self.map_slot(field, *index, false, path)?;
            let (outer_ty, inner_ty, value_ty) =
                super::super::validate::maps::child_types(self.declarations, slot, path)?;
            let inner =
                self.map_operand(inner, &inner_ty, scope, bindings, &format!("{path}.inner"))?;
            let outer =
                self.map_operand(outer, &outer_ty, scope, bindings, &format!("{path}.outer"))?;
            return Ok((Operation::NestedMapLookup { slot, outer, inner }, value_ty));
        }
        let query = match expression {
            Expr::MapMember { field, index, key } => MapQuery::Member(field, *index, key),
            Expr::MapLookup { field, index, key } => MapQuery::Lookup(field, *index, key),
            Expr::MapIsEmpty { field, index } => MapQuery::IsEmpty(field, *index),
            Expr::MapSize { field, index } => MapQuery::Size(field, *index),
            _ => {
                return Err(invalid(
                    path,
                    "Map expression dispatch requires a Map query",
                ));
            }
        };
        self.map_query(query, scope, bindings, path)
    }
    pub(super) fn map_query(
        &mut self,
        query: MapQuery<'_>,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        match query {
            MapQuery::Size(name, index) | MapQuery::IsEmpty(name, index) => {
                // Metadata queries never decode V; nested native Map nodes remain valid.
                let (slot, _, _) = self.map_slot(name, index, false, path)?;
                Ok(if matches!(query, MapQuery::Size(..)) {
                    (Operation::MapSize { slot }, uint("18446744073709551615"))
                } else {
                    (Operation::MapIsEmpty { slot }, Type::Boolean)
                })
            }
            MapQuery::Member(name, index, expression)
            | MapQuery::Lookup(name, index, expression) => {
                let lookup = matches!(query, MapQuery::Lookup(..));
                let (slot, key_type, value_type) = self.map_slot(name, index, lookup, path)?;
                let key = self.map_operand(
                    expression,
                    &key_type,
                    scope,
                    bindings,
                    &format!("{path}.key"),
                )?;
                Ok(if lookup {
                    (
                        Operation::MapLookup { slot, key },
                        value_type.expect("scalar lookup declaration"),
                    )
                } else {
                    (Operation::MapMember { slot, key }, Type::Boolean)
                })
            }
        }
    }
    pub(super) fn map_action(
        &mut self,
        action: &StateAction,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(), RecordingGap> {
        let (name, index) = match action {
            StateAction::MapInsert { field, index, .. }
            | StateAction::MapRemove { field, index, .. }
            | StateAction::MapInsertDefault { field, index, .. }
            | StateAction::NestedMapInsert { field, index, .. }
            | StateAction::MapInitChild { field, index, .. }
            | StateAction::MapReset { field, index } => (field, *index),
            _ => return Err(invalid(path, "Map action dispatch requires a Map mutation")),
        };
        let insert_default = matches!(action, StateAction::MapInsertDefault { .. });
        let (declaration, _) = self.field(name, index, path)?;
        let slot = MapSlot { declaration, index };
        let scalar = !matches!(
            action,
            StateAction::MapRemove { .. }
                | StateAction::NestedMapInsert { .. }
                | StateAction::MapInitChild { .. }
        ) && !(insert_default
            && super::super::validate::maps::has_merkle_value(self.declarations, slot));
        let (key_type, value_type) =
            super::super::validate::maps::types(self.declarations, slot, scalar, path)?;
        let key_type = key_type.clone();
        let value_type = value_type.clone();
        let operation = match action {
            StateAction::NestedMapInsert {
                outer,
                inner,
                value,
                ..
            } => {
                let (outer_ty, inner_ty, value_ty) =
                    super::super::validate::maps::child_types(self.declarations, slot, path)?;
                let inner =
                    self.map_operand(inner, &inner_ty, scope, bindings, &format!("{path}.inner"))?;
                let value =
                    self.map_operand(value, &value_ty, scope, bindings, &format!("{path}.value"))?;
                let outer =
                    self.map_operand(outer, &outer_ty, scope, bindings, &format!("{path}.outer"))?;
                Operation::NestedMapInsert {
                    slot,
                    outer,
                    inner,
                    value,
                }
            }
            StateAction::MapInitChild { key, .. } => {
                super::super::validate::maps::child_slot_types(self.declarations, slot, path)?;
                let key =
                    self.map_operand(key, &key_type, scope, bindings, &format!("{path}.key"))?;
                Operation::MapInitChild { slot, key }
            }
            StateAction::MapReset { .. } => Operation::MapReset { slot },
            StateAction::MapInsert { key, value, .. } => {
                let key =
                    self.map_operand(key, &key_type, scope, bindings, &format!("{path}.key"))?;
                let value = self.map_operand(
                    value,
                    &value_type,
                    scope,
                    bindings,
                    &format!("{path}.value"),
                )?;
                Operation::MapInsert { slot, key, value }
            }
            StateAction::MapInsertDefault { key, .. } | StateAction::MapRemove { key, .. } => {
                let key =
                    self.map_operand(key, &key_type, scope, bindings, &format!("{path}.key"))?;
                if insert_default {
                    if !super::super::validate::maps::default_insert_value(
                        self.declarations,
                        &value_type,
                        path,
                    )? {
                        return Err(invalid(
                            path,
                            "Map default insertion requires a defaultable value",
                        ));
                    }
                    Operation::MapInsertDefault { slot, key }
                } else {
                    Operation::MapRemove { slot, key }
                }
            }
            _ => return Err(invalid(path, "Map action dispatch requires a Map mutation")),
        };
        self.emit(operation, Type::Unit, bindings, path);
        Ok(())
    }
}
