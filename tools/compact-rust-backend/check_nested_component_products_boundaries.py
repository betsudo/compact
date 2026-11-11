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
        "outer_dimension": ["expression has type Vector", "length: 1", "length: 2"],
        "inner_dimension": ["expression has type Vector { element: Tuple { elements: [Tuple",
                            "expected Vector { element: Tuple { elements: [Tuple", "length: 1", "length: 2"],
        "nested_arity": ["expression has type Tuple", "expected Tuple", 'max: "65535"'],
        "tuple_to_scalar": ["expression has type Tuple", 'expected Unsigned { max: "65535" }'],
        "first_narrowing": ['expression has type Unsigned { max: "255" }', 'expected Unsigned { max: "1" }'],
        "second_narrowing": ['expression has type Unsigned { max: "65535" }', 'expected Unsigned { max: "255" }'],
        "boolean_leaf": ['expression has type Unsigned { max: "255" }', "expected Boolean"],
        "dead_branch": ["line 31 char 1", 'expression has type Unsigned { max: "255" }', "expected Boolean"],
        "static_empty": ["line 36 char 1", 'expression has type Unsigned { max: "255" }', "expected Boolean"],
        "source_binding": ["line 27 char 1", "expression has type Vector", 'max: "65535"', 'max: "255"'],
        "binding_type": ["line 27 char 1", "expression has type Vector", "expected Boolean"],
        "misplaced_action": ["unknown variant `cell_write`"],
    }

    def circuit(ir, name):
        return next(c for c in ir["stateful_circuits"] if c["name"] == name)

    def children(ty):
        return ty["element"]["elements"][0]["element"]["elements"]

    for name, expected in checks.items():
        positive = copy.deepcopy(original)
        base = ("branch" if name == "dead_branch" else "empty" if name == "static_empty"
                else "local" if name in {"source_binding", "binding_type"} else "parameter")
        c = circuit(positive, base)
        if name == "dead_branch":
            c["return_value"]["body"]["condition"] = {"kind": "boolean", "value": True}
            unreachable = c["return_value"]["body"]["otherwise"]["result"]["value"]
            unreachable["value"] = {"kind": "coerce", "value": unreachable["value"],
                                    "ty": copy.deepcopy(c["parameters"][1]["ty"])}
        malformed = copy.deepcopy(positive)
        c = circuit(malformed, base)
        body = c["return_value"]["body"]
        binding = first_local_binding(body) if base == "local" else None
        expr = (binding["value"] if binding else body["otherwise"]["result"]["value"]
                if base == "branch" else body["result"]["value"])
        if expr["kind"] != "coerce":
            raise ValueError("missing intended nested aggregate conversion")
        if name == "outer_dimension":
            expr["ty"]["length"] = 2
        elif name == "inner_dimension":
            expr["ty"]["element"]["elements"][0]["length"] = 2
        elif name == "nested_arity":
            children(expr["ty"])[0]["elements"].pop()
        elif name == "tuple_to_scalar":
            children(expr["ty"])[0] = {"kind": "unsigned", "max": "65535"}
        elif name == "first_narrowing":
            children(expr["ty"])[0]["elements"][0]["max"] = "1"
        elif name == "second_narrowing":
            children(expr["ty"])[0]["elements"][1]["max"] = "255"
        elif name in {"boolean_leaf", "static_empty"}:
            children(expr["ty"])[0]["elements"][0] = {"kind": "boolean"}
        elif name == "dead_branch":
            children(expr["value"]["ty"])[0]["elements"][0] = {"kind": "boolean"}
        elif name == "source_binding":
            argument = expr["value"]["arguments"][0]
            if argument["kind"] != "coerce":
                raise ValueError("missing supplied helper argument conversion")
            children(argument["ty"])[0]["elements"][0]["max"] = "65535"
        elif name == "binding_type":
            binding["ty"] = {"kind": "boolean"}
        else:
            c["return_value"] = {"kind": "expression", "value": {
                "kind": "cell_write", "field": "seen", "index": 0,
                "value": {"kind": "field_literal", "value": "1"}}}
        # Force actual Coerce checking, rather than a surrounding return mismatch.
        # In particular, length zero does not excuse an invalid child conversion.
        if name in {"outer_dimension", "inner_dimension", "nested_arity", "tuple_to_scalar",
                    "first_narrowing", "second_narrowing", "boolean_leaf", "static_empty"}:
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
    require(set(controls) == {"first_bound", "second_bound", "nested_arity", "component_dimension",
                             "outer_dimension", "result", "program", "context", "witness"},
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
                    scope=("whole private13export IR; schema decoding refusal only, no emitted Rust"
                           if name == "misplaced_action" else
                           "whole private13export IR; intended child/type/owner refusal, no emitted Rust; not source/proof acceptance"))

    # Preserve the refused side of both recursion boundaries in the maintained
    # current-head compiler gate, rather than merely hash-pinning unused sources.
    result["source_refusals"] = {}
    table = json.loads((fixtures / "scope-controls.json").read_text())
    require(set(table) == {"top_level_nested_tuple", "vector_inside_nested_product"}, "source refusal roster differs")
    historical = ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements"
    legacy = json.loads((historical / "scope-controls.json").read_text())["vector_inside_component_product"]
    controls = [(name, fixtures / "unsupported" / (name + ".compact"), spec) for name, spec in table.items()]
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
            scope="complete current source lanes; exact diagnostics on refusals and artifacts on reclassified success; not proof execution")

    family_source = ROOT / "examples/rust_backend/nested_component_products/nested_component_products.compact"
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
            scope="complete typed13export producer; unchanged consumer fails on first native lane, not all mutated routes")

    ts = output / "ts-results.json"
    wrong = json.loads(ts.read_text())
    next(row for row in wrong["rows"] if row["id"] == "parameter")["result"]["valueAtoms"][0][0] ^= 1
    wrong_path = output / "changed-expected-result.json"
    wrong_path.write_text(json.dumps(wrong, indent=2) + "\n")
    bind(wrong_path)
    environment["COMPACT_NESTED_COMPONENT_PRODUCTS_TS"] = str(wrong_path)
    try:
        stdout = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", base_consumer / "Cargo.toml",
            "--lib", "parameter", "--", "--exact"], expected_exit=101)
        require("test result: FAILED. 0 passed; 1 failed;" in stdout and "assertion `left == right` failed" in stdout,
                "controlled corrupt result table not consumed")
    finally:
        environment["COMPACT_NESTED_COMPONENT_PRODUCTS_TS"] = str(ts)
    result["changed_expected_result"] = dict(sha256=digest(wrong_path),
        scope="controlled corrupt expected result, not fresh TS execution or proof acceptance")
    return result
