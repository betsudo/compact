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

//! One interpreter-polymorphic body over the original checked graph.
//!
//! Successful binds delimit operand evaluation. Neither a later clone/default
//! nor an inactive branch is evaluated after a preceding operation fails.
use super::functional_paths::rust_type;
use super::{
    RecordingGap, declarations::Declarations, functional::FunctionalProgram, invalid, model::*,
};
use crate::{ident, ir::Type};
use quote::{ToTokens, format_ident};

/// A logical Cell identity and its authoritative physical declaration.
pub(super) struct FunctionalCellSlot {
    pub id: u32,
    pub declaration: LedgerId,
}

pub(super) struct FunctionalCounterSlot {
    pub id: u32,
    pub declaration: LedgerId,
}

pub(super) struct FunctionalWitness {
    pub id: u32,
    pub declaration: WitnessId,
}

pub(super) struct RenderedFunctional {
    /// Expects `__compact_executor` and the ordinary `__compact_param_i` inputs.
    /// Returns a Circuit; the request wrapper chooses when to call finish.
    pub body: syn::Expr,
    pub executor_bounds: Vec<syn::TypeParamBound>,
    pub slots: Vec<FunctionalCellSlot>,
    pub counters: Vec<FunctionalCounterSlot>,
    pub witnesses: Vec<FunctionalWitness>,
}

