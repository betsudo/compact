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

"""Private whole-IR and complete changed-producer controls, never source authority."""
import copy
import json
import shutil

from check_stateful_product_vector_widening import first_local_binding, source_manifest_snapshot


def malformed_ir_controls(original):
    def circuit(ir, name):
        return next(c for c in ir["stateful_circuits"] if c["name"] == name)

    def children(ty):
        return ty["element"]["elements"]

    def fields(ty):
        return children(ty)[0]["elements"]

    names = ["outer_dimension", "outer_arity", "inner_arity", "tuple_vector_dimension",
             "leaf_narrowing", "second_narrowing", "boolean_leaf", "foreign_identity",
             "field_narrowing", "dead_branch", "static_empty", "static_empty_leaf",
             "empty_leaf_dimension", "empty_tuple_arity", "source_binding", "binding_type",
             "canonical_bound", "resource_bound", "misplaced_action"]
    for name in names:
        base = ("branch" if name == "dead_branch" else "empty" if name == "static_empty"
                else "empty_leaf" if name in {"static_empty_leaf", "empty_leaf_dimension"}
                else "tuple_empty" if name == "empty_tuple_arity"
                else "field_identity" if name == "field_narrowing"
                else "local" if name in {"source_binding", "binding_type"} else "parameter")
        positive = copy.deepcopy(original)
        if name == "dead_branch":
            circuit(positive, base)["return_value"]["body"]["condition"] = {"kind": "boolean", "value": True}
        if name == "tuple_vector_dimension":
            # The existing explicit checked relation permits a nonempty tuple
            # to a same-length Vector. Preserve that law as the positive twin.
            c = circuit(positive, base)
            expr = c["return_value"]["body"]["result"]["value"]
            children(expr["ty"])[0] = {"kind": "vector", "length": 1,
                                      "element": {"kind": "unsigned", "max": "65535"}}
            c["result"] = copy.deepcopy(expr["ty"])
        malformed = copy.deepcopy(positive)
        c = circuit(malformed, base)
        body = c["return_value"]["body"]
        binding = first_local_binding(body) if base == "local" else None
        expr = (binding["value"] if binding else body["otherwise"]["result"]["value"]
                if base == "branch" else body["result"]["value"])
        if expr["kind"] != "coerce":
            raise ValueError("missing original conversion: " + name)
        expected = ["expression has type", "expected"]
        if name == "outer_dimension":
            expr["ty"]["length"] += 1
            expected += ["Vector", "length: 2", "length: 3"]
        elif name == "outer_arity":
            children(expr["ty"]).pop()
            expected += ["Tuple"]
        elif name == "inner_arity":
            fields(expr["ty"]).clear()
            expected += ["Tuple"]
        elif name == "tuple_vector_dimension":
            children(expr["ty"])[0] = {"kind": "vector", "length": 2, "element": {"kind": "unsigned", "max": "65535"}}
            expected += ["Tuple", "Vector"]
        elif name == "leaf_narrowing":
            fields(expr["ty"])[0]["max"] = "1"
            expected += ['max: "255"', 'max: "1"']
        elif name == "second_narrowing":
            children(expr["ty"])[1]["max"] = "15"
            expected += ['max: "255"', 'max: "15"']
        elif name in {"boolean_leaf", "static_empty"}:
            fields(expr["ty"])[0] = {"kind": "boolean"}
            expected += ["Boolean"]
        elif name == "foreign_identity":
            children(expr["ty"])[0] = {"kind": "struct", "name": "Foreign", "fields": [
                {"name": "value", "ty": {"kind": "unsigned", "max": "65535"}}]}
            expected += ["Struct", "Foreign"]
        elif name == "field_narrowing":
            fields(expr["ty"])[0] = {"kind": "unsigned", "max": "65535"}
            expected += ["Field", 'Unsigned { max: "65535" }']
        elif name == "dead_branch":
            argument = expr["value"]["arguments"][0]
            if argument["kind"] != "coerce":
                raise ValueError("missing original helper operand conversion")
            fields(argument["ty"])[0] = {"kind": "boolean"}
            expected += ["Boolean"]
        elif name == "static_empty_leaf":
            fields(expr["ty"])[0]["element"] = {"kind": "boolean"}
            expected += ["Boolean"]
        elif name == "empty_leaf_dimension":
            fields(expr["ty"])[0]["length"] = 1
            expected += ["length: 0", "length: 1"]
        elif name == "empty_tuple_arity":
            # The real frontend canonicalizes this source's empty tuple to
            # Unit. A nonempty tuple target cannot be invented from that leaf.
            if fields(expr["ty"])[0] != {"kind": "unit"}:
                raise ValueError("empty tuple source transport changed")
            fields(expr["ty"])[0] = {"kind": "tuple", "elements": [{"kind": "unsigned", "max": "65535"}]}
            expected += ["Unit", "Tuple"]
        elif name == "source_binding":
            argument = expr["value"]["arguments"][0]
            if argument["kind"] != "coerce":
                raise ValueError("missing original helper operand conversion")
            fields(argument["ty"])[0]["max"] = "65535"
            expected += ['max: "65535"', 'max: "255"']
        elif name == "binding_type":
            binding["ty"] = {"kind": "boolean"}
            expected += ["Boolean"]
        elif name in {"canonical_bound", "resource_bound"}:
            fields(expr["ty"])[0]["max"] = "00" if name == "canonical_bound" else "9" * 262145
            expected = ["canonical decimal"] if name == "canonical_bound" else ["compiler resource string_bytes exceeds 262144 (observed 263859)"]
        else:
            c["return_value"] = {"kind": "expression", "value": {"kind": "cell_write",
                "field": "seen", "index": 0, "value": {"kind": "field_literal", "value": "1"}}}
            expected = ["unknown variant", "cell_write"]
        if name not in {"dead_branch", "source_binding", "binding_type", "misplaced_action"}:
            c["result"] = copy.deepcopy(expr["ty"])
        # Type failures must retain the original complete source owner. Schema
        # and resource preflight can refuse before source location is decoded.
        if name not in {"misplaced_action", "resource_bound", "canonical_bound"}:
            expected += [f"line {c['source']['line']} char {c['source']['column']}"]
        yield name, positive, malformed, expected


