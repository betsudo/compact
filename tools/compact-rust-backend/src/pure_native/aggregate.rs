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

//! Aggregate output from checked values, names, ownership and conversion plans.
//! No source Expr/Type pairs or support decisions enter these formatters.
use super::leaf::NativeLeafSink;
use crate::{
    naming::checked_ident,
    pure_semantics::{aggregate::CheckedAggregate, identifier::CheckedName, sink::AggregateSink},
};
mod literal;
pub(crate) mod projection;
mod vector;

impl AggregateSink for NativeLeafSink {
    fn binding(&mut self, name: &CheckedName) -> syn::Ident {
        checked_ident(name)
    }
    fn aggregate(
        &mut self,
        checked: CheckedAggregate<syn::Expr, syn::Ident, syn::Type>,
    ) -> syn::Expr {
        match checked {
            CheckedAggregate::StructField {
                value,
                field,
                ownership,
            } => projection::field(value, field, ownership),
            CheckedAggregate::TupleIndex { value, index } => projection::tuple(value, index),
            CheckedAggregate::VectorIndex {
                value,
                index,
                ownership,
            } => projection::vector(value, index, ownership),
            CheckedAggregate::StructLiteral { name, fields } => literal::structure(name, fields),
            CheckedAggregate::Tuple(values) => literal::tuple(values),
            CheckedAggregate::Vector(values) => {
                syn::parse_quote!(runtime::FixedVector::new([#(#values),*]))
            }
            CheckedAggregate::EmptyVector(element) => {
                syn::parse_quote!(runtime::FixedVector::<#element, 0>::new([]))
            }
            CheckedAggregate::Map {
                source,
                source_plan,
                binding,
                body,
                length,
            } => vector::map(source, &source_plan, binding, body, length),
            CheckedAggregate::Fold {
                function,
                initial,
                initial_plan,
                source,
                source_plan,
            } => vector::fold(function, initial, &initial_plan, source, &source_plan),
            CheckedAggregate::Call {
                function,
                arguments,
            } => {
                let function = checked_ident(&function);
                syn::parse_quote!(crate::pure_circuits::#function(#(#arguments),*)?)
            }
        }
    }
}
