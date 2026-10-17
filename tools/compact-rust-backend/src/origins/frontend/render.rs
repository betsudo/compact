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

//! Native rendering runs once; metadata publication status is independently held.
use super::{FrontendArtifacts, InvocationFingerprint, digest};
use crate::origins::{
    binding::{self, BoundOccurrences, Owner, Slot},
    diagnostic::SourceDiagnostic,
    native_observer::NativeObserver,
    recording::{Recorder, RecordingDiagnostic},
    source_text::SourceSnapshots,
    status::{self, BindingSummary, MetadataProblem, MetadataState, MetadataStatus},
    transport::{UntrustedTransport, model::Extent},
};
use crate::{RenderError, RenderedContract, ir::Contract};
use syn::visit_mut::VisitMut;
#[path = "functional_sources.rs"]
mod functional_sources;

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderOptions {
    pub apply_frontend_proof_metadata: bool,
    pub require_recording: bool,
}

#[derive(Debug)]
pub enum NativeFailureCause {
    IrDecoding(String),
    Render(RenderError),
    ProofMetadata(String),
    RequireRecording(Vec<RecordingRequirement>),
}

#[derive(Debug)]
pub struct RecordingRequirement {
    pub circuit: String,
    pub missing_api: &'static str,
    pub code: String,
    pub path: String,
    pub detail: String,
}

#[derive(Debug)]
pub struct NativeCompileFailure {
    pub cause: NativeFailureCause,
    pub metadata: MetadataStatus,
    pub diagnostic: Option<Box<SourceDiagnostic>>,
    pub recording_diagnostics: Vec<RecordingDiagnostic>,
}
impl NativeCompileFailure {
    /// Constructed only after the failed native/metadata phase has returned.
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
impl std::fmt::Display for NativeCompileFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            NativeFailureCause::Render(error) => write!(f, "{error}"),
            other => write!(f, "native compilation: {other:?}"),
        }
    }
}
impl std::error::Error for NativeCompileFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            NativeFailureCause::Render(error) => Some(error),
            _ => None,
        }
    }
}

/// Errors of the new opt-in surface, without extending the existing exhaustive
/// NativeFailureCause enum used by default callers.
#[derive(Debug)]
pub enum FunctionalCompileFailure {
    Native(Box<NativeCompileFailure>),
    Projection(String),
}
impl From<Box<NativeCompileFailure>> for FunctionalCompileFailure {
    fn from(error: Box<NativeCompileFailure>) -> Self {
        Self::Native(error)
    }
}
impl std::fmt::Display for FunctionalCompileFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(error) => write!(f, "{error}"),
            Self::Projection(detail) => write!(f, "functional prototype: {detail}"),
        }
    }
}
impl std::error::Error for FunctionalCompileFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native(error) => Some(error.as_ref()),
            Self::Projection(_) => None,
        }
    }
}

pub struct FrontendRender {
    rendered: RenderedContract,
    metadata: MetadataStatus,
    fingerprint: InvocationFingerprint,
    recording_diagnostics: Vec<RecordingDiagnostic>,
}
impl FrontendRender {
    pub fn recording_diagnostics(&self) -> &[RecordingDiagnostic] {
        &self.recording_diagnostics
    }
    pub fn rendered(&self) -> &RenderedContract {
        &self.rendered
    }
    pub fn metadata(&self) -> &MetadataStatus {
        &self.metadata
    }
    pub fn fingerprint(&self) -> &InvocationFingerprint {
        &self.fingerprint
    }
    /// Separate publication check, after semantic and selected recording gates.
    /// Missing/budget metadata is a successful unavailable outcome.
    pub fn finish_metadata(&self) -> Result<(), MetadataProblem> {
        match &self.metadata {
            MetadataStatus::Invalid(problem) => Err(problem.clone()),
            _ => Ok(()),
        }
    }
}

pub(super) fn render(
    artifacts: &FrontendArtifacts,
    options: RenderOptions,
) -> Result<FrontendRender, Box<NativeCompileFailure>> {
    render_selected(artifacts, options, None)
}

pub(super) fn render_functional(
    artifacts: &FrontendArtifacts,
    options: RenderOptions,
) -> Result<FrontendRender, FunctionalCompileFailure> {
    render_selected(
        artifacts,
        options,
        Some(FunctionalCompileFailure::Projection),
    )
}

