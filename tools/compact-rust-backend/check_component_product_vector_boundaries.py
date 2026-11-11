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

"""Negative controls for the complete nested component product API gate.

Private malformed IR and complete changed producers are deliberately invalid
controls, never new source/proof authority. Genuine generated SDKs stay unedited.
"""
import copy
import json
from pathlib import Path
import shutil
import subprocess

from check_constant_vector_projection import compile_failure
from check_stateful_product_vector_widening import first_local_binding, source_manifest_snapshot

ROOT = Path(__file__).resolve().parents[2]


def malformed_ir_controls(original):
    checks = {
        "outer_dimension": ["expression has type Vector", "expected Vector", "length: 1", "length: 2"],
        "component_dimension": ["expression has type Vector", "expected Vector", "length: 1", "length: 2"],
        "leaf_dimension": ["expression has type Vector", "expected Vector", "length: 2", "length: 3"],
        "component_arity": ["expression has type Tuple", "expected Tuple"],
        "vector_to_scalar": ["expression has type Vector", 'expected Unsigned { max: "65535" }'],
        "leaf_narrowing": ['expression has type Unsigned { max: "255" }', 'expected Unsigned { max: "1" }'],
        "middle_narrowing": ['expression has type Unsigned { max: "255" }', 'expected Unsigned { max: "15" }'],
        "boolean_leaf": ['expression has type Unsigned { max: "255" }', "expected Boolean"],
        "dead_branch": ["line 31 char 1", 'expression has type Unsigned { max: "255" }', "expected Boolean"],
        "static_empty": ["line 36 char 1", 'expression has type Unsigned { max: "255" }', "expected Boolean"],
        "static_empty_leaf": ["line 40 char 1", 'expression has type Unsigned { max: "255" }', "expected Boolean"],
        "source_binding": ["line 27 char 1", "expression has type Vector", 'max: "65535"', 'max: "255"'],
        "binding_type": ["line 27 char 1", "expression has type Vector", "expected Boolean"],
        "misplaced_action": ["unknown variant `cell_write`"],
    }
    def circuit(ir, name):
        return next(c for c in ir["stateful_circuits"] if c["name"] == name)
    def component(ty):
        return ty["element"]["elements"][0]
    def fields(ty):
        return component(ty)["element"]["elements"]
    def leaf(ty):
        return fields(ty)[0]["element"]
    for name, expected in checks.items():
        positive = copy.deepcopy(original)
        base = ("branch" if name == "dead_branch" else "empty" if name == "static_empty"
                else "empty_leaf" if name == "static_empty_leaf"
                else "local" if name in {"source_binding", "binding_type"} else "parameter")
        c = circuit(positive, base)
        if name == "dead_branch":
            c["return_value"]["body"]["condition"] = {"kind": "boolean", "value": True}
            expr = c["return_value"]["body"]["otherwise"]["result"]["value"]
            expr["value"] = {"kind": "coerce", "value": expr["value"], "ty": copy.deepcopy(c["parameters"][1]["ty"])}
        malformed = copy.deepcopy(positive)
        c = circuit(malformed, base)
        body = c["return_value"]["body"]
        binding = first_local_binding(body) if base == "local" else None
        expr = (binding["value"] if binding else body["otherwise"]["result"]["value"]
                if base == "branch" else body["result"]["value"])
        if expr["kind"] != "coerce":
            raise ValueError("missing intended Vector/product conversion")
        if name == "outer_dimension": expr["ty"]["length"] = 2
        elif name == "component_dimension": component(expr["ty"])["length"] = 2
        elif name == "leaf_dimension": fields(expr["ty"])[0]["length"] = 3
        elif name == "component_arity": fields(expr["ty"]).pop()
        elif name == "vector_to_scalar": fields(expr["ty"])[0] = {"kind":"unsigned", "max":"65535"}
        elif name == "leaf_narrowing": leaf(expr["ty"])["max"] = "1"
        elif name == "middle_narrowing": fields(expr["ty"])[1]["max"] = "15"
        elif name in {"boolean_leaf", "static_empty", "static_empty_leaf"}: fields(expr["ty"])[0]["element"] = {"kind":"boolean"}
        elif name == "dead_branch": fields(expr["value"]["ty"])[0]["element"] = {"kind":"boolean"}
        elif name == "source_binding":
            argument = expr["value"]["arguments"][0]
            if argument["kind"] != "coerce": raise ValueError("missing supplied helper conversion")
            leaf(argument["ty"])["max"] = "65535"
        elif name == "binding_type": binding["ty"] = {"kind":"boolean"}
        else: c["return_value"] = {"kind":"expression", "value":{"kind":"cell_write", "field":"seen", "index":0, "value":{"kind":"field_literal", "value":"1"}}}
        if name in {"outer_dimension", "component_dimension", "leaf_dimension", "component_arity", "vector_to_scalar", "leaf_narrowing", "middle_narrowing", "boolean_leaf", "static_empty", "static_empty_leaf"}:
            c["result"] = copy.deepcopy(expr["ty"])
        yield name, positive, malformed, expected


