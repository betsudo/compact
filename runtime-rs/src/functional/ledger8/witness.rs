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

/// Generated handlers map a checked WitnessId to the existing typed witness API
/// and project the existing metered ledger view. Handwritten handlers and their
/// host effects are not certified by this interface. Ordinary aligned results
/// reuse the existing frame conversion. Exact opaque transport requires a
/// separately checked bridge and is not selected by this interface.
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

impl<Private, D: DB, Args, T, W> WitnessCall<Args, T> for NativeLedger8<'_, Private, D, W>
where
    W: WitnessHandler<Private, D, Args, T>,
    T: Clone,
    AlignedValue: From<T>,
{
    fn witness(
        self,
        witness: Witness<Args, T>,
        arguments: Args,
    ) -> Result<(Self, T), CompactError> {
        let (frame, value) = self.frame.try_witness_metered(|context, meter| {
            self.witnesses.call(witness.id(), arguments, context, meter)
        })?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
impl<Private, D: DB, Args, T, W> WitnessCall<Args, T> for RecordedLedger8<'_, Private, D, W>
where
    W: WitnessHandler<Private, D, Args, T>,
    T: Clone,
    AlignedValue: From<T>,
{
    fn witness(
        self,
        witness: Witness<Args, T>,
        arguments: Args,
    ) -> Result<(Self, T), CompactError> {
        let (frame, value) = self.frame.try_witness_metered(|context, meter| {
            self.witnesses.call(witness.id(), arguments, context, meter)
        })?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
