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

//! Product operations are checked independently against canonical source types.
use super::*;

pub(super) fn validate(
    binding: &Binding,
    declarations: &Declarations,
    scope: &HashMap<usize, Type>,
) -> Result<bool, RecordingGap> {
    let ty = &binding.value.ty;
    let path = &binding.path;
    let use_value = |value: &Value| used(value, scope, declarations, path);
    Ok(match &binding.operation {
        Operation::EnumVariant { variant } => {
            matches!(ty, Type::Enum { variants, .. } if variants.contains(variant))
        }
        Operation::Default => default_value_type(ty),
        Operation::StructConstruct { fields: values } => {
            let Type::Struct { fields, .. } = ty else {
                return Ok(false);
            };
            if fields.len() != values.len() {
                return Ok(false);
            }
            for (field, value) in fields.iter().zip(values) {
                use_value(value)?;
                if field.ty != value.ty {
                    return Ok(false);
                }
            }
            true
        }
        Operation::StructProject {
            value,
            field,
            index,
        } => {
            use_value(value)?;
            let Type::Struct { fields, .. } = &value.ty else {
                return Ok(false);
            };
            fields
                .get(*index)
                .is_some_and(|member| member.name == *field && member.ty == *ty)
        }
        Operation::TupleProject { value, index } => {
            use_value(value)?;
            let Type::Tuple { elements } = &value.ty else {
                return Ok(false);
            };
            elements.get(*index).is_some_and(|element| element == ty)
        }
        Operation::VectorProject { value, index } => {
            use_value(value)?;
            let Type::Vector { element, length } = &value.ty else {
                return Ok(false);
            };
            *index < *length
                && element.as_ref() == ty
                && declarations.vector_projection_matches(binding.value.id, value.id, *index)
        }
        Operation::Vector { elements: values } => {
            let Type::Vector { element, length } = ty else {
                return Ok(false);
            };
            if *length != values.len() {
                return Ok(false);
            }
            for value in values {
                use_value(value)?;
                if value.ty != **element {
                    return Ok(false);
                }
            }
            true
        }
        Operation::Tuple { elements: values } => {
            let Type::Tuple { elements } = ty else {
                return Ok(false);
            };
            if elements.len() != values.len() {
                return Ok(false);
            }
            for (element, value) in elements.iter().zip(values) {
                use_value(value)?;
                if *element != value.ty {
                    return Ok(false);
                }
            }
            true
        }
        _ => return Err(invalid(path, "expected rich value operation")),
    })
}
