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

//! Conversion syntax from checked facts and one already checked child.
//! No source Type pair, support predicate or numeric parsing enters formatting.
use super::leaf::NativeLeafSink;
use crate::{
    pure_semantics::{conversion::ConversionFacts, sink::ConversionSink},
    value_lowering::{
        coercion, field_to_bytes_32_syntax, fixed_bytes_conversion_syntax,
        jubjub_cast_syntax, secp256k1_bytes_cast_syntax,
    },
};

impl ConversionSink for NativeLeafSink {
    fn conversion(&mut self, facts: ConversionFacts, child: syn::Expr) -> syn::Expr {
        match facts {
            ConversionFacts::UintToField { wide: false } => {
                syn::parse_quote!(runtime::Field::from((#child).value()))
            }
            ConversionFacts::UintToField { wide: true } => syn::parse_quote!((#child).as_field()),
            ConversionFacts::FieldToBytes32 => field_to_bytes_32_syntax(child),
            ConversionFacts::JubjubCast(cast) => jubjub_cast_syntax(child, cast),
            ConversionFacts::EnumCast(cast) => crate::value_lowering::enum_cast_syntax(child, cast),
            ConversionFacts::Secp256k1BytesCast(cast) => secp256k1_bytes_cast_syntax(child, cast),
            ConversionFacts::FixedBytes(operation) => {
                fixed_bytes_conversion_syntax(child, operation)
            }
            ConversionFacts::Coercion(plan) => coercion::render(&plan, child, 0),
            ConversionFacts::ExplicitUnsignedCast(cast) => {
                coercion::render_unsigned_cast(child, cast)
            }
        }
    }
}
