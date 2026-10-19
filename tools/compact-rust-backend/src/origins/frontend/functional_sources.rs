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

//! Original occurrence binding, explicitly weaker than consumed-byte attestation.
use crate::{
    ir::{Contract, SourceLocation, StatefulCircuit},
    origins::{
        binding::BoundOccurrences, diagnostic::Capture, source_text::SourceSnapshots,
        transport::UntrustedTransport,
    },
    source_diagnostics::{Owner, RefusalSite},
};
use std::{collections::HashMap, path::Path};

pub(super) struct RetainedSources<'ir> {
    roots: HashMap<usize, SourceLocation>,
    original: std::marker::PhantomData<&'ir Contract>,
}
impl<'ir> RetainedSources<'ir> {
    pub(super) fn new(
        contract: &'ir Contract,
        bound: Option<&BoundOccurrences<'ir>>,
        transport: Option<&UntrustedTransport>,
        snapshots: Option<&SourceSnapshots>,
        cwd: &Path,
    ) -> Self {
        let roots = contract
            .stateful_circuits
            .iter()
            .filter_map(|original| {
                let mut capture = Capture::default();
                capture.observe_site(
                    RefusalSite::node(Owner::Stateful(original)),
                    bound,
                    transport,
                );
                let diagnostic = capture.expand(transport, snapshots, cwd)?;
                // Physical original occurrence and current snapshot correspondence
                // are both required. This does not prove consumed-source bytes.
                if !diagnostic.range_matches_post_invocation_snapshot {
                    return None;
                }
                let source = SourceLocation {
                    file: diagnostic.descriptor_path,
                    line: usize::try_from(diagnostic.start.line).ok()?,
                    column: usize::try_from(diagnostic.start.column).ok()?,
                };
                Some((std::ptr::from_ref(original) as usize, source))
            })
            .collect();
        Self {
            roots,
            original: std::marker::PhantomData,
        }
    }
}
impl crate::recorded::FunctionalSources for RetainedSources<'_> {
    fn root<'a>(&'a self, original: &StatefulCircuit) -> Option<&'a SourceLocation> {
        self.roots.get(&(std::ptr::from_ref(original) as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recorded::FunctionalSources;

    fn root() -> StatefulCircuit {
        serde_json::from_value(serde_json::json!({
            "name": "update", "actions": [], "result": {"kind": "unit"}
        }))
        .unwrap()
    }

    #[test]
    fn cloned_candidate_cannot_borrow_original_root_binding() {
        let original = root();
        let candidate = original.clone();
        let source = SourceLocation {
            file: "retained.compact".into(),
            line: 3,
            column: 7,
        };
        let retained = RetainedSources {
            roots: HashMap::from([(std::ptr::from_ref(&original) as usize, source.clone())]),
            original: std::marker::PhantomData,
        };
        assert_eq!(retained.root(&original), Some(&source));
        assert_eq!(retained.root(&candidate), None);
    }

    #[test]
    fn missing_bound_metadata_never_uses_candidate_source_annotation() {
        let mut original = root();
        original.source = Some(SourceLocation {
            file: "candidate.compact".into(),
            line: 1,
            column: 1,
        });
        let contract = Contract {
            schema_version: crate::SCHEMA_VERSION,
            type_aliases: vec![],
            ledger_fields: vec![],
            constructor: None,
            witnesses: vec![],
            circuits: vec![],
            stateful_circuits: vec![original],
        };
        let retained = RetainedSources::new(&contract, None, None, None, Path::new("."));
        assert_eq!(retained.root(&contract.stateful_circuits[0]), None);
    }
}
