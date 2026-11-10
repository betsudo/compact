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

//! Optional, bounded evidence for direct transient values reaching persistent primitives.
//!
//! This pass is deliberately separate from rendering and capability selection. A missing
//! finding never certifies a value as stable. Unsupported expression forms and exhausted
//! budgets are reported as unavailable, including when a known route was already found.

use crate::ir::{
    Contract, Expr, LocalBinding, Parameter, PureCircuit, ReturnPlan, SCHEMA_VERSION,
    SourceLocation, StateAction, StateReturn, StatefulCircuit, Type,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_circuits: usize,
    pub max_call_depth: usize,
    pub max_nodes: usize,
    /// Total nodes permitted while materializing reusable helper return summaries.
    pub max_summary_nodes: usize,
    pub max_parameters: usize,
    pub max_component_nodes: usize,
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
            max_call_depth: 8,
            max_nodes: 4096,
            max_summary_nodes: 4096,
            max_parameters: 32,
            max_component_nodes: 64,
            max_depth: 64,
            max_routes: 32,
            max_steps: 32,
            max_findings: 64,
            max_unavailable: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerKind {
    TransientHash,
    TransientCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub kind: ProducerKind,
    pub circuit: String,
    pub source: Option<SourceLocation>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub operation: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ParameterDependency {
    pub circuit: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormalDependency {
    pub parameter: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HelperSummaryEvidence {
    pub circuit: String,
    pub source: Option<SourceLocation>,
    pub formal_dependencies: Vec<FormalDependency>,
    pub producers: Vec<Producer>,
    /// Unknown contribution other than a symbolic formal; false never certifies stability.
    pub has_nonformal_unknown: bool,
    pub nodes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub producer: Producer,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub circuit: String,
    pub source: Option<SourceLocation>,
    pub sink: String,
    pub sink_circuit: String,
    pub sink_source: Option<SourceLocation>,
    pub sink_path: String,
    pub input: String,
    pub routes: Vec<Route>,
    /// Provenance of a condition that controls whether this sink executes.
    pub control_routes: Vec<Route>,
    /// Root-circuit parameters that may affect this sink input.
    pub parameter_dependencies: Vec<ParameterDependency>,
    /// Root-circuit parameters that may affect whether this sink executes.
    pub control_parameter_dependencies: Vec<ParameterDependency>,
    /// Branch arms containing the sink; an empty list means no lexical branch.
    pub conditions: Vec<String>,
    /// True when the sink input also depends on an ordinary argument, witness,
    /// or an expression whose origin cannot be established by this slice.
    pub has_unknown_input: bool,
    pub has_unknown_control: bool,
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
    /// Reusable, sink-free helper return summaries used at call sites.
    pub helper_summaries: Vec<HelperSummaryEvidence>,
    pub unavailable: Vec<Unavailable>,
    /// A resource limit stopped analysis or omitted further diagnostic records.
    pub truncated: bool,
}

/// Exact caller-supplied acknowledgement of one producer route at one sink input.
/// It changes advisory severity only; the producer and dependency evidence remains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRole {
    Data,
    Control,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Acknowledgement {
    pub circuit: String,
    pub sink: String,
    pub sink_circuit: String,
    pub sink_path: String,
    pub input: String,
    pub role: DependencyRole,
    pub route: Route,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdvisoryFinding {
    pub finding: Finding,
    pub acknowledged_data_routes: Vec<bool>,
    pub acknowledged_control_routes: Vec<bool>,
    /// True if any known route remains unacknowledged.
    pub warning: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AdvisoryReport {
    pub findings: Vec<AdvisoryFinding>,
    pub helper_summaries: Vec<HelperSummaryEvidence>,
    pub unavailable: Vec<Unavailable>,
    pub unused_acknowledgements: Vec<Acknowledgement>,
    pub truncated: bool,
}

/// Apply an explicit, bounded acknowledgement policy to an `analyze` report.
/// Matching is exact over circuit, sink kind/circuit/path/input, dependency role,
/// and the full route. No finding is removed. `warning == false` means all known
/// routes were acknowledged, not that unknown input is stable. An empty or
/// whitespace-only reason never acknowledges.
pub fn advise(
    report: &Report,
    acknowledgements: &[Acknowledgement],
    max_acknowledgements: usize,
) -> AdvisoryReport {
    let mut result = AdvisoryReport {
        helper_summaries: report.helper_summaries.clone(),
        unavailable: report.unavailable.clone(),
        truncated: report.truncated,
        ..AdvisoryReport::default()
    };
    if acknowledgements.len() > max_acknowledgements {
        result.unavailable.push(Unavailable {
            circuit: "acknowledgements".into(),
            path: "policy".into(),
            reason: "acknowledgement limit".into(),
        });
        result.truncated = true;
    }
    let active = if acknowledgements.len() <= max_acknowledgements {
        acknowledgements
    } else {
        &[]
    };
    let mut used = vec![false; active.len()];
    for finding in &report.findings {
        let mut mark = |route: &Route, role: DependencyRole| {
            let mut acknowledged = false;
            for (index, entry) in active.iter().enumerate() {
                if !entry.reason.trim().is_empty()
                    && entry.circuit == finding.circuit
                    && entry.sink == finding.sink
                    && entry.sink_circuit == finding.sink_circuit
                    && entry.sink_path == finding.sink_path
                    && entry.input == finding.input
                    && entry.role == role
                    && entry.route == *route
                {
                    used[index] = true;
                    acknowledged = true;
                }
            }
            acknowledged
        };
        let acknowledged_data_routes = finding
            .routes
            .iter()
            .map(|route| mark(route, DependencyRole::Data))
            .collect::<Vec<_>>();
        let acknowledged_control_routes = finding
            .control_routes
            .iter()
            .map(|route| mark(route, DependencyRole::Control))
            .collect::<Vec<_>>();
        let warning = acknowledged_data_routes
            .iter()
            .chain(&acknowledged_control_routes)
            .any(|acknowledged| !acknowledged);
        result.findings.push(AdvisoryFinding {
            finding: finding.clone(),
            acknowledged_data_routes,
            acknowledged_control_routes,
            warning,
        });
    }
    result.unused_acknowledgements = active
        .iter()
        .zip(used)
        .filter_map(|(entry, used)| (!used).then(|| entry.clone()))
        .collect();
    result
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
    formals: Vec<FormalRoute>,
    route_order: Vec<RouteSlot>,
    parameters: BTreeSet<ParameterDependency>,
    unknown: bool,
    components: Option<Components>,
}

#[derive(Clone)]
enum Components {
    Indexed(Vec<Origin>),
    Named(Vec<(String, Origin)>),
}

#[derive(Clone, PartialEq, Eq)]
struct FormalRoute {
    index: usize,
    dependencies: Vec<Dependency>,
}

#[derive(Clone)]
enum RouteSlot {
    Known(usize),
    Formal(usize),
}

#[derive(Clone)]
struct HelperSummary {
    result: Origin,
    nodes: usize,
    max_depth: usize,
    max_call_stack: usize,
}

fn same_simple_literal(left: &Expr, right: &Expr) -> bool {
    match (left, right) {
        (Expr::Unit, Expr::Unit) => true,
        (Expr::Boolean { value: a }, Expr::Boolean { value: b }) => a == b,
        (Expr::FieldLiteral { value: a }, Expr::FieldLiteral { value: b }) => {
            a.len() <= 128 && b.len() <= 128 && a == b
        }
        (Expr::BytesLiteral { bytes: a }, Expr::BytesLiteral { bytes: b }) => {
            a.len() <= 64 && b.len() <= 64 && a == b
        }
        (
            Expr::UnsignedLiteral { value: a, max: am },
            Expr::UnsignedLiteral { value: b, max: bm },
        ) => {
            a.len() <= 128
                && b.len() <= 128
                && am.len() <= 128
                && bm.len() <= 128
                && a == b
                && am == bm
        }
        _ => false,
    }
}

impl Origin {
    fn unknown() -> Self {
        Self {
            routes: Vec::new(),
            formals: Vec::new(),
            route_order: Vec::new(),
            parameters: BTreeSet::new(),
            unknown: true,
            components: None,
        }
    }

    fn parameter(circuit: &str, name: &str) -> Self {
        let mut origin = Self::unknown();
        origin.parameters.insert(ParameterDependency {
            circuit: circuit.into(),
            name: name.into(),
        });
        origin
    }

    fn formal(index: usize) -> Self {
        Self {
            formals: vec![FormalRoute {
                index,
                dependencies: Vec::new(),
            }],
            route_order: vec![RouteSlot::Formal(0)],
            ..Self::default()
        }
    }
}

fn count_component_nodes(origin: &Origin, used: &mut usize, limit: usize) -> bool {
    if *used >= limit {
        return false;
    }
    *used += 1;
    match &origin.components {
        Some(Components::Indexed(values)) => values
            .iter()
            .all(|value| count_component_nodes(value, used, limit)),
        Some(Components::Named(values)) => values
            .iter()
            .all(|(_, value)| count_component_nodes(value, used, limit)),
        None => true,
    }
}

fn summary_path(path: &str, call_body: &str) -> String {
    if path == "body" {
        call_body.into()
    } else if let Some(rest) = path.strip_prefix("body.") {
        format!("{call_body}.{rest}")
    } else {
        format!("{call_body}.{path}")
    }
}

struct CircuitAnalysis<'c, 'r> {
    name: &'c str,
    source: &'c Option<SourceLocation>,
    current_circuit: &'c str,
    current_source: &'c Option<SourceLocation>,
    circuits: &'c BTreeMap<&'c str, &'c PureCircuit>,
    ambiguous: &'c BTreeSet<&'c str>,
    stateful_circuits: &'c BTreeMap<&'c str, &'c StatefulCircuit>,
    ambiguous_stateful: &'c BTreeSet<&'c str>,
    summaries: Option<&'c BTreeMap<&'c str, HelperSummary>>,
    call_stack: Vec<&'c str>,
    conditions: Vec<String>,
    control_origin: Origin,
    return_env: Option<BTreeMap<String, Origin>>,
    limits: Limits,
    nodes: usize,
    max_depth_seen: usize,
    max_call_stack_seen: usize,
    saw_sink: bool,
    saw_stateful_call: bool,
    saw_assert_or_witness: bool,
    report: &'r mut Report,
}

impl<'c, 'r> CircuitAnalysis<'c, 'r> {
    fn unavailable(&mut self, path: &str, reason: &str) {
        push_unavailable(self.report, self.limits, self.name, path, reason);
    }

    fn instantiate_summary_origin(
        &mut self,
        template: Origin,
        actuals: &[Origin],
        formals: &[Parameter],
        path: &str,
        call_body: &str,
    ) -> Origin {
        let Origin {
            mut routes,
            formals: formal_routes,
            route_order,
            parameters,
            unknown,
            components,
        } = template;
        for route in &mut routes {
            route.producer.path = summary_path(&route.producer.path, call_body);
            for dependency in &mut route.dependencies {
                dependency.path = summary_path(&dependency.path, call_body);
            }
        }
        let retain_actual_components = matches!(route_order.as_slice(), [RouteSlot::Formal(_)])
            && !unknown
            && parameters.is_empty()
            && components.is_none();
        let mut result = Origin {
            routes: Vec::new(),
            formals: Vec::new(),
            route_order: Vec::new(),
            parameters,
            unknown,
            components: None,
        };
        for slot in route_order {
            match slot {
                RouteSlot::Known(index) => {
                    let route = Origin {
                        routes: vec![routes[index].clone()],
                        route_order: vec![RouteSlot::Known(0)],
                        ..Origin::default()
                    };
                    result = self.combine(result, route, path);
                }
                RouteSlot::Formal(index) => {
                    let formal = &formal_routes[index];
                    let Some((parameter, actual)) =
                        formals.get(formal.index).zip(actuals.get(formal.index))
                    else {
                        self.unavailable(path, "summary formal unavailable");
                        return Origin::unknown();
                    };
                    let mut actual = self.append(
                        actual.clone(),
                        &format!("call_argument:{}", parameter.name),
                        path,
                    );
                    for dependency in &formal.dependencies {
                        actual = self.append(
                            actual,
                            &dependency.operation,
                            &summary_path(&dependency.path, call_body),
                        );
                    }
                    result = if retain_actual_components {
                        actual
                    } else {
                        self.combine(result, actual, path)
                    };
                }
            }
        }
        if let Some(components) = components {
            result.components = Some(match components {
                Components::Indexed(values) => Components::Indexed(
                    values
                        .into_iter()
                        .map(|value| {
                            self.instantiate_summary_origin(
                                value, actuals, formals, path, call_body,
                            )
                        })
                        .collect(),
                ),
                Components::Named(values) => Components::Named(
                    values
                        .into_iter()
                        .map(|(name, value)| {
                            (
                                name,
                                self.instantiate_summary_origin(
                                    value, actuals, formals, path, call_body,
                                ),
                            )
                        })
                        .collect(),
                ),
            });
            let mut used = 0;
            if !count_component_nodes(&result, &mut used, self.limits.max_component_nodes) {
                self.unavailable(path, "aggregate component limit");
                return Origin::unknown();
            }
        }
        result
    }

    fn cached_summary_result(
        &mut self,
        name: &str,
        formals: &[Parameter],
        actuals: &[Origin],
        path: &str,
        call_body: &str,
        depth: usize,
    ) -> Option<Origin> {
        let summary = self.summaries?.get(name)?.clone();
        let route_upper_bound =
            summary
                .result
                .formals
                .iter()
                .fold(summary.result.routes.len(), |used, formal| {
                    used.saturating_add(
                        actuals
                            .get(formal.index)
                            .map_or(self.limits.max_routes.saturating_add(1), |actual| {
                                actual.routes.len()
                            }),
                    )
                });
        if self.nodes.saturating_add(summary.nodes) > self.limits.max_nodes
            || depth.saturating_add(1).saturating_add(summary.max_depth) > self.limits.max_depth
            || self.call_stack.len().saturating_add(summary.max_call_stack)
                > self.limits.max_call_depth.saturating_add(1)
            || route_upper_bound > self.limits.max_routes
        {
            return None;
        }
        self.nodes += summary.nodes;
        let result =
            self.instantiate_summary_origin(summary.result, actuals, formals, path, call_body);
        Some(self.append(result, &format!("call_return:{name}"), path))
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
        for formal in &mut origin.formals {
            if formal.dependencies.len() >= self.limits.max_steps {
                self.unavailable(path, "dependency step limit");
                return Origin::unknown();
            }
            formal.dependencies.push(Dependency {
                operation: operation.into(),
                path: path.into(),
            });
        }
        match &mut origin.components {
            Some(Components::Indexed(values)) => {
                for value in values {
                    *value = self.append(value.clone(), operation, path);
                }
            }
            Some(Components::Named(values)) => {
                for (_, value) in values {
                    *value = self.append(value.clone(), operation, path);
                }
            }
            None => {}
        }
        origin
    }

    fn combine(&mut self, left: Origin, right: Origin, path: &str) -> Origin {
        let mut routes = left.routes;
        let mut formals = left.formals;
        let mut route_order = left.route_order;
        for slot in right.route_order {
            match slot {
                RouteSlot::Known(index) => {
                    let route = &right.routes[index];
                    if !routes.contains(route) {
                        if routes.len() >= self.limits.max_routes {
                            self.unavailable(path, "producer route limit");
                            return Origin::unknown();
                        }
                        route_order.push(RouteSlot::Known(routes.len()));
                        routes.push(route.clone());
                    }
                }
                RouteSlot::Formal(index) => {
                    let formal = &right.formals[index];
                    if !formals.contains(formal) {
                        if formals.len() >= self.limits.max_routes {
                            self.unavailable(path, "formal route limit");
                            return Origin::unknown();
                        }
                        route_order.push(RouteSlot::Formal(formals.len()));
                        formals.push(formal.clone());
                    }
                }
            }
        }
        let mut parameters = left.parameters;
        parameters.extend(right.parameters);
        Origin {
            routes,
            formals,
            route_order,
            parameters,
            unknown: left.unknown || right.unknown,
            components: None,
        }
    }

    fn aggregate(&mut self, components: Components, path: &str) -> Origin {
        let values: Vec<&Origin> = match &components {
            Components::Indexed(values) => values.iter().collect(),
            Components::Named(values) => values.iter().map(|(_, value)| value).collect(),
        };
        let mut component_nodes = 1;
        if !values.iter().all(|value| {
            count_component_nodes(value, &mut component_nodes, self.limits.max_component_nodes)
        }) {
            self.unavailable(path, "aggregate component limit");
            return Origin::unknown();
        }
        let had_known = values
            .iter()
            .any(|value| !value.routes.is_empty() || !value.formals.is_empty());
        let mut whole = Origin::default();
        for value in values {
            whole = self.combine(whole, value.clone(), path);
        }
        if had_known && whole.routes.is_empty() && whole.formals.is_empty() {
            return Origin::unknown();
        }
        whole.components = Some(components);
        whole
    }

    fn branch_components(
        &mut self,
        then: &Origin,
        otherwise: &Origin,
        path: &str,
    ) -> Option<Components> {
        match (&then.components, &otherwise.components) {
            (Some(Components::Indexed(left)), Some(Components::Indexed(right)))
                if left.len() == right.len() =>
            {
                Some(Components::Indexed(
                    left.iter()
                        .zip(right)
                        .map(|(a, b)| self.combine(a.clone(), b.clone(), path))
                        .collect(),
                ))
            }
            (Some(Components::Named(left)), Some(Components::Named(right)))
                if left.len() == right.len()
                    && left.iter().zip(right).all(|((a, _), (b, _))| a == b) =>
            {
                Some(Components::Named(
                    left.iter()
                        .zip(right)
                        .map(|((name, a), (_, b))| {
                            (name.clone(), self.combine(a.clone(), b.clone(), path))
                        })
                        .collect(),
                ))
            }
            (Some(components), None)
                if otherwise.routes.is_empty() && otherwise.formals.is_empty() =>
            {
                Some(self.components_with_unknown(components, otherwise, path))
            }
            (None, Some(components)) if then.routes.is_empty() && then.formals.is_empty() => {
                Some(self.components_with_unknown(components, then, path))
            }
            (None, None) => None,
            _ => {
                self.unavailable(path, "branch aggregate shape unavailable");
                None
            }
        }
    }

    fn components_with_unknown(
        &mut self,
        components: &Components,
        other: &Origin,
        path: &str,
    ) -> Components {
        let mut unknown = Origin::unknown();
        unknown.parameters = other.parameters.clone();
        unknown.formals = other.formals.clone();
        match components {
            Components::Indexed(values) => Components::Indexed(
                values
                    .iter()
                    .map(|value| self.combine(value.clone(), unknown.clone(), path))
                    .collect(),
            ),
            Components::Named(values) => Components::Named(
                values
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            self.combine(value.clone(), unknown.clone(), path),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn sink(&mut self, kind: &str, path: &str, input: &str, origin: &Origin) {
        self.saw_sink = true;
        if origin.routes.is_empty() && self.control_origin.routes.is_empty() {
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
            sink_circuit: self.current_circuit.into(),
            sink_source: self.current_source.clone(),
            sink_path: path.into(),
            input: input.into(),
            routes: origin.routes.clone(),
            control_routes: self.control_origin.routes.clone(),
            parameter_dependencies: origin.parameters.iter().cloned().collect(),
            control_parameter_dependencies: self
                .control_origin
                .parameters
                .iter()
                .cloned()
                .collect(),
            conditions: self.conditions.clone(),
            has_unknown_input: origin.unknown,
            has_unknown_control: self.control_origin.unknown,
        });
    }

    fn action_node(&mut self, path: &str, depth: usize) -> bool {
        self.max_depth_seen = self.max_depth_seen.max(depth);
        if depth > self.limits.max_depth {
            self.unavailable(path, "action depth limit");
            return false;
        }
        if self.nodes >= self.limits.max_nodes {
            self.unavailable(path, "action node limit");
            return false;
        }
        self.nodes += 1;
        true
    }

    fn bindings(
        &mut self,
        bindings: &[LocalBinding],
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
    ) -> BTreeMap<String, Origin> {
        let mut local = env.clone();
        for (index, binding) in bindings.iter().enumerate() {
            if !self.action_node(&format!("{path}.bindings[{index}]"), depth) {
                break;
            }
            let binding_path = format!("{path}.bindings[{index}]");
            let origin = self.visit(
                &binding.value,
                &local,
                &format!("{binding_path}.value"),
                depth + 1,
            );
            local.insert(
                binding.name.clone(),
                self.append(origin, &format!("binding:{}", binding.name), &binding_path),
            );
        }
        local
    }

    fn action(
        &mut self,
        action: &StateAction,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
        terminal: bool,
    ) {
        if !self.action_node(path, depth) {
            return;
        }
        match action {
            StateAction::Sequence { actions } => {
                for (index, action) in actions.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "action node limit");
                        break;
                    }
                    self.action(
                        action,
                        env,
                        &format!("{path}.actions[{index}]"),
                        depth + 1,
                        terminal && index + 1 == actions.len(),
                    );
                }
            }
            StateAction::Let { bindings, action } => {
                let local = self.bindings(bindings, env, path, depth + 1);
                if terminal {
                    self.return_env = Some(local.clone());
                }
                self.action(
                    action,
                    &local,
                    &format!("{path}.action"),
                    depth + 1,
                    terminal,
                );
            }
            StateAction::If {
                condition,
                then,
                otherwise,
            } => {
                let condition_origin =
                    self.visit(condition, env, &format!("{path}.condition"), depth + 1);
                let prior_control = self.control_origin.clone();
                let branch_control = self.append(condition_origin, "branch_execution", path);
                self.control_origin = self.combine(prior_control.clone(), branch_control, path);
                self.conditions.push(format!("{path}:then"));
                self.action(then, env, &format!("{path}.then"), depth + 1, false);
                self.conditions.pop();
                self.conditions.push(format!("{path}:otherwise"));
                self.action(
                    otherwise,
                    env,
                    &format!("{path}.otherwise"),
                    depth + 1,
                    false,
                );
                self.conditions.pop();
                self.control_origin = prior_control;
            }
            StateAction::Expression { value } => {
                self.visit(value, env, &format!("{path}.value"), depth + 1);
            }
            StateAction::Assert { condition, .. } => {
                let condition_origin =
                    self.visit(condition, env, &format!("{path}.condition"), depth + 1);
                if !condition_origin.routes.is_empty() || condition_origin.unknown {
                    self.unavailable(path, "assertion execution control unavailable");
                }
            }
            StateAction::CellWrite { value, .. } => {
                self.state_sink("cell_write", "value", value, env, path, depth)
            }
            StateAction::SetInsert { value, .. } => {
                self.state_sink("set_insert", "value", value, env, path, depth)
            }
            StateAction::SetRemove { value, .. } => {
                self.state_sink("set_remove", "value", value, env, path, depth)
            }
            StateAction::ListPushFront { value, .. } => {
                self.state_sink("list_push_front", "value", value, env, path, depth)
            }
            StateAction::MerkleInsert { value, .. } => {
                self.state_sink("merkle_insert", "value", value, env, path, depth)
            }
            StateAction::MerkleInsertHash { hash, .. } => {
                self.state_sink("merkle_insert_hash", "hash", hash, env, path, depth)
            }
            StateAction::HistoricMerkleInsert { value, .. } => {
                self.state_sink("historic_merkle_insert", "value", value, env, path, depth)
            }
            StateAction::HistoricMerkleInsertHash { hash, .. } => self.state_sink(
                "historic_merkle_insert_hash",
                "hash",
                hash,
                env,
                path,
                depth,
            ),
            StateAction::MerkleInsertHashIndex { hash, position, .. } => {
                self.state_sink("merkle_insert_hash_index", "hash", hash, env, path, depth);
                self.state_sink(
                    "merkle_insert_hash_index",
                    "position",
                    position,
                    env,
                    path,
                    depth,
                );
            }
            StateAction::HistoricMerkleInsertHashIndex { hash, position, .. } => {
                self.state_sink(
                    "historic_merkle_insert_hash_index",
                    "hash",
                    hash,
                    env,
                    path,
                    depth,
                );
                self.state_sink(
                    "historic_merkle_insert_hash_index",
                    "position",
                    position,
                    env,
                    path,
                    depth,
                );
            }
            StateAction::MerkleInsertIndexDefault { position, .. } => self.state_sink(
                "merkle_insert_index_default",
                "position",
                position,
                env,
                path,
                depth,
            ),
            StateAction::HistoricMerkleInsertIndexDefault { position, .. } => self.state_sink(
                "historic_merkle_insert_index_default",
                "position",
                position,
                env,
                path,
                depth,
            ),
            StateAction::MapInsert { key, value, .. } => {
                self.state_sink("map_insert", "key", key, env, path, depth);
                self.state_sink("map_insert", "value", value, env, path, depth);
            }
            StateAction::MapInsertDefault { key, .. } => {
                self.state_sink("map_insert_default", "key", key, env, path, depth)
            }
            StateAction::MapRemove { key, .. } => {
                self.state_sink("map_remove", "key", key, env, path, depth)
            }
            StateAction::MerkleInsertIndex {
                value, position, ..
            } => {
                self.state_sink(
                    "merkle_insert_index",
                    "position",
                    position,
                    env,
                    path,
                    depth,
                );
                self.state_sink("merkle_insert_index", "value", value, env, path, depth);
            }
            StateAction::HistoricMerkleInsertIndex {
                value, position, ..
            } => {
                self.state_sink(
                    "historic_merkle_insert_index",
                    "position",
                    position,
                    env,
                    path,
                    depth,
                );
                self.state_sink(
                    "historic_merkle_insert_index",
                    "value",
                    value,
                    env,
                    path,
                    depth,
                );
            }
            StateAction::PureCall { name, arguments } => {
                if self.ambiguous.contains(name.as_str())
                    || self.stateful_circuits.contains_key(name.as_str())
                {
                    self.unavailable(path, "ambiguous pure action helper");
                    return;
                }
                let Some(callee) = self.circuits.get(name.as_str()).copied() else {
                    self.unavailable(path, "unknown pure action helper");
                    return;
                };
                if !matches!(callee.result, Type::Unit) {
                    self.unavailable(path, "non-unit pure action helper");
                    return;
                }
                if arguments.len() != callee.parameters.len() {
                    self.unavailable(path, "pure action helper arity mismatch");
                    return;
                }
                if callee.parameters.len() > self.limits.max_nodes.min(self.limits.max_parameters) {
                    self.unavailable(path, "parameter limit");
                    return;
                }
                if callee
                    .parameters
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != callee.parameters.len()
                {
                    self.unavailable(path, "duplicate pure action helper parameter");
                    return;
                }
                if self.call_stack.contains(&callee.name.as_str()) {
                    self.unavailable(path, "recursive pure action helper");
                    return;
                }
                if self.call_stack.len() > self.limits.max_call_depth {
                    self.unavailable(path, "pure action helper call depth limit");
                    return;
                }
                let mut actuals = Vec::with_capacity(arguments.len().min(self.limits.max_nodes));
                for (index, argument) in arguments.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "action node limit");
                        return;
                    }
                    actuals.push(self.visit(
                        argument,
                        env,
                        &format!("{path}.arguments[{index}]"),
                        depth + 1,
                    ));
                }
                let mut callee_env = BTreeMap::new();
                for (formal, actual) in callee.parameters.iter().zip(actuals) {
                    callee_env.insert(
                        formal.name.clone(),
                        self.append(actual, &format!("call_argument:{}", formal.name), path),
                    );
                }
                self.call_stack.push(&callee.name);
                let previous_circuit = self.current_circuit;
                let previous_source = self.current_source;
                self.current_circuit = &callee.name;
                self.current_source = &callee.source;
                self.visit(
                    &callee.body,
                    &callee_env,
                    &format!("{path}.callee:{}.body", callee.name),
                    depth + 1,
                );
                self.current_circuit = previous_circuit;
                self.current_source = previous_source;
                self.call_stack.pop();
            }
            StateAction::CircuitCall { name, arguments } => {
                if self.ambiguous_stateful.contains(name.as_str()) {
                    self.unavailable(path, "ambiguous stateful helper");
                    return;
                }
                let Some(callee) = self.stateful_circuits.get(name.as_str()).copied() else {
                    self.unavailable(path, "unknown stateful helper");
                    return;
                };
                if arguments.len() != callee.parameters.len() {
                    self.unavailable(path, "stateful helper arity mismatch");
                    return;
                }
                if callee.parameters.len() > self.limits.max_nodes.min(self.limits.max_parameters) {
                    self.unavailable(path, "parameter limit");
                    return;
                }
                if callee
                    .parameters
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != callee.parameters.len()
                {
                    self.unavailable(path, "duplicate stateful helper parameter");
                    return;
                }
                if self.call_stack.contains(&callee.name.as_str()) {
                    self.unavailable(path, "recursive stateful helper");
                    return;
                }
                if self.call_stack.len() > self.limits.max_call_depth {
                    self.unavailable(path, "stateful helper call depth limit");
                    return;
                }
                if !matches!(callee.return_value, StateReturn::Unit) {
                    self.unavailable(path, "non-unit stateful action helper");
                    return;
                }
                let mut actuals = Vec::with_capacity(arguments.len().min(self.limits.max_nodes));
                for (index, argument) in arguments.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "action node limit");
                        return;
                    }
                    actuals.push(self.visit(
                        argument,
                        env,
                        &format!("{path}.arguments[{index}]"),
                        depth + 1,
                    ));
                }
                let mut callee_env = BTreeMap::new();
                for (formal, actual) in callee.parameters.iter().zip(actuals) {
                    callee_env.insert(
                        formal.name.clone(),
                        self.append(actual, &format!("call_argument:{}", formal.name), path),
                    );
                }
                self.call_stack.push(&callee.name);
                let previous_circuit = self.current_circuit;
                let previous_source = self.current_source;
                self.current_circuit = &callee.name;
                self.current_source = &callee.source;
                for (index, action) in callee.actions.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "action node limit");
                        break;
                    }
                    self.action(
                        action,
                        &callee_env,
                        &format!("{path}.callee:{}.actions[{index}]", callee.name),
                        depth + 1,
                        false,
                    );
                }
                self.current_circuit = previous_circuit;
                self.current_source = previous_source;
                self.call_stack.pop();
            }
            _ => self.unavailable(path, "unsupported state action"),
        }
    }

    fn state_sink(
        &mut self,
        kind: &str,
        input: &str,
        value: &Expr,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
    ) {
        let origin = self.visit(value, env, &format!("{path}.{input}"), depth + 1);
        self.sink(kind, path, input, &origin);
    }

    fn ledger_query(
        &mut self,
        operand: &Expr,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
        input: &str,
        operation: &str,
    ) -> Origin {
        let operand_origin = self.visit(operand, env, &format!("{path}.{input}"), depth);
        // The query operand can affect the result, while ledger contents are external.
        let result = self.combine(operand_origin, Origin::unknown(), path);
        self.append(result, operation, path)
    }

    fn return_plan(
        &mut self,
        plan: &ReturnPlan,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
    ) -> Origin {
        if !self.action_node(path, depth) {
            return Origin::unknown();
        }
        match plan {
            ReturnPlan::Value { value } => {
                self.visit(value, env, &format!("{path}.value"), depth + 1)
            }
            ReturnPlan::Sequence { actions, result } => {
                for (index, action) in actions.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "action node limit");
                        return Origin::unknown();
                    }
                    self.action(
                        action,
                        env,
                        &format!("{path}.actions[{index}]"),
                        depth + 1,
                        false,
                    );
                }
                self.return_plan(result, env, &format!("{path}.result"), depth + 1)
            }
            ReturnPlan::Let { bindings, result } => {
                let local = self.bindings(bindings, env, path, depth + 1);
                self.return_plan(result, &local, &format!("{path}.result"), depth + 1)
            }
            ReturnPlan::Conditional {
                condition,
                then,
                otherwise,
            } => {
                let condition_origin =
                    self.visit(condition, env, &format!("{path}.condition"), depth + 1);
                let prior_control = self.control_origin.clone();
                let branch_control =
                    self.append(condition_origin.clone(), "branch_execution", path);
                self.control_origin = self.combine(prior_control.clone(), branch_control, path);
                self.conditions.push(format!("{path}:then"));
                let then_origin = self.return_plan(then, env, &format!("{path}.then"), depth + 1);
                self.conditions.pop();
                self.conditions.push(format!("{path}:otherwise"));
                let otherwise_origin =
                    self.return_plan(otherwise, env, &format!("{path}.otherwise"), depth + 1);
                self.conditions.pop();
                self.control_origin = prior_control;
                let same_literal = matches!((then.as_ref(), otherwise.as_ref()),
                    (ReturnPlan::Value { value: a }, ReturnPlan::Value { value: b }) if same_simple_literal(a, b));
                self.conditional_result(
                    condition_origin,
                    then_origin,
                    otherwise_origin,
                    same_literal,
                    path,
                )
            }
        }
    }

    fn conditional_result(
        &mut self,
        condition_origin: Origin,
        then_origin: Origin,
        otherwise_origin: Origin,
        same_literal: bool,
        path: &str,
    ) -> Origin {
        let had_known = !then_origin.routes.is_empty()
            || !otherwise_origin.routes.is_empty()
            || !then_origin.formals.is_empty()
            || !otherwise_origin.formals.is_empty()
            || (!same_literal
                && (!condition_origin.routes.is_empty() || !condition_origin.formals.is_empty()));
        let components = self.branch_components(&then_origin, &otherwise_origin, path);
        let mut result = self.combine(then_origin, otherwise_origin, path);
        let control = if same_literal {
            Origin::default()
        } else {
            self.append(condition_origin, "conditional_control", path)
        };
        result = self.combine(result, control.clone(), path);
        if had_known && result.routes.is_empty() && result.formals.is_empty() {
            return Origin::unknown();
        }
        result.components = components.map(|mut components| {
            match &mut components {
                Components::Indexed(values) => {
                    for value in values {
                        *value = self.combine(value.clone(), control.clone(), path);
                    }
                }
                Components::Named(values) => {
                    for (_, value) in values {
                        *value = self.combine(value.clone(), control.clone(), path);
                    }
                }
            }
            components
        });
        result
    }

    fn stateful_value_call(
        &mut self,
        callee: &'c StatefulCircuit,
        actuals: Vec<Origin>,
        path: &str,
        depth: usize,
    ) -> Origin {
        if actuals.len() != callee.parameters.len() {
            self.unavailable(path, "stateful value helper arity mismatch");
            return Origin::unknown();
        }
        if callee.parameters.len() > self.limits.max_nodes.min(self.limits.max_parameters) {
            self.unavailable(path, "parameter limit");
            return Origin::unknown();
        }
        if callee
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != callee.parameters.len()
        {
            self.unavailable(path, "duplicate stateful value helper parameter");
            return Origin::unknown();
        }
        if self.call_stack.contains(&callee.name.as_str()) {
            self.unavailable(path, "recursive stateful value helper");
            return Origin::unknown();
        }
        if self.call_stack.len() > self.limits.max_call_depth {
            self.unavailable(path, "stateful value helper call depth limit");
            return Origin::unknown();
        }
        if callee.actions.is_empty() {
            let call_body = format!("{path}.callee:{}", callee.name);
            if let Some(result) = self.cached_summary_result(
                &callee.name,
                &callee.parameters,
                &actuals,
                path,
                &call_body,
                depth,
            ) {
                return result;
            }
        }
        let mut callee_env = BTreeMap::new();
        for (formal, actual) in callee.parameters.iter().zip(actuals) {
            callee_env.insert(
                formal.name.clone(),
                self.append(actual, &format!("call_argument:{}", formal.name), path),
            );
        }
        self.call_stack.push(&callee.name);
        let previous_circuit = self.current_circuit;
        let previous_source = self.current_source;
        let previous_return_env = self.return_env.take();
        self.current_circuit = &callee.name;
        self.current_source = &callee.source;
        for (index, action) in callee.actions.iter().enumerate() {
            if self.nodes >= self.limits.max_nodes {
                self.unavailable(path, "action node limit");
                break;
            }
            self.action(
                action,
                &callee_env,
                &format!("{path}.callee:{}.actions[{index}]", callee.name),
                depth + 1,
                index + 1 == callee.actions.len(),
            );
        }
        let return_env = self.return_env.take().unwrap_or(callee_env);
        let result = match &callee.return_value {
            StateReturn::Expression { value } => self.visit(
                value,
                &return_env,
                &format!("{path}.callee:{}.return_value.value", callee.name),
                depth + 1,
            ),
            StateReturn::SetMember { value, .. } => self.ledger_query(
                value,
                &return_env,
                &format!("{path}.callee:{}.return_value", callee.name),
                depth + 1,
                "value",
                "set_membership",
            ),
            StateReturn::MapMember { key, .. } => self.ledger_query(
                key,
                &return_env,
                &format!("{path}.callee:{}.return_value", callee.name),
                depth + 1,
                "key",
                "map_membership",
            ),
            StateReturn::MapLookup { key, .. } => self.ledger_query(
                key,
                &return_env,
                &format!("{path}.callee:{}.return_value", callee.name),
                depth + 1,
                "key",
                "map_lookup_selection",
            ),
            StateReturn::Effectful { body } if callee.actions.is_empty() => self.return_plan(
                body,
                &return_env,
                &format!("{path}.callee:{}.return_value.body", callee.name),
                depth + 1,
            ),
            _ => {
                self.unavailable(path, "non-expression stateful value result unavailable");
                Origin::unknown()
            }
        };
        self.current_circuit = previous_circuit;
        self.current_source = previous_source;
        self.return_env = previous_return_env;
        self.call_stack.pop();
        self.append(result, &format!("call_return:{}", callee.name), path)
    }

    fn visit(
        &mut self,
        expr: &Expr,
        env: &BTreeMap<String, Origin>,
        path: &str,
        depth: usize,
    ) -> Origin {
        self.max_depth_seen = self.max_depth_seen.max(depth);
        self.max_call_stack_seen = self.max_call_stack_seen.max(self.call_stack.len());
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
            Expr::SetMember { value, .. } => {
                self.ledger_query(value, env, path, depth + 1, "value", "set_membership")
            }
            Expr::MapMember { key, .. } => {
                self.ledger_query(key, env, path, depth + 1, "key", "map_membership")
            }
            Expr::Tuple { elements } | Expr::Vector { elements, .. } => {
                if elements.len() > self.limits.max_nodes {
                    self.unavailable(path, "expression node limit");
                    return Origin::unknown();
                }
                if elements.len() >= self.limits.max_component_nodes {
                    self.unavailable(path, "aggregate component limit");
                    return Origin::unknown();
                }
                let mut values = Vec::with_capacity(elements.len());
                for (index, element) in elements.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        return Origin::unknown();
                    }
                    values.push(self.visit(
                        element,
                        env,
                        &format!("{path}.elements[{index}]"),
                        depth + 1,
                    ));
                }
                self.aggregate(Components::Indexed(values), path)
            }
            Expr::StructLiteral { ty, fields } => {
                let Type::Struct {
                    fields: declared, ..
                } = ty
                else {
                    self.unavailable(path, "struct literal shape unavailable");
                    return Origin::unknown();
                };
                if declared.len() != fields.len() {
                    self.unavailable(path, "struct literal arity mismatch");
                    return Origin::unknown();
                }
                if fields.len() > self.limits.max_nodes {
                    self.unavailable(path, "expression node limit");
                    return Origin::unknown();
                }
                if fields.len() >= self.limits.max_component_nodes {
                    self.unavailable(path, "aggregate component limit");
                    return Origin::unknown();
                }
                let mut values = Vec::with_capacity(fields.len());
                for (index, (field, declaration)) in fields.iter().zip(declared).enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        return Origin::unknown();
                    }
                    values.push((
                        declaration.name.clone(),
                        self.visit(field, env, &format!("{path}.fields[{index}]"), depth + 1),
                    ));
                }
                self.aggregate(Components::Named(values), path)
            }
            Expr::StructField {
                value,
                field,
                index,
            } => {
                let origin = self.visit(value, env, &child("value"), depth + 1);
                match origin.components {
                    Some(Components::Named(values))
                        if values.get(*index).is_some_and(|(name, _)| name == field) =>
                    {
                        self.append(
                            values[*index].1.clone(),
                            &format!("projection:{field}"),
                            path,
                        )
                    }
                    _ => {
                        self.unavailable(path, "struct projection shape unavailable");
                        Origin::unknown()
                    }
                }
            }
            Expr::TupleIndex { value, index } => {
                let origin = self.visit(value, env, &child("value"), depth + 1);
                match origin.components {
                    Some(Components::Indexed(values)) if *index < values.len() => {
                        self.append(values[*index].clone(), "tuple_projection", path)
                    }
                    _ => {
                        self.unavailable(path, "tuple projection shape unavailable");
                        Origin::unknown()
                    }
                }
            }
            Expr::Equal { left, right }
            | Expr::NotEqual { left, right }
            | Expr::Compare { left, right, .. } => {
                let left = self.visit(left, env, &child("left"), depth + 1);
                let right = self.visit(right, env, &child("right"), depth + 1);
                let combined = self.combine(left, right, path);
                self.append(combined, "comparison", path)
            }
            Expr::If {
                condition,
                then,
                otherwise,
            } => {
                let condition_origin = self.visit(condition, env, &child("condition"), depth + 1);
                let prior_control = self.control_origin.clone();
                let branch_control =
                    self.append(condition_origin.clone(), "branch_execution", path);
                self.control_origin = self.combine(prior_control.clone(), branch_control, path);
                self.conditions.push(format!("{path}:then"));
                let then_origin = self.visit(then, env, &child("then"), depth + 1);
                self.conditions.pop();
                self.conditions.push(format!("{path}:otherwise"));
                let otherwise_origin = self.visit(otherwise, env, &child("otherwise"), depth + 1);
                self.conditions.pop();
                self.control_origin = prior_control;
                let same_literal = same_simple_literal(then, otherwise);
                self.conditional_result(
                    condition_origin,
                    then_origin,
                    otherwise_origin,
                    same_literal,
                    path,
                )
            }
            Expr::Call { name, arguments } => {
                if self.ambiguous.contains(name.as_str()) {
                    self.unavailable(path, "ambiguous pure helper");
                    return Origin::unknown();
                }
                let pure = self.circuits.get(name.as_str()).copied();
                if pure.is_some() && self.stateful_circuits.contains_key(name.as_str()) {
                    self.unavailable(path, "ambiguous helper kind");
                    return Origin::unknown();
                }
                let stateful = if pure.is_none() {
                    if self.ambiguous_stateful.contains(name.as_str()) {
                        self.unavailable(path, "ambiguous stateful value helper");
                        return Origin::unknown();
                    }
                    self.stateful_circuits.get(name.as_str()).copied()
                } else {
                    None
                };
                self.saw_stateful_call |= stateful.is_some();
                let Some(formals) = pure
                    .map(|callee| &callee.parameters)
                    .or_else(|| stateful.map(|callee| &callee.parameters))
                else {
                    self.unavailable(path, "unknown pure helper");
                    return Origin::unknown();
                };
                if formals.len() > self.limits.max_nodes.min(self.limits.max_parameters) {
                    self.unavailable(path, "parameter limit");
                    return Origin::unknown();
                }
                if arguments.len() != formals.len() {
                    self.unavailable(
                        path,
                        if pure.is_some() {
                            "pure helper arity mismatch"
                        } else {
                            "stateful value helper arity mismatch"
                        },
                    );
                    return Origin::unknown();
                }
                if formals
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != formals.len()
                {
                    self.unavailable(
                        path,
                        if pure.is_some() {
                            "duplicate pure helper parameter"
                        } else {
                            "duplicate stateful value helper parameter"
                        },
                    );
                    return Origin::unknown();
                }
                if self.call_stack.contains(&name.as_str()) {
                    self.unavailable(
                        path,
                        if pure.is_some() {
                            "recursive pure helper"
                        } else {
                            "recursive stateful value helper"
                        },
                    );
                    return Origin::unknown();
                }
                if self.call_stack.len() > self.limits.max_call_depth {
                    self.unavailable(
                        path,
                        if pure.is_some() {
                            "pure helper call depth limit"
                        } else {
                            "stateful value helper call depth limit"
                        },
                    );
                    return Origin::unknown();
                }
                if stateful.is_some_and(|callee| {
                    !callee.actions.is_empty()
                        && matches!(callee.return_value, StateReturn::Effectful { .. })
                }) {
                    self.unavailable(path, "effectful return with separate actions");
                    return Origin::unknown();
                }
                let mut actuals = Vec::with_capacity(arguments.len().min(self.limits.max_nodes));
                for (index, argument) in arguments.iter().enumerate() {
                    if self.nodes >= self.limits.max_nodes {
                        self.unavailable(path, "expression node limit");
                        return Origin::unknown();
                    }
                    let argument_path = format!("{path}.arguments[{index}]");
                    actuals.push(self.visit(argument, env, &argument_path, depth + 1));
                }
                if self.ambiguous.contains(name.as_str()) {
                    self.unavailable(path, "ambiguous pure helper");
                    return Origin::unknown();
                }
                let Some(callee) = self.circuits.get(name.as_str()).copied() else {
                    if self.ambiguous_stateful.contains(name.as_str()) {
                        self.unavailable(path, "ambiguous stateful value helper");
                        return Origin::unknown();
                    }
                    if let Some(callee) = self.stateful_circuits.get(name.as_str()).copied() {
                        return self.stateful_value_call(callee, actuals, path, depth);
                    }
                    self.unavailable(path, "unknown pure helper");
                    return Origin::unknown();
                };
                if actuals.len() != callee.parameters.len() {
                    self.unavailable(path, "pure helper arity mismatch");
                    return Origin::unknown();
                }
                if callee
                    .parameters
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != callee.parameters.len()
                {
                    self.unavailable(path, "duplicate pure helper parameter");
                    return Origin::unknown();
                }
                if self.call_stack.contains(&callee.name.as_str()) {
                    self.unavailable(path, "recursive pure helper");
                    return Origin::unknown();
                }
                if self.call_stack.len() > self.limits.max_call_depth {
                    self.unavailable(path, "pure helper call depth limit");
                    return Origin::unknown();
                }
                let call_body = format!("{path}.callee:{}.body", callee.name);
                if let Some(result) = self.cached_summary_result(
                    &callee.name,
                    &callee.parameters,
                    &actuals,
                    path,
                    &call_body,
                    depth,
                ) {
                    return result;
                }
                let mut callee_env = BTreeMap::new();
                for (formal, actual) in callee.parameters.iter().zip(actuals) {
                    callee_env.insert(
                        formal.name.clone(),
                        self.append(actual, &format!("call_argument:{}", formal.name), path),
                    );
                }
                self.call_stack.push(&callee.name);
                let previous_circuit = self.current_circuit;
                let previous_source = self.current_source;
                self.current_circuit = &callee.name;
                self.current_source = &callee.source;
                let result = self.visit(
                    &callee.body,
                    &callee_env,
                    &format!("{path}.callee:{}.body", callee.name),
                    depth + 1,
                );
                self.current_circuit = previous_circuit;
                self.current_source = previous_source;
                self.call_stack.pop();
                self.append(result, &format!("call_return:{}", callee.name), path)
            }
            Expr::WitnessCall { arguments, .. } => {
                self.saw_assert_or_witness = true;
                let unavailable_before = self.report.unavailable.len();
                let mut known_argument = false;
                let mut parameters = BTreeSet::new();
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
                    known_argument |=
                        !argument_origin.routes.is_empty() || !argument_origin.formals.is_empty();
                    parameters.extend(argument_origin.parameters);
                }
                if known_argument || self.report.unavailable.len() > unavailable_before {
                    self.unavailable(path, "witness result provenance is unavailable");
                }
                let mut result = Origin::unknown();
                result.parameters = parameters;
                result
            }
            Expr::NativeWitnessCall { .. } | Expr::CellRead { .. } => Origin::unknown(),
            Expr::MapLookup { key, .. } => {
                self.ledger_query(key, env, path, depth + 1, "key", "map_lookup_selection")
            }
            Expr::TransientHash { value } | Expr::TransientCommit { value, opening: _ } => {
                // Retain upstream transient producers through this operation.
                let input = self.visit(value, env, &child("value"), depth + 1);
                let mut unknown = input.unknown;
                let mut parameters = input.parameters.clone();
                let mut upstream = self.append(input, "transient_value", path);
                if let Expr::TransientCommit { opening, .. } = expr {
                    let opening = self.visit(opening, env, &child("opening"), depth + 1);
                    unknown |= opening.unknown;
                    parameters.extend(opening.parameters.iter().cloned());
                    let opening = self.append(opening, "transient_opening", path);
                    upstream = self.combine(upstream, opening, path);
                }
                if self.limits.max_routes == 0 {
                    self.unavailable(path, "producer route limit");
                    return Origin::unknown();
                }
                let direct = Origin {
                    routes: vec![Route {
                        producer: Producer {
                            kind: if matches!(expr, Expr::TransientHash { .. }) {
                                ProducerKind::TransientHash
                            } else {
                                ProducerKind::TransientCommit
                            },
                            circuit: self.current_circuit.into(),
                            source: self.current_source.clone(),
                            path: path.into(),
                        },
                        dependencies: Vec::new(),
                    }],
                    formals: Vec::new(),
                    route_order: vec![RouteSlot::Known(0)],
                    parameters,
                    unknown,
                    components: None,
                };
                self.combine(direct, upstream, path)
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
            Expr::Assert { condition, .. } => {
                self.saw_assert_or_witness = true;
                let condition_origin = self.visit(condition, env, &child("condition"), depth + 1);
                if !condition_origin.routes.is_empty() || condition_origin.unknown {
                    self.unavailable(path, "assertion execution control unavailable");
                }
                Origin::default()
            }
            _ => {
                self.unavailable(path, "unsupported expression form");
                Origin::unknown()
            }
        }
    }
}

