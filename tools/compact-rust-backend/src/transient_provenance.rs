// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Optional, bounded evidence for direct transient values reaching persistent primitives.
//!
//! This pass is deliberately separate from rendering and capability selection. A missing
//! finding never certifies a value as stable. Unsupported expression forms and exhausted
//! budgets are reported as unavailable, including when a known route was already found.

use crate::ir::{Contract, Expr, SourceLocation, StateReturn};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_circuits: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_routes: usize,
    pub max_steps: usize,
    pub max_findings: usize,
    pub max_unavailable: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_circuits: 128,
            max_nodes: 4096,
            max_depth: 64,
            max_routes: 32,
            max_steps: 32,
            max_findings: 64,
            max_unavailable: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerKind {
    TransientHash,
    TransientCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Producer {
    pub kind: ProducerKind,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Dependency {
    pub operation: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Route {
    pub producer: Producer,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub circuit: String,
    pub source: Option<SourceLocation>,
    pub sink: String,
    pub sink_path: String,
    pub input: String,
    pub routes: Vec<Route>,
    /// True when the sink input also depends on an ordinary argument, witness,
    /// or an expression whose origin cannot be established by this slice.
    pub has_unknown_input: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unavailable {
    pub circuit: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub unavailable: Vec<Unavailable>,
    /// A resource limit stopped analysis or omitted further diagnostic records.
    pub truncated: bool,
}

fn push_unavailable(report: &mut Report, limits: Limits, circuit: &str, path: &str, reason: &str) {
    if reason.ends_with("limit") {
        report.truncated = true;
    }
    // Reserve one slot for an explanation even when a caller requests zero.
    if report.unavailable.len() >= limits.max_unavailable.max(1) {
        report.truncated = true;
        return;
    }
    report.unavailable.push(Unavailable {
        circuit: circuit.into(),
        path: path.into(),
        reason: reason.into(),
    });
}

#[derive(Clone, Default)]
struct Origin {
    routes: Vec<Route>,
    unknown: bool,
}

impl Origin {
    fn unknown() -> Self {
        Self {
            routes: Vec::new(),
            unknown: true,
        }
    }
}

struct CircuitAnalysis<'a> {
    name: &'a str,
    source: &'a Option<SourceLocation>,
    limits: Limits,
    nodes: usize,
    report: &'a mut Report,
}

impl CircuitAnalysis<'_> {
    fn unavailable(&mut self, path: &str, reason: &str) {
        push_unavailable(self.report, self.limits, self.name, path, reason);
    }

    fn append(&mut self, mut origin: Origin, operation: &str, path: &str) -> Origin {
        for route in &mut origin.routes {
            if route.dependencies.len() >= self.limits.max_steps {
                self.unavailable(path, "dependency step limit");
                return Origin::unknown();
            }
            route.dependencies.push(Dependency {
                operation: operation.into(),
                path: path.into(),
            });
        }
        origin
    }

    fn combine(&mut self, left: Origin, right: Origin, path: &str) -> Origin {
        let mut routes = left.routes;
        for route in right.routes {
            if !routes.contains(&route) {
                if routes.len() >= self.limits.max_routes {
                    self.unavailable(path, "producer route limit");
                    return Origin::unknown();
                }
                routes.push(route);
            }
        }
        Origin {
            routes,
            unknown: left.unknown || right.unknown,
        }
    }

    fn sink(&mut self, kind: &str, path: &str, input: &str, origin: &Origin) {
        if origin.routes.is_empty() {
            return;
        }
        if self.report.findings.len() >= self.limits.max_findings {
            self.unavailable(path, "finding limit");
            self.report.truncated = true;
            return;
        }
        self.report.findings.push(Finding {
            circuit: self.name.into(),
            source: self.source.clone(),
            sink: kind.into(),
            sink_path: path.into(),
            input: input.into(),
            routes: origin.routes.clone(),
            has_unknown_input: origin.unknown,
        });
    }

    fn visit(
        &mut self,
        expr: &Expr,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
    ) -> Origin {
        if depth > self.limits.max_depth {
            self.unavailable(path, "expression depth limit");
            return Origin::unknown();
        }
        if self.nodes >= self.limits.max_nodes {
            self.unavailable(path, "expression node limit");
            return Origin::unknown();
        }
        self.nodes += 1;
        let child = |name: &str| format!("{path}.{name}");
        match expr {
            Expr::Unit
            | Expr::Default { .. }
            | Expr::Boolean { .. }
            | Expr::FieldLiteral { .. }
            | Expr::BytesLiteral { .. }
            | Expr::UnsignedLiteral { .. }
            | Expr::EnumVariant { .. } => Origin::default(),
            Expr::Parameter { name } => env.get(name).cloned().unwrap_or_else(Origin::unknown),
            Expr::WitnessCall { arguments, .. } => {
                let unavailable_before = self.report.unavailable.len();
                let mut known_argument = false;
                for (index, argument) in arguments.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        break;
                    }
                    let argument_origin = self.visit(
                        argument,
                        env,
                        &format!("{path}.arguments[{index}]"),
                        depth + 1,
                    );
                    known_argument |= !argument_origin.routes.is_empty();
                }
                if known_argument || self.report.unavailable.len() > unavailable_before {
                    self.unavailable(path, "witness result provenance is unavailable");
                }
                Origin::unknown()
            }
            Expr::NativeWitnessCall { .. } | Expr::CellRead { .. } | Expr::MapLookup { .. } => {
                Origin::unknown()
            }
            Expr::TransientHash { value } | Expr::TransientCommit { value, opening: _ } => {
                // Analyze children for nested sinks; a producer starts at this operation.
                let input = self.visit(value, env, &child("value"), depth + 1);
                let mut unknown = input.unknown;
                if let Expr::TransientCommit { opening, .. } = expr {
                    unknown |= self
                        .visit(opening, env, &child("opening"), depth + 1)
                        .unknown;
                }
                if self.limits.max_routes == 0 {
                    self.unavailable(path, "producer route limit");
                    return Origin::unknown();
                }
                Origin {
                    routes: vec![Route {
                        producer: Producer {
                            kind: if matches!(expr, Expr::TransientHash { .. }) {
                                ProducerKind::TransientHash
                            } else {
                                ProducerKind::TransientCommit
                            },
                            path: path.into(),
                        },
                        dependencies: Vec::new(),
                    }],
                    unknown,
                }
            }
            Expr::PersistentHash { value } => {
                let input = child("value");
                let origin = self.visit(value, env, &input, depth + 1);
                self.sink("persistent_hash", path, "value", &origin);
                self.append(origin, "persistent_hash", path)
            }
            Expr::PersistentCommit { value, opening } => {
                let value_origin = self.visit(value, env, &child("value"), depth + 1);
                let opening_origin = self.visit(opening, env, &child("opening"), depth + 1);
                self.sink("persistent_commit", path, "value", &value_origin);
                self.sink("persistent_commit", path, "opening", &opening_origin);
                let joined = self.combine(value_origin, opening_origin, path);
                self.append(joined, "persistent_commit", path)
            }
            Expr::UnsignedCast { value, .. }
            | Expr::FieldCast { value }
            | Expr::FieldToBytes32 { value }
            | Expr::Coerce { value, .. }
            | Expr::UpgradeFromTransient { value }
            | Expr::DegradeToTransient { value } => {
                let origin = self.visit(value, env, &child("value"), depth + 1);
                self.append(origin, "conversion", path)
            }
            Expr::Add { left, right }
            | Expr::Subtract { left, right }
            | Expr::Multiply { left, right }
            | Expr::UnsignedAdd { left, right, .. }
            | Expr::UnsignedSubtract { left, right, .. }
            | Expr::UnsignedMultiply { left, right, .. } => {
                let left = self.visit(left, env, &child("left"), depth + 1);
                let right = self.visit(right, env, &child("right"), depth + 1);
                let joined = self.combine(left, right, path);
                self.append(joined, "arithmetic", path)
            }
            Expr::Let { bindings, body } => {
                let mut local = env.clone();
                for (index, binding) in bindings.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        return Origin::unknown();
                    }
                    let binding_path = format!("{path}.bindings[{index}]");
                    let value = self.visit(
                        &binding.value,
                        &local,
                        &format!("{binding_path}.value"),
                        depth + 1,
                    );
                    local.insert(
                        binding.name.clone(),
                        self.append(value, &format!("binding:{}", binding.name), &binding_path),
                    );
                }
                self.visit(body, &local, &child("body"), depth + 1)
            }
            Expr::Sequence { steps, value } => {
                for (index, step) in steps.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        return Origin::unknown();
                    }
                    self.visit(step, env, &format!("{path}.steps[{index}]"), depth + 1);
                }
                self.visit(value, env, &child("value"), depth + 1)
            }
            _ => {
                self.unavailable(path, "unsupported expression form");
                Origin::unknown()
            }
        }
    }
}