pub(super) fn render(
    program: &FunctionalProgram<'_, '_>,
) -> Result<RenderedFunctional, RecordingGap> {
    let checked = program.checked();
    let mut renderer = Renderer {
        declarations: checked.declarations(),
        slots: Vec::new(),
        counters: Vec::new(),
        witnesses: Vec::new(),
        executor_bounds: Vec::new(),
    };
    let mut parameters = Vec::<syn::Stmt>::new();
    for (index, parameter) in checked.plan().parameters.iter().enumerate() {
        if parameter.ty != Type::Unit {
            let target = name(parameter);
            let input = format_ident!("__compact_param_{index}");
            let ty = rust_type(&parameter.ty)
                .map_err(|error| invalid("functional.parameters", error.to_string()))?;
            parameters.push(syn::parse_quote!(let #target: #ty = #input;));
        }
    }
    let execution = syn::parse_quote!(crate::runtime::functional::Circuit::begin(
        __compact_executor
    ));
    let body = renderer.block(&checked.plan().body, execution)?;
    Ok(RenderedFunctional {
        body: syn::parse_quote!({ #(#parameters)* #body }),
        executor_bounds: renderer.executor_bounds,
        slots: renderer.slots,
        counters: renderer.counters,
        witnesses: renderer.witnesses,
    })
}

fn name(value: &Value) -> syn::Ident {
    format_ident!("__compact_effect_{}", value.id)
}

fn value(value: &Value) -> syn::Expr {
    if value.ty == Type::Unit {
        syn::parse_quote!(())
    } else {
        let name = name(value);
        crate::retained_value(syn::parse_quote!(#name), &value.ty)
    }
}

struct Renderer<'a> {
    declarations: &'a Declarations,
    slots: Vec<FunctionalCellSlot>,
    counters: Vec<FunctionalCounterSlot>,
    witnesses: Vec<FunctionalWitness>,
    executor_bounds: Vec<syn::TypeParamBound>,
}

impl Renderer<'_> {
    fn block(&mut self, block: &Block, execution: syn::Expr) -> Result<syn::Expr, RecordingGap> {
        self.sequence(&block.bindings, &block.result, execution)
    }

    fn sequence(
        &mut self,
        bindings: &[Binding],
        result: &Value,
        execution: syn::Expr,
    ) -> Result<syn::Expr, RecordingGap> {
        let Some((binding, remaining)) = bindings.split_first() else {
            let result = value(result);
            return Ok(syn::parse_quote!((#execution).try_returning(|| Ok(#result))));
        };
        let step = self.step(binding, execution)?;
        let continuation =
            self.sequence(remaining, result, syn::parse_quote!(__compact_execution))?;
        if binding.value.ty == Type::Unit {
            Ok(syn::parse_quote!((#step).bind(|(), __compact_execution| #continuation)))
        } else {
            let output = name(&binding.value);
            let ty = rust_type(&binding.value.ty)
                .map_err(|error| invalid(&binding.path, error.to_string()))?;
            Ok(syn::parse_quote!((#step).bind(|#output: #ty, __compact_execution| #continuation)))
        }
    }

    fn step(&mut self, binding: &Binding, execution: syn::Expr) -> Result<syn::Expr, RecordingGap> {
        // This outer success boundary also protects operand clones on writes,
        // conditions and rule arguments. Carrier methods cannot make eager Rust
        // method arguments lazy on their own.
        let current: syn::Expr = syn::parse_quote!(__compact_execution);
        let step: syn::Expr = match &binding.operation {
            Operation::CounterRead { slot } => {
                let counter = self.counter(*slot, &binding.path)?;
                self.bound(syn::parse_quote!(crate::runtime::functional::CounterRead));
                let ty = rust_type(&binding.value.ty)
                    .map_err(|error| invalid(&binding.path, error.to_string()))?;
                syn::parse_quote! {
                    #current.read_counter(#counter).bind(|__compact_counter_value, __compact_execution| {
                        __compact_execution.try_returning(|| <#ty>::new(__compact_counter_value as u128))
                    })
                }
            }
            Operation::CounterIncrement { slot, amount } => {
                let counter = self.counter(*slot, &binding.path)?;
                self.bound(syn::parse_quote!(
                    crate::runtime::functional::CounterIncrement
                ));
                let amount = value(amount);
                syn::parse_quote!(#current.increment(#counter, (#amount).value() as u16))
            }
            Operation::Witness {
                declaration,
                arguments,
            } => {
                let witness = self.declarations.witness(*declaration).ok_or_else(|| {
                    invalid(&binding.path, "functional witness declaration missing")
                })?;
                let [argument] = arguments.as_slice() else {
                    return Err(invalid(
                        &binding.path,
                        "functional witness requires one Boolean argument",
                    ));
                };
                if witness.result != Type::Boolean
                    || binding.value.ty != Type::Boolean
                    || witness.parameters.len() != 1
                    || witness.parameters[0].ty != Type::Boolean
                    || argument.ty != Type::Boolean
                {
                    return Err(invalid(
                        &binding.path,
                        "functional witness requires a Boolean to Boolean signature",
                    ));
                }
                let id = u32::try_from(declaration.0).map_err(|_| {
                    invalid(&binding.path, "functional witness identity exceeds u32")
                })?;
                if !self.witnesses.iter().any(|present| present.id == id) {
                    self.witnesses.push(FunctionalWitness {
                        id,
                        declaration: *declaration,
                    });
                }
                self.bound(syn::parse_quote!(crate::runtime::functional::WitnessCall<bool, bool>));
                let argument = value(argument);
                syn::parse_quote! {
                    #current.witness(
                        crate::runtime::functional::Witness::<bool, bool>::new(crate::runtime::functional::WitnessId::new(#id)),
                        #argument,
                    )
                }
            }
            Operation::CellRead(slot) => {
                let cell = self.cell(*slot, &binding.value.ty, false, &binding.path)?;
                syn::parse_quote!(#current.read(#cell))
            }
            Operation::CellWrite { slot, value: input } => {
                let cell = self.cell(*slot, &input.ty, true, &binding.path)?;
                let input = value(input);
                syn::parse_quote!(#current.write(#cell, #input))
            }
            Operation::Assert { condition, message } => {
                let condition = value(condition);
                syn::parse_quote!(#current.require(crate::runtime::functional::Rule::new(#condition, #message)))
            }
            Operation::If {
                condition,
                then,
                otherwise,
            } => {
                let condition = value(condition);
                let then = self.block(then, syn::parse_quote!(__compact_execution))?;
                let otherwise = self.block(otherwise, syn::parse_quote!(__compact_execution))?;
                syn::parse_quote!(if #condition { #then } else { #otherwise })
            }
            operation => {
                let expression = pure(binding, operation)?;
                syn::parse_quote!(#current.try_returning(|| Ok(#expression)))
            }
        };
        Ok(syn::parse_quote!((#execution).bind(|(), __compact_execution| #step)))
    }

    fn cell(
        &mut self,
        slot: CellSlot,
        ty: &Type,
        write: bool,
        path: &str,
    ) -> Result<syn::Expr, RecordingGap> {
        let id = u32::try_from(slot.declaration.0)
            .map_err(|_| invalid(path, "functional Cell identity exceeds u32"))?;
        if !self.slots.iter().any(|present| present.id == id) {
            self.slots.push(FunctionalCellSlot {
                id,
                declaration: slot.declaration,
            });
        }
        let ty = rust_type(ty).map_err(|error| invalid(path, error.to_string()))?;
        let bound: syn::TypeParamBound = if write {
            syn::parse_quote!(crate::runtime::functional::CellWrite<#ty>)
        } else {
            syn::parse_quote!(crate::runtime::functional::CellRead<#ty>)
        };
        self.bound(bound);
        Ok(
            syn::parse_quote!(crate::runtime::functional::Cell::<#ty>::new(crate::runtime::functional::SlotId::new(#id))),
        )
    }

    fn counter(&mut self, slot: CounterSlot, path: &str) -> Result<syn::Expr, RecordingGap> {
        let id = u32::try_from(slot.declaration.0)
            .map_err(|_| invalid(path, "functional Counter identity exceeds u32"))?;
        if !self.counters.iter().any(|present| present.id == id) {
            self.counters.push(FunctionalCounterSlot {
                id,
                declaration: slot.declaration,
            });
        }
        Ok(
            syn::parse_quote!(crate::runtime::functional::Counter::new(crate::runtime::functional::SlotId::new(#id))),
        )
    }

    fn bound(&mut self, bound: syn::TypeParamBound) {
        if !self.executor_bounds.iter().any(|present| {
            present.to_token_stream().to_string() == bound.to_token_stream().to_string()
        }) {
            self.executor_bounds.push(bound);
        }
    }
}

fn pure(binding: &Binding, operation: &Operation) -> Result<syn::Expr, RecordingGap> {
    let ty =
        rust_type(&binding.value.ty).map_err(|error| invalid(&binding.path, error.to_string()))?;
    Ok(match operation {
        Operation::Literal(literal) => match literal {
            Literal::Unit => syn::parse_quote!(()),
            Literal::Boolean(boolean) => syn::parse_quote!(#boolean),
            Literal::Bytes(bytes) => {
                syn::parse_quote!(crate::runtime::FixedBytes::new([#(#bytes),*]))
            }
            Literal::Field(bytes) => {
                syn::parse_quote!(crate::runtime::Field::from_le_bytes(&[#(#bytes),*]).ok_or(crate::runtime::CompactError::InvalidUnsignedValue)?)
            }
            Literal::Unsigned(number) => {
                let number =
                    syn::LitInt::new(&format!("{number}u128"), proc_macro2::Span::call_site());
                syn::parse_quote!(<#ty>::new(#number)?)
            }
            Literal::WideUnsigned(bytes) => {
                syn::parse_quote!(<#ty>::from_le_bytes(&[#(#bytes),*])?)
            }
        },
        Operation::EnumVariant { variant } => {
            let variant =
                ident(variant).map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(#ty::#variant)
        }
        Operation::Default => syn::parse_quote!(<#ty as ::core::default::Default>::default()),
        Operation::Equal(left, right) => {
            let (left, right) = (value(left), value(right));
            syn::parse_quote!(#left == #right)
        }
        Operation::NotEqual(left, right) => {
            let (left, right) = (value(left), value(right));
            syn::parse_quote!(#left != #right)
        }
        _ => {
            return Err(invalid(
                &binding.path,
                "operation outside functional rendering subset",
            ));
        }
    })
}