fn render_selected<E: From<Box<NativeCompileFailure>>>(
    artifacts: &FrontendArtifacts,
    options: RenderOptions,
    functional: Option<fn(String) -> E>,
) -> Result<FrontendRender, E> {
    let contract: Contract = serde_json::from_slice(&artifacts.exact_ir).map_err(|error| {
        NativeCompileFailure {
            cause: NativeFailureCause::IrDecoding(error.to_string()),
            metadata: artifacts.metadata.prior_status(),
            diagnostic: None,
            recording_diagnostics: Vec::new(),
        }
        .boxed()
    })?;
    let binding = bind_metadata(artifacts, &contract);
    let metadata = binding.status.clone();
    let observer = NativeObserver::new(binding.index.as_ref(), binding.transport);
    let recorder = Recorder::new(
        binding.index.as_ref(),
        binding.transport,
        binding.snapshots.as_ref(),
        &artifacts.receipt.cwd,
    );
    // Retain the actual borrowed index throughout one semantic render. Callback
    // source/error references end at each invocation; only a small ID survives.
    let native_result =
        crate::render_with_refusals(&contract, None, Some(&observer), Some(&recorder));
    let mut rendered = native_result.map_err(|error| {
        NativeCompileFailure {
            cause: NativeFailureCause::Render(error),
            metadata: metadata.clone(),
            diagnostic: observer.expand(binding.snapshots.as_ref(), &artifacts.receipt.cwd),
            recording_diagnostics: recorder.take_completed(),
        }
        .boxed()
    })?;
    if let Some(fail) = functional {
        if !contract
            .stateful_circuits
            .iter()
            .any(|circuit| !circuit.internal)
        {
            return Err(fail("functional ledger8 programs require an exported stateful circuit; pure-only contracts use the existing pure API".into()));
        }
        let sources = functional_sources::RetainedSources::new(
            &contract,
            binding.index.as_ref(),
            binding.transport,
            binding.snapshots.as_ref(),
            &artifacts.receipt.cwd,
        );
        let mut module = crate::recorded::render_functional_contract(&contract, &sources)
            .map_err(|error| fail(format!("{error:?}")))?;
        crate::CompactSignatureLint.visit_item_mod_mut(&mut module);
        crate::CompactBooleanLiterals.visit_item_mod_mut(&mut module);
        let mut file =
            syn::parse_file(&rendered.source).map_err(|error| fail(error.to_string()))?;
        let ledger = file
            .items
            .iter_mut()
            .find_map(|item| match item {
                syn::Item::Mod(item) if item.ident == "ledger_contract" => item.content.as_mut(),
                _ => None,
            })
            .ok_or_else(|| fail("generated ledger_contract module missing".into()))?;
        ledger.1.push(syn::Item::Mod(module));
        rendered.source = format!(
            "{}// Generated by compactc. Do not edit.\n\n{}",
            crate::GENERATED_HEADER,
            prettyplease::unparse(&file),
        );
    }
    if options.apply_frontend_proof_metadata {
        let bytes = artifacts
            .contract_info
            .as_ref()
            .map_err(|error| {
                NativeCompileFailure {
                    cause: NativeFailureCause::ProofMetadata(error.clone()),
                    metadata: metadata.clone(),
                    diagnostic: None,
                    recording_diagnostics: recorder.take_completed(),
                }
                .boxed()
            })?
            .as_ref()
            .ok_or_else(|| {
                NativeCompileFailure {
                    cause: NativeFailureCause::ProofMetadata("contract-info.json missing".into()),
                    metadata: metadata.clone(),
                    diagnostic: None,
                    recording_diagnostics: recorder.take_completed(),
                }
                .boxed()
            })?;
        let info = serde_json::from_slice(bytes).map_err(|error| {
            NativeCompileFailure {
                cause: NativeFailureCause::ProofMetadata(error.to_string()),
                metadata: metadata.clone(),
                diagnostic: None,
                recording_diagnostics: recorder.take_completed(),
            }
            .boxed()
        })?;
        rendered
            .capabilities
            .apply_contract_info(&info)
            .map_err(|error| {
                NativeCompileFailure {
                    cause: NativeFailureCause::Render(RenderError::ProofApplicability(error)),
                    metadata: metadata.clone(),
                    diagnostic: None,
                    recording_diagnostics: recorder.take_completed(),
                }
                .boxed()
            })?;
    }
    if options.require_recording {
        if !options.apply_frontend_proof_metadata {
            return Err(NativeCompileFailure {
                cause: NativeFailureCause::ProofMetadata(
                    "require_recording needs frontend proof applicability".into(),
                ),
                metadata,
                diagnostic: None,
                recording_diagnostics: recorder.take_completed(),
            }
            .boxed()
            .into());
        }
        let missing = recording_requirements(&rendered);
        if !missing.is_empty() {
            return Err(NativeCompileFailure {
                cause: NativeFailureCause::RequireRecording(missing),
                metadata,
                diagnostic: None,
                recording_diagnostics: recorder.take_completed(),
            }
            .boxed()
            .into());
        }
    }
    Ok(FrontendRender {
        rendered,
        metadata,
        fingerprint: artifacts.fingerprint(),
        recording_diagnostics: recorder.take_completed(),
    })
}