/// Analyze only explicitly selected typed IR. No rendering, capability, or ABI effect.
pub fn analyze(contract: &Contract, limits: Limits) -> Report {
    let mut report = Report::default();
    let mut count = 0;
    for circuit in &contract.circuits {
        if count >= limits.max_circuits {
            push_unavailable(&mut report, limits, &circuit.name, "body", "circuit limit");
            report.truncated = true;
            return report;
        }
        count += 1;
        if circuit.parameters.len() > limits.max_nodes {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "parameter limit",
            );
            continue;
        }
        let mut analysis = CircuitAnalysis {
            name: &circuit.name,
            source: &circuit.source,
            limits,
            nodes: 0,
            report: &mut report,
        };
        let env = circuit
            .parameters
            .iter()
            .map(|p| (p.name.clone(), Origin::unknown()))
            .collect();
        analysis.visit(&circuit.body, &env, "body", 0);
    }
    for circuit in &contract.stateful_circuits {
        if count >= limits.max_circuits {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "actions",
                "circuit limit",
            );
            report.truncated = true;
            return report;
        }
        count += 1;
        if circuit.parameters.len() > limits.max_nodes {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "parameter limit",
            );
            continue;
        }
        if circuit.actions.is_empty() {
            if let StateReturn::Expression { value } = &circuit.return_value {
                let mut analysis = CircuitAnalysis {
                    name: &circuit.name,
                    source: &circuit.source,
                    limits,
                    nodes: 0,
                    report: &mut report,
                };
                let env = circuit
                    .parameters
                    .iter()
                    .map(|p| (p.name.clone(), Origin::unknown()))
                    .collect();
                analysis.visit(value, &env, "return_value.value", 0);
                continue;
            }
        }
        push_unavailable(
            &mut report,
            limits,
            &circuit.name,
            "actions",
            "stateful action/return flow is outside this slice",
        );
    }
    if contract.constructor.is_some() {
        push_unavailable(
            &mut report,
            limits,
            "constructor",
            "steps",
            "constructor flow is outside this slice",
        );
    }
    report
}
