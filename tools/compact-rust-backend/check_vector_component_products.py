#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Focused complete nested-product SDK API/static/ingress execution. Proof qualification is separate.

Requires a clean source and its original compiler recipe receipt. Every run writes
an absent evidence directory. Never edits a generated SDK or normalizes its IR.
"""
import argparse
import historical_source_licenses
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib

from check_stateful_product_vector_widening import (
    digest, inventory, reject_compiler_overrides, source_manifest_snapshot, first_local_binding,
)
from check_constant_vector_projection import compile_failure

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "vector_component_products"
SOURCES = ROOT / "examples/rust_backend/vector_component_products"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def evaluation_order_controls(irs):
    """Inspect the original source value and its one conversion at its lexical site.

    Identity casts on call arguments remain explicit. This structural control is
    separate from native/TS witness/error observations, not a formal equivalence proof.
    """
    def nodes(value):
        if isinstance(value, dict):
            yield value
            for child in value.values():
                yield from nodes(child)
        elif isinstance(value, list):
            for child in value:
                yield from nodes(child)

    observations = {}
    for name in ["helper", "local", "aliases", "checked_helper", "witness_before", "witness_after"]:
        ir = irs[name]
        circuit = next(c for c in ir["stateful_circuits"] if c["name"] == "run")
        body = circuit["return_value"]["body"]
        before_write = name in {"local", "aliases", "witness_before"}
        if before_write:
            binding_body, sequence = body, body["result"]
        elif name == "helper":
            binding_body, sequence = None, body
        else:
            binding_body, sequence = body["result"], body
        require(sequence["kind"] == "sequence" and len(sequence["actions"]) == 1,
                "original write sequence changed: " + name)
        action = sequence["actions"][0]
        writes = [n for n in nodes(action) if n.get("kind") == "cell_write"]
        require(len(writes) == 1 and writes[0]["field"] == "seen", "write identity changed: " + name)
        if binding_body is not None:
            require(binding_body["kind"] == "let" and len(binding_body["bindings"]) == 1,
                    "original local conversion binding changed: " + name)
            binding = binding_body["bindings"][0]
            require(binding["ty"] == circuit["result"], "converted local type changed: " + name)
            coerce = binding["value"]
            tail = sequence["result"] if before_write else binding_body["result"]
            require(tail == {"kind": "value", "value": {"kind": "parameter", "name": binding["name"]}},
                    "converted local was reevaluated or discarded: " + name)
        else:
            coerce = body["result"]["value"]
        require(coerce["kind"] == "coerce" and coerce["ty"] == circuit["result"],
                "original widening missing: " + name)
        value = coerce["value"]
        if name in {"helper", "local", "checked_helper"}:
            callee = "checked" if name == "checked_helper" else "keep"
            calls = [n for n in nodes(body) if n.get("kind") == "call"]
            require(len(calls) == 1 and calls[0] is value and value["name"] == callee,
                    "original helper evaluated more than once or replaced: " + name)
            declaration = next(c for c in ir["circuits"] if c["name"] == callee)
            original = declaration["result"]
        elif name.startswith("witness_"):
            calls = [n for n in nodes(body) if n.get("kind") == "witness_call"]
            require(len(calls) == 1 and calls[0] is value and value["name"] == "vector_value",
                    "typed original witness evaluated more than once or replaced: " + name)
            original = next(w for w in ir["witnesses"] if w["name"] == "vector_value")["result"]
        else:
            require(value == {"kind": "parameter", "name": "input"}, "original alias value changed")
            original = next(p for p in circuit["parameters"] if p["name"] == "input")["ty"]
        expected_original = {"kind": "vector", "length": 2, "element": {"kind": "tuple", "elements": [
            {"kind": "vector", "length": 2, "element": {"kind": "tuple", "elements": [
                {"kind": "unsigned", "max": "255"}, {"kind": "unsigned", "max": "255"}]}},
            {"kind": "unsigned", "max": "65535"}]}}
        require(original == expected_original, "original source bounds were widened before conversion: " + name)
        all_nodes = list(nodes(body))
        coerce_count = sum(n.get("kind") == "coerce" for n in all_nodes)
        require(coerce_count == ({"helper": 2, "local": 2, "checked_helper": 3}.get(name, 1)),
                "conversion structure duplicated or lost: " + name)
        require(sum(n.get("kind") == "cell_write" for n in all_nodes) == 1, "write duplicated: " + name)
        observations[name] = dict(original_type=original, coerce_count=coerce_count,
                                  conversion_position="before write" if before_write else "after write")
    return observations


def malformed_ir_controls(irs):
    """Private ingress controls; complete genuine SDKs remain unchanged."""
    checks = {
        "outer_dimension": ["expression has type Vector", "length: 2", "length: 3"],
        "inner_dimension": ["expression has type Vector", "length: 2", "length: 3"],
        "product_arity": ["expression has type Tuple", "expected Tuple", "Boolean"],
        "inner_leaf": ["expression has type Unsigned", "expected Boolean"],
        "inner_narrowing": ["expression has type Unsigned", 'max: "255"', 'max: "1"'],
        "input_type": ["expression has type Boolean", "expected Vector"],
        "binding_type": ["expression has type Vector", "expected Boolean"],
        "source_binding": ["expression has type Vector", 'max: "255"', 'max: "65535"'],
        "dead_branch": ["expression has type Unsigned", "expected Boolean"],
        "empty_body": ["expression has type Boolean", "expected Unsigned"],
        "inner_product": ["expression has type Tuple", "expected Tuple"],
        "second_leaf_narrowing": ["expression has type Unsigned", 'max: "255"', 'max: "1"'],
        "misplaced_action": ["cell_write"],
    }
    for name, expected in checks.items():
        base = ("local" if name in {"binding_type", "source_binding"} else
                "proof_candidate" if name == "dead_branch" else
                "empty_component" if name == "empty_body" else "parameter")
        positive = copy.deepcopy(irs[base])
        circuit = next(c for c in positive["stateful_circuits"] if c["name"] == "run")
        if name == "dead_branch":
            circuit["return_value"]["body"]["condition"] = {"kind": "boolean", "value": True}
            unreachable = circuit["return_value"]["body"]["otherwise"]["result"]["value"]
            unreachable["value"] = {"kind": "coerce", "value": unreachable["value"],
                                    "ty": copy.deepcopy(circuit["parameters"][0]["ty"])}
        malformed = copy.deepcopy(positive)
        circuit = next(c for c in malformed["stateful_circuits"] if c["name"] == "run")
        body = circuit["return_value"]["body"]
        binding = first_local_binding(body) if base == "local" else None
        expr = (binding["value"] if binding else
                body["otherwise"]["result"]["value"] if name == "dead_branch" else
                body["result"]["value"])
        require(expr["kind"] == "coerce", "missing intended aggregate conversion")
        inner = expr["ty"]["element"]["elements"][0]
        require(inner["kind"] == "vector", "missing intended Vector component")
        if name == "outer_dimension":
            expr["ty"]["length"] = 3
        elif name == "inner_dimension":
            inner["length"] = 3
        elif name == "product_arity":
            expr["ty"]["element"]["elements"].append({"kind": "boolean"})
        elif name == "inner_leaf":
            inner["element"]["elements"][1] = {"kind": "boolean"}
        elif name == "dead_branch":
            expr["value"]["ty"]["element"]["elements"][0]["element"]["elements"][1] = {"kind": "boolean"}
        elif name == "empty_body":
            circuit["parameters"][0]["ty"]["element"]["elements"][0]["element"]["elements"][1] = {"kind": "boolean"}
        elif name == "inner_narrowing":
            inner["element"]["elements"][0]["max"] = "1"
        elif name == "second_leaf_narrowing":
            inner["element"]["elements"][1]["max"] = "1"
        elif name == "input_type":
            circuit["parameters"][0]["ty"] = {"kind": "boolean"}
        elif name == "binding_type":
            binding["ty"] = {"kind": "boolean"}
        elif name == "source_binding":
            argument = expr["value"]["arguments"][0]
            require(argument["kind"] == "coerce", "missing supplied helper argument conversion")
            argument["ty"]["element"]["elements"][0]["element"]["elements"][0]["max"] = "65535"
        elif name == "inner_product":
            inner["element"] = {"kind": "tuple", "elements": [
                {"kind": "unsigned", "max": "65535"}]}
        else:
            circuit["return_value"] = {"kind": "expression", "value": {
                "kind": "cell_write", "field": "seen", "index": 0,
                "value": {"kind": "field_literal", "value": "1"}}}
        # Keep the declared result equal to the forged conversion target so a
        # contextual return-type mismatch cannot substitute for checking Coerce.
        if name in {"outer_dimension", "inner_dimension", "product_arity", "inner_leaf",
                    "inner_narrowing", "second_leaf_narrowing", "inner_product"}:
            circuit["result"] = copy.deepcopy(expr["ty"])
        yield name, positive, malformed, expected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "output must be absent")
    env = dict(os.environ)
    configuration_environment = dict(env)
    configuration = reject_compiler_overrides(configuration_environment)
    env.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "use Rust1.99.0")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "clean source required")
    build = json.loads(args.compiler_receipt.read_text())
    require(build["status"] == "passed" and build["head"] == head and build["tree"] == tree,
            "original recipe receipt must bind current clean HEAD/tree")
    compiler, scheme = [Path(build[k]["path"]).resolve(strict=True) for k in ["compactc", "compactc-scheme"]]
    require(compiler.parent == scheme.parent, "one original compiler package required")
    runtime = args.ts_runtime.resolve(strict=True)
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "flake.nix", ROOT / "flake.lock",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/scope-controls-compiler180.json",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/unsupported/product_below_component.compact",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/scope-controls.json",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/unsupported/nested_product_below_component.compact",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/unsupported/vector_inside_component_product.compact"]
    for directory in [ROOT / "compiler", ROOT / "runtime-rs", ROOT / "runtime-rs-macros",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES, SOURCES]:
        paths.extend(p for p in directory.rglob("*") if p.suffix in {".ss", ".rs", ".toml", ".json", ".mjs", ".compact"})
    paths.extend(Path(__file__).parent.glob("*.py"))
    paths.extend(historical_source_licenses.inputs())
    pins = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    tools = {str(p): digest(p) for p in [compiler, scheme, args.compiler_receipt.resolve(), Path(shutil.which("node")).resolve()]}
    for key, path in [("compactc", compiler), ("compactc-scheme", scheme)]:
        require(digest(path) == build[key]["sha256"], "compiler changed")
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    rust_tools = {}
    for name in ["cargo", "rustc", "rustfmt", "clippy-driver", "cargo-clippy"]:
        path = Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name], text=True).strip()).resolve(strict=True)
        rust_tools[name] = {"path": str(path), "sha256": digest(path)}
        tools[str(path)] = digest(path)
    tools[str(rustup)] = digest(rustup)
    env["PATH"] = str(Path(rust_tools["cargo"]["path"]).parent) + os.pathsep + env["PATH"]
    env["RUSTC"] = rust_tools["rustc"]["path"]
    output.mkdir(parents=True)
    report = dict(status="incomplete", head=head, tree=tree, commands=[], inputs=pins, tools=tools,
                  rust_tools=rust_tools, cargo_configuration_sha256=configuration,
                  environment={key: env.get(key) for key in ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS"]},
                  scope="Focused whole unedited SDK API execution and strict root compilation, including checked helper and witness errors. Marked static misuses and independent twins execute; 13 malformed native ingress controls and valid twins execute; Two exact complete outside-scope refusals execute; Three whole typed producer mutations and a controlled result-oracle corruption execute; model/proof/Linux/integration gates remain open.")
    report["build_inputs_before_use"] = {}
    report["resolved_locks"] = {}
    root_packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    root_versions = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in root_packages}
    allowed_local_packages = {("vector-component-products-consumer", "0.1.0")}

    def bind_build_inputs(paths):
        for path in paths:
            key = str(path)
            sha = digest(path)
            require(key not in report["build_inputs_before_use"] or report["build_inputs_before_use"][key] == sha,
                    "build input changed before reuse: " + key)
            report["build_inputs_before_use"][key] = sha

    def bind_lock(path):
        packages = tomllib.loads(path.read_text())["package"]
        for package in packages:
            key = (package["name"], package["version"], package.get("source"))
            if package.get("source") is not None:
                require(key in root_versions and package.get("checksum") == root_versions[key],
                        "resolved registry/Git package differs from original root lock")
            else:
                require(key in root_versions or key[:2] in allowed_local_packages,
                        "unexpected resolved local package")
        bind_build_inputs([path])
        report["resolved_locks"][str(path)] = {"sha256": digest(path), "packages": packages}

    def check_bound_inputs():
        for path, sha in report["build_inputs_before_use"].items():
            require(digest(path) == sha, "bound build input changed before Cargo use: " + path)
    start = time.monotonic()

    def run(label, command, *, success=True, stdin=None):
        command = list(map(str, command))
        if command[0] == "cargo":
            check_bound_inputs()
        began = time.monotonic()
        try:
            p = subprocess.run(command, cwd=ROOT, env=env, input=stdin, capture_output=True, text=True, timeout=300)
        except subprocess.TimeoutExpired as error:
            for suffix, value in [("stdout", error.stdout), ("stderr", error.stderr)]:
                (output / f"{label}.{suffix}").write_bytes(
                    value if isinstance(value, bytes) else (value or "").encode())
            report["commands"].append(dict(label=label, command=command, timeout=True))
            raise
        for suffix in ["stdout", "stderr"]:
            (output / f"{label}.{suffix}").write_text(getattr(p, suffix))
        report["commands"].append(dict(label=label, command=command, exit=p.returncode, seconds=time.monotonic() - began))
        require(not success or p.returncode == 0, label + " failed: retained diagnostics")
        return p

    try:
        historical = ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements"
        original_hash = "6fb741f1171b87312d8e9e0815c148e7d2f010cfcecfefd135f819a5d26df07c"
        require(digest(historical / "scope-controls-compiler180.json") ==
                "04f33133d10a0e1be883450e7968682ccde4b860e73239aa7638a31c5863b4a3",
                "literal historical refusal table changed")
        old_scope = json.loads((historical / "scope-controls-compiler180.json").read_text())["product_below_component"]
        require(old_scope["source_sha256"] == original_hash
                and old_scope["lanes"]["rust"]["exit"] == 255,
                "complete original source/refusal authority changed")
        historical_source_licenses.verify_source(historical / "unsupported/product_below_component.compact", original_hash)
        historical_source_licenses.verify_source(SOURCES / "original_regression.compact", original_hash)
        definitions = json.loads((FIXTURES / "sources.json").read_text())
        cases = json.loads((FIXTURES / "cases.json").read_text())
        names = {s["source"] for s in definitions}
        require(len(names) == len(definitions) == 24
                and names == {p.stem for p in SOURCES.glob("*.compact")}, "whole source roster differs")
        require(len(cases) == len({r["id"] for r in cases}) == 35
                and {r["source"] for r in cases} == names, "case roster differs")
        for spec in definitions:
            require({r["entry"] for r in cases if r["source"] == spec["source"]}
                    == set(spec["entries"]), "an export has no declared execution case")
        generated = output / "generated"
        snapshots = {}
        irs = {}
        report["sources"] = {}
        for spec in definitions:
            name = spec["source"]
            target = generated / name
            source = (SOURCES / (name + ".compact")).relative_to(ROOT)
            run("generate-" + name, [compiler, "--target", "rust", "--target", "ts", "--skip-zk", "--rust-runtime-root", ROOT,
                *(["--rust-functional-ledger8", "--rust-require-recording"] if spec["stateful"] else []), source, target])
            run("independent-" + name, [scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk", source, output / "independent" / name])
            require((target / "contract/compact-rust-ir.json").read_bytes() ==
                    (output / "independent" / name / "contract/compact-rust-ir.json").read_bytes(), "independent IR differs: " + name)
            irs[name] = json.loads((target / "contract/compact-rust-ir.json").read_text())
            programs = inventory(target / "contract/checked-programs")
            require(len(programs) == (len(spec["entries"]) if spec["stateful"] else 0), "Program count differs")
            caps = json.loads((target / "contract/rust-capabilities.json").read_text())["circuits"]
            if spec["stateful"]:
                require({c["name"] for c in caps} == set(spec["entries"]) and
                        all(c["recorded"] and c["observed_call"] and c["proof_required"] is True for c in caps), "required APIs missing")
            else:
                require(not caps, "pure source acquired proof APIs")
            report["sources"][name] = dict(source_sha256=digest(ROOT / source), manifest=source_manifest_snapshot(target, spec["stateful"]), programs=programs)
            snapshots[name] = inventory(target)
        report["evaluation_order"] = evaluation_order_controls(irs)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        ts = output / "ts-results.json"
        run("ts", ["node", FIXTURES / "capture.mjs", "--generated", generated, "--runtime", runtime,
                   "--version", "0.16.102", "--output", ts])
        reference = json.loads(ts.read_text())
        require(reference["status"] == "passed" and len(reference["rows"]) == len(cases), "TS case count differs")
        consumer = output / "consumer"
        consumer.mkdir()
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "consumer.rs")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        dependencies = []
        for spec in definitions:
            name = spec["source"]
            path = generated / name / "contract"
            package = tomllib.loads((path / "Cargo.toml").read_text())["package"]["name"]
            allowed_local_packages.add((package, "0.1.0"))
            dependencies.append(name + '={package=' + json.dumps(package) + ',path=' + json.dumps(str(path)) + ',features=["ledger-transaction"]}')
        manifest = consumer / "Cargo.toml"
        manifest.write_text('[workspace]\n[package]\nname="vector-component-products-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + '\n'.join(dependencies) + '\n')
        env["COMPACT_VECTOR_COMPONENT_PRODUCTS_TS"] = str(ts)
        report["generated_inputs_before_build"] = {name: inventory(generated / name) for name in names}
        bind_build_inputs([manifest, consumer / "consumer.rs", ts])
        run("resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
        bind_lock(consumer / "Cargo.lock")
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--lib"])
        require(f"test result: ok. {len(cases)} passed; 0 failed; 0 ignored;" in result.stdout, "all Rust cases must execute")
        run("consumer-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest, "--lib", "--tests", "--", "-D", "warnings"])
        report["strict_sdk_roots"] = {}
        for spec in definitions:
            name = spec["source"]
            target = output / "strict-sdk-roots" / name
            shutil.copytree(generated / name / "contract", target)
            before = inventory(target)
            shutil.copyfile(consumer / "Cargo.lock", target / "Cargo.lock")
            bind_build_inputs([target / "Cargo.toml"])
            run(name + "-root-resolve", ["cargo", "metadata", "--offline", "--manifest-path", target / "Cargo.toml", "--format-version", "1"])
            bind_lock(target / "Cargo.lock")
            bind_build_inputs([target / "Cargo.toml"])
            run(name + "-root-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", target / "Cargo.toml", "--all-targets", "--all-features", "--", "-D", "warnings"])
            require(all(digest(target / path) == sha for path, sha in before.items()), "root Clippy edited emitted files")
            report["strict_sdk_roots"][name] = before
        report["compile_fail"] = {}
        controls = json.loads((FIXTURES / "compile-fail.json").read_text())
        require(len(controls) == 14 and set(controls) | {"consumer"} == {p.stem for p in FIXTURES.glob("*.rs")},
                "ordinary misuse roster differs")
        require({spec["twin"] for spec in controls.values()} ==
                {str(p.relative_to(FIXTURES)) for p in (FIXTURES / "compile-pass").glob("*.rs")},
                "positive twin roster differs")
        bins = consumer / "src/bin"
        bins.mkdir(parents=True)
        for name, spec in controls.items():
            source = (FIXTURES / (name + ".rs")).read_text()
            twin = (FIXTURES / spec["twin"]).read_text()
            require("// expected-error" not in twin and "c::run(context(), input())" in twin,
                    "positive twin must retain an actual correct-type call")
            positive = bins / (name + "_twin.rs")
            positive.write_text(twin)
            bind_build_inputs([positive])
            run(name + "-correct-twin", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", name + "_twin"])
            negative = bins / (name + ".rs")
            negative.write_text(source)
            bind_build_inputs([negative])
            rejected = run(name + "-misuse", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                           "--bin", name, "--message-format=json"], success=False)
            report["compile_fail"][name] = {"source_sha256": digest(negative), "twin_sha256": digest(positive),
                "errors": compile_failure(rejected, name, spec["code"], source)}
        report["native_ir_controls"] = {}
        ir_folder = output / "native-ir-controls"
        ir_folder.mkdir()
        backend = ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "compact-rust-backend", "--bin", "compact-rust-backend"]
        for name, positive, malformed, expected in malformed_ir_controls(irs):
            original_path = ir_folder / (name + "-positive.json")
            original_path.write_text(json.dumps(positive, indent=2) + "\n")
            bind_build_inputs([original_path])
            positive_bytes = original_path.read_text()
            run(name + "-valid-ir", backend, stdin=positive_bytes)
            rejected_path = ir_folder / (name + "-malformed.json")
            rejected_path.write_text(json.dumps(malformed, indent=2) + "\n")
            bind_build_inputs([rejected_path])
            rejected = run(name + "-malformed-ir", backend, success=False, stdin=rejected_path.read_text())
            require(rejected.returncode != 0 and not rejected.stdout and
                    all(part in rejected.stderr for part in expected), "wrong malformed-IR refusal: " + name)
            report["native_ir_controls"][name] = {"positive_sha256": digest(original_path),
                "malformed_sha256": digest(rejected_path), "stderr": rejected.stderr,
                "scope": "native untrusted JSON ingress, with a separate valid private-IR twin; not proof acceptance"}
        report["producer_mutations"] = {}
        target_type = "Vector<2,[Vector<2,[Uint<16>,Uint<16>]>,Uint<32>]>"
        before = f"const wide:{target_type} = disclose(vector_value());seen=1;"
        after = f"seen=1;const wide:{target_type} = disclose(vector_value());"
        for name, old, new in [("helper", "return input;", "return [input[1],input[0]];"),
                               ("witness_before", before, after), ("witness_after", after, before)]:
            original = SOURCES / (name + ".compact")
            text = original.read_text()
            require(text.count(old) == 1, "complete producer mutation premise differs: " + name)
            folder = output / "producer-mutations" / name
            folder.mkdir(parents=True)
            changed_source = folder / (name + ".compact")
            changed_source.write_text(text.replace(old, new))
            bind_build_inputs([changed_source])
            changed = folder / "generated"
            run(name + "-mutant-generate", [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", ROOT,
                "--rust-functional-ledger8", "--rust-require-recording", changed_source, changed])
            changed_manifest = source_manifest_snapshot(changed, True)
            changed_snapshot = inventory(changed)
            changed_consumer = folder / "consumer"
            changed_consumer.mkdir()
            shutil.copyfile(consumer / "consumer.rs", changed_consumer / "consumer.rs")
            shutil.copyfile(consumer / "Cargo.lock", changed_consumer / "Cargo.lock")
            text = manifest.read_text()
            original_path = json.dumps(str(generated / name / "contract"))
            require(text.count(original_path) == 1, "producer dependency membership differs")
            changed_manifest_path = changed_consumer / "Cargo.toml"
            changed_manifest_path.write_text(text.replace(original_path, json.dumps(str(changed / "contract"))))
            bind_build_inputs([changed_source, changed_manifest_path, changed_consumer / "consumer.rs"])
            for relative in changed_snapshot:
                bind_build_inputs([changed / relative])
            run(name + "-mutant-resolve", ["cargo", "metadata", "--offline", "--manifest-path", changed_manifest_path, "--format-version", "1"])
            bind_lock(changed_consumer / "Cargo.lock")
            rejected = run(name + "-mutant-consume", ["cargo", "test", "--locked", "--offline", "--manifest-path", changed_manifest_path,
                           "--lib", name, "--", "--exact"], success=False)
            require(rejected.returncode != 0 and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                    and "assertion `left == right` failed" in rejected.stdout, "changed producer not consumed: " + name)
            require(name == "helper" or "initializer witness order" in rejected.stdout, "wrong producer refusal")
            require(inventory(changed) == changed_snapshot, "changed generated producer edited")
            report["producer_mutations"][name] = {"original_source_sha256": digest(original),
                "changed_source_sha256": digest(changed_source), "manifest": changed_manifest,
                "generated_files": changed_snapshot,
                "scope": "complete typed producer; unchanged old consumer rejects on first native lane, not all mutated routes"}
        wrong = copy.deepcopy(reference)
        next(row for row in wrong["rows"] if row["id"] == "parameter")["result"]["valueAtoms"][0][0] ^= 1
        wrong_path = output / "changed-expected-result.json"
        wrong_path.write_text(json.dumps(wrong) + "\n")
        bind_build_inputs([wrong_path])
        env["COMPACT_VECTOR_COMPONENT_PRODUCTS_TS"] = str(wrong_path)
        rejected = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                       "--lib", "parameter", "--", "--exact"], success=False)
        require(rejected.returncode != 0 and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                and "assertion `left == right` failed" in rejected.stdout, "changed expected result not consumed")
        report["changed_expected_result"] = {"sha256": digest(wrong_path),
            "scope": "controlled corrupt expected-result table, not a new genuine TS verdict, typed proof result or proof acceptance"}
        env["COMPACT_VECTOR_COMPONENT_PRODUCTS_TS"] = str(ts)
        report["scope_refusals"] = {}
        require(digest(historical / "scope-controls-compiler184.json") == "c495f390d053c4a1e6d8709da3d08a05de5b81b534dfbd2707205a892abb46fc",
                "literal compiler184 scope authority changed")
        scope_table = json.loads((historical / "scope-controls.json").read_text())
        for name in ["nested_product_below_component", "vector_inside_component_product"]:
            expected = scope_table[name]
            source = historical / "unsupported" / (name + ".compact")
            historical_source_licenses.verify_source(source, expected["source_sha256"])
            for lane, outcome in expected["lanes"].items():
                command = ([scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk"] if lane == "ir"
                           else [compiler, "--target", lane, "--skip-zk"])
                if lane == "rust":
                    command += ["--rust-runtime-root", ROOT, "--rust-functional-ledger8", "--rust-require-recording"]
                rejected = run("scope-" + name + "-" + lane,
                               command + [source.relative_to(ROOT), output / ("scope-" + name + "-" + lane)],
                               success=outcome["exit"] == 0)
                require(rejected.returncode == outcome["exit"], "scope outcome changed")
                if outcome["exit"]:
                    require(rejected.stdout == outcome["stdout"] and rejected.stderr == outcome["stderr"],
                            "complete scope refusal changed")
            report["scope_refusals"][name] = expected
        for name, before in snapshots.items():
            require(inventory(generated / name) == before, "generated artifact changed: " + name)
        for path, sha in pins.items():
            require(digest(ROOT / path) == sha, "source changed: " + path)
        for path, sha in tools.items():
            require(digest(path) == sha, "tool changed: " + path)
        for path, sha in report["build_inputs_before_use"].items():
            require(digest(path) == sha, "resolved build input changed: " + path)
        require(reject_compiler_overrides(configuration_environment) == configuration, "Cargo configuration changed")
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source HEAD changed")
        pure_routes = sum(not next(s["stateful"] for s in definitions if s["source"] == row["source"]) for row in cases)
        error_cases = sum("error" in row for row in cases)
        report.update(status="passed", rust_tests=len(cases), stateful_routes=4*(len(cases)-pure_routes),
                      successful_stateful_routes=4*(len(cases)-pure_routes-error_cases),
                      expected_error_routes=4*error_cases, public_transcript_comparisons=2*(len(cases)-pure_routes-error_cases),
                      pure_routes=pure_routes, generated_hashes=snapshots)
    except Exception as error:
        report.update(status="failed", error=str(error))
        raise
    finally:
        report["seconds"] = time.monotonic() - start
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(dict(status=report["status"], head=head, commands=len(report["commands"]), error=report.get("error"))))


if __name__ == "__main__":
    main()
