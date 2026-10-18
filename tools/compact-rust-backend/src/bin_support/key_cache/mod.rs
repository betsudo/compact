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

//! Shared frontend key preparation: fresh inputs/models; no cached proof verdicts.
mod error;
mod identity;
mod json;
mod local;
mod model;
mod paths;
mod policy;
mod process;
mod publication;
mod report;
use error::{CacheError, Result, require};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    schema: u8,
    inputs: PathBuf,
    keys: PathBuf,
    policy: Option<PathBuf>,
    cache: Option<PathBuf>,
    final_output: Option<PathBuf>,
    #[serde(default)]
    report: bool,
}
pub struct Request<'a> {
    pub source: &'a Path,
    pub bytes: &'a [u8],
    pub destination: &'a Path,
}
/// Owned private work area. No fixtures/exports are shared or modified.
pub struct Stage(PathBuf);
impl Stage {
    pub fn new(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".compact-keys-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err(CacheError::Refused(
            "cannot allocate private key stage".into(),
        ))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn roster(directory: &Path) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    require(
        directory.is_dir() && !directory.is_symlink(),
        "regular ZKIR directory required",
    )?;
    for row in fs::read_dir(directory)? {
        let path = row?.path();
        if path.extension().is_some_and(|s| s == "bzkir") {
            require(
                path.with_extension("zkir").is_file(),
                "binary-only circuit refused",
            )?;
        }
        if !path.extension().is_some_and(|s| s == "zkir") {
            continue;
        }
        let bytes = json::read(&path, identity::MAX_JSON)?;
        let value = json::parse(&bytes)?;
        require(
            value
                .pointer("/version/major")
                .and_then(serde_json::Value::as_u64)
                == Some(2)
                && value
                    .pointer("/version/minor")
                    .and_then(serde_json::Value::as_u64)
                    == Some(0),
            "local cache requires ZKIRv2.0",
        )?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| CacheError::Refused("circuit filename must be UTF-8".into()))?;
        require(!name.is_empty(), "empty circuit filename")?;
        result.insert(name.into(), identity::digest(&bytes));
    }
    Ok(result)
}
pub fn run() -> Result<()> {
    require(
        std::env::args_os().len() == 1,
        "helper accepts only its internal scoped request",
    )?;
    let request = std::env::var("COMPACT_ZKIR_CACHE_REQUEST")
        .map_err(|_| CacheError::Refused("internal request missing".into()))?;
    require(
        request.len() as u64 <= identity::MAX_JSON,
        "request too large",
    )?;
    let options: Options = serde_json::from_value(json::parse(request.as_bytes())?)?;
    prepare(options)
}
fn prepare(options: Options) -> Result<()> {
    require(options.schema == 1, "unsupported internal request schema")?;
    // No provider, policy, parameters or output access for empty circuit sets.
    if !options.inputs.try_exists()? {
        return report::emit(options.report, &report::Report::new());
    }
    require(
        !options.inputs.is_symlink(),
        "ZKIR directory symlink refused",
    )?;
    let inputs = options.inputs.canonicalize()?;
    let before = roster(&inputs)?;
    if before.is_empty() {
        return report::emit(options.report, &report::Report::new());
    }
    let policy_path = options.policy.as_ref().ok_or_else(|| {
        CacheError::Refused("--zkir-cache-policy required for local key preparation".into())
    })?;
    let mut protected = Vec::new();
    if let Some(output) = &options.final_output {
        paths::require_disjoint(output, &[policy_path, &inputs, &options.keys])?;
        for path in [policy_path, &inputs, &options.keys] {
            protected.push((path.to_path_buf(), paths::prospective(path)?));
        }
    }
    let policy_bytes = json::read(policy_path, identity::MAX_JSON)?;
    let policy = policy::Policy::parse(&policy_bytes)?;
    if let Some(output) = &options.final_output {
        paths::require_disjoint(
            output,
            &[
                &policy.parameter_directory,
                &policy.zkir.path,
                &policy.forge.path,
            ],
        )?;
        for path in [
            &policy.parameter_directory,
            &policy.zkir.path,
            &policy.forge.path,
        ] {
            protected.push((path.to_path_buf(), paths::prospective(path)?));
        }
    }
    policy::validate_executable(&policy.zkir)?;
    let names: Vec<_> = before.keys().cloned().collect();
    publication::validate_targets(&inputs, &options.keys, &names)?;
    let mut provider = None;
    let mut forge_available = None;
    let mut store_identity: Option<(PathBuf, PathBuf)> = None;
    let stage = Stage::new(
        inputs
            .parent()
            .ok_or_else(|| CacheError::Refused("ZKIR has no parent".into()))?,
    )?;
    let mut parameters = BTreeMap::new();
    let mut prepared = BTreeMap::new();
    let mut receipt = report::Report::new();
    let unchanged = |parameters: &BTreeMap<u8, String>,
                     forge_available: Option<bool>,
                     store_identity: &Option<(PathBuf, PathBuf)>|
     -> Result<()> {
        require(
            roster(&inputs)? == before,
            "current circuit roster/bytes changed",
        )?;
        require(
            json::read(policy_path, identity::MAX_JSON)? == policy_bytes,
            "qualification policy changed",
        )?;
        if let Some(output) = &options.final_output {
            for (original, canonical) in &protected {
                paths::require_disjoint(output, &[original])?;
                require(
                    paths::prospective(original)? == *canonical,
                    "protected compiler input alias changed",
                )?;
            }
        }
        policy::validate_executable(&policy.zkir)?;
        if let Some((original, canonical)) = store_identity {
            paths::require_disjoint(
                original,
                &[
                    &inputs,
                    &options.keys,
                    stage.path(),
                    inputs
                        .parent()
                        .ok_or_else(|| CacheError::Refused("ZKIR has no parent".into()))?,
                ],
            )?;
            require(
                paths::prospective(original)? == *canonical,
                "Forge store alias changed during preparation",
            )?;
        }
        if let Some(available) = forge_available {
            policy.validate_tools(available)?;
        }
        for (k, sha) in parameters {
            require(
                policy.parameter(*k)? == *sha,
                "parameters changed during preparation",
            )?;
        }
        Ok(())
    };
    for (name, sha) in &before {
        let path = inputs.join(format!("{name}.zkir"));
        let source = json::read(&path, identity::MAX_JSON)?;
        require(
            identity::digest(&source) == *sha,
            "current circuit changed before model",
        )?;
        let destination = stage.path().join(name);
        fs::create_dir(&destination)?;
        let model_dir = destination.join("model");
        fs::create_dir(&model_dir)?;
        let model = model::fresh(&policy, &source, &model_dir)
            .map_err(|error| CacheError::Refused(format!("circuit {name}: {error}")))?;
        parameters.insert(model.k, model.parameter_sha.clone());
        let use_forge = policy.uses_forge(model.k);
        // Tiny cases skip Forge execution/profile/contents and all store access.
        // Earlier path-only safety guards can still resolve the configured tool.
        if use_forge && forge_available.is_none() {
            let store = paths::effective_cache(options.cache.as_deref())?;
            paths::require_disjoint(
                &store,
                &[
                    &inputs,
                    &options.keys,
                    stage.path(),
                    inputs
                        .parent()
                        .ok_or_else(|| CacheError::Refused("ZKIR has no parent".into()))?,
                ],
            )?;
            let canonical = paths::prospective(&store)?;
            store_identity = Some((store, canonical.clone()));
            let available = policy.forge.path.try_exists()?;
            require(
                available || matches!(policy.unavailable, policy::Unavailable::Keygen),
                "qualified Forge unavailable; policy refuses fallback",
            )?;
            policy.validate_tools(available)?;
            if available {
                provider = Some(local::Local::connect(&policy, canonical)?);
            }
            forge_available = Some(available);
        }
        let acquisition = if use_forge && let Some(provider) = &provider {
            let (hit, artifacts) = provider
                .prepare(
                    Request {
                        source: &path,
                        bytes: &source,
                        destination: &destination,
                    },
                    &model,
                )
                .map_err(|error| CacheError::Refused(format!("circuit {name}: {error}")))?;
            prepared.insert(name.clone(), artifacts);
            eprintln!(
                "ZKIR local cache {}: {name}; fresh k={} model checked",
                if hit { "hit" } else { "miss" },
                model.k
            );
            if hit {
                report::Acquisition::ForgeHit
            } else {
                report::Acquisition::ForgeMiss
            }
        } else {
            eprintln!(
                "ZKIR ordinary key generation: {name}; {}",
                if use_forge {
                    "explicit unavailable policy"
                } else {
                    "automatic tiny-circuit bypass"
                }
            );
            prepared.insert(
                name.clone(),
                fallback(&policy, &source, &destination, &model)
                    .map_err(|error| CacheError::Refused(format!("circuit {name}: {error}")))?,
            );
            if use_forge {
                report::Acquisition::UnavailableKeygen
            } else {
                report::Acquisition::AutoBypass
            }
        };
        if options.report {
            receipt.circuits.push(report::Circuit {
                name: name.clone(),
                source_sha256: sha.clone(),
                k: model.k,
                rows: model.rows,
                parameter_sha256: model.parameter_sha.clone(),
                acquisition_path: acquisition,
                artifacts: prepared
                    .get(name)
                    .ok_or_else(|| {
                        CacheError::Refused("prepared circuit artifact record missing".into())
                    })?
                    .clone(),
            });
        }
        unchanged(&parameters, forge_available, &store_identity)?;
    }
    unchanged(&parameters, forge_available, &store_identity)?;
    publication::publish(stage.path(), &inputs, &options.keys, &prepared)?;
    unchanged(&parameters, forge_available, &store_identity)?;
    report::emit(options.report, &receipt)
}
fn fallback(
    policy: &policy::Policy,
    source: &[u8],
    destination: &Path,
    model: &model::Model,
) -> Result<Vec<identity::Artifact>> {
    let inputs = destination.join("uncached");
    let keys = destination.join("uncached-keys");
    fs::create_dir(&inputs)?;
    fs::write(inputs.join("current.zkir"), source)?;
    process::model_output(
        Command::new(&policy.zkir.path)
            .arg("compile-many")
            .arg(&inputs)
            .arg(&keys)
            .env("MIDNIGHT_PP", &policy.parameter_directory),
    )?;
    require(
        json::read(&inputs.join("current.zkir"), identity::MAX_JSON)? == source,
        "uncached source changed",
    )?;
    require(
        publication::hash_file(&inputs.join("current.bzkir"), identity::MAX_ARTIFACT)?
            == model.binary_sha,
        "uncached binary differs from fresh model",
    )?;
    let mut artifacts = Vec::new();
    for (kind, path) in [
        ("binary_ir", inputs.join("current.bzkir")),
        ("prover", keys.join("current.prover")),
        ("verifier", keys.join("current.verifier")),
    ] {
        let artifact = identity::Artifact {
            kind: kind.into(),
            digest: publication::hash_file(&path, identity::MAX_ARTIFACT)?,
            size: fs::symlink_metadata(&path)?.len(),
        };
        require(
            artifact.size > 0 && artifact.size <= identity::MAX_ARTIFACT,
            "uncached artifact size invalid",
        )?;
        publication::copy_verified(
            &path,
            &destination.join(format!("artifact.{}", artifact.extension())),
            &artifact,
        )?;
        artifacts.push(artifact);
    }
    Ok(artifacts)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_roster_never_reads_policy() {
        let s = Stage::new(&std::env::temp_dir()).unwrap();
        let inputs = s.path().join("zkir");
        fs::create_dir(&inputs).unwrap();
        prepare(Options {
            schema: 1,
            inputs,
            keys: s.path().join("keys"),
            policy: Some(s.path().join("absent-policy")),
            cache: None,
            final_output: None,
            report: false,
        })
        .unwrap();
        assert!(!s.path().join("keys").exists());
    }
    #[test]
    fn internal_request_rejects_unknown_or_duplicate_fields() {
        for s in [
            r#"{"schema":1,"inputs":"i","keys":"k","policy":null,"cache":null,"remote":true}"#,
            r#"{"schema":1,"schema":1}"#,
        ] {
            assert!(
                json::parse(s.as_bytes())
                    .and_then(|v| Ok(serde_json::from_value::<Options>(v)?))
                    .is_err()
            );
        }
    }
    #[test]
    fn roster_detects_content_and_membership() {
        let s = Stage::new(&std::env::temp_dir()).unwrap();
        let a = s.path().join("a.zkir");
        fs::write(&a, br#"{"version":{"major":2,"minor":0}}"#).unwrap();
        let before = roster(s.path()).unwrap();
        fs::write(&a, br#"{"version":{"major":2,"minor":0},"changed":true}"#).unwrap();
        assert_ne!(before, roster(s.path()).unwrap());
        fs::remove_file(a).unwrap();
        assert_ne!(before, roster(s.path()).unwrap());
    }
    #[cfg(unix)]
    #[test]
    fn automatic_tiny_bypasses_missing_forge_but_force_refuses() {
        use std::os::unix::fs::PermissionsExt;
        let stage = Stage::new(&std::env::temp_dir()).unwrap();
        let inputs = stage.path().join("inputs");
        fs::create_dir(&inputs).unwrap();
        fs::write(
            inputs.join("circuit.zkir"),
            br#"{"version":{"major":2,"minor":0}}"#,
        )
        .unwrap();
        let tool = stage.path().join("actual-zkir");
        fs::write(&tool,br##"#!/bin/sh
case "$1" in
--version) printf 'zkir 2.1.1\n';;
mock-compile) printf 'binary' > "${2%.zkir}.bzkir"; printf '(k=8, rows=1)\n';;
compile-many) mkdir "$3"; printf 'binary' > "$2/current.bzkir"; printf 'prover' > "$3/current.prover"; printf 'verifier' > "$3/current.verifier";;
*) exit 9;;
esac
"##).unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            stage.path().join("bls_midnight_2p8"),
            b"official-test-parameters",
        )
        .unwrap();
        let policy_path = stage.path().join("policy.json");
        let mut value = serde_json::json!({"schema":1,"forge":{"path":stage.path().join("absent-forge"),"sha256":"a".repeat(64)},"zkir":{"path":tool,"sha256":publication::hash_file(&tool,identity::MAX_ARTIFACT).unwrap()},"profile":identity::PROFILE,"ledger_revision":identity::REVISION,"build_fingerprint":"b".repeat(64),"serialization":policy::serialization(),"keygen_recipe":"library-keygen/default-options/no-rng","parameter_directory":stage.path(),"parameters":{"8":identity::digest(b"official-test-parameters")},"unavailable":"refuse"});
        fs::write(&policy_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let options = || Options {
            schema: 1,
            inputs: inputs.clone(),
            keys: stage.path().join("keys"),
            policy: Some(policy_path.clone()),
            cache: Some(stage.path().join("must-not-exist")),
            final_output: None,
            report: false,
        };
        let p = policy::Policy::read(&policy_path).unwrap();
        assert!(!p.uses_forge(8));
        assert!(p.uses_forge(9));
        prepare(options()).unwrap();
        assert_eq!(
            fs::read(stage.path().join("keys/circuit.prover")).unwrap(),
            b"prover"
        );
        assert!(!stage.path().join("must-not-exist").exists());
        fs::remove_dir_all(stage.path().join("keys")).unwrap();
        fs::remove_file(inputs.join("circuit.bzkir")).unwrap();
        value["selection"] = serde_json::json!("force");
        fs::write(&policy_path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            prepare(options())
                .unwrap_err()
                .to_string()
                .contains("must be disjoint")
        );
        assert!(!stage.path().join("keys").exists());
        let mut disjoint = options();
        disjoint.cache = Some(stage.path().with_extension("independent-cache"));
        assert!(
            prepare(disjoint)
                .unwrap_err()
                .to_string()
                .contains("policy refuses fallback")
        );
        value["selection"] = serde_json::json!("auto");
        value["tiny_max_k"] = serde_json::json!(0);
        fs::write(&policy_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let zero = policy::Policy::read(&policy_path).unwrap();
        assert!(!zero.uses_forge(0));
        assert!(zero.uses_forge(1));
        value["tiny_max_k"] = serde_json::json!(255);
        fs::write(&policy_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let all = policy::Policy::read(&policy_path).unwrap();
        assert!(!all.uses_forge(255));
    }
    #[cfg(unix)]
    #[test]
    fn static_bridge_treats_paths_as_data_and_never_falls_through() {
        use std::os::unix::fs::PermissionsExt;
        let stage = Stage::new(&std::env::temp_dir()).unwrap();
        let directory = stage.path().join("bin ' $ ; `literal`");
        fs::create_dir(&directory).unwrap();
        let helper = directory.join("compact-zkir-cache");
        fs::write(
            &helper,
            b"#!/bin/sh\nprintf '%s' \"$COMPACT_ZKIR_CACHE_REQUEST\"\n",
        )
        .unwrap();
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
        let fixed = "if [ -n \"$COMPACT_ZKIR_CACHE_HELPER_DIRECTORY\" ]; then exec \"$COMPACT_ZKIR_CACHE_HELPER_DIRECTORY/compact-zkir-cache\"; else exec compact-zkir-cache; fi";
        let invoke = || {
            Command::new("/bin/sh")
                .arg("-c")
                .arg(fixed)
                .env("COMPACT_ZKIR_CACHE_HELPER_DIRECTORY", &directory)
                .env("COMPACT_ZKIR_CACHE_REQUEST", "literal ' $ ; `data`")
                .output()
                .unwrap()
        };
        let output = invoke();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"literal ' $ ; `data`");
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(!invoke().status.success());
        fs::remove_file(&helper).unwrap();
        assert!(!invoke().status.success());
    }
    #[cfg(unix)]
    #[test]
    fn overlapping_helper_store_refuses_before_forge_profile() {
        use std::os::unix::fs::PermissionsExt;
        let stage = Stage::new(&std::env::temp_dir()).unwrap();
        let inputs = stage.path().join("zkir");
        fs::create_dir(&inputs).unwrap();
        fs::write(
            inputs.join("circuit.zkir"),
            br#"{"version":{"major":2,"minor":0}}"#,
        )
        .unwrap();
        let zkir = stage.path().join("zkir-tool");
        fs::write(&zkir,b"#!/bin/sh\ncase \"$1\" in\n--version) printf 'zkir 2.1.1\\n';;\nmock-compile) printf 'binary' > \"${2%.zkir}.bzkir\"; printf '(k=9, rows=1)\\n';;\n*) exit 8;;\nesac\n").unwrap();
        fs::set_permissions(&zkir, fs::Permissions::from_mode(0o700)).unwrap();
        let forge = stage.path().join("forge");
        fs::write(
            &forge,
            b"#!/bin/sh\nprintf invoked > \"${0%/*}/ForgeNeverInvoked\"\nexit 9\n",
        )
        .unwrap();
        fs::set_permissions(&forge, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(stage.path().join("bls_midnight_2p9"), b"parameters").unwrap();
        let policy_path = stage.path().join("policy.json");
        let value = serde_json::json!({"schema":1,"forge":{"path":forge,"sha256":publication::hash_file(&forge,identity::MAX_ARTIFACT).unwrap()},"zkir":{"path":zkir,"sha256":publication::hash_file(&zkir,identity::MAX_ARTIFACT).unwrap()},"profile":identity::PROFILE,"ledger_revision":identity::REVISION,"build_fingerprint":"b".repeat(64),"serialization":policy::serialization(),"keygen_recipe":"library-keygen/default-options/no-rng","parameter_directory":stage.path(),"parameters":{"9":identity::digest(b"parameters")},"unavailable":"refuse"});
        fs::write(&policy_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let marker = stage.path().join("ForgeNeverInvoked");
        assert!(!marker.exists());
        for cache in [
            inputs.clone(),
            inputs.join("missing/cache"),
            stage.path().join("keys"),
            stage.path().to_path_buf(),
        ] {
            let error = prepare(Options {
                schema: 1,
                inputs: inputs.clone(),
                keys: stage.path().join("keys"),
                policy: Some(policy_path.clone()),
                cache: Some(cache),
                final_output: None,
                report: false,
            })
            .unwrap_err();
            assert!(error.to_string().contains("must be disjoint"));
            assert!(!marker.exists());
            assert!(!stage.path().join("keys").exists());
            assert!(!inputs.join("circuit.bzkir").exists());
            assert_eq!(
                fs::read(inputs.join("circuit.zkir")).unwrap(),
                br#"{"version":{"major":2,"minor":0}}"#
            );
        }
    }
    #[cfg(unix)]
    #[test]
    fn final_output_cannot_delete_policy_parameters_or_pinned_tools() {
        use std::os::unix::fs::PermissionsExt;
        for inside in ["policy", "parameters", "zkir", "forge"] {
            let stage = Stage::new(&std::env::temp_dir()).unwrap();
            let output = stage.path().join("existing output");
            fs::create_dir(&output).unwrap();
            fs::write(output.join("old-output"), b"preserved").unwrap();
            let location = |kind: &str, name: &str| {
                if inside == kind {
                    output.join(name)
                } else {
                    stage.path().join(name)
                }
            };
            let policy_path = location("policy", "policy.json");
            let parameters = location("parameters", "parameters");
            fs::create_dir(&parameters).unwrap();
            let parameter_path = parameters.join("bls_midnight_2p8");
            fs::write(&parameter_path, b"pinned params").unwrap();
            let tool_bytes = b"#!/bin/sh\nprintf invoked > \"${0%/*}/ToolNeverInvoked\"\nexit 9\n";
            let zkir = location("zkir", "actual-zkir");
            let forge = location("forge", "forge");
            for tool in [&zkir, &forge] {
                fs::write(tool, tool_bytes).unwrap();
                fs::set_permissions(tool, fs::Permissions::from_mode(0o700)).unwrap();
            }
            let value = serde_json::json!({"schema":1,"forge":{"path":forge,"sha256":identity::digest(tool_bytes)},"zkir":{"path":zkir,"sha256":identity::digest(tool_bytes)},"profile":identity::PROFILE,"ledger_revision":identity::REVISION,"build_fingerprint":"b".repeat(64),"serialization":policy::serialization(),"keygen_recipe":"library-keygen/default-options/no-rng","parameter_directory":parameters,"parameters":{"8":identity::digest(b"pinned params")},"unavailable":"refuse"});
            let policy_bytes = serde_json::to_vec(&value).unwrap();
            fs::write(&policy_path, &policy_bytes).unwrap();
            let inputs = stage.path().join("zkir-inputs");
            fs::create_dir(&inputs).unwrap();
            fs::write(
                inputs.join("tiny.zkir"),
                br#"{"version":{"major":2,"minor":0}}"#,
            )
            .unwrap();
            let error = prepare(Options {
                schema: 1,
                inputs: inputs.clone(),
                keys: stage.path().join("keys"),
                policy: Some(policy_path.clone()),
                cache: None,
                final_output: Some(output.clone()),
                report: false,
            })
            .unwrap_err();
            assert!(
                error.to_string().contains("must be disjoint"),
                "{inside}: {error}"
            );
            assert_eq!(fs::read(&policy_path).unwrap(), policy_bytes);
            assert_eq!(fs::read(parameter_path).unwrap(), b"pinned params");
            for tool in [&zkir, &forge] {
                assert_eq!(fs::read(tool).unwrap(), tool_bytes);
            }
            assert_eq!(fs::read(output.join("old-output")).unwrap(), b"preserved");
            assert!(!output.join("ToolNeverInvoked").exists());
            assert!(!stage.path().join("ToolNeverInvoked").exists());
            assert!(!stage.path().join("keys").exists());
            assert!(!inputs.join("tiny.bzkir").exists());
        }
    }
}