fn build_helper_summaries<'c>(
    contract: &'c Contract,
    circuits: &'c BTreeMap<&'c str, &'c PureCircuit>,
    ambiguous: &'c BTreeSet<&'c str>,
    stateful_circuits: &'c BTreeMap<&'c str, &'c StatefulCircuit>,
    ambiguous_stateful: &'c BTreeSet<&'c str>,
    limits: Limits,
) -> (BTreeMap<&'c str, HelperSummary>, Vec<HelperSummaryEvidence>) {
    let mut summaries = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut used_nodes = 0usize;
    for circuit in contract.circuits.iter().take(limits.max_circuits) {
        let remaining = limits.max_summary_nodes.saturating_sub(used_nodes);
        if remaining == 0 {
            break;
        }
        if ambiguous.contains(circuit.name.as_str())
            || stateful_circuits.contains_key(circuit.name.as_str())
            || circuit.parameters.len() > limits.max_nodes.min(limits.max_parameters)
            || circuit
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != circuit.parameters.len()
        {
            continue;
        }
        let mut scratch = Report::default();
        let mut analysis = CircuitAnalysis {
            name: &circuit.name,
            source: &circuit.source,
            current_circuit: &circuit.name,
            current_source: &circuit.source,
            circuits,
            ambiguous,
            stateful_circuits,
            ambiguous_stateful,
            summaries: None,
            call_stack: vec![&circuit.name],
            conditions: Vec::new(),
            control_origin: Origin::default(),
            return_env: None,
            limits: Limits {
                max_nodes: limits.max_nodes.min(remaining),
                ..limits
            },
            nodes: 0,
            max_depth_seen: 0,
            max_call_stack_seen: 1,
            saw_sink: false,
            saw_stateful_call: false,
            saw_assert_or_witness: false,
            report: &mut scratch,
        };
        let env = circuit
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| (parameter.name.clone(), Origin::formal(index)))
            .collect();
        let result = analysis.visit(&circuit.body, &env, "body", 0);
        let nodes = analysis.nodes;
        let max_depth = analysis.max_depth_seen;
        let max_call_stack = analysis.max_call_stack_seen;
        let eligible =
            !analysis.saw_sink && !analysis.saw_stateful_call && !analysis.saw_assert_or_witness;
        drop(analysis);
        used_nodes = used_nodes.saturating_add(nodes);
        if !eligible || !scratch.unavailable.is_empty() || scratch.truncated {
            continue;
        }
        let formal_dependencies = result
            .formals
            .iter()
            .filter_map(|formal| {
                circuit
                    .parameters
                    .get(formal.index)
                    .map(|parameter| FormalDependency {
                        parameter: parameter.name.clone(),
                        dependencies: formal.dependencies.clone(),
                    })
            })
            .collect();
        let mut producers = Vec::new();
        for route in &result.routes {
            if !producers.contains(&route.producer) {
                producers.push(route.producer.clone());
            }
        }
        evidence.push(HelperSummaryEvidence {
            circuit: circuit.name.clone(),
            source: circuit.source.clone(),
            formal_dependencies,
            producers,
            has_nonformal_unknown: result.unknown,
            nodes,
        });
        summaries.insert(
            circuit.name.as_str(),
            HelperSummary {
                result,
                nodes,
                max_depth,
                max_call_stack,
            },
        );
    }
    for circuit in contract
        .stateful_circuits
        .iter()
        .take(limits.max_circuits.saturating_sub(contract.circuits.len()))
    {
        let remaining = limits.max_summary_nodes.saturating_sub(used_nodes);
        if remaining == 0 {
            break;
        }
        if !circuit.actions.is_empty()
            || ambiguous_stateful.contains(circuit.name.as_str())
            || circuits.contains_key(circuit.name.as_str())
            || circuit.parameters.len() > limits.max_nodes.min(limits.max_parameters)
            || circuit
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != circuit.parameters.len()
        {
            continue;
        }
        let mut scratch = Report::default();
        let mut analysis = CircuitAnalysis {
            name: &circuit.name,
            source: &circuit.source,
            current_circuit: &circuit.name,
            current_source: &circuit.source,
            circuits,
            ambiguous,
            stateful_circuits,
            ambiguous_stateful,
            summaries: None,
            call_stack: vec![&circuit.name],
            conditions: Vec::new(),
            control_origin: Origin::default(),
            return_env: None,
            limits: Limits {
                max_nodes: limits.max_nodes.min(remaining),
                ..limits
            },
            nodes: 0,
            max_depth_seen: 0,
            max_call_stack_seen: 1,
            saw_sink: false,
            saw_stateful_call: false,
            saw_assert_or_witness: false,
            report: &mut scratch,
        };
        let env = circuit
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| (parameter.name.clone(), Origin::formal(index)))
            .collect();
        let result = match &circuit.return_value {
            StateReturn::Expression { value } => {
                analysis.visit(value, &env, "return_value.value", 0)
            }
            StateReturn::SetMember { value, .. } => {
                analysis.ledger_query(value, &env, "return_value", 0, "value", "set_membership")
            }
            StateReturn::MapMember { key, .. } => {
                analysis.ledger_query(key, &env, "return_value", 0, "key", "map_membership")
            }
            StateReturn::MapLookup { key, .. } => {
                analysis.ledger_query(key, &env, "return_value", 0, "key", "map_lookup_selection")
            }
            _ => continue,
        };
        let nodes = analysis.nodes;
        let max_depth = analysis.max_depth_seen;
        let max_call_stack = analysis.max_call_stack_seen;
        let eligible =
            !analysis.saw_sink && !analysis.saw_stateful_call && !analysis.saw_assert_or_witness;
        drop(analysis);
        used_nodes = used_nodes.saturating_add(nodes);
        if !eligible || !scratch.unavailable.is_empty() || scratch.truncated {
            continue;
        }
        let formal_dependencies = result
            .formals
            .iter()
            .filter_map(|formal| {
                circuit
                    .parameters
                    .get(formal.index)
                    .map(|parameter| FormalDependency {
                        parameter: parameter.name.clone(),
                        dependencies: formal.dependencies.clone(),
                    })
            })
            .collect();
        let mut producers = Vec::new();
        for route in &result.routes {
            if !producers.contains(&route.producer) {
                producers.push(route.producer.clone());
            }
        }
        evidence.push(HelperSummaryEvidence {
            circuit: circuit.name.clone(),
            source: circuit.source.clone(),
            formal_dependencies,
            producers,
            has_nonformal_unknown: result.unknown,
            nodes,
        });
        summaries.insert(
            circuit.name.as_str(),
            HelperSummary {
                result,
                nodes,
                max_depth,
                max_call_stack,
            },
        );
    }
    (summaries, evidence)
}

