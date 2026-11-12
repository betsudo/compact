#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Complete pure Struct member literal widening, API and refusal checks.

No key generation, proof execution, full CI or integration acceptance is implied.
Every run requires a new evidence directory and the original current-head build.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_constant_vector_projection import compile_failure
from check_functional_effects import generated_manifest_snapshot
from check_compactc_target import check_manifest
from check_functional_source_sdk import cargo_configuration

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "struct_member_literal_widening"
SOURCES = ROOT / "examples/rust_backend/struct_member_literal_widening"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def inventory(root):
    return {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob("*")) if p.is_file()}


def source_manifest_snapshot(output, stateful):
    if stateful:
        return generated_manifest_snapshot(output)
    # Pure-only exports emit no ZKIR or Programs. Authenticate every declared
    # asset through the established manifest checker without inventing a proof
    # obligation; stateful helper manifests use the unchanged full checker.
    require(__debug__, "manifest qualification requires Python without -O/-OO")
    check_manifest(output, require_zkir=False)
    path = output / "compiler/contract-manifest.json"
    manifest = json.loads(path.read_text())
    require("zkir" not in manifest and not (output / "zkir").exists(), "pure-only source acquired ZKIR")
    files = {}
    def visit(root, entries):
        for name, entry in entries.items():
            if name == "type":
                continue
            child = root / name
            if entry["type"] == "directory":
                visit(child, entry)
            else:
                files[str(child.relative_to(output))] = digest(child)
    for directory in ["compiler", "contract"]:
        visit(output / directory, manifest[directory])
    require(not inventory(output / "contract/checked-programs"), "pure-only source acquired checked Programs")
    return {"manifest_sha256": digest(path), "assets": files,
            "declared_asset_bytes": sum((output / p).stat().st_size for p in files),
            "proof_applicability": "pure-only; no ZKIR/Programs"}


def check_projection_origins(ir, raw):
    """Projection transport label/source check only; native Coerce owns legality."""
    require(raw["schema_version"] == 3, "unexpected raw transport")
    nodes, paths, origins = [], [], {}
    for parent, edge, kind, origin in raw["addresses"]:
        if parent is None:
            node, path = ir, ()
        else:
            key = edge.get("field", edge.get("index"))
            node, path = nodes[parent][key], paths[parent] + (key,)
        require(kind is None or isinstance(node, dict) and node.get("kind") == kind, "raw address differs from semantic IR")
        nodes.append(node)
        paths.append(path)
        origins[path] = None if origin is None else raw["origins"][origin]
    count = 0
    for node, path in zip(nodes, paths):
        if (path and path[0] == "circuits" and isinstance(node, dict)
                and node.get("kind") == "coerce"
                and node.get("value", {}).get("kind") == "struct_field"):
            for suffix, label in [((), "derived"), (("value",), "provided")]:
                origin = origins.get(path + suffix)
                require(origin is not None and origin[1:] == [label, "expression"], "Projection element origin label/category differs")
                require(raw["spans"][origin[0]][7]["kind"] == "physical_source", "Projection element lacks physical source")
            count += 1
    return count


