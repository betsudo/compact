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

"""Finite complete-source tuple scope expectations, not an IR/type checker.

The normal compiler importers retain IR, type and origin authority. These
assertions pin characterized conditions and their original callback ranges;
generation does not establish Rust compilation, execution or proof acceptance.
"""
import copy
import json
from pathlib import Path

from check_constructor_equality_scope import original_origins
from check_stateful_product_vector_widening import digest, require

FIXTURES = Path(__file__).with_name("tests") / "constructor_equality_tuple_guards"
ROLES = ("aliased_tuple_variable", "aliased_uint_component", "aliased_vector_variable",
         "asymmetric_eq", "direct_tuple_variable", "direct_vector_variable", "flat_eq",
         "flat_ne", "hash_tuple_argument", "mixed_literal_eq", "nested_eq",
         "owned_capture_eq", "reversed_eq", "struct_literal_field", "tuple_to_vector",
         "uint_to_field_tuple", "vector_to_tuple", "widened_vector_capture")
RAW_REFUSALS = {"aliased_tuple_variable": "Rust backend does not yet support this cast"}
RUST_REFUSALS = {
    **RAW_REFUSALS,
    "aliased_uint_component": 'expression has type Tuple { elements: [Unsigned { max: "65535" }, Unsigned { max: "65535" }] }, expected Tuple { elements: [Unsigned { max: "255" }, Unsigned { max: "255" }] }',
}


def expectations():
    rows = json.loads((FIXTURES / "scope_expectations.json").read_bytes())
    require(set(rows) == set(ROLES), "complete tuple scope roster differs")
    sources = {str(p.relative_to(FIXTURES)) for p in FIXTURES.rglob("*.compact")}
    require({row["path"] for row in rows.values()} == sources,
            "complete tuple source membership differs")
    for row in rows.values():
        source = FIXTURES / row["path"]
        require(source.resolve().is_relative_to(FIXTURES.resolve()) and
                digest(source) == row["source_sha256"], "tuple scope source bytes differ")
    return rows


def at(value, path):
    for key in path:
        value = value[key]
    return value


def validate(role, ir, origins, source, expected):
    """Exact finite shapes and physical original origins, with no type inference."""
    require(digest(source) == expected["source_sha256"], "tuple scope source bytes differ")
    mapped, addresses = original_origins(ir, origins, source)
    ctor = ir["constructor"]
    require(ctor["parameters"] == expected["parameters"], "tuple original parameters differ: " + role)
    loops = [s for s in ctor["steps"] if s["kind"] == "for_each_vector"]
    require(len(loops) == len(expected["loops"]), "tuple complete loop count differs: " + role)
    for loop, wanted in zip(loops, expected["loops"]):
        require({k: loop[k] for k in ("binding", "source", "length")} ==
                {k: wanted[k] for k in ("binding", "source", "length")},
                "tuple original loop differs: " + role)
        require(len(loop["steps"]) == 1 and loop["steps"][0]["kind"] == "if" and
                loop["steps"][0]["condition"] == wanted["condition"],
                "tuple condition transport differs: " + role)
    require([c["name"] for c in ir["stateful_circuits"]] == expected["exports"] ==
            ["read", "advance", "advance_other"], "tuple intact export roster differs")
    contents = source.read_bytes()

    def physical(path, wanted):
        origin = mapped.get(tuple(path))
        require(origin is not None, "tuple original origin missing")
        require(origin[1:] == [wanted["origin_status"], wanted["origin_role"]],
                "tuple original origin authority differs")
        span = origins["spans"][origin[0]]
        require(len(span) == 8 and span[7] == {"kind": "physical_source"} and
                span[:7] == wanted["physical_span"], "tuple original physical span differs")
        require(type(span[3]) is int and type(span[6]) is int and
                0 <= span[3] < span[6] <= len(contents) and
                contents[span[3]:span[6]].decode("utf-8") == wanted["source_fragment"],
                "tuple original source fragment differs")

    for wanted in expected["condition_origins"]:
        path = wanted["path"]
        require(at(ir, path)["kind"] == wanted["kind"], "tuple original node kind differs")
        physical(path, wanted)
        if wanted["kind"] in ("coerce", "field_cast"):
            # An aggregate cast can span the original whole comparison while
            # its value retains the narrower variable use-site. Pin both;
            # never manufacture a shared range or copy the parent's range.
            physical([*path, "value"], wanted["original_value_origin"])
    return dict(raw_origin_addresses=addresses, loops=len(loops),
                original_condition_origins=len(expected["condition_origins"]),
                rust_generation="current-typed-refusal" if role in RUST_REFUSALS else "supported-generation")


def host_controls(cases, output, expected):
    """Mutate actual completed producer copies; never compile/execute HOST IR."""
    def condition(ir, index=0):
        loops = [s for s in ir["constructor"]["steps"] if s["kind"] == "for_each_vector"]
        return loops[index]["steps"][0]["condition"]

    def first(ir):
        return condition(ir)["left"]["elements"][0]

    def change_origin(raw, *, missing=False):
        index = next(i for i, row in enumerate(raw["addresses"]) if row[2] == "coerce")
        if missing:
            child = next(row for row in raw["addresses"] if row[0] == index and row[1] == {"field": "value"})
            child[3] = None
        else:
            target = raw["addresses"][index][3]
            root = next(row[3] for row in raw["addresses"] if row[2] == "equal")
            raw["origins"][target][0] = raw["origins"][root][0]

    controls = (
        ("missing-component-coerce", "flat_eq", lambda ir, raw: condition(ir)["left"]["elements"].__setitem__(0, first(ir)["value"]), "scope origin kind differs"),
        ("wrong-component-bound", "flat_eq", lambda ir, raw: first(ir)["ty"].update(max="255"), "tuple condition transport differs"),
        ("reversed-operand-loss", "reversed_eq", lambda ir, raw: condition(ir).update(left=condition(ir)["right"], right=condition(ir)["left"]), "scope origin kind differs"),
        ("nested-component-loss", "nested_eq", lambda ir, raw: condition(ir)["left"]["elements"].__setitem__(1, first(ir)), "scope origin kind differs"),
        ("wrong-capture-loop-length", "owned_capture_eq", lambda ir, raw: next(s for s in ir["constructor"]["steps"] if s["kind"] == "for_each_vector").update(length=3), "tuple original loop differs"),
        ("wrong-original-parameter", "direct_vector_variable", lambda ir, raw: ir["constructor"]["parameters"][0]["ty"].update(length=3), "tuple original parameters differ"),
        ("copied-condition-range", "flat_eq", lambda ir, raw: change_origin(raw), "tuple original physical span differs"),
        ("missing-original-value-origin", "flat_eq", lambda ir, raw: change_origin(raw, missing=True), "tuple original origin missing"),
    )
    output.mkdir()
    results = {}
    for label, role, mutate, message in controls:
        original_ir, original_raw, source = cases[role]
        ir, raw = copy.deepcopy(original_ir), copy.deepcopy(original_raw)
        mutate(ir, raw)
        try:
            validate(role, ir, raw, source, expected[role])
        except ValueError as error:
            require(message in str(error), "unrelated HOST tuple refusal: " + label)
        else:
            raise ValueError("HOST tuple scope mutation accepted: " + label)
        path = output / (label + ".json")
        path.write_text(json.dumps(dict(scope="HOST-only, never executed", ir=ir, origins=raw), indent=2) + "\n")
        results[label] = dict(scope_assertion_refused=True, diagnostic=message, sha256=digest(path))
    return results
