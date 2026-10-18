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

use super::{
    Request,
    error::{CacheError, Result, require},
    identity::{self, Artifact},
    json,
    model::Model,
    policy::Policy,
    process, publication,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

const PREPARE_RESPONSE_FIELDS: &[&str] = &[
    "schema",
    "bundle_id",
    "derivation_id",
    "artifact_set_id",
    "format",
    "worker",
    "profile",
    "build_fingerprint",
    "ledger_revision",
    "descriptor",
    "export",
    "cache",
    "lock_wait_ms",
];

pub struct Local<'a> {
    policy: &'a Policy,
    cache: PathBuf,
}
impl<'a> Local<'a> {
    pub fn connect(policy: &'a Policy, cache: PathBuf) -> Result<Self> {
        let this = Self { policy, cache };
        let value = json::parse(&process::output(this.command().arg("profile"))?)?;
        fields(
            &value,
            &[
                "schema",
                "formats",
                "selectors",
                "profile",
                "ledger_revision",
                "build_fingerprint",
            ],
        )?;
        require(
            json::number(&value, "schema")? == 1
                && json::text(&value, "profile")? == policy.profile
                && json::text(&value, "ledger_revision")? == policy.ledger_revision
                && json::text(&value, "build_fingerprint")? == policy.build_fingerprint,
            "Forge profile/build differs from independent policy",
        )?;
        Ok(this)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(&self.policy.forge.path);
        c.arg("--profile")
            .arg("ledger8-v2")
            .arg("--cache-dir")
            .arg(&self.cache);
        c
    }
    pub fn prepare(&self, request: Request<'_>, model: &Model) -> Result<(bool, Vec<Artifact>)> {
        let name = identity::name(request.bytes);
        let response = json::parse(&process::output(
            self.command()
                .arg("--params-dir")
                .arg(&self.policy.parameter_directory)
                .arg("prepare")
                .arg(request.source)
                .arg("--name")
                .arg(&name),
        )?)?;
        fields(&response, PREPARE_RESPONSE_FIELDS)?;
        fields(
            &response["cache"],
            &[
                "hit",
                "elapsed_ms",
                "generation_ms",
                "estimated_generation_avoided_ms",
            ],
        )?;
        require(
            response["format"] == identity::FORMAT && response["worker"].is_null(),
            "Forge dialect/worker differs from qualified local profile",
        )?;
        let hit = response
            .pointer("/cache/hit")
            .and_then(Value::as_bool)
            .ok_or_else(|| CacheError::Refused("Forge cache outcome missing".into()))?;
        require(
            json::number(&response, "schema")? == 1
                && json::text(&response, "profile")? == identity::PROFILE
                && json::text(&response, "ledger_revision")? == identity::REVISION
                && json::text(&response, "build_fingerprint")? == self.policy.build_fingerprint,
            "Forge preparation profile/build changed",
        )?;
        let export = &response["export"];
        fields(export, &["base", "files", "lifetime", "descriptor"])?;
        require(
            export["descriptor"] == response["descriptor"],
            "export descriptor differs from response",
        )?;
        require(
            json::text(export, "lifetime")? == "persistent_until_owner_removal",
            "persistent export required",
        )?;
        let base = Path::new(json::text(export, "base")?);
        let manifest_path = base.join("manifest.json");
        publication::regular_inside(base, &manifest_path)?;
        let manifest = json::parse(&json::read(&manifest_path, identity::MAX_JSON)?)?;
        require(
            json::number(&manifest, "schema")? == 1,
            "Forge manifest schema differs",
        )?;
        for field in [
            "profile",
            "build_fingerprint",
            "ledger_revision",
            "bundle_id",
            "derivation_id",
            "artifact_set_id",
        ] {
            require(
                manifest.get(field).is_some() && manifest[field] == response[field],
                "Forge manifest and response identities differ",
            )?;
        }
        validate_descriptor(&response, &manifest, base, self.policy)?;
        require(
            json::text(&manifest, "name")? == name,
            "Forge manifest name differs from requested content name",
        )?;
        let source_sha = identity::digest(request.bytes);
        require(
            json::text(&response, "bundle_id")? == identity::bundle(&name, &source_sha)?,
            "Forge bundle identity differs",
        )?;
        require(
            json::number(&manifest, "k")? == u64::from(model.k)
                && json::text(&manifest, "parameter_digest")? == model.parameter_sha,
            "Forge parameters differ from actual verified consumer parameters",
        )?;
        require(
            json::text(&response, "derivation_id")?
                == identity::derivation(
                    &source_sha,
                    &self.policy.build_fingerprint,
                    model.k,
                    &model.parameter_sha,
                )?,
            "Forge derivation identity differs",
        )?;
        let artifacts: Vec<_> = manifest["artifacts"]
            .as_array()
            .ok_or_else(|| CacheError::Refused("Forge artifacts missing".into()))?
            .iter()
            .map(Artifact::from_record)
            .collect::<Result<_>>()?;
        require(
            json::text(&response, "artifact_set_id")? == identity::artifact_set(&artifacts)?,
            "Forge artifact-set identity differs",
        )?;
        let source = Artifact::from_record(&manifest["source"])?;
        require(
            source.kind == "source"
                && source.digest == source_sha
                && source.size == request.bytes.len() as u64,
            "Forge retained source differs",
        )?;
        let mut declared: BTreeMap<_, _> = artifacts
            .iter()
            .cloned()
            .map(|a| (a.kind.clone(), a))
            .collect();
        declared.insert("source".into(), source);
        let rows = export["files"]
            .as_array()
            .ok_or_else(|| CacheError::Refused("Forge exported files missing".into()))?;
        require(
            rows.len() == 4,
            "Forge export must contain exactly four artifacts",
        )?;
        for row in rows {
            fields(row, &["kind", "path", "sha256", "bytes"])?;
            let kind = json::text(row, "kind")?;
            let expected = declared
                .remove(kind)
                .ok_or_else(|| CacheError::Refused("unknown or duplicate exported kind".into()))?;
            require(
                json::text(row, "sha256")? == expected.digest
                    && json::number(row, "bytes")? == expected.size,
                "Forge export and manifest digest/length differ",
            )?;
            let path = Path::new(json::text(row, "path")?);
            publication::regular_inside(base, path)?;
            let target = request
                .destination
                .join(format!("artifact.{}", expected.extension()));
            publication::copy_verified(path, &target, &expected)?;
        }
        require(declared.is_empty(), "incomplete Forge export")?;
        require(
            publication::hash_file(
                &request.destination.join("artifact.bzkir"),
                identity::MAX_ARTIFACT,
            )? == model.binary_sha,
            "Forge binary differs from fresh actual serialization",
        )?;
        Ok((hit, artifacts))
    }
}