struct RenderBinding<'ir, 'metadata> {
    status: MetadataStatus,
    index: Option<BoundOccurrences<'ir>>,
    transport: Option<&'metadata UntrustedTransport>,
    snapshots: Option<SourceSnapshots>,
}
impl RenderBinding<'_, '_> {
    fn unavailable(status: MetadataStatus) -> Self {
        Self {
            status,
            index: None,
            transport: None,
            snapshots: None,
        }
    }
}

fn bind_metadata<'ir, 'metadata>(
    artifacts: &'metadata FrontendArtifacts,
    contract: &'ir Contract,
) -> RenderBinding<'ir, 'metadata> {
    let MetadataState::Valid(transport) = &artifacts.metadata else {
        return RenderBinding::unavailable(artifacts.metadata.prior_status());
    };
    if digest(&artifacts.exact_ir) != artifacts.receipt.ir_sha256 {
        return RenderBinding::unavailable(MetadataStatus::Invalid(MetadataProblem::PairChanged {
            artifact: "exact IR",
        }));
    }
    if Some(transport.raw_sha256()) != artifacts.receipt.companion_sha256 {
        return RenderBinding::unavailable(MetadataStatus::Invalid(MetadataProblem::PairChanged {
            artifact: "raw origins",
        }));
    }
    let bound = match binding::bind(contract, &artifacts.exact_ir, transport) {
        Ok(bound) => bound,
        Err(error) => return RenderBinding::unavailable(status::binding_problem(error)),
    };
    if bound.address(Owner::Contract(contract), Slot::Node) != Some(0) {
        return RenderBinding::unavailable(MetadataStatus::Invalid(MetadataProblem::Binding(
            "typed root address absent".into(),
        )));
    }
    let snapshots = SourceSnapshots::read(transport, &artifacts.receipt.cwd);
    let physical = transport
        .origins()
        .iter()
        .filter(|origin| matches!(origin.extent, Extent::PhysicalSource {}));
    let physical_count = physical.clone().count();
    // One bounded source-text sample, not a full source-input authentication pass.
    // Future failure capture validates only the selected range after unwinding.
    let sample_match = physical
        .take(1)
        .filter(|origin| snapshots.range_matches_snapshot(origin))
        .count();
    let (files, origins, addresses) = transport.table_counts();
    let status = MetadataStatus::Available(BindingSummary {
        files,
        origins,
        addresses,
        typed_slots: bound.typed_slot_count(),
        transport_only: bound.transport_only_count(),
        post_invocation_source_snapshots: snapshots.loaded(),
        source_snapshots_unavailable: snapshots.unavailable(),
        physical_ranges_matching_snapshot: sample_match,
        physical_ranges_unverified: physical_count - sample_match,
        compiler_context_ranges: origins - physical_count,
        post_invocation_snapshot_fingerprints: snapshots.fingerprints(),
    });
    RenderBinding {
        status,
        index: Some(bound),
        transport: Some(transport),
        snapshots: Some(snapshots),
    }
}

fn recording_requirements(rendered: &RenderedContract) -> Vec<RecordingRequirement> {
    rendered
        .capabilities
        .circuits
        .iter()
        .filter(|circuit| {
            circuit.proof_required == Some(true) && (!circuit.recorded || !circuit.observed_call)
        })
        .map(|circuit| {
            let (missing_api, gap) = if !circuit.recorded {
                ("recorded", circuit.recording_unavailable.as_ref())
            } else {
                ("observed-call", circuit.observed_call_unavailable.as_ref())
            };
            // Defensive invariant reporting, never manufacture a supported capability.
            let (code, path, detail) = gap.map_or_else(
                || {
                    (
                        "capability_invariant".into(),
                        String::new(),
                        "unavailable API has no reason".into(),
                    )
                },
                |gap| {
                    (
                        gap.code.as_str().into(),
                        gap.path.clone(),
                        gap.detail.clone(),
                    )
                },
            );
            RecordingRequirement {
                circuit: circuit.name.clone(),
                missing_api,
                code,
                path,
                detail,
            }
        })
        .collect()
}
