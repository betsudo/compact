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

use compact_rust_backend::ir::{Contract, SCHEMA_VERSION};
use compact_rust_backend::transient_provenance::{Limits, advise, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!("transient-provenance-source-ir.json")).unwrap()
}

#[test]
fn direct_analysis_rejects_other_schema_versions_without_findings() {
    let valid = source_contract();
    assert!(!analyze(&valid, Limits::default()).findings.is_empty());
    for schema in [SCHEMA_VERSION - 1, SCHEMA_VERSION + 1] {
        let mut contract = valid.clone();
        contract.schema_version = schema;
        let report = analyze(&contract, Limits::default());
        assert_eq!(report, analyze(&contract, Limits::default()));
        assert!(report.findings.is_empty());
        assert!(report.helper_summaries.is_empty());
        assert!(!report.truncated);
        assert_eq!(report.unavailable.len(), 1);
        assert_eq!(report.unavailable[0].circuit, "contract");
        assert_eq!(report.unavailable[0].path, "schema_version");
        assert_eq!(
            report.unavailable[0].reason,
            format!("unsupported typed IR schema {schema}; expected {SCHEMA_VERSION}")
        );
        let advisory = advise(&report, &[], 0);
        assert!(advisory.findings.is_empty());
        assert_eq!(advisory.unavailable, report.unavailable);
    }
}
