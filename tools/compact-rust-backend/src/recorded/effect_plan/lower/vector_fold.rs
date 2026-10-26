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

//! Legacy named Fold enters the general accumulator-carrying checked loop.
use super::*;

impl Lowerer<'_> {
    pub(super) fn vector_fold_call(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let Expr::VectorFoldCall {
            name,
            initial,
            source,
            accumulator,
            element,
            length,
        } = expression
        else {
            return Err(self.unsupported_expression(expression, path));
        };
        let Some((declaration, signature)) = self.declarations.resolve_call(name) else {
            return Err(invalid(path, "unknown Fold reducer"));
        };
        if signature.kind != super::super::declarations::CallKind::Pure
            || signature.parameters.len() != 2
            || signature.parameters[0].ty != *accumulator
            || signature.parameters[1].ty != *element
            || signature.result != *accumulator
        {
            return Err(invalid(
                path,
                "Fold reducer signature differs from declared accumulator/items",
            ));
        }
        // Static traversal and runtime producer bindings retain seed-before-source.
        let initial_path = format!("{path}.initial");
        let initial = self.expression(initial, scope, bindings, &initial_path)?;
        let initial = self.coerce(initial, accumulator, bindings, &initial_path)?;
        let source_path = format!("{path}.source");
        let source = self.expression(source, scope, bindings, &source_path)?;
        let source = self.coerce(
            source,
            &Type::Vector {
                element: Box::new(element.clone()),
                length: *length,
            },
            bindings,
            &source_path,
        )?;
        self.spend(path)?;
        let accumulator_value = self.fresh(accumulator.clone());
        self.spend(path)?;
        let item = self.fresh(element.clone());
        // A named reducer has its declared lexical scope, never caller captures.
        // These private aliases refer to already checked formals, not new source
        // occurrences. The actual Fold remains the diagnostic call site.
        let local = Scope::from([
            ("__fold_accumulator".into(), accumulator_value.clone()),
            ("__fold_item".into(), item.clone()),
        ]);
        let arguments = [
            Expr::Parameter {
                name: "__fold_accumulator".into(),
            },
            Expr::Parameter {
                name: "__fold_item".into(),
            },
        ];
        let mut body_bindings = Vec::new();
        let result = self.pure_call(
            (declaration, calls::CallSite::Expression(expression)),
            name,
            &arguments,
            &local,
            &mut body_bindings,
            &format!("{path}.body"),
        )?;
        Ok((
            Operation::VectorFold {
                initial,
                sources: vec![source],
                accumulator: accumulator_value,
                items: vec![item],
                body: Block {
                    bindings: body_bindings,
                    result,
                },
            },
            accumulator.clone(),
        ))
    }
}
