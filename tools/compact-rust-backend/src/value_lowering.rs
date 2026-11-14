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

//! Shared typed-value lowering for pure, native and recorded emitters.
//!
//! Maps Compact types and bounds to Rust carriers and checked conversion syntax.
//! Runtime operations remain authoritative; this module neither executes ledger
//! queries nor decides whole-circuit purity or recording admission.

use crate::RenderError;
use crate::ir::{Expr, Type};
use proc_macro2::Span;

use crate::pure_semantics::arithmetic::Secp256k1Unary;
use crate::pure_semantics::conversion::{FixedBytesConversion, JubjubCast, Secp256k1BytesCast};
use crate::pure_semantics::numeric::{
    ArithmeticKind, UnsignedArithmeticStrategy, checked_unsigned_arithmetic, checked_unsigned_cast,
};
pub(crate) use crate::pure_semantics::numeric::{
    UnsignedMaximum, field_literal_bytes, unsigned_maximum,
};

pub(crate) fn wide_uint_type(high: u128, low: u128) -> syn::Type {
    let high = syn::LitInt::new(&format!("{high}u128"), Span::call_site());
    let low = syn::LitInt::new(&format!("{low}u128"), Span::call_site());
    syn::parse_quote!(runtime::WideUint<#high, #low>)
}

/// Unit comparisons still evaluate both checked operands once, left to right.
/// Typed discard bindings avoid inferring Unit temporaries or emitting a Rust
/// Unit comparison that strict Clippy rejects.
pub(crate) fn unit_comparison_syntax(left: syn::Expr, right: syn::Expr, result: bool) -> syn::Expr {
    syn::parse_quote!({
        let _: () = #left;
        let _: () = #right;
        #result
    })
}

pub(crate) fn field_to_bytes_32_syntax(value: syn::Expr) -> syn::Expr {
    // Compact's Field-to-Bytes cast is zero-padded little endian. The pinned
    // midnight-zk Field has a canonical 32-byte representation.
    syn::parse_quote!(runtime::FixedBytes::<32>::new(
        (#value).as_le_bytes().try_into().expect("canonical Field is 32 bytes")
    ))
}

pub(crate) fn fixed_bytes_conversion_syntax(
    value: syn::Expr,
    operation: &FixedBytesConversion,
) -> syn::Expr {
    match *operation {
        FixedBytesConversion::ToVector { .. } => syn::parse_quote!(runtime::FixedVector::new(
            (#value).into_array().map(|byte| runtime::BoundedUint::<255>::new(u128::from(byte)).expect("byte fits Uint<8>"))
        )),
        FixedBytesConversion::FromVector { .. } => syn::parse_quote!(runtime::FixedBytes::new(
            (#value).into_array().map(|byte| u8::try_from(byte.value()).expect("Uint<8> fits a byte"))
        )),
        FixedBytesConversion::UintToBytes { length } => syn::parse_quote!({
            let bytes = (#value).value().to_le_bytes();
            let len = #length.min(bytes.len());
            if bytes[len..].iter().any(|&byte| byte != 0) {
                Err(runtime::CompactError::InvalidUnsignedValue)
            } else {
                let mut output = [0u8; #length];
                output[..len].copy_from_slice(&bytes[..len]);
                Ok(runtime::FixedBytes::new(output))
            }?
        }),
        FixedBytesConversion::BytesToField => syn::parse_quote!({
            let input = (#value).into_array();
            let mut bytes = [0u8; 32];
            let len = input.len().min(bytes.len());
            let field = if input[len..].iter().any(|&byte| byte != 0) {
                None
            } else {
                bytes[..len].copy_from_slice(&input[..len]);
                runtime::Field::from_le_bytes(&bytes)
            };
            field.ok_or_else(|| runtime::CompactError::AssertionFailed(
                "byte vector exceeds maximum value of Field type".into()
            ))?
        }),
        FixedBytesConversion::WideUintToBytes { length } => syn::parse_quote!({
            let value = #value;
            let bytes = value.as_le_bytes();
            let len = #length.min(bytes.len());
            if bytes[len..].iter().any(|&byte| byte != 0) {
                Err(runtime::CompactError::InvalidUnsignedValue)
            } else {
                let mut output = [0u8; #length];
                output[..len].copy_from_slice(&bytes[..len]);
                Ok(runtime::FixedBytes::new(output))
            }?
        }),
        FixedBytesConversion::BytesToUint {
            bound: UnsignedMaximum::Wide { high, low },
            ..
        } => {
            let target = wide_uint_type(high, low);
            syn::parse_quote!({
                let input = (#value).into_array();
                let len = input.len().min(32);
                if input[len..].iter().any(|&byte| byte != 0) {
                    Err(runtime::CompactError::InvalidUnsignedValue)
                } else {
                    <#target>::from_le_bytes(&input[..len])
                }?
            })
        }
        FixedBytesConversion::BytesToUint {
            bound: UnsignedMaximum::Small(max),
            ..
        } => {
            let max = syn::LitInt::new(&format!("{max}u128"), Span::call_site());
            syn::parse_quote!({
                let input = (#value).into_array();
                let mut bytes = [0u8; 16];
                let len = input.len().min(bytes.len());
                if input[len..].iter().any(|&byte| byte != 0) {
                    Err(runtime::CompactError::InvalidUnsignedValue)
                } else {
                    bytes[..len].copy_from_slice(&input[..len]);
                    runtime::BoundedUint::<#max>::new(u128::from_le_bytes(bytes))
                }?
            })
        }
        FixedBytesConversion::ZeroPad { length } => syn::parse_quote!({
            let input = (#value).into_array();
            let mut output = [0u8; #length];
            output[..input.len()].copy_from_slice(&input);
            runtime::FixedBytes::new(output)
        }),
        FixedBytesConversion::FromChunks {
            length,
            chunk_count,
        } => {
            let indices = (0..chunk_count).map(syn::Index::from);
            syn::parse_quote!({
                let chunks = #value;
                let mut output = [0u8; #length];
                let mut remaining = output.as_mut_slice();
                #(
                    let chunk = chunks.#indices.into_array();
                    let (destination, suffix) = remaining.split_at_mut(chunk.len());
                    destination.copy_from_slice(&chunk);
                    remaining = suffix;
                )*
                debug_assert!(remaining.is_empty());
                runtime::FixedBytes::new(output)
            })
        }
        FixedBytesConversion::Slice { start, length } => {
            let end = start + length;
            syn::parse_quote!(runtime::FixedBytes::<#length>::new(
                (#value).into_array()[#start..#end].try_into().expect("checked Bytes slice length")
            ))
        }
    }
}

pub(crate) fn jubjub_cast_syntax(value: syn::Expr, cast: JubjubCast) -> syn::Expr {
    use crate::pure_semantics::numeric::UnsignedMaximum;
    match cast {
        JubjubCast::Scalar => {
            syn::parse_quote!(runtime::JubjubScalar::from_field_reduced(#value))
        }
        JubjubCast::Field => syn::parse_quote!((#value).as_field()),
        JubjubCast::Unsigned(UnsignedMaximum::Small(max)) => {
            syn::parse_quote!((#value).try_to_uint::<#max>()?)
        }
        JubjubCast::Unsigned(UnsignedMaximum::Wide { high, low }) => {
            syn::parse_quote!((#value).try_to_wide_uint::<#high, #low>()?)
        }
    }
}

pub(crate) fn enum_cast_syntax(
    value: syn::Expr,
    cast: crate::pure_semantics::conversion::EnumCast,
) -> syn::Expr {
    use crate::pure_semantics::conversion::{EnumCast, EnumOrdinalSource};
    match cast {
        EnumCast::ToField => syn::parse_quote!(runtime::FieldRepr::field_vec(&(#value))[0]),
        EnumCast::ToUnsigned {
            max_ordinal,
            strategy,
        } => {
            let bound = syn::LitInt::new(&format!("{max_ordinal}u128"), Span::call_site());
            let ordinal = syn::parse_quote!({
                let field = runtime::FieldRepr::field_vec(&(#value))[0];
                let ordinal = <u128 as runtime::FromFieldRepr>::from_field_repr(&[field])
                    .ok_or(runtime::CompactError::InvalidUnsignedValue)?;
                runtime::BoundedUint::<#bound>::new(ordinal)?
            });
            coercion::render_unsigned_cast(ordinal, strategy)
        }
        EnumCast::FromNumber { name, source } => {
            let name = crate::naming::checked_ident(&name);
            let field: syn::Expr = match source {
                EnumOrdinalSource::Field => value,
                EnumOrdinalSource::Small => {
                    syn::parse_quote!(runtime::Field::from((#value).value()))
                }
                EnumOrdinalSource::Wide => syn::parse_quote!((#value).as_field()),
            };
            syn::parse_quote!(<crate::types::#name as runtime::FromFieldRepr>::from_field_repr(&[#field])
                .ok_or(runtime::CompactError::InvalidUnsignedValue)?)
        }
    }
}

pub(crate) fn secp256k1_bytes_cast_syntax(value: syn::Expr, cast: Secp256k1BytesCast) -> syn::Expr {
    match cast {
        Secp256k1BytesCast::ToBase => {
            syn::parse_quote!(runtime::Secp256k1Base::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToScalar => {
            syn::parse_quote!(runtime::Secp256k1Scalar::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToSecp256r1Base => {
            syn::parse_quote!(runtime::Secp256r1Base::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToSecp256r1Scalar => {
            syn::parse_quote!(runtime::Secp256r1Scalar::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToCurve25519Base => {
            syn::parse_quote!(runtime::Curve25519Base::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToCurve25519Scalar => {
            syn::parse_quote!(runtime::Curve25519Scalar::from_le_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::WideToCurve25519Scalar => {
            syn::parse_quote!(runtime::Curve25519Scalar::from_le_wide_bytes_reduced((#value).into_array()))
        }
        Secp256k1BytesCast::ToBytes => {
            syn::parse_quote!(runtime::FixedBytes::<32>::new((#value).to_le_bytes()))
        }
    }
}

pub(crate) fn secp256k1_unary_syntax(kind: Secp256k1Unary, value: syn::Expr) -> syn::Expr {
    match kind {
        Secp256k1Unary::Negate => syn::parse_quote!(-(#value)),
        Secp256k1Unary::Invert => syn::parse_quote!((#value).inverse()?),
        Secp256k1Unary::PointX => syn::parse_quote!((#value).x()?),
        Secp256k1Unary::PointY => syn::parse_quote!((#value).y()?),
    }
}

pub(crate) fn unsigned_cast_syntax(
    value: syn::Expr,
    source_max: &str,
    target_max: &str,
) -> Result<syn::Expr, RenderError> {
    Ok(coercion::render_unsigned_cast(
        value,
        checked_unsigned_cast(source_max, target_max)?,
    ))
}

/// Both expression emitters consume the same checked runtime arithmetic strategy.
pub(crate) fn unsigned_arithmetic_syntax(
    operation: &Expr,
    left: syn::Expr,
    right: syn::Expr,
    left_max: &str,
    right_max: &str,
    result_max: &str,
) -> Result<syn::Expr, RenderError> {
    let kind = match operation {
        Expr::UnsignedAdd { .. } => ArithmeticKind::Add,
        Expr::UnsignedSubtract { .. } => ArithmeticKind::Subtract,
        Expr::UnsignedMultiply { .. } => ArithmeticKind::Multiply,
        _ => unreachable!("unsigned arithmetic caller"),
    };
    let strategy = checked_unsigned_arithmetic(kind, left_max, right_max, result_max)?;
    Ok(render_unsigned_arithmetic(strategy, left, right))
}

/// Format a checked strategy without parsing or deciding operation support.
pub(crate) fn render_unsigned_arithmetic(
    strategy: UnsignedArithmeticStrategy,
    left: syn::Expr,
    right: syn::Expr,
) -> syn::Expr {
    match strategy {
        UnsignedArithmeticStrategy::WideZeroProduct { result } => {
            let result = syn::LitInt::new(&result.to_string(), Span::call_site());
            syn::parse_quote!({
                let _ = #left;
                let _ = #right;
                runtime::BoundedUint::<#result>::new(0)?
            })
        }
        UnsignedArithmeticStrategy::WideAdd { high, low } => {
            let high = syn::LitInt::new(&format!("{high}u128"), Span::call_site());
            let low = syn::LitInt::new(&format!("{low}u128"), Span::call_site());
            syn::parse_quote!(runtime::add_wide_unsigned::<#high,#low,_,_>(#left,#right)?)
        }
        UnsignedArithmeticStrategy::Small {
            kind,
            left: left_bound,
            right: right_bound,
            result,
        } => {
            let left_max = syn::LitInt::new(&left_bound.to_string(), Span::call_site());
            let right_max = syn::LitInt::new(&right_bound.to_string(), Span::call_site());
            let result_max = syn::LitInt::new(&result.to_string(), Span::call_site());
            let helper: syn::Path = match kind {
                ArithmeticKind::Add => syn::parse_quote!(runtime::add_unsigned),
                ArithmeticKind::Subtract => syn::parse_quote!(runtime::subtract_unsigned),
                ArithmeticKind::Multiply => syn::parse_quote!(runtime::multiply_unsigned),
            };
            syn::parse_quote!(#helper::<#left_max,#right_max,#result_max>(#left,#right)?)
        }
    }
}

pub(crate) fn rust_type(ty: &Type) -> Result<syn::Type, RenderError> {
    crate::pure_semantics::carrier::fold_carrier(ty, &mut carrier::NativeCarrierSink)
}
pub(crate) fn rust_type_observed(
    ty: &Type,
    observer: Option<&dyn crate::source_diagnostics::RefusalObserver>,
) -> Result<syn::Type, RenderError> {
    crate::pure_semantics::carrier::fold_carrier_observed(
        ty,
        &mut carrier::NativeCarrierSink,
        observer,
    )
}

fn map_slot_value_type(value: &Type) -> Result<Option<syn::Type>, RenderError> {
    if let Type::LedgerMap { key, value } = value {
        let Some((key, value)) = map_slot_types(key, value)? else {
            return Ok(None);
        };
        return Ok(Some(
            syn::parse_quote!(runtime::slots::MapNode<#key, #value>),
        ));
    }
    if let Type::LedgerMerkle {
        historic,
        depth,
        ty,
    } = value
    {
        if !(2..=32).contains(depth) {
            return Err(RenderError::UnsupportedLedgerValueType(value.clone()));
        }
        let leaf = rust_type(ty)?;
        return Ok(Some(
            syn::parse_quote!(runtime::slots::MerkleNode<#leaf, #depth, #historic>),
        ));
    }
    match rust_type(value) {
        Ok(ty) => Ok(Some(ty)),
        Err(RenderError::UnsupportedLedgerValueType(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn map_slot_types(
    key: &Type,
    value: &Type,
) -> Result<Option<(syn::Type, syn::Type)>, RenderError> {
    let key = match rust_type(key) {
        Ok(ty) => ty,
        Err(RenderError::UnsupportedLedgerValueType(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let value = match map_slot_value_type(value)? {
        Some(ty) => ty,
        None => return Ok(None),
    };
    Ok(Some((key, value)))
}

pub(crate) fn coerce_expression(
    value: syn::Expr,
    actual: &Type,
    target: &Type,
    depth: usize,
) -> Result<syn::Expr, RenderError> {
    let plan = crate::pure_semantics::coercion::plan_coercion(actual, target)?;
    Ok(coercion::render(&plan, value, depth))
}

// Existing output contracts inspect conversion fallibility independently of
// source-expression evaluation. Production support belongs to the neutral plan.
#[cfg(test)]
struct Coercion {
    expression: syn::Expr,
    may_fail: bool,
}

#[cfg(test)]
fn coercion(
    value: syn::Expr,
    actual: &Type,
    target: &Type,
    depth: usize,
) -> Result<Coercion, RenderError> {
    let plan = crate::pure_semantics::coercion::plan_coercion(actual, target)?;
    Ok(Coercion {
        expression: coercion::render(&plan, value, depth),
        may_fail: plan.may_fail(),
    })
}

pub(crate) fn copy_type(ty: &Type) -> bool {
    crate::pure_semantics::ownership::copy_type(ty)
}

/// Keep an owned value available at an emission boundary without cloning Copy types.
pub(crate) fn retained_value(value: syn::Expr, ty: &Type) -> syn::Expr {
    use crate::pure_semantics::ownership::Ownership;
    let ownership = if copy_type(ty) {
        Ownership::Copy
    } else {
        Ownership::Owned
    };
    retained_value_with_ownership(value, ownership)
}

/// Output-only retention; the semantic owner has already chosen ownership.
pub(crate) fn retained_value_with_ownership(
    value: syn::Expr,
    ownership: crate::pure_semantics::ownership::Ownership,
) -> syn::Expr {
    match ownership {
        crate::pure_semantics::ownership::Ownership::Copy => value,
        crate::pure_semantics::ownership::Ownership::Owned => syn::parse_quote!((#value).clone()),
    }
}

#[cfg(test)]
mod tests;

pub(crate) mod carrier;

pub(crate) mod coercion;