def qualify(output, root, compiler, env, run, lock, bind, digest, require):
    result = {"native_ingress": {}, "producer_mutations": {}}
    controls = output / "private-ir-controls"
    controls.mkdir()
    original = json.loads((output / "independent/family/contract/compact-rust-ir.json").read_bytes())
    backend = ["cargo", "run", "--locked", "--offline", "--quiet", "-p",
               "compact-rust-backend", "--bin", "compact-rust-backend"]
    for name, positive, malformed, expected in malformed_ir_controls(original):
        paths = {}
        for label, ir in [("positive", positive), ("malformed", malformed)]:
            path = controls / (name + "-" + label + ".json")
            path.write_text(json.dumps(ir, indent=2) + "\n")
            bind(path)
            paths[label] = digest(path)
            command = ["bash", "-c", 'ir_input_file=$1; shift; exec "$@" < "$ir_input_file"',
                       "private-ir", path, *backend]
            stdout = run(name + "-" + label + "-ir", command,
                         expected_exit=0 if label == "positive" else 1)
            if label == "positive":
                require("pub mod ledger_contract" in stdout, "valid private IR did not render: " + name)
            else:
                diagnostic = (output / (name + "-malformed-ir.stderr")).read_bytes().decode()
                require(not stdout and all(part.lower() in diagnostic.lower() for part in expected),
                        "wrong private native IR refusal: " + name)
                result["native_ingress"][name] = {"inputs": paths, "stderr": diagnostic,
                    "scope": "whole private15export IR and valid twin; no source/proof authority"}
    require(len(result["native_ingress"]) == 19, "private IR control roster differs")

    consumer = output / "consumer"
    source = root / "examples/rust_backend/outer_vector_product_widening/outer_vector_product_widening.compact"
    original_text = source.read_text()
    mutations = {
        "helper": ("pure circuit keep(input: Narrow): Narrow { return input; }",
                   "pure circuit keep(input: Narrow): Narrow { return [[[0], input[0][1]], [input[1][0], input[1][1]]]; }"),
        "witness_before": ("const wide: Wide = disclose(product_value()); seen = 5; return wide;",
                           "seen = 5; const wide: Wide = disclose(product_value()); return wide;"),
        "witness_after": ("seen = 6; const wide: Wide = disclose(product_value()); return wide;",
                          "const wide: Wide = disclose(product_value()); seen = 6; return wide;"),
    }
    for name, (before, after) in mutations.items():
        require(original_text.count(before) == 1, "changed producer site differs")
        directory = output / "producer-mutations" / name
        directory.mkdir(parents=True)
        changed_source = directory / source.name
        changed_source.write_text(original_text.replace(before, after))
        bind(changed_source)
        generated = directory / "generated"
        run(name + "-mutant-generate", [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", root,
            "--rust-functional-ledger8", "--rust-require-recording", changed_source, generated])
        files = {str(p.relative_to(generated)): digest(p) for p in generated.rglob("*") if p.is_file()}
        for path in files:
            bind(generated / path)
        changed = directory / "consumer"
        changed.mkdir()
        for filename in ["consumer.rs", "Cargo.lock"]:
            shutil.copyfile(consumer / filename, changed / filename)
        old = json.dumps(str(output / "generated/family/contract"))
        manifest = (consumer / "Cargo.toml").read_text()
        require(manifest.count(old) == 1, "changed producer dependency differs")
        changed_manifest = changed / "Cargo.toml"
        changed_manifest.write_text(manifest.replace(old, json.dumps(str(generated / "contract"))))
        bind(changed_manifest)
        bind(changed / "consumer.rs")
        run(name + "-mutant-resolve", ["cargo", "metadata", "--offline", "--manifest-path", changed_manifest, "--format-version", "1"])
        lock(changed / "Cargo.lock")
        stdout = run(name + "-mutant-consume", ["cargo", "test", "--locked", "--offline", "--manifest-path", changed_manifest,
            "--lib", name, "--", "--exact"], expected_exit=101)
        require("test result: FAILED. 0 passed; 1 failed;" in stdout and "assertion `left == right` failed" in stdout,
                "unchanged consumer did not detect the changed complete producer: " + name)
        require(name == "helper" or "nested witness order" in stdout, "wrong changed witness refusal")
        result["producer_mutations"][name] = {"source_sha256": digest(changed_source),
            "manifest": source_manifest_snapshot(generated, True), "raw_files": files,
            "scope": "complete typed15export producer; unchanged consumer refuses on first native lane, not all mutated routes"}

    ts = output / "ts-results.json"
    changed = json.loads(ts.read_bytes())
    next(row for row in changed["rows"] if row["id"] == "parameter")["result"]["valueAtoms"][0][0] ^= 1
    path = output / "changed-expected-result.json"
    path.write_text(json.dumps(changed, indent=2) + "\n")
    bind(path)
    env["COMPACT_OUTER_VECTOR_PRODUCT_TS"] = str(path)
    try:
        stdout = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", consumer / "Cargo.toml",
            "--lib", "parameter", "--", "--exact"], expected_exit=101)
        require("test result: FAILED. 0 passed; 1 failed;" in stdout and "assertion `left == right` failed" in stdout,
                "controlled changed expected result was not consumed")
    finally:
        env["COMPACT_OUTER_VECTOR_PRODUCT_TS"] = str(ts)
    result["changed_expected_result"] = {"sha256": digest(path), "scope": "controlled corrupt expectation, not fresh TS/proof authority"}
    return result
