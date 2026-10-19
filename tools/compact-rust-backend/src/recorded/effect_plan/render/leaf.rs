// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Nonrecursive opcode syntax. Large quote temporaries stay off recursive block frames.
use super::*;
use crate::ir::Type;

pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    steps: &mut Vec<syn::Stmt>,
) -> Result<(), RecordingGap> {
    let path = &binding.path;
    let output = name(&binding.value);
    let ty = rust_type(&binding.value.ty).map_err(|e| invalid(path, e.to_string()))?;
    let expression: syn::Expr = match &binding.operation {
        Operation::Literal(literal) => match literal {
            Literal::Unit => syn::parse_quote!(()),
            Literal::Bytes(bytes) => syn::parse_quote!(runtime::FixedBytes::new([#(#bytes),*])),
            Literal::Boolean(b) => syn::parse_quote!(#b),
            Literal::Field(bytes) => {
                syn::parse_quote!(runtime::Field::from_le_bytes(&[#(#bytes),*]).expect("validated Compact Field literal"))
            }
            Literal::WideUnsigned(bytes) => {
                syn::parse_quote!(<#ty>::from_le_bytes(&[#(#bytes),*])?)
            }
            Literal::Unsigned(n) => {
                let n = syn::LitInt::new(&format!("{n}u128"), Span::call_site());
                syn::parse_quote!(<#ty>::new(#n)?)
            }
        },
        Operation::EnumVariant { variant } => {
            let variant = ident(variant).map_err(|e| invalid(path, e.to_string()))?;
            syn::parse_quote!(#ty::#variant)
        }
        Operation::Default => syn::parse_quote!(<#ty as ::core::default::Default>::default()),
        Operation::StructConstruct { .. }
        | Operation::StructProject { .. }
        | Operation::TupleProject { .. }
        | Operation::Vector { .. }
        | Operation::Tuple { .. }
        | Operation::Add(..)
        | Operation::Subtract(..)
        | Operation::Multiply(..)
        | Operation::Compare { .. }
        | Operation::Equal(..)
        | Operation::NotEqual(..)
        | Operation::UnsignedAdd(..)
        | Operation::UnsignedSubtract(..)
        | Operation::UnsignedMultiply(..)
        | Operation::Convert { .. } => pure::expression(binding, &ty, value, |input| {
            let source = name(input);
            syn::parse_quote!(#source)
        })?
        .expect("matched shared checked value expression"),
        Operation::CreateZswapInput { coin } => {
            steps.push(zswap::input(coin));
            syn::parse_quote!(())
        }
        Operation::CreateZswapOutput { coin, recipient } => {
            steps.push(zswap::output(coin, recipient));
            syn::parse_quote!(())
        }
        Operation::KernelClaim { kind, value: input } => {
            use crate::ir::KernelClaimKind;
            let input = value(input);
            let claim: syn::Expr = match kind {
                KernelClaimKind::Nullifier => {
                    syn::parse_quote!(runtime::ledger::KernelClaim::Nullifier(
                        runtime::ledger::CoinNullifier(runtime::ledger::HashOutput((#input).into_array()))))
                }
                KernelClaimKind::CoinSpend => {
                    syn::parse_quote!(runtime::ledger::KernelClaim::CoinSpend(
                        runtime::ledger::CoinCommitment(runtime::ledger::HashOutput((#input).into_array()))))
                }
                KernelClaimKind::CoinReceive => {
                    syn::parse_quote!(runtime::ledger::KernelClaim::CoinReceive(
                        runtime::ledger::CoinCommitment(runtime::ledger::HashOutput((#input).into_array()))))
                }
            };
            steps.push(syn::parse_quote!(let frame = frame.kernel_claim(#claim)?;));
            syn::parse_quote!(())
        }
        Operation::KernelMintShielded { domain, amount } => {
            let (domain, amount) = (value(domain), value(amount));
            // The checked exact Uint64 signature proves this conversion fits.
            steps.push(syn::parse_quote!(let frame = frame.kernel_mint_shielded(
                    runtime::ledger::HashOutput((#domain).into_array()), (#amount).value() as u64,
                )?;));
            syn::parse_quote!(())
        }
        Operation::OwnPublicKey => {
            let observed = syn::Ident::new(
                &format!("__compact_effect_own_key_{}", binding.value.id),
                Span::call_site(),
            );
            steps.push(syn::parse_quote!(let (frame, #observed) = frame.own_coin_public_key()?;));
            syn::parse_quote!(#ty { bytes: runtime::FixedBytes::new(#observed) })
        }
        Operation::KernelSelf => {
            let observed = syn::Ident::new(
                &format!("__compact_effect_address_{}", binding.value.id),
                Span::call_site(),
            );
            steps.push(syn::parse_quote!(let (frame, #observed) = frame.kernel_self()?;));
            syn::parse_quote!(#ty { bytes: runtime::ledger::contract_address_bytes(&#observed) })
        }
        Operation::PersistentCommit {
            value: input,
            opening,
        } => {
            let (input, opening) = (value(input), value(opening));
            syn::parse_quote!(runtime::persistent_commit(#input, #opening))
        }
        Operation::FieldToBytes32(input) => crate::field_to_bytes_32_syntax(value(input)),
        Operation::PersistentHash(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::persistent_hash(#input))
        }
        Operation::HashToCurve(input) => {
            let input = crate::hash_input::syntax(value(input), &input.ty);
            syn::parse_quote!(runtime::hash_to_curve(#input))
        }
        Operation::Keccak256(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::keccak256(#input))
        }
        Operation::JubjubPointX(input) | Operation::JubjubPointY(input) => {
            let coordinate = if matches!(binding.operation, Operation::JubjubPointX(_)) {
                crate::jubjub_value::Coordinate::X
            } else {
                crate::jubjub_value::Coordinate::Y
            };
            coordinate
                .lower(operand(input))
                .map(|(syntax, _)| syntax)
                .map_err(|e| invalid(path, e.to_string()))?
        }
        Operation::JubjubScalarFromNative(input) => {
            jubjub(crate::jubjub_value::Operation::Reduce(operand(input)), path)?
        }
        Operation::EcMulGenerator(input) => jubjub(
            crate::jubjub_value::Operation::Generator(operand(input)),
            path,
        )?,
        Operation::EcNeg(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::ec_neg(#input))
        }
        Operation::ConstructJubjubPoint { x, y } => {
            let (x, y) = (value(x), value(y));
            syn::parse_quote!(runtime::construct_jubjub_point(#x, #y)?)
        }
        Operation::EcMul { point, scalar } => jubjub(
            crate::jubjub_value::Operation::Multiply(operand(point), operand(scalar)),
            path,
        )?,
        Operation::EcAdd { left, right } => jubjub(
            crate::jubjub_value::Operation::Add(operand(left), operand(right)),
            path,
        )?,
        Operation::DegradeToTransient(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::degrade_to_transient(#input))
        }
        Operation::UpgradeFromTransient(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::upgrade_from_transient(#input))
        }
        Operation::TransientHash(input) => {
            let input = crate::hash_input::syntax(value(input), &input.ty);
            syn::parse_quote!(runtime::transient_hash(#input))
        }
        Operation::TransientCommit {
            value: input,
            opening,
        } => {
            let (input, opening) = (
                crate::hash_input::syntax(value(input), &input.ty),
                value(opening),
            );
            syn::parse_quote!(runtime::transient_commit(#input, #opening))
        }
        Operation::Merkle { slot, operation } => {
            return merkle::render(binding, *slot, operation, declarations, steps);
        }
        Operation::List { slot, operation } => {
            return lists::render(binding, *slot, operation, declarations, steps);
        }
        Operation::CellRead(slot) => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(syn::parse_quote!(let (frame,#output): (_,#ty) = crate::ledger_slots::#slot.record_read(frame)?;));
            return Ok(());
        }
        Operation::CellWrite { slot, value: input } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let input = value(input);
            steps.push(syn::parse_quote!(let frame = crate::ledger_slots::#slot.record_write(frame,#input)?;));
            syn::parse_quote!(())
        }
        Operation::CellWriteCoin {
            slot,
            coin,
            recipient,
        } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(qualified_coins::write(&slot, coin, recipient));
            syn::parse_quote!(())
        }
        Operation::SetInsertCoin {
            slot,
            coin,
            recipient,
        } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(qualified_coins::insert(&slot, coin, recipient));
            syn::parse_quote!(())
        }
        Operation::MapMember { .. }
        | Operation::MapLookup { .. }
        | Operation::MapInsert { .. }
        | Operation::MapRemove { .. }
        | Operation::MapInsertDefault { .. }
        | Operation::MapReset { .. }
        | Operation::MapSize { .. }
        | Operation::MapIsEmpty { .. } => {
            return maps::render(binding, declarations, steps);
        }
        Operation::SetMember { slot, value: input } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let input = value(input);
            steps.push(syn::parse_quote!(let (frame, #output): (_, #ty) = crate::ledger_slots::#slot.record_member(frame, #input)?;));
            return Ok(());
        }
        Operation::SetInsert { slot, value: input }
        | Operation::SetRemove { slot, value: input } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let input = value(input);
            let method = ident(
                if matches!(binding.operation, Operation::SetInsert { .. }) {
                    "record_insert"
                } else {
                    "record_remove"
                },
            )
            .map_err(|e| invalid(path, e.to_string()))?;
            steps.push(
                syn::parse_quote!(let frame = crate::ledger_slots::#slot.#method(frame, #input)?;),
            );
            syn::parse_quote!(())
        }
        Operation::SetReset { slot } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(
                syn::parse_quote!(let frame = crate::ledger_slots::#slot.record_reset(frame)?;),
            );
            syn::parse_quote!(())
        }
        Operation::SetIsEmpty { slot } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(syn::parse_quote!(let (frame, #output): (_, #ty) = crate::ledger_slots::#slot.record_is_empty(frame)?;));
            return Ok(());
        }
        Operation::SetSize { slot } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let raw = syn::Ident::new(
                &format!("__compact_effect_set_size_{}", binding.value.id),
                Span::call_site(),
            );
            steps.push(syn::parse_quote!(let (frame, #raw) = crate::ledger_slots::#slot.record_size(frame)?;));
            syn::parse_quote!(<#ty>::new(#raw as u128)?)
        }
        Operation::CounterRead { slot } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let raw = syn::Ident::new(
                &format!("__compact_effect_raw_{}", binding.value.id),
                Span::call_site(),
            );
            steps.push(syn::parse_quote!(let (frame,#raw) = crate::ledger_slots::#slot.record_read(frame)?;));
            syn::parse_quote!(<#ty>::new(#raw as u128)?)
        }
        Operation::CounterIncrement { slot, amount }
        | Operation::CounterDecrement { slot, amount } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let amount = value(amount);
            let method = if matches!(&binding.operation, Operation::CounterIncrement { .. }) {
                syn::Ident::new("record_increment", Span::call_site())
            } else {
                syn::Ident::new("record_decrement", Span::call_site())
            };
            steps.push(syn::parse_quote!(let frame = crate::ledger_slots::#slot.#method(frame,(#amount).value() as u16)?;));
            syn::parse_quote!(())
        }
        Operation::CounterReset { slot } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            steps.push(
                syn::parse_quote!(let frame = crate::ledger_slots::#slot.record_reset(frame)?;),
            );
            syn::parse_quote!(())
        }
        Operation::CounterLessThan { slot, threshold } => {
            let slot = slot_name(declarations, slot.declaration, path)?;
            let threshold = value(threshold);
            steps.push(syn::parse_quote!(let (frame,#output): (_,#ty) = crate::ledger_slots::#slot.record_less_than(frame,(#threshold).value() as u64)?;));
            return Ok(());
        }
        Operation::Witness {
            declaration,
            arguments,
        } => {
            let witness = declarations
                .witness(*declaration)
                .ok_or_else(|| invalid(path, "unknown checked witness"))?;
            let method = ident(&witness.name).map_err(|e| invalid(path, e.to_string()))?;
            let args: Vec<_> = arguments.iter().map(value).collect();
            let frame_method = crate::hash_input::witness_method(&witness.result);
            steps.push(syn::parse_quote! {
                let (frame,#output): (_,#ty) = frame.#frame_method(|context,meter| {
                    witnesses.#method(context.witness_context_with(super::LedgerView {
                        state: context.query.state.get_ref(),meter,
                    }),#(#args),*)
                })?;
            });
            return Ok(());
        }
        Operation::Assert { condition, message } => {
            let condition = value(condition);
            steps.push(syn::parse_quote!(if !#condition { return Err(runtime::CompactError::AssertionFailed(#message.to_owned())); }));
            syn::parse_quote!(())
        }
        Operation::PureCall { .. } => pure::certified_call(binding, declarations, value)?,
        Operation::Call { .. } | Operation::If { .. } | Operation::VectorMap { .. } => {
            unreachable!("control operations are rendered by block traversal")
        }
    };
    // Every operation above has already emitted its effects/failures. Suppress
    // only a payload-free literal, never a call or fallible Unit expression.
    let literal_unit = binding.value.ty == Type::Unit
        && matches!(&expression, syn::Expr::Tuple(tuple) if tuple.elems.is_empty() && tuple.attrs.is_empty());
    if !literal_unit {
        steps.push(syn::parse_quote!(let #output: #ty = #expression;));
    }
    Ok(())
}
