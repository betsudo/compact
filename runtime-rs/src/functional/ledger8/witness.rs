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

//! Metered witness bridge; the handler is supplied to an execution, not a program.
use super::super::{Witness, WitnessCall, WitnessId};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    context::{CircuitContext, WitnessReadMeter},
    fab::AlignedValue,
    ledger::DB,
};

pub struct NoWitnesses;

/// Select the existing exact opaque-bearing private-output encoding.
/// This is an in-memory transport choice, not a proof-serialization guarantee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactWitnessOutput;

/// Generated handlers map a checked WitnessId to the existing typed witness API
/// and project the existing metered ledger view. Handwritten handlers and their
/// host effects are not certified by this interface. Ordinary aligned results
/// reuse the existing frame conversion. A separately selected ExactWitnessOutput
/// strategy delegates the existing exact frame conversion; neither strategy
/// certifies that its output can pass later proof-serialization checks.
///
/// The borrowed context matches the existing frame callback API; a handwritten
/// handler can bypass the meter by reading it directly. Generated handlers must
/// use their metered ledger view for every projected read. This interface does
/// not make unmetered host access impossible or certify callback behavior.
pub trait WitnessHandler<Private, D: DB, Args, T> {
    fn call(
        &self,
        id: WitnessId,
        arguments: Args,
        context: &CircuitContext<Private, D>,
        meter: &WitnessReadMeter<'_, D>,
    ) -> Result<(Private, T), CompactError>;
}
impl<Private, D: DB, Args, T, H: WitnessHandler<Private, D, Args, T>>
    WitnessHandler<Private, D, Args, T> for &H
{
    fn call(
        &self,
        id: WitnessId,
        arguments: Args,
        context: &CircuitContext<Private, D>,
        meter: &WitnessReadMeter<'_, D>,
    ) -> Result<(Private, T), CompactError> {
        (**self).call(id, arguments, context, meter)
    }
}

// Only frame ownership differs between the native and recorded interpreters.
// Keep their four transport implementations mechanical; codecs remain in frames.
macro_rules! witness_bridge {
    ($adapter:ident, $mode:ty, $method:ident, $($transport_bound:tt)*) => {
        impl<Private, D: DB, Args, T, W> WitnessCall<Args, T, $mode>
            for $adapter<'_, Private, D, W>
        where
            W: WitnessHandler<Private, D, Args, T>,
            T: Clone,
            $($transport_bound)*
        {
            fn witness(
                self,
                witness: Witness<Args, T, $mode>,
                arguments: Args,
            ) -> Result<(Self, T), CompactError> {
                let (frame, value) = self.frame.$method(|context, meter| {
                    self.witnesses.call(witness.id(), arguments, context, meter)
                })?;
                Ok((Self { frame, bindings: self.bindings, witnesses: self.witnesses }, value))
            }
        }
    };
}
witness_bridge!(NativeLedger8, (), try_witness_metered, AlignedValue: From<T>,);
witness_bridge!(RecordedLedger8, (), try_witness_metered, AlignedValue: From<T>,);
witness_bridge!(NativeLedger8, ExactWitnessOutput, try_witness_metered_exact, T: crate::HashInputValue,);
witness_bridge!(RecordedLedger8, ExactWitnessOutput, try_witness_metered_exact, T: crate::HashInputValue,);