def qualify(output, fixtures, compiler, environment, run, lock, bind, digest, require):
    """Called only after the main gate has accepted fresh whole-source behavior."""
    result = {}
    base_consumer = output / "consumer"
    consumer = output / "boundary-consumer"
    consumer.mkdir()
    for name in ["consumer.rs", "Cargo.toml", "Cargo.lock"]:
        shutil.copyfile(base_consumer / name, consumer / name)
        bind(consumer / name)
    manifest = consumer / "Cargo.toml"
    bins = consumer / "src/bin"
    bins.mkdir(parents=True)
    controls = json.loads((fixtures / "boundary/controls.json").read_text())
    require(set(controls) == {"leaf_bound", "middle_bound", "outer_bound", "leaf_dimension", "component_dimension",
                             "outer_dimension", "component_arity", "tuple_instead_vector", "result", "program", "context", "witness"},
            "marked control roster differs")
    result["static"] = {}
    for name, spec in controls.items():
        twin = spec["twin"]
        require(twin == name + "_twin.rs", "matching positive filename differs")
        for filename in [name + ".rs", twin]:
            shutil.copyfile(fixtures / "boundary" / filename, bins / filename)
            bind(bins / filename)
        run(name + "-twin", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                            "--bin", name + "_twin"])
        out = run(name + "-misuse", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                                    "--bin", name, "--message-format=json"], expected_exit=101)
        error = subprocess.CompletedProcess([], 101, stdout=out)
        result["static"][name] = dict(source_sha256=digest(bins / (name + ".rs")),
                                     twin_sha256=digest(bins / twin), scope=spec["scope"],
                                     errors=compile_failure(error, name, spec["code"], (bins / (name + ".rs")).read_text()))
    result["native_ingress"] = {}
    ir_folder = output / "private-ir-controls"
    ir_folder.mkdir()
    original = json.loads((output / "independent/family/contract/compact-rust-ir.json").read_text())
    backend = ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "compact-rust-backend", "--bin", "compact-rust-backend"]
    for name, positive, malformed, expected in malformed_ir_controls(original):
        for label, ir in [("positive", positive), ("malformed", malformed)]:
            path = ir_folder / (name + "-" + label + ".json")
            path.write_text(json.dumps(ir, indent=2) + "\n")
            bind(path)
            # Structured argv and quoted positional arguments preserve the stdin
            # input under the shared bounded POSIX process-group owner.
            command = ["bash", "-c", 'ir_input_file=$1; shift; exec "$@" < "$ir_input_file"',
                       "native-ir", path, *backend]
            stdout = run(name + "-" + label + "-ir", command, expected_exit=0 if label == "positive" else 1)
            if label == "positive":
                require("pub mod ledger_contract" in stdout, "valid IR twin did not render")
            else:
                stderr = (output / (name + "-malformed-ir.stderr")).read_text()
                require(not stdout and all(part in stderr for part in expected), "wrong native IR refusal: " + name)
                result["native_ingress"][name] = dict(positive_sha256=digest(ir_folder / (name + "-positive.json")),
                    malformed_sha256=digest(path), stderr=stderr,
                    scope=("whole private14export IR; schema decoding refusal only, no emitted Rust"
                           if name == "misplaced_action" else
                           "whole private14export IR; intended child/type/owner refusal, no emitted Rust; not source/proof acceptance"))

    # Preserve the refused side of both recursion boundaries in the maintained
    # current-head compiler gate, rather than merely hash-pinning unused sources.
    result["source_refusals"] = {}
    previous = ROOT / "tools/compact-rust-backend/tests/nested_component_products"
    require(digest(previous / "scope-controls-compiler186.json") == "6bc3b02c74083bdd443c9945d6360d5357b70050e35b8917ce6a995108f7136d", "literal nested compiler186 table changed")
    historical = ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements"
    require(digest(historical / "scope-controls-compiler186.json") == "619d74bd560ddb5879be700e38cdba57fb18eb5e69c7ff9ca3c61fad7e28a9a3", "literal direct compiler186 table changed")
    table = json.loads((previous / "scope-controls.json").read_text())
    require(set(table) == {"top_level_nested_tuple", "vector_inside_nested_product"}, "source refusal roster differs")
    historical = ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements"
    legacy = json.loads((historical / "scope-controls.json").read_text())["vector_inside_component_product"]
    controls = [(name, previous / "unsupported" / (name + ".compact"), spec) for name, spec in table.items()]
    controls.append(("vector_inside_component_product", historical / "unsupported/vector_inside_component_product.compact", legacy))
    for name, source, spec in controls:
        require(digest(source) == spec["source_sha256"], "complete refused source identity differs")
        for lane in ["ts", "rust", "ir"]:
            folder = output / "source-refusals" / (name + "-" + lane)
            command = ([compiler.with_name("compactc-scheme"), "--emit-rust-ir", "--skip-ts", "--skip-zk"]
                       if lane == "ir" else [compiler, "--target", lane, "--skip-zk"] +
                       (["--rust-runtime-root", ROOT, "--rust-functional-ledger9", "--rust-require-recording"] if lane == "rust" else []))
            label = name + "-" + lane + "-scope"
            expected = spec["lanes"][lane]
            stdout = run(label, [*command, source, folder], expected_exit=expected["exit"])
            if lane != "ts":
                if expected["exit"]:
                    require(stdout == expected["stdout"] and (output / (label + ".stderr")).read_text() == expected["stderr"], "source refusal diagnostic differs: " + label)
                    require(not (folder / "contract/lib.rs").exists() and not (folder / "contract/compact-rust-ir.json").exists(), "refused source published Rust/IR")
                else:
                    artifact = "lib.rs" if lane == "rust" else "compact-rust-ir.json"
                    require((folder / "contract" / artifact).is_file(), "accepted original omitted artifact: " + label)
        result["source_refusals"][name] = dict(source_sha256=digest(source), lanes=spec["lanes"],
            scope="complete current source lane checks, exact diagnostics on refusals; accepted original API is independently exercised in the native cohort; not proof execution")

    family_source = ROOT / "examples/rust_backend/component_product_vector_widening/component_product_vector_widening.compact"
    source_text = family_source.read_text()
    mutations = {
        "helper": ("pure circuit keep(input: Narrow): Narrow { return input; }",
                   "pure circuit keep(input: Narrow): Narrow { return [[[[[0, input[0][0][0][0][1]], input[0][0][0][1]]], input[0][1]]]; }"),
        "witness_before": ("const wide: Wide = disclose(vector_value()); seen = 5; return wide;",
                           "seen = 5; const wide: Wide = disclose(vector_value()); return wide;"),
        "witness_after": ("seen = 6; const wide: Wide = disclose(vector_value()); return wide;",
                          "const wide: Wide = disclose(vector_value()); seen = 6; return wide;"),
    }
    result["producer_mutations"] = {}
    for name, (before, after) in mutations.items():
        require(source_text.count(before) == 1, "producer mutation site differs")
        folder = output / "producer-mutations" / name
        folder.mkdir(parents=True)
        source = folder / family_source.name
        source.write_text(source_text.replace(before, after))
        bind(source)
        generated = folder / "generated"
        run(name + "-mutant-generate", [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", ROOT,
            "--rust-functional-ledger9", "--rust-require-recording", source, generated])
        snapshot = {str(p.relative_to(generated)): digest(p) for p in generated.rglob("*") if p.is_file()}
        for path in snapshot:
            bind(generated / path)
        changed = folder / "consumer"
        changed.mkdir()
        for filename in ["consumer.rs", "Cargo.lock"]:
            shutil.copyfile(base_consumer / filename, changed / filename)
        old = json.dumps(str(output / "generated/family/contract"))
        text = (base_consumer / "Cargo.toml").read_text()
        require(text.count(old) == 1, "producer dependency membership differs")
        changed_manifest = changed / "Cargo.toml"
        changed_manifest.write_text(text.replace(old, json.dumps(str(generated / "contract"))))
        bind(changed_manifest)
        bind(changed / "consumer.rs")
        run(name + "-mutant-resolve", ["cargo", "metadata", "--offline", "--manifest-path", changed_manifest, "--format-version", "1"])
        lock(changed / "Cargo.lock")
        stdout = run(name + "-mutant-consume", ["cargo", "test", "--locked", "--offline", "--manifest-path", changed_manifest,
            "--lib", name, "--", "--exact"], expected_exit=101)
        require("test result: FAILED. 0 passed; 1 failed;" in stdout and "assertion `left == right` failed" in stdout,
                "changed complete producer was not consumed: " + name)
        require(name == "helper" or "nested witness order" in stdout, "wrong witness producer refusal")
        require(all(digest(generated / path) == sha for path, sha in snapshot.items()), "changed producer edited after generation")
        result["producer_mutations"][name] = dict(source_sha256=digest(source),
            source_manifest=source_manifest_snapshot(generated, True), raw_files=snapshot,
            scope="complete typed14export producer; unchanged consumer fails on first native lane, not all mutated routes")

    ts = output / "ts-results.json"
    wrong = json.loads(ts.read_text())
    next(row for row in wrong["rows"] if row["id"] == "parameter")["result"]["valueAtoms"][0][0] ^= 1
    wrong_path = output / "changed-expected-result.json"
    wrong_path.write_text(json.dumps(wrong, indent=2) + "\n")
    bind(wrong_path)
    environment["COMPACT_COMPONENT_PRODUCT_VECTOR_TS"] = str(wrong_path)
    try:
        stdout = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", base_consumer / "Cargo.toml",
            "--lib", "parameter", "--", "--exact"], expected_exit=101)
        require("test result: FAILED. 0 passed; 1 failed;" in stdout and "assertion `left == right` failed" in stdout,
                "controlled corrupt result table not consumed")
    finally:
        environment["COMPACT_COMPONENT_PRODUCT_VECTOR_TS"] = str(ts)
    result["changed_expected_result"] = dict(sha256=digest(wrong_path),
        scope="controlled corrupt expected result, not fresh TS execution or proof acceptance")
    return result