def reject_compiler_overrides(environment):
    names = ["COMPACTC_SCHEME", "COMPACT_RUST_BACKEND", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
             "RUSTC", "CARGO_BUILD_RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
             "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP"]
    require(not any(environment.get(name) for name in names),
            "compiler/backend/unchecked Rust substitutions are not qualified")
    return cargo_configuration(environment)


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
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "use the pinned Rust1.99.0 toolchain")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "qualification requires a clean source HEAD")
    build = json.loads(args.compiler_receipt.read_text())
    require(build["status"] == "passed" and build["head"] == head and build["tree"] == tree,
            "original compiler receipt must bind this exact clean HEAD/tree")
    compiler, scheme = [Path(build[k]["path"]).resolve(strict=True) for k in ["compactc", "compactc-scheme"]]
    require(compiler.parent == scheme.parent, "use one default native compiler package")
    for key, path in [("compactc", compiler), ("compactc-scheme", scheme)]:
        require(digest(path) == build[key]["sha256"], "original compiler identity changed")
    runtime = args.ts_runtime.resolve(strict=True)
    paths = [ Path(__file__), ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "flake.nix", ROOT / "flake.lock"]
    for directory in [ROOT / "compiler", ROOT / "runtime-rs", ROOT / "runtime-rs-macros",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES, SOURCES]:
        paths.extend(p for p in directory.rglob("*") if p.suffix in {".ss", ".rs", ".toml", ".json", ".mjs", ".compact"})
    paths.extend(Path(__file__).parent.glob("*.py"))
    pins = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    tools = {str(p): digest(p) for p in [compiler, scheme, args.compiler_receipt.resolve(), Path(shutil.which("node")).resolve()]}
    rust_tools = {}
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    for name in ["cargo", "rustc", "rustfmt", "clippy-driver", "cargo-clippy"]:
        path = Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name], text=True).strip()).resolve(strict=True)
        rust_tools[name] = {"path": str(path), "sha256": digest(path)}
        tools[str(path)] = digest(path)
    tools[str(rustup)] = digest(rustup)
    env["PATH"] = str(Path(rust_tools["cargo"]["path"]).parent) + os.pathsep + env["PATH"]
    env["RUSTC"] = rust_tools["rustc"]["path"]
    output.mkdir(parents=True)
    report = {"status": "incomplete", "head": head, "tree": tree, "commands": [], "inputs": pins,
              "cargo_configuration_sha256": configuration,
              "tools": tools, "rust_tools": rust_tools, "environment": {k: env.get(k) for k in
                    ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_TARGET_DIR", "MIDNIGHT_PP"]}, "scope": "Focused current complete-source/API/refusal checks; no proofs, Linux or integration"}
    start = time.monotonic()

    def run(label, command, *, success=True, stdin=None):
        command = list(map(str, command))
        began = time.monotonic()
        try:
            p = subprocess.run(command, cwd=ROOT, env=env, input=stdin, capture_output=True, text=True, timeout=300)
        except subprocess.TimeoutExpired as e:
            for suffix, value in [("stdout", e.stdout), ("stderr", e.stderr)]:
                (output / f"{label}.{suffix}").write_bytes(value if isinstance(value, bytes) else (value or "").encode())
            report["commands"].append({"label": label, "command": command, "timeout": True})
            raise
        for suffix in ["stdout", "stderr"]:
            (output / f"{label}.{suffix}").write_text(getattr(p, suffix))
        report["commands"].append({"label": label, "command": command, "exit": p.returncode, "seconds": time.monotonic() - began})
        require(not success or p.returncode == 0, label + " failed: see retained diagnostics")
        return p

    try:
        definitions = json.loads((FIXTURES / "sources.json").read_text())
        require(len(definitions) == 18 and {s["source"] for s in definitions} == {p.stem for p in SOURCES.glob("*.compact")}, "source roster differs")
        cases = json.loads((FIXTURES / "cases.json").read_text())
        case_count = len(cases)
        require(case_count == 28 and len({r["id"] for r in cases}) == case_count, "case roster differs")
        stateful_sources = {r["source"] for r in definitions if r["stateful"]}
        require({r["source"] for r in cases} == {r["source"] for r in definitions}, "a complete source has no consumer")
        stateful_cases = sum(r["source"] in stateful_sources for r in cases)
        pure_routes, stateful_routes = case_count - stateful_cases, stateful_cases * 4
        generated = output / "generated"
        snapshots, irs = {}, {}
        report["sources"] = {}
        for spec in definitions:
            name = spec["source"]
            target = generated / name
            run("generate-" + name, [compiler, "--target", "rust", "--target", "ts", "--skip-zk", "--feature-zkir-v3", "--rust-runtime-root", ROOT,
                    *(["--rust-functional-ledger9", "--rust-require-recording"] if spec["stateful"] else []),
                    (SOURCES / (name + ".compact")).relative_to(ROOT), target])
            run("independent-" + name, [scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk",
                    (SOURCES / (name + ".compact")).relative_to(ROOT), output / "independent" / name])
            ir_path = target / "contract/compact-rust-ir.json"
            require(ir_path.read_bytes() == (output / "independent" / name / "contract/compact-rust-ir.json").read_bytes()
                    == (FIXTURES / "goldens" / name / "source-ir.json").read_bytes(), "complete independent/golden IR differs: " + name)
            irs[name] = json.loads(ir_path.read_text())
            raw = json.loads((target / "contract/compact-rust-origins.raw.json").read_text())
            origin_count = check_projection_origins(irs[name], raw)
            if name == "small_unsigned":
                require(origin_count == 2, "projection element coercion source origin missing")
                forged = copy.deepcopy(raw)
                for origin in forged["origins"]:
                    if origin[1:] == ["derived", "expression"]:
                        origin[1] = "provided"
                try:
                    check_projection_origins(irs[name], forged)
                except ValueError as e:
                    require("origin label/category differs" in str(e), "origin mutation hit unrelated guard")
                else:
                    raise ValueError("derived projection conversion relabeled as supplied")
            programs = inventory(target / "contract/checked-programs")
            require(programs == inventory(FIXTURES / "ledger9-goldens" / name / "checked-programs"),
                    "complete Ledger 9 Program roster differs: " + name)
            require(len(programs) == (2 if spec["stateful"] else 0), "unexpected proof applicability: " + name)
            caps = json.loads((target / "contract/rust-capabilities.json").read_text())["circuits"]
            if spec["stateful"]:
                require({c["name"] for c in caps} == {"run", "run_field"}
                        and all(all(c[k] for k in ["recorded", "observed_call", "proof_required"]) for c in caps), "helper APIs unavailable")
            else:
                require(not caps, "pure-only source unexpectedly acquired ledger proof APIs")
            report["sources"][name] = {"source_sha256": digest(SOURCES / (name + ".compact")),
                    "manifest": source_manifest_snapshot(target, spec["stateful"]), "programs": programs, "projection_origin_pairs": origin_count}
            snapshots[name] = inventory(target)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        ts = output / "typescript.json"
        run("typescript", ["node", FIXTURES / "capture.mjs", "--generated", generated, "--runtime", runtime,
                "--version", json.loads((ROOT / "runtime/package.json").read_text())["version"], "--output", ts])
        reference = json.loads(ts.read_text())
        require(reference["status"] == "passed" and len(reference["rows"]) == case_count, "incomplete TypeScript oracle")
        report["typescript"] = {"path": str(ts), "sha256": digest(ts), "cases": case_count}
        require(all("nativeError" not in row for row in reference["rows"]), "undeclared native error substitution")
        env["COMPACT_STRUCT_MEMBER_LITERAL_TS"] = str(ts)
        consumer = output / "consumer"
        (consumer / "src/bin").mkdir(parents=True)
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "consumer.rs")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        dependencies = []
        for spec in definitions:
            name = spec["source"]
            path = generated / name / "contract"
            package = tomllib.loads((path / "Cargo.toml").read_text())["package"]["name"]
            dependencies.append(name + '={package=' + json.dumps(package) + ',path=' + json.dumps(str(path)) + ',features=["ledger-transaction"]}')
        manifest = consumer / "Cargo.toml"
        manifest.write_text('[workspace]\n[package]\nname="struct-member-literal-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + '\n'.join(dependencies) + '\n')
        run("resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--lib"])
        require(f"test result: ok. {case_count} passed; 0 failed; 0 ignored;" in result.stdout, "all consumer cases must execute")
        run("consumer-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest, "--lib", "--tests", "--", "-D", "warnings"])
        report["strict_sdk_roots"] = {}
        for spec in definitions:
            name = spec["source"]
            target = output / "strict-sdk-roots" / name
            shutil.copytree(generated / name / "contract", target)
            before = inventory(target)
            shutil.copyfile(consumer / "Cargo.lock", target / "Cargo.lock")
            run(name + "-root-resolve", ["cargo", "metadata", "--offline", "--manifest-path", target / "Cargo.toml", "--format-version", "1"])
            run(name + "-root-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", target / "Cargo.toml", "--all-targets", "--all-features", "--", "-D", "warnings"])
            require(all(digest(target / path) == sha for path, sha in before.items()), "root Clippy changed an emitted source")
            report["strict_sdk_roots"][name] = {"unchanged_emitted_files": before, "lock_sha256": digest(target / "Cargo.lock")}
        report["compile_fail"] = {}
        for name, code in [("wrong_length", "E0308"), ("wrong_element", "E0308"), ("wrong_result_max", "E0308"), ("field_result", "E0308"), ("program_reuse", "E0382")]:
            source = (FIXTURES / (name + ".rs")).read_text()
            (consumer / "src/bin" / (name + ".rs")).write_text(source)
            rejected = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", name, "--message-format=json"], success=False)
            report["compile_fail"][name] = compile_failure(rejected, name, code, source)
        report["source_refusals"] = {}
        for name, message in [("wrong_length", "mismatch between actual return type"), ("wrong_element", "mismatch between actual return type"),
                ("narrow_implicit", "mismatch between actual return type"), ("missing_field", "structure Input has no field named absent"), ("nominal_implicit", "mismatch between actual return type")]:
            rejected = run("refused-" + name, [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", ROOT,
                FIXTURES / (name + ".compact"), output / ("refused-" + name)], success=False)
            require(rejected.returncode and message.lower() in rejected.stderr.lower(), "source refusal changed: " + name)
            report["source_refusals"][name] = message
        wrong = copy.deepcopy(reference)
        next(row for row in wrong["rows"] if row["id"] == "unsigned_bounds")["result"]["valueAtoms"] = [[42, 0], [0, 0]]
        wrong_path = output / "changed-result.json"
        wrong_path.write_text(json.dumps(wrong) + "\n")
        env["COMPACT_STRUCT_MEMBER_LITERAL_TS"] = str(wrong_path)
        rejected = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--lib", "unsigned_bounds", "--", "--exact"], success=False)
        require(rejected.returncode and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                and "assertion `left == right` failed" in rejected.stdout, "changed expected result was not consumed")
        env["COMPACT_STRUCT_MEMBER_LITERAL_TS"] = str(ts)
        report["source_mutations"] = {}
        for name, old, new, test in [
                ("small_unsigned", "return [input.right,input.left];", "return [input.left,input.right];", "small_unsigned"),
                ("source_checked", 'assert(ok,"first denied");', '', "both_denied")]:
            # Mutate a complete producer, then regenerate it. No generated Rust
            # or expected results are edited. The second producer intentionally
            # lets the later assertion win instead of the input assertion.
            text = (SOURCES / (name + ".compact")).read_text()
            require(text.count(old) == 1, "producer mutation premise changed: " + name)
            mutant = output / (name + "-producer-mutant")
            mutant.mkdir()
            source = mutant / (name + ".compact")
            source.write_text(text.replace(old, new))
            run(name + "-mutant-generate", [compiler, "--target", "rust", "--skip-zk", "--feature-zkir-v3", "--rust-runtime-root", ROOT, source, mutant / "generated"])
            changed_manifest = manifest.read_text().replace(str(generated / name / "contract"), str(mutant / "generated/contract"))
            mutant_consumer = mutant / "consumer"
            mutant_consumer.mkdir()
            (mutant_consumer / "Cargo.toml").write_text(changed_manifest)
            shutil.copyfile(consumer / "consumer.rs", mutant_consumer / "consumer.rs")
            shutil.copyfile(consumer / "Cargo.lock", mutant_consumer / "Cargo.lock")
            run(name + "-mutant-resolve", ["cargo", "metadata", "--offline", "--manifest-path", mutant_consumer / "Cargo.toml", "--format-version", "1"])
            rejected = run(name + "-producer-consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", mutant_consumer / "Cargo.toml", "--lib", test, "--", "--exact"], success=False)
            require(rejected.returncode and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                    and "assertion `left == right` failed" in rejected.stdout, "changed complete source did not fail actual consumer: " + name)
            report["source_mutations"][name] = {"sha256": digest(source), "consumer_test": test, "scope": "same-type projection order / first-before-second-source assertion change"}
        report["malformed_ir_refusals"] = {}
        for name in ["wrong_dimension", "wrong_element", "wrong_field", "wrong_index", "wrong_input_type", "dead_branch"]:
            malformed = copy.deepcopy(irs["conditional" if name == "dead_branch" else "small_unsigned"])
            circuit = next(c for c in malformed["circuits"] if c["name"] == "value")
            expr = circuit["body"]
            if name == "wrong_dimension":
                expr["elements"].append(copy.deepcopy(expr["elements"][0]))
            elif name == "wrong_element":
                expr["elements"][0]["ty"] = {"kind": "boolean"}
            elif name == "wrong_field":
                expr["elements"][0]["value"]["field"] = "absent"
            elif name == "wrong_index":
                # A real field name at the wrong declaration ordinal is invalid.
                expr["elements"][0]["value"]["index"] = 0
            elif name == "wrong_input_type":
                circuit["parameters"][0]["ty"] = {"kind": "boolean"}
            else:
                expr["condition"] = {"kind": "boolean", "value": True}
                expr["otherwise"]["value"]["elements"][0]["ty"] = {"kind": "boolean"}
            (output / (name + "-ir.json")).write_text(json.dumps(malformed, indent=2) + "\n")
            rejected = run(name + "-ir", ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "compact-rust-backend", "--bin", "compact-rust-backend"], success=False, stdin=json.dumps(malformed))
            expected = {
                "wrong_dimension": ['expression has type Vector', 'length: 2', 'length: 3'],
                "wrong_element": ['expression has type Unsigned', 'max: "255"', 'expected Boolean'],
                "wrong_field": ['invalid struct field "absent"'],
                "wrong_index": ['invalid struct field "right"'],
                "wrong_input_type": ['invalid struct field "right"'],
                "dead_branch": ['expression has type Unsigned', 'max: "255"', 'expected Boolean'],
            }[name]
            require(rejected.returncode and all(message in rejected.stderr for message in expected),
                    "native malformed IR refusal changed: " + name)
            report["malformed_ir_refusals"][name] = rejected.stderr
        for name, before in snapshots.items():
            require(inventory(generated / name) == before, "generated artifact changed: " + name)
        for path, sha in pins.items():
            require(digest(ROOT / path) == sha, "source changed: " + path)
        for path, sha in tools.items():
            require(digest(path) == sha, "tool/compiler receipt changed: " + path)
        require(reject_compiler_overrides(configuration_environment) == configuration,
                "Cargo configuration membership or bytes changed during qualification")
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "clean HEAD changed during qualification")
        report.update(status="passed", rust_tests=case_count, pure_routes=pure_routes, stateful_routes=stateful_routes, generated_hashes=snapshots)
    except Exception as e:
        report.update(status="failed", error=str(e))
        raise
    finally:
        report["seconds"] = time.monotonic() - start
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({k: report[k] for k in ["status", "seconds", "head"]} | {"commands": len(report["commands"]), "error": report.get("error")}))


if __name__ == "__main__":
    main()

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
