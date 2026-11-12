#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Complete stateful product Vector widening, API and refusal checks.

No key generation, proof execution, full CI or integration acceptance is implied.
Every run requires a new evidence directory and the original current-head build.
"""
import argparse
import historical_source_licenses
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
FIXTURES = Path(__file__).with_name("tests") / "stateful_product_vector_widening"
SOURCES = ROOT / "examples/rust_backend/stateful_product_vector_widening"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def inventory(root):
    return {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob("*")) if p.is_file()}


def source_manifest_snapshot(output, proof_required):
    if proof_required:
        return generated_manifest_snapshot(output)
    # Pure execution has no public ledger proof obligation.
    # Authenticate every declared asset through the existing manifest checker;
    # proof-required manifests use the unchanged full checker above.
    require(__debug__, "manifest qualification requires Python without -O/-OO")
    check_manifest(output, require_zkir=False)
    path = output / "compiler/contract-manifest.json"
    manifest = json.loads(path.read_text())
    require("zkir" not in manifest and not (output / "zkir").exists(), "proof-not-applicable source acquired ZKIR")
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
    return {"manifest_sha256": digest(path), "assets": files,
            "declared_asset_bytes": sum((output / p).stat().st_size for p in files),
            "proof_applicability": "no applicable ledger proof; explicit classification and no ZKIR"}


def check_local_origins(ir, raw):
    """Check local cast transport; native binding/coercion checking is separate."""
    require(raw["schema_version"] == 3, "unexpected raw transport")
    nodes, paths, origins = [], [], {}
    for parent, edge, kind, origin in raw["addresses"]:
        if parent is None:
            node, path = ir, ()
        else:
            key = edge.get("field", edge.get("index"))
            node, path = nodes[parent][key], paths[parent] + (key,)
        require(kind is None or isinstance(node, dict) and node.get("kind") == kind,
                "raw address differs from semantic IR")
        nodes.append(node); paths.append(path)
        origins[path] = None if origin is None else raw["origins"][origin]
    by_path = dict(zip(paths, nodes))
    count = 0
    for node, path in zip(nodes, paths):
        if (len(path) >= 3 and path[0] == "stateful_circuits"
                and path[-3] == "bindings" and path[-1] == "value"
                and isinstance(node, dict) and node.get("kind") == "coerce"
                and node.get("ty", {}).get("kind") == "vector"):
            binding = by_path[path[:-1]]
            require(binding["ty"] == node["ty"], "local target differs from declared binding")
            origin = origins.get(path)
            require(origin is not None and origin[1:] == ["derived", "expression"],
                    "local conversion origin label/category differs")
            require(raw["spans"][origin[0]][7]["kind"] == "physical_source",
                    "local conversion lacks physical source")
            count += 1
    return count


def first_local_binding(node):
    if isinstance(node, dict):
        if (set(node) >= {"name", "ty", "value"} and node["ty"].get("kind") == "vector"
                and node["value"].get("kind") == "coerce"):
            return node
        for value in node.values():
            found = first_local_binding(value)
            if found is not None:
                return found
    elif isinstance(node, list):
        for value in node:
            found = first_local_binding(value)
            if found is not None:
                return found
    return None


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
    paths.extend(historical_source_licenses.inputs())
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
        require(len(definitions) == 25 and {s["source"] for s in definitions} == {p.stem for p in SOURCES.glob("*.compact")}, "source roster differs")
        cases = json.loads((FIXTURES / "cases.json").read_text())
        case_count = len(cases)
        require(case_count == 32 and len({r["id"] for r in cases}) == case_count, "case roster differs")
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
            origin_count = check_local_origins(irs[name], raw)
            require(origin_count == spec["local_conversions"], "local conversion origin roster differs: " + name)
            if name == "local_product":
                require(origin_count == 1, "local conversion source origin missing")
                forged = copy.deepcopy(raw)
                for origin in forged["origins"]:
                    if origin[1:] == ["derived", "expression"]:
                        origin[1] = "provided"
                try:
                    check_local_origins(irs[name], forged)
                except ValueError as e:
                    require("origin label/category differs" in str(e), "origin mutation hit unrelated guard")
                else:
                    raise ValueError("local conversion relabeled as supplied")
            programs = inventory(target / "contract/checked-programs")
            require(programs == inventory(FIXTURES / "ledger9-goldens" / name / "checked-programs"),
                    "complete Ledger 9 Program roster differs: " + name)
            require(len(programs) == (len(spec["entries"]) if spec["stateful"] else 0), "complete checked Program count differs: " + name)
            caps = json.loads((target / "contract/rust-capabilities.json").read_text())["circuits"]
            if spec["stateful"]:
                require({c["name"] for c in caps} == set(spec["entries"])
                        and all(c["recorded"] is True and c["observed_call"] is True
                                and c["proof_required"] is spec["proof_required"]
                                and c["recording_status"] == ("available" if spec["proof_required"] else "not_applicable")
                                for c in caps), "stateful Vector APIs unavailable")
            else:
                require(not caps, "pure-only source unexpectedly acquired ledger proof APIs")
            report["sources"][name] = {"source_sha256": digest(SOURCES / (name + ".compact")),
                    "manifest": source_manifest_snapshot(target, spec["proof_required"]), "programs": programs, "local_conversion_origins": origin_count}
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
        env["COMPACT_STATEFUL_PRODUCT_VECTOR_TS"] = str(ts)
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
        manifest.write_text('[workspace]\n[package]\nname="stateful-product-vector-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + '\n'.join(dependencies) + '\n')
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
        for name, code in [("wrong_input_bound", "E0308"), ("wrong_input_length", "E0308"), ("wrong_input_arity", "E0308"), ("wrong_length", "E0308"), ("wrong_element", "E0308"), ("wrong_result_max", "E0308"), ("wrong_result_arity", "E0308"), ("field_result", "E0308"), ("program_reuse", "E0382"), ("context_reuse", "E0382")]:
            source = (FIXTURES / (name + ".rs")).read_text()
            # Execute an independent compilation with only the intended misuse
            # removed. Value/shape controls retain a real correctly typed call;
            # ownership controls already contain their first valid execution.
            if name not in {"program_reuse", "context_reuse"}:
                source = source.replace("fn main() {", "fn main() {\n    let _: r::FixedVector<(r::BoundedUint<65535>,r::BoundedUint<4294967295>), 2> = c::run(context(), input()).unwrap().result;")
            twin = "\n".join(line for line in source.splitlines() if "// expected-error" not in line) + "\n"
            (consumer / "src/bin" / (name + "_twin.rs")).write_text(twin)
            run(name + "-correct-twin", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", name + "_twin"])
            (consumer / "src/bin" / (name + ".rs")).write_text(source)
            rejected = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", name, "--message-format=json"], success=False)
            report["compile_fail"][name] = compile_failure(rejected, name, code, source)
        report["source_refusals"] = {}
        refusals = json.loads((FIXTURES / "frontend-refusals.json").read_text())
        require(set(refusals) == {"wrong_length", "boolean_numeric", "refused_narrow", "refused_wrong_arity", "nominal_implicit", "private_condition", "explicit_narrow"}, "frontend refusal roster differs")
        require({p.stem for p in (FIXTURES / "negative").glob("*.compact")} == set(refusals), "frontend source membership differs")
        for name, expected in refusals.items():
            source = FIXTURES / "negative" / (name + ".compact")
            historical_source_licenses.verify_source(source, expected["source_sha256"])
            rejected = run("refused-" + name, [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", ROOT,
                source, output / ("refused-" + name)], success=False)
            require(rejected.returncode == expected["exit"] != 0 and rejected.stderr == expected["stderr"]
                    and rejected.stdout == expected["stdout"], "complete frontend refusal diagnostic differs: " + name)
            report["source_refusals"][name] = expected
        wrong = copy.deepcopy(reference)
        next(row for row in wrong["rows"] if row["id"] == "parameter")["result"]["valueAtoms"][0][0] ^= 1
        wrong_path = output / "changed-result.json"
        wrong_path.write_text(json.dumps(wrong) + "\n")
        env["COMPACT_STATEFUL_PRODUCT_VECTOR_TS"] = str(wrong_path)
        rejected = run("changed-expected-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--lib", "parameter", "--", "--exact"], success=False)
        require(rejected.returncode and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                and "assertion `left == right` failed" in rejected.stdout, "changed expected result was not consumed")
        env["COMPACT_STATEFUL_PRODUCT_VECTOR_TS"] = str(ts)
        report["source_mutations"] = {}
        for name, old, new, test in [
                ("helper", "return input;", "return [input[1],input[0]];", "helper"),
                ("checked_helper", 'assert(ok,"helper denied");', '', "checked_guard_first"),
                ("witness_after", 'seen=1;const wide:Vector<2,[Uint<16>,Uint<32>]> =disclose(vector_value());', 'const wide:Vector<2,[Uint<16>,Uint<32>]> =disclose(vector_value());seen=1;', "witness_after"),
                ("witness_before", 'const wide:Vector<2,[Uint<16>,Uint<32>]> =disclose(vector_value());seen=1;', 'seen=1;const wide:Vector<2,[Uint<16>,Uint<32>]> =disclose(vector_value());', "witness_before")]:
            # Mutate a complete producer, then regenerate it. No generated Rust
            # or expected results are edited. The assertion producer changes
            # first-error order; the witness producer moves its call before the write.
            text = (SOURCES / (name + ".compact")).read_text()
            require(text.count(old) == 1, "producer mutation premise changed: " + name)
            mutant = output / (name + "-producer-mutant")
            mutant.mkdir()
            source = mutant / (name + ".compact")
            source.write_text(text.replace(old, new))
            run(name + "-mutant-generate", [compiler, "--target", "rust", "--skip-zk", "--feature-zkir-v3", "--rust-runtime-root", ROOT, "--rust-functional-ledger9", "--rust-require-recording", source, mutant / "generated"])
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
            report["source_mutations"][name] = {"sha256": digest(source), "consumer_test": test, "scope": "same-type helper projection / first-error / witness-before-and-after-write ordering change"}
        report["malformed_ir_refusals"] = {}
        for name in ["wrong_dimension", "wrong_product_arity", "wrong_element", "narrowing_component", "wrong_input_type", "wrong_binding_type", "dead_branch", "empty_element", "misplaced_action"]:
            base = "proofs" if name == "dead_branch" else "empty_product" if name == "empty_element" else "local_product"
            malformed = copy.deepcopy(irs[base])
            circuit = next(c for c in malformed["stateful_circuits"] if c["name"] == "run")
            body = circuit["return_value"]["body"]
            if name == "dead_branch":
                body["condition"] = {"kind": "boolean", "value": True}
                expr = body["otherwise"]["result"]["value"]
            elif name == "empty_element":
                expr = body["result"]["value"]
            else:
                binding = first_local_binding(body)
                require(binding is not None, "missing intended malformed local binding")
                expr = binding["value"]
            require(expr["kind"] == "coerce", "missing intended checked aggregate conversion")
            if name == "wrong_dimension":
                expr["ty"]["length"] = 3
            elif name == "wrong_product_arity":
                expr["ty"]["element"]["elements"].append({"kind": "boolean"})
            elif name in ["wrong_element", "dead_branch", "empty_element"]:
                expr["ty"]["element"]["elements"][0] = {"kind": "boolean"}
            elif name == "narrowing_component":
                expr["ty"]["element"]["elements"][0]["max"] = "1"
            elif name == "wrong_input_type":
                circuit["parameters"][0]["ty"] = {"kind": "boolean"}
            elif name == "wrong_binding_type":
                binding["ty"] = {"kind": "boolean"}
            else:
                circuit["return_value"] = {"kind": "expression", "value": {"kind": "cell_write", "field": "seen", "index": 0, "value": {"kind": "field_literal", "value": "1"}}}
            (output / (name + "-ir.json")).write_text(json.dumps(malformed, indent=2) + "\n")
            rejected = run(name + "-ir", ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "compact-rust-backend", "--bin", "compact-rust-backend"], success=False, stdin=json.dumps(malformed))
            expected = {
                "wrong_dimension": ['expression has type Vector', 'length: 2', 'length: 3'],
                "wrong_product_arity": ['expression has type Tuple', 'expected Tuple', 'Boolean'],
                "wrong_element": ['expression has type Unsigned', 'expected Boolean'],
                "narrowing_component": ['expression has type Unsigned', 'max: "255"', 'max: "1"'],
                "wrong_input_type": ['expression has type Boolean', 'expected Vector'],
                "wrong_binding_type": ['expression has type Vector', 'expected Boolean'],
                "dead_branch": ['expression has type Unsigned', 'expected Boolean'],
                "empty_element": ['expression has type Unsigned', 'expected Boolean'],
                "misplaced_action": ['cell_write'],
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
