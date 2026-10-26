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

//! Test-only syntax-free visitor using the real generic owners, not copies.
//! Deliberately excludes control; it is never production body admission.
use crate::{
    RenderError,
    ir::{Expr, PureCircuit, Type},
    pure_semantics::{
        aggregate::{CheckedAggregate, fold_aggregate},
        arithmetic::{ArithmeticFacts, fold_arithmetic},
        carrier::CheckSink,
        children::{Children, ScopedChildren},
        conversion::{ConversionFacts, fold_conversion},
        crypto::{CheckedCrypto, fold_crypto},
        identifier::CheckedName,
        leaf::{LeafSource, fold_leaf},
        sink::{
            AggregateSink, ArithmeticSink, CheckedLeaf, ConversionSink, CryptoSink, LeafSink, Typed,
        },
    },
};
use std::collections::HashMap;
mod source;

pub(crate) struct Consumer {
    carrier: CheckSink,
    pub(crate) outputs: Vec<CheckedAggregate<(), (), ()>>,
}
impl Default for Consumer {
    fn default() -> Self {
        Self {
            carrier: CheckSink,
            outputs: vec![],
        }
    }
}
impl LeafSink for Consumer {
    type Value = ();
    type Binding = ();
    type Carriers = CheckSink;
    fn carriers(&mut self) -> &mut CheckSink {
        &mut self.carrier
    }
    fn leaf(&mut self, _: CheckedLeaf<'_, (), ()>) {}
}
impl AggregateSink for Consumer {
    fn binding(&mut self, _: &CheckedName) {}
    fn aggregate(&mut self, checked: CheckedAggregate<(), (), ()>) {
        self.outputs.push(checked);
    }
}
impl ArithmeticSink for Consumer {
    fn arithmetic(&mut self, _: ArithmeticFacts, _: (), _: ()) {}
}
impl ConversionSink for Consumer {
    fn conversion(&mut self, _: ConversionFacts, _: ()) {}
}
impl CryptoSink for Consumer {
    fn crypto(&mut self, _: CheckedCrypto<()>) {}
}

pub(crate) struct Visitor<'a> {
    bindings: HashMap<&'a str, (&'a Type, ())>,
    declarations: &'a HashMap<&'a str, &'a PureCircuit>,
    pub(crate) visits: Vec<usize>,
}
impl<'a> Visitor<'a> {
    pub(crate) fn new(
        bindings: HashMap<&'a str, (&'a Type, ())>,
        declarations: &'a HashMap<&'a str, &'a PureCircuit>,
    ) -> Self {
        Self {
            bindings,
            declarations,
            visits: vec![],
        }
    }
    pub(crate) fn check(
        &mut self,
        source: &Expr,
        sink: &mut Consumer,
    ) -> Result<Typed<()>, RenderError> {
        self.visit(source, sink)
    }
}
impl Children<Consumer> for Visitor<'_> {
    fn visit(&mut self, expr: &Expr, sink: &mut Consumer) -> Result<Typed<()>, RenderError> {
        self.visits.push(std::ptr::from_ref(expr) as usize);
        let declarations = self.declarations;
        if let Some(source) = source::aggregate(expr) {
            return fold_aggregate(source, self, declarations, sink);
        }
        if let Some(source) = source::arithmetic(expr) {
            return fold_arithmetic(source, self, sink);
        }
        if let Some(source) = source::conversion(expr) {
            return fold_conversion(source, self, sink);
        }
        if let Some(source) = source::crypto(expr) {
            return fold_crypto(source, self, sink);
        }
        let leaf = match expr {
            Expr::Unit => LeafSource::Unit,
            Expr::Default { ty } => LeafSource::Default(ty),
            Expr::Boolean { value } => LeafSource::Boolean(*value),
            Expr::FieldLiteral { value } => LeafSource::FieldLiteral(value),
            Expr::BytesLiteral { bytes } => LeafSource::BytesLiteral(bytes),
            Expr::UnsignedLiteral { value, max } => LeafSource::UnsignedLiteral { value, max },
            Expr::Parameter { name } => LeafSource::Parameter(name),
            Expr::EnumVariant { ty, variant } => LeafSource::EnumVariant { ty, variant },
            Expr::KernelSelf { .. } => LeafSource::Effectful,
            _ => unreachable!("test-only prior/aggregate protocol excludes unmigrated control"),
        };
        fold_leaf(leaf, &self.bindings, sink)
    }
}
impl ScopedChildren<Consumer> for Visitor<'_> {
    fn visit_bound(
        &mut self,
        source: &Expr,
        source_key: &str,
        ty: &Type,
        _: &(),
        sink: &mut Consumer,
    ) -> Result<Typed<()>, RenderError> {
        let mut scope: HashMap<&str, (&Type, ())> = self.bindings.clone();
        scope.insert(source_key, (ty, ()));
        let mut inner = Visitor::new(scope, self.declarations);
        let result = inner.visit(source, sink);
        self.visits.extend(inner.visits);
        result
    }
    fn visit_bound_many(
        &mut self,
        source: &Expr,
        bindings: &[(&str, &Type, &())],
        sink: &mut Consumer,
    ) -> Result<Typed<()>, RenderError> {
        let mut scope: HashMap<&str, (&Type, ())> = self.bindings.clone();
        for (key, ty, _) in bindings {
            scope.insert(key, (ty, ()));
        }
        let mut inner = Visitor::new(scope, self.declarations);
        let result = inner.visit(source, sink);
        self.visits.extend(inner.visits);
        result
    }
}