fn fields(value: &Value, expected: &[&str]) -> Result<()> {
    let actual = value
        .as_object()
        .ok_or_else(|| CacheError::Refused("JSON object required".into()))?;
    require(
        actual.len() == expected.len() && expected.iter().all(|k| actual.contains_key(*k)),
        "unknown/missing Forge descriptor fields",
    )
}
fn validate_descriptor(
    response: &Value,
    manifest: &Value,
    base: &Path,
    policy: &Policy,
) -> Result<()> {
    fields(
        manifest,
        &[
            "schema",
            "profile",
            "build_fingerprint",
            "ledger_revision",
            "bundle_id",
            "derivation_id",
            "artifact_set_id",
            "name",
            "k",
            "parameter_digest",
            "source",
            "artifacts",
        ],
    )?;
    for row in std::iter::once(&manifest["source"]).chain(
        manifest["artifacts"]
            .as_array()
            .ok_or_else(|| CacheError::Refused("artifact array required".into()))?,
    ) {
        fields(row, &["kind", "digest", "size"])?;
    }
    let d = &response["descriptor"];
    fields(
        d,
        &[
            "schema",
            "record",
            "materialized",
            "input_format",
            "input_version",
            "keygen_recipe",
            "serialization",
        ],
    )?;
    fields(&d["input_version"], &["major", "minor"])?;
    require(
        d["schema"] == 1
            && d["materialized"] == true
            && d["input_version"] == serde_json::json!({"major":2,"minor":0})
            && d["input_format"] == identity::FORMAT,
        "Forge descriptor dialect/materialization differs",
    )?;
    require(
        d["serialization"] == serde_json::to_value(&policy.serialization)?
            && d["keygen_recipe"] == policy.keygen_recipe,
        "Forge descriptor recipe/serialization differs",
    )?;
    let mut record = manifest.clone();
    record["generation_ms"] = Value::Null;
    require(
        d["record"] == record,
        "Forge descriptor record differs from checked manifest",
    )?;
    let path = base.join("descriptor.json");
    publication::regular_inside(base, &path)?;
    require(
        json::parse(&json::read(&path, identity::MAX_JSON)?)? == *d,
        "Forge descriptor sidecar differs",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modern_prepare_roster_requires_lock_wait_and_refuses_unknown_fields() {
        // Mirrors the actual v1 prepare response (application::ensure_with_profile),
        // not its descriptor or export alone. Timing is not cache identity.
        let response = serde_json::json!({"schema":1,"bundle_id":null,"derivation_id":null,
            "artifact_set_id":null,"format":null,"worker":null,"profile":null,
            "build_fingerprint":null,"ledger_revision":null,"descriptor":null,
            "export":null,"cache":null,"lock_wait_ms":0.125});
        fields(&response, PREPARE_RESPONSE_FIELDS).unwrap();
        let mut missing = response.clone();
        missing.as_object_mut().unwrap().remove("lock_wait_ms");
        assert!(fields(&missing, PREPARE_RESPONSE_FIELDS).is_err());
        let mut unknown = response;
        unknown["unreviewed_field"] = Value::Null;
        assert!(fields(&unknown, PREPARE_RESPONSE_FIELDS).is_err());
    }
}
