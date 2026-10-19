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

//! Functional crypto proof overlay; ordinary execution is comparison only.
use super::*;
use functional_crypto_contract::{ledger_contract as c, runtime as r};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct Marker(Rc<RefCell<Vec<(u64, Field)>>>);
impl c::TryWitnesses<u64> for Marker {
    fn marker(&self, context: WitnessContext<'_, u64, c::LedgerView<'_>>, value: Field)
        -> Result<(u64, Field), r::CompactError> {
        self.0.borrow_mut().push((*context.private_state, value));
        Ok((*context.private_state + 1, value))
    }
}
pub(super) fn arguments(arguments: &mut impl Iterator<Item=std::ffi::OsString>) -> Result<(),Box<dyn Error>> {
    let mode=arguments.next().ok_or("missing crypto circuit")?;
    let root=arguments.next().ok_or("missing crypto artifact directory")?;
    if arguments.next().is_some() { return Err("unexpected crypto proof arguments".into()); }
    match mode.to_str() {
        Some("hashing")=>hashing(Path::new(&root)),
        Some("jubjub")=>jubjub(Path::new(&root)),
        _=>Err("expected hashing or jubjub".into())
    }
}
macro_rules! prove_case {
    ($root:ident,$entry:ident,$call:ident,[$($arg:expr),*],$check:expr) => {{
        let name=stringify!($entry);
        let mut rng=StdRng::seed_from_u64(0x6750_0007);
        let initial=c::initial_state(ConstructorContext::new(7_u64))?;
        let deploy=make_deploy($root,name,initial.ledger_state.get_ref().clone(),&mut rng)?;
        let functional=Marker::default();
        let recorded=c::functional::$entry::Program::new($($arg),*)
            .execute_recorded(initial.into_circuit_context(deploy.address()),&functional)?;
        ($check)(&recorded.execution.result)?;
        let seed=Field::from(7_u64);
        if recorded.execution.context.private_state!=8
            || recorded.execution.private_transcript_outputs!=[AlignedValue::from(seed)]
            || *functional.0.borrow()!=[(7,seed)] {
            return Err("crypto functional witness/private transcript changed".into());
        }
        let expected_state=recorded.execution.context.query.state.get_ref().clone();
        if functional_crypto_contract::ledger_slots::stored.inspect(&expected_state)?!=seed {
            return Err("crypto functional witness ledger value changed".into());
        }
        let manual=check_generated_trace($root,name,recorded,($($arg,)*))?;
        let observed=pure_field_arguments::observed_state(&deploy);
        let ordinary=Marker::default();
        // The observed ordinary call is only the independently compared reference.
        let prepared=c::Contract::from(ordinary.clone()).recording()
            .$call(&observed,7_u64,$($arg),*)?
            .prepare(pure_field_arguments::verifier($root,name)?,Fr::from(0_u64))?;
        if format!("{manual:?}")!=format!("{prepared:?}") || *ordinary.0.borrow()!=[(7,seed)] {
            return Err("functional crypto differs from ordinary observed preparation".into());
        }
        check_transaction($root,name,deploy,manual,&mut rng,|state| {
            if state.data.get_ref()!=&expected_state { return Err("crypto proof ledger state changed".into()); }
            Ok(())
        })?;
        println!("functional crypto {} proved, verified and ledger-applied",name);
        Ok(())
    }};
}
fn hashing(root:&Path)->Result<(),Box<dyn Error>> {
    let seed=Field::from(7_u64);
    let opening=r::FixedBytes::new([3_u8;32]);
    prove_case!(root,hashing,hashing_call,[seed,opening],|value:&functional_crypto_contract::types::HashReceipt|->Result<(),Box<dyn Error>> {
        let encoded=r::FixedBytes::<32>::new(seed.as_le_bytes().try_into().expect("Field encoding has 32 bytes"));
        let digest=r::persistent_hash(encoded);
        let commitment=r::persistent_commit(encoded,opening);
        let degraded=r::degrade_to_transient(digest);
        let hashed=r::transient_hash(degraded);
        let transient=r::transient_commit(hashed,seed);
        if value.encoded!=encoded || value.digest!=digest || value.commitment!=commitment
            || value.degraded!=degraded || value.hashed!=hashed || value.transient!=transient
            || value.upgraded!=r::upgrade_from_transient(transient) {
            return Err("crypto hashing independent official intermediate mismatch".into());
        }
        Ok(())
    })
}
fn jubjub(root:&Path)->Result<(),Box<dyn Error>> {
    let seed=Field::from(7_u64);
    let point=r::ec_mul_generator(Field::from(5_u64))?;
    prove_case!(root,jubjub,jubjub_call,[seed,point],|value:&functional_crypto_contract::types::CurveReceipt|->Result<(),Box<dyn Error>> {
        let x=r::jubjub_point_x(point); let y=r::jubjub_point_y(point);
        let scalar=r::jubjub_scalar_from_native(seed);
        let generator=r::ec_mul_generator(scalar)?;
        let multiplied=r::ec_mul(point,scalar)?;
        let added=r::ec_add(generator,multiplied);
        let negated=r::ec_neg(added);
        if value.curve!=r::hash_to_curve(seed) || value.x!=x || value.y!=y
            || value.reconstructed!=r::construct_jubjub_point(x,y)? || value.scalar!=scalar
            || value.generator!=generator || value.multiplied!=multiplied || value.added!=added || value.negated!=negated {
            return Err("crypto Jubjub independent official intermediate mismatch".into());
        }
        Ok(())
    })
}
