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

//! Independent reduction shape, lexical SSA and body/result validation.
use super::*;

pub(super) fn validate(
    binding: &Binding,
    declarations: &Declarations,
    incoming: &HashMap<usize, Type>,
    state: &mut Validation,
) -> Result<(), RecordingGap> {
    let path = &binding.path;
    let Operation::VectorFold {
        initial,
        sources,
        accumulator,
        items,
        body,
    } = &binding.operation
    else {
        return Err(invalid(path, "expected VectorFold operation"));
    };
    used(initial, incoming, declarations, path)?;
    if sources.is_empty() || sources.len() != items.len() {
        return Err(invalid(
            path,
            "Fold requires an item formal for each nonempty iterable list",
        ));
    }
    if initial.ty != accumulator.ty
        || binding.value.ty != accumulator.ty
        || body.result.ty != accumulator.ty
    {
        return Err(invalid(
            path,
            "Fold accumulator/initial/body/result types differ",
        ));
    }
    let mut length = None;
    for (source, item) in sources.iter().zip(items) {
        used(source, incoming, declarations, path)?;
        let Type::Vector {
            element,
            length: actual_length,
        } = &source.ty
        else {
            return Err(invalid(path, "Fold iterable is not a fixed vector"));
        };
        if **element != item.ty || length.is_some_and(|length| length != *actual_length) {
            return Err(invalid(path, "Fold iterable lengths or item types differ"));
        }
        length = Some(*actual_length);
    }
    let mut local = incoming.clone();
    // Formals and the whole body remain obligations even for zero iterations.
    for formal in std::iter::once(accumulator).chain(items) {
        declarations.types().validate_at(&formal.ty, path)?;
        if !value_type(&formal.ty) {
            return Err(invalid(path, "unsupported Fold formal type"));
        }
        state.charge(1, path)?;
        if !state.allocated.insert(formal.id) {
            return Err(invalid(path, "invalid or duplicate Fold formal"));
        }
        local.insert(formal.id, formal.ty.clone());
    }
    block(body, declarations, &local, state, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vector_fold_empty_body_and_formals_consume_finite_work() {
        let initial = Value {
            id: 0,
            ty: Type::Field,
        };
        let source = Value {
            id: 1,
            ty: Type::Vector {
                element: Box::new(Type::Field),
                length: 0,
            },
        };
        let accumulator = Value {
            id: 2,
            ty: Type::Field,
        };
        let item = Value {
            id: 3,
            ty: Type::Field,
        };
        let binding = Binding {
            value: Value {
                id: 4,
                ty: Type::Field,
            },
            path: "fold".into(),
            operation: Operation::VectorFold {
                initial: initial.clone(),
                sources: vec![source.clone()],
                accumulator: accumulator.clone(),
                items: vec![item],
                body: Block {
                    bindings: vec![],
                    result: accumulator,
                },
            },
        };
        let incoming = HashMap::from([(0, initial.ty), (1, source.ty)]);
        for (remaining, accepted) in [(3, true), (2, false)] {
            let mut state = Validation {
                allocated: HashSet::from([0, 1]),
                remaining,
                active_calls: vec![],
                block_depth: 1,
            };
            let result = validate(&binding, &Declarations::default(), &incoming, &mut state);
            if accepted {
                result.unwrap();
                assert_eq!(state.remaining, 0);
            } else {
                assert_eq!(
                    result.unwrap_err().detail,
                    "recording plan expanded work limit exceeded"
                );
            }
        }
    }
}