/// Analyze only explicitly selected typed IR. No rendering, capability, or ABI effect.
pub fn analyze(contract: &Contract, limits: Limits) -> Report {
    let mut report = Report::default();
    if contract.schema_version != SCHEMA_VERSION {
        push_unavailable(
            &mut report,
            limits,
            "contract",
            "schema_version",
            &format!(
                "unsupported typed IR schema {}; expected {SCHEMA_VERSION}",
                contract.schema_version
            ),
        );
        return report;
    }
    let mut circuits = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for circuit in contract.circuits.iter().take(limits.max_circuits) {
        if circuits.insert(circuit.name.as_str(), circuit).is_some() {
            ambiguous.insert(circuit.name.as_str());
        }
    }
    let mut stateful_circuits = BTreeMap::new();
    let mut ambiguous_stateful = BTreeSet::new();
    for circuit in contract.stateful_circuits.iter().take(limits.max_circuits) {
        if stateful_circuits
            .insert(circuit.name.as_str(), circuit)
            .is_some()
        {
            ambiguous_stateful.insert(circuit.name.as_str());
        }
    }
    let (summaries, summary_evidence) = build_helper_summaries(
        contract,
        &circuits,
        &ambiguous,
        &stateful_circuits,
        &ambiguous_stateful,
        limits,
    );
    report.helper_summaries = summary_evidence;
    let mut count = 0;
    for circuit in &contract.circuits {
        if count >= limits.max_circuits {
            push_unavailable(&mut report, limits, &circuit.name, "body", "circuit limit");
            report.truncated = true;
            return report;
        }
        count += 1;
        if circuit.parameters.len() > limits.max_nodes.min(limits.max_parameters) {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "parameter limit",
            );
            continue;
        }
        if circuit
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != circuit.parameters.len()
        {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "duplicate circuit parameter",
            );
            continue;
        }
        let mut analysis = CircuitAnalysis {
            name: &circuit.name,
            source: &circuit.source,
            current_circuit: &circuit.name,
            current_source: &circuit.source,
            circuits: &circuits,
            ambiguous: &ambiguous,
            stateful_circuits: &stateful_circuits,
            ambiguous_stateful: &ambiguous_stateful,
            summaries: Some(&summaries),
            call_stack: vec![&circuit.name],
            conditions: Vec::new(),
            control_origin: Origin::default(),
            return_env: None,
            limits,
            nodes: 0,
            max_depth_seen: 0,
            max_call_stack_seen: 1,
            saw_sink: false,
            saw_stateful_call: false,
            saw_assert_or_witness: false,
            report: &mut report,
        };
        let env = circuit
            .parameters
            .iter()
            .map(|p| (p.name.clone(), Origin::parameter(&circuit.name, &p.name)))
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
        if circuit.parameters.len() > limits.max_nodes.min(limits.max_parameters) {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "parameter limit",
            );
            continue;
        }
        if circuit
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != circuit.parameters.len()
        {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "parameters",
                "duplicate circuit parameter",
            );
            continue;
        }
        if !circuit.actions.is_empty()
            && matches!(circuit.return_value, StateReturn::Effectful { .. })
        {
            push_unavailable(
                &mut report,
                limits,
                &circuit.name,
                "return_value",
                "effectful return with separate actions",
            );
            continue;
        }
        {
            let mut analysis = CircuitAnalysis {
                name: &circuit.name,
                source: &circuit.source,
                current_circuit: &circuit.name,
                current_source: &circuit.source,
                circuits: &circuits,
                ambiguous: &ambiguous,
                stateful_circuits: &stateful_circuits,
                ambiguous_stateful: &ambiguous_stateful,
                summaries: Some(&summaries),
                call_stack: vec![&circuit.name],
                conditions: Vec::new(),
                control_origin: Origin::default(),
                return_env: None,
                limits,
                nodes: 0,
                max_depth_seen: 0,
                max_call_stack_seen: 1,
                saw_sink: false,
                saw_stateful_call: false,
                saw_assert_or_witness: false,
                report: &mut report,
            };
            let env = circuit
                .parameters
                .iter()
                .map(|p| (p.name.clone(), Origin::parameter(&circuit.name, &p.name)))
                .collect();
            for (index, action) in circuit.actions.iter().enumerate() {
                if analysis.nodes >= limits.max_nodes {
                    analysis.unavailable("actions", "action node limit");
                    break;
                }
                analysis.action(
                    action,
                    &env,
                    &format!("actions[{index}]"),
                    0,
                    index + 1 == circuit.actions.len(),
                );
            }
            let return_env = analysis.return_env.clone().unwrap_or_else(|| env.clone());
            match &circuit.return_value {
                StateReturn::Unit => {}
                StateReturn::Expression { value } => {
                    analysis.visit(value, &return_env, "return_value.value", 0);
                }
                StateReturn::Effectful { body } if circuit.actions.is_empty() => {
                    analysis.return_plan(body, &env, "return_value.body", 0);
                }
                StateReturn::Effectful { .. } => {
                    analysis.unavailable("return_value", "effectful return with separate actions")
                }
                StateReturn::SetMember { value, .. } => {
                    analysis.visit(value, &return_env, "return_value.value", 0);
                }
                StateReturn::MapMember { key, .. } | StateReturn::MapLookup { key, .. } => {
                    analysis.visit(key, &return_env, "return_value.key", 0);
                }
                StateReturn::HistoricMerkleCheckRoot { root, .. }
                | StateReturn::MerkleCheckRoot { root, .. } => {
                    analysis.visit(root, &return_env, "return_value.root", 0);
                }
                _ => {}
            }
        }
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
