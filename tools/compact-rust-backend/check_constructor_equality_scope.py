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

"""Pin genuine constructor-condition coercion scope and current refusal boundaries.

Source/IR/origin and generation checks only, not SDK execution or proofs.
The implicit mixed-width tuple case requires current component coercions;
complete compiler209 mismatch fixtures separately retain historical refusals.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import time

from check_passport_current_profile import bounded_command, COMMAND_TIMEOUT_SECONDS
from check_stateful_product_vector_widening import digest, inventory, reject_compiler_overrides, require

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "constructor_equality_boundaries" / "scope"
ROLES = ("ordinary_after_condition", "initializer_after_condition", "literal_condition",
         "nested_explicit_tuple_condition", "narrowing_condition", "nested_tuple_condition")
REFUSALS = {"narrowing_condition": 'constructor can initialize root Cells from parameters only: "if"'}


def unsigned(maximum):
    return {"kind": "unsigned", "max": str(maximum)}


def coerce():
    return {"kind": "coerce", "value": {"kind": "parameter", "name": "item"}, "ty": unsigned(65535)}


def original_origins(ir, origins, source):
    """Walk original producer addresses; callers add their finite assertions."""
    require(ir["schema_version"] == 20 and origins["schema_version"] == 3, "IR/origin schema differs")
    require(len(origins["files"]) == 1 and Path(origins["files"][0]["path"]).resolve() == source.resolve(),
            "scope source origin file differs")
    require(all(len(row) == 3 and type(row[0]) is int and 0 <= row[0] < len(origins["spans"])
                for row in origins["origins"]), "scope origin span index differs")
    nodes, paths, mapped = [], [], {}
    for parent, edge, kind, origin in origins["addresses"]:
        if parent is None:
            require(not nodes and edge == {"root": True}, "scope origin root differs")
            node, path = ir, ()
        else:
            require(type(parent) is int and 0 <= parent < len(nodes), "scope origin parent differs")
            key = edge.get("field", edge.get("index"))
            node, path = nodes[parent][key], paths[parent] + (key,)
        require(kind is None or isinstance(node, dict) and node.get("kind") == kind,
                "scope origin kind differs from original IR")
        require(path not in mapped, "duplicate scope origin address")
        require(origin is None or type(origin) is int and 0 <= origin < len(origins["origins"]),
                "scope origin index differs")
        mapped[path] = None if origin is None else origins["origins"][origin]
        nodes.append(node)
        paths.append(path)
    return mapped, len(nodes)


def validate_scope(role, ir, origins, source):
    """Finite source regression assertions, not a replacement IR/origin importer."""
    mapped, node_count = original_origins(ir, origins, source)
    ctor = ir["constructor"]
    loops = [(i, s) for i, s in enumerate(ctor["steps"]) if s["kind"] == "for_each_vector"]
    require(len(loops) == 1, "one complete scope loop required")
    index, loop = loops[0]
    require(loop["length"] == 2 and loop["source"] == {"kind": "parameter", "name": "input"}, "scope loop source differs")
    require(len(loop["steps"]) == 1 and loop["steps"][0]["kind"] == "if", "one original guard required")
    guard = loop["steps"][0]
    condition = guard["condition"]
    maximum = 65535 if role == "narrowing_condition" else 255
    require(loop["binding"] == {"name": "item", "ty": unsigned(maximum)}, "original loop bound differs")
    require(ctor["parameters"][0] == {"name": "input", "ty": {"kind": "vector", "length": 2, "element": unsigned(maximum)}}, "original input bound differs")
    right_type = ({"kind": "tuple", "elements": [unsigned(65535), unsigned(65535)]}
                  if role.startswith("nested_") else unsigned(255 if role == "narrowing_condition" else 65535))
    require(ctor["parameters"][1] == {"name": "right", "ty": right_type}, "original right bound differs")
    expected = {"kind": "equal", "left": coerce(), "right": {"kind": "parameter", "name": "right"}}
    if role == "literal_condition":
        expected.update(left={"kind": "parameter", "name": "item"},
                        right={"kind": "unsigned_literal", "value": "2", "max": "255"})
    elif role == "narrowing_condition":
        expected["left"] = {"kind": "unsigned_cast", "max": "255", "value": {"kind": "parameter", "name": "item"}}
    elif role in ("nested_tuple_condition", "nested_explicit_tuple_condition"):
        expected["left"] = {"kind": "tuple", "elements": [coerce(), coerce()]}
    require(condition == expected, "scope condition transport differs: " + role)
    if role == "ordinary_after_condition":
        functions = [c for c in ir["circuits"] if c["name"] == "widen"]
        require(len(functions) == 1 and functions[0]["body"] == {
            "kind": "unsigned_cast", "max": "65535", "value": {"kind": "parameter", "name": "value"}},
            "ordinary widening transport changed")
    if role == "initializer_after_condition":
        bindings = [b for s in guard["then_steps"] if s["kind"] == "let" for b in s["bindings"]
                    if b["ty"] == unsigned(65535)]
        require(len(bindings) == 1 and bindings[0]["value"] == {
            "kind": "unsigned_cast", "max": "65535", "value": {"kind": "parameter", "name": "item"}},
            "initializer widening transport changed")
    prefix = ("constructor", "steps", index, "steps", 0, "condition")
    require(mapped.get(prefix) is not None, "scope guard lost original origin")
    return {"raw_origin_addresses": node_count, "original_input_max": maximum,
            "rust_generation": "current-typed-refusal" if role in REFUSALS else "supported-generation",
            "implicit_mixed_tuple_gap_open": False}


def host_controls(cases, output):
    """Mutate copies of actual raw outputs; never execute these HOST objects."""
    def loop(ir):
        return next(s for s in ir["constructor"]["steps"] if s["kind"] == "for_each_vector")

    def condition(ir):
        return loop(ir)["steps"][0]["condition"]

    def initializer(ir):
        return next(b for s in loop(ir)["steps"][0]["then_steps"] if s["kind"] == "let"
                    for b in s["bindings"] if b["ty"] == unsigned(65535))["value"]

    def ordinary(ir):
        return next(c for c in ir["circuits"] if c["name"] == "widen")["body"]

    controls = (
        ("wrong-guard-bound", "ordinary_after_condition",
         lambda ir, raw: condition(ir)["left"]["ty"].update(max="255"), "scope condition transport differs"),
        ("wrong-ordinary-bound", "ordinary_after_condition",
         lambda ir, raw: ordinary(ir).update(max="255"), "ordinary widening transport changed"),
        ("wrong-initializer-bound", "initializer_after_condition",
         lambda ir, raw: initializer(ir).update(max="255"), "initializer widening transport changed"),
        ("wrong-literal-inference", "literal_condition",
         lambda ir, raw: condition(ir)["right"].update(max="65535"), "scope condition transport differs"),
        ("wrong-original-input", "narrowing_condition",
         lambda ir, raw: ir["constructor"]["parameters"][0]["ty"].update(element=unsigned(255)), "original input bound differs"),
        ("wrong-original-tuple", "nested_tuple_condition",
         lambda ir, raw: ir["constructor"]["parameters"][1].update(ty={"kind": "tuple", "elements": [unsigned(255)] * 2}), "original right bound differs"),
        ("wrong-origin-kind", "ordinary_after_condition",
         lambda ir, raw: next(row for row in raw["addresses"] if row[2] == "coerce").__setitem__(2, "unsigned_cast"), "scope origin kind differs"),
        ("duplicate-origin-address", "literal_condition",
         lambda ir, raw: raw["addresses"].append(copy.deepcopy(raw["addresses"][-1])), "duplicate scope origin address"),
        ("negative-origin-index", "literal_condition",
         lambda ir, raw: raw["addresses"][0].__setitem__(3, -1), "scope origin index differs"),
        ("negative-origin-span", "literal_condition",
         lambda ir, raw: raw["origins"][0].__setitem__(0, -1), "scope origin span index differs"),
    )
    output.mkdir()
    results = {}
    for label, role, mutate, message in controls:
        original_ir, original_raw, source = cases[role]
        ir, raw = copy.deepcopy(original_ir), copy.deepcopy(original_raw)
        mutate(ir, raw)
        try:
            validate_scope(role, ir, raw, source)
        except ValueError as error:
            require(message in str(error), "unrelated HOST scope refusal: " + label)
        else:
            raise ValueError("HOST scope mutation accepted: " + label)
        path = output / (label + ".json")
        path.write_text(json.dumps({"scope": "HOST-only, never executed", "ir": ir, "origins": raw}, indent=2) + "\n")
        results[label] = {"scope_assertion_refused": True, "diagnostic": message, "sha256": digest(path)}
    return results


def main(*, tuple_guards=False):
    require(__debug__ and not os.environ.get("PYTHONOPTIMIZE"), "optimized Python is not admitted")
    if tuple_guards:
        import constructor_tuple_scope as tuple_scope
        tuple_expected = tuple_scope.expectations()
        roles, fixtures = tuple_scope.ROLES, tuple_scope.FIXTURES
        raw_refusals, refusals = tuple_scope.RAW_REFUSALS, tuple_scope.RUST_REFUSALS
    else:
        roles, fixtures, raw_refusals, refusals = ROLES, FIXTURES, {}, REFUSALS
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    require(not out.exists() and not args.output.is_symlink() and not out.is_relative_to(ROOT), "fresh output outside source required")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "clean source required")
    recipe = json.loads(args.compiler_receipt.read_bytes())
    require(recipe["status"] == "passed" and recipe["head"] == head and recipe["tree"] == tree,
            "current clean original compiler recipe required")
    tools = {k: Path(recipe[k]["path"]).resolve(strict=True) for k in ("compactc", "compactc-scheme")}
    require(tools["compactc"].parent == tools["compactc-scheme"].parent and
            all(digest(p) == recipe[k]["sha256"] for k, p in tools.items()), "compiler identity differs")
    environment = dict(os.environ)
    configuration = reject_compiler_overrides(environment)
    environment.pop("LD_LIBRARY_PATH", None)
    if not tuple_guards:
        require({p.stem for p in fixtures.glob("*.compact")} == set(roles), "complete scope source roster differs")
    pins = {str(ROOT / p): digest(ROOT / p) for p in git("ls-files").splitlines() if (ROOT / p).is_file()}
    pins.update({str(p): digest(p) for p in [args.compiler_receipt.resolve(), *tools.values()]})
    out.mkdir(parents=True)
    report = dict(status="running", head=head, tree=tree, inputs=pins, commands=[], sources={},
                  scope=__doc__, sdk_execution=False, proof_execution=False)
    bound = {}
    frozen_directories = {}
    save = lambda: (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    save()

    def unchanged():
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "scope source changed")
        require(reject_compiler_overrides(dict(os.environ)) == configuration, "scope environment changed")
        require(all(Path(p).is_file() and digest(p) == h for p, h in {**pins, **bound}.items()), "scope input changed")
        require(all(inventory(Path(p)) == expected for p, expected in frozen_directories.items()),
                "complete scope output membership or bytes changed")

    def run(label, argv, expected=0):
        unchanged()
        row = dict(label=label, argv=list(map(str, argv)), timeout_seconds=COMMAND_TIMEOUT_SECONDS)
        report["commands"].append(row)
        save()
        started = time.monotonic()
        try:
            result = bounded_command(row["argv"], environment)
        except subprocess.TimeoutExpired as error:
            row.update(exit=None, timed_out=True)
            for stream in ("stdout", "stderr"):
                value = getattr(error, stream) or ""
                (out / (label + "." + stream)).write_text(value.decode(errors="replace") if isinstance(value, bytes) else value)
            raise
        else:
            row.update(exit=result.returncode, timed_out=False)
            for stream in ("stdout", "stderr"):
                (out / (label + "." + stream)).write_text(getattr(result, stream))
            require(result.returncode == 0 if expected == 0 else result.returncode != 0, "wrong scope generation outcome: " + label)
            return result
        finally:
            row["seconds"] = time.monotonic() - started
            for stream in ("stdout", "stderr"):
                p = out / (label + "." + stream)
                if p.exists(): row[stream + "_sha256"] = digest(p)
            save()
            unchanged()

    try:
        cases = {}
        for role in roles:
            source = fixtures / (tuple_expected[role]["path"] if tuple_guards else role + ".compact")
            raw, ts, rust = [out / role / target for target in ("raw", "ts", "rust")]
            result = run(role + "-raw", [tools["compactc-scheme"], "--emit-rust-ir", "--skip-zk", "--skip-ts", source, raw],
                         expected=255 if role in raw_refusals else 0)
            if role in raw_refusals:
                require(result.returncode == 255 and raw_refusals[role] in result.stderr,
                        "unrelated raw tuple scope refusal: " + role)
                report["sources"][role] = dict(raw_generation="known-open-transport-refusal",
                                                rust_generation="current-typed-refusal")
            else:
                ir = json.loads((raw / "contract/compact-rust-ir.json").read_bytes())
                origins = json.loads((raw / "contract/compact-rust-origins.raw.json").read_bytes())
                report["sources"][role] = (tuple_scope.validate(role, ir, origins, source, tuple_expected[role])
                                           if tuple_guards else validate_scope(role, ir, origins, source))
                cases[role] = (ir, origins, source)
            report["sources"][role]["source_sha256"] = digest(source)
            if raw.exists():
                bound.update({str(raw / p): h for p, h in inventory(raw).items()})
                frozen_directories[str(raw)] = inventory(raw)
            run(role + "-ts", [tools["compactc"], "--target", "ts", "--skip-zk", source, ts])
            bound.update({str(ts / p): h for p, h in inventory(ts).items()})
            frozen_directories[str(ts)] = inventory(ts)
            result = run(role + "-rust", [tools["compactc"], "--target", "rust", "--skip-zk",
                "--rust-functional-ledger8", "--rust-require-recording", "--rust-runtime-root", ROOT,
                source, rust], expected=1 if role in refusals else 0)
            if role in refusals:
                require(refusals[role] in result.stderr, "unrelated scope refusal: " + role)
                if tuple_guards:
                    require(result.returncode == (255 if role in raw_refusals else 1),
                            "wrong exact tuple refusal exit: " + role)
            if rust.exists():
                bound.update({str(rust / p): h for p, h in inventory(rust).items()})
                frozen_directories[str(rust)] = inventory(rust)
            save()
        report["HOST_controls"] = (tuple_scope.host_controls(cases, out / "HOST-transport-controls", tuple_expected)
                                   if tuple_guards else host_controls(cases, out / "HOST-transport-controls"))
        unchanged()
        report.update(status="passed", complete_sources=len(roles), supported_rust_generations=len(roles) - len(refusals),
                      current_typed_refusals=len(refusals),
                      cohort="implicit-tuple-guards" if tuple_guards else "constructor-scope")
    except BaseException as error:
        report.update(status="failed", error=repr(error))
        raise
    finally:
        report["frozen_directories"] = frozen_directories
        report["artifacts"] = inventory(out)
        report["artifacts"].pop("receipt.json", None)
        save()


if __name__ == "__main__":
    main()
