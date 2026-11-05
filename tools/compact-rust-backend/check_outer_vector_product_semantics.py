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

"""Complete original/family outer-Vector source, API and untrusted-boundary checks.

Fresh current-head original recipe required. Whole sources and generated SDKs
stay unedited. Models, keys, proofs, applications and integration are separate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib

from check_constant_vector_projection import compile_failure
from check_outer_vector_product_boundaries import qualify
from check_passport_current_profile import cargo_target
from check_source_map_publication import capture_command
from check_stateful_product_vector_widening import reject_compiler_overrides, source_manifest_snapshot

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "outer_vector_product_semantics"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    out = args.output.absolute()
    require(not out.exists() and not out.is_symlink(), "output must be absent")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "clean source required")
    build = json.loads(args.compiler_receipt.read_bytes())
    require(build["status"] == "passed" and build["head"] == head and build["tree"] == tree,
            "original recipe must bind current clean source")
    compiler, scheme = [Path(build[k]["path"]).resolve(strict=True) for k in ("compactc", "compactc-scheme")]
    require(compiler.parent == scheme.parent and compiler.name == "compactc" and scheme.name == "compactc-scheme",
            "one original compiler package required")
    for key, path in (("compactc", compiler), ("compactc-scheme", scheme)):
        require(digest(path) == build[key]["sha256"], "compiler executable identity changed")
    runtime = args.ts_runtime.resolve(strict=True)
    require(json.loads((runtime / "package.json").read_bytes())["version"] == "0.16.102", "runtime version differs")
    env = dict(os.environ)
    config = reject_compiler_overrides(dict(env))
    env.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "use Rust1.99.0")
    env.update(CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    env["CARGO_TARGET_DIR"] = str(cargo_target(env, out))
    env.pop("LD_LIBRARY_PATH", None)
    node = Path(shutil.which("node")).resolve(strict=True)
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    rust = {n: Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", n], text=True).strip()).resolve(strict=True)
            for n in ("cargo", "rustc", "clippy-driver", "cargo-clippy")}
    env["PATH"] = str(rust["cargo"].parent) + os.pathsep + env["PATH"]
    env["RUSTC"] = str(rust["rustc"])
    definitions = json.loads((FIXTURES / "sources.json").read_bytes())
    cases = json.loads((FIXTURES / "cases.json").read_bytes())
    require([s["role"] for s in definitions] == ["original", "family"]
            and [len(s["entries"]) for s in definitions] == [1, 15], "whole source roster differs")
    require(len(cases) == len({c["id"] for c in cases}) == 26, "case roster differs")
    require(definitions[0]["path"] == "tools/compact-rust-backend/tests/nested_component_products/unsupported/top_level_nested_tuple.compact"
            and definitions[0]["sha256"] == "36d8ab210f5981a88cdc6e497d2fd26bf4070ea226a5c1b2a75212c9dac983e2", "original source identity differs")
    require(definitions[1]["path"] == "examples/rust_backend/outer_vector_product_widening/outer_vector_product_widening.compact"
            and definitions[1]["sha256"] == "ae8b7d1406c2011e07b4069361b4d1178e5e04856838044bd8b17c4906854f1d", "whole family identity differs")
    source_files = [ROOT / s["path"] for s in definitions]
    require(all(digest(p) == s["sha256"] for p, s in zip(source_files, definitions)), "whole source bytes differ")
    historical = ROOT / "tools/compact-rust-backend/tests/nested_component_products"
    historical188 = historical / "scope-controls-compiler188.json"
    # Pin the complete old table, rather than deriving history from a new row.
    require(digest(historical188) == "76690e368d2fb8782101362c0ea73cfb8e46c08b3882777ee384c547645d0ab6", "historical compiler188 table changed")
    paths = [Path(__file__), args.compiler_receipt.resolve(), compiler, scheme, node, rustup, *rust.values(),
             ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "flake.nix", ROOT / "flake.lock", *source_files,
             Path(sys.executable).resolve(strict=True), runtime / "package.json",
             historical188, historical / "scope-controls.json",
             ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements/scope-controls.json"]
    paths.extend(FIXTURES.rglob("*"))
    paths.extend(Path(__file__).parent.glob("*.py"))
    for directory in ("compiler", "runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src"):
        paths.extend(p for p in (ROOT / directory).rglob("*") if p.suffix in {".ss", ".rs", ".json", ".toml"})
    pins = {str(p): digest(p) for p in paths if p.is_file()}
    out.mkdir(parents=True)
    bound = {}
    report = {"status": "running", "head": head, "tree": tree, "inputs": pins,
              "cargo_configuration_sha256": config, "commands": [], "sources": {}, "proof_execution": False,
              "scope": "Complete two-source16export SDK,26currentTS/four-routeRust,10markedrefusal/executed-twin, private nativeIR/recorded/source/producer controls; no model/proof/CI/integration acceptance. Pins are not transitive build or consumed-source attestation."}
    started = time.monotonic()
    root_lock = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
    allowed = {("outer-vector-product-consumer", "0.1.0")}

    def unchanged():
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source changed")
        require(reject_compiler_overrides(dict(os.environ)) == config, "Cargo configuration changed")
        require(all(Path(p).is_file() and digest(p) == h for p, h in {**pins, **bound}.items()), "bound input changed")

    def bind(path):
        bound[str(path)] = digest(path)

    def lock(path):
        for package in tomllib.loads(path.read_text())["package"]:
            key = (package["name"], package["version"], package.get("source"))
            require(key in root_lock or key[2] is None and key[:2] in allowed, "unexpected resolved dependency: " + str(key))
            if key[2] is not None:
                require(package.get("checksum") == root_lock[key], "resolved checksum differs")
        bind(path)

    def run(label, command, *, expected_exit=0):
        unchanged()
        command = list(map(str, command))
        report["current"] = label
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        began = time.monotonic()
        row = {"label": label, "argv": command}
        try:
            result = capture_command(command, env, ROOT, out / (label + ".stdout"), out / (label + ".stderr"))
            row["exit"] = result.returncode
        except subprocess.TimeoutExpired:
            row["timeout"] = True
            raise
        finally:
            row["seconds"] = time.monotonic() - began
            for stream in ("stdout", "stderr"):
                path = out / (label + "." + stream)
                if path.is_file():
                    row[stream + "_sha256"] = digest(path)
            report["commands"].append(row)
            (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        unchanged()
        require(result.returncode == expected_exit, f"{label} failed ({result.returncode}); raw diagnostics retained")
        return result.stdout.decode("utf-8")

    try:
        deps, raw = [], {}
        for spec in definitions:
            role = spec["role"]
            generated = out / "generated" / role
            independent = out / "independent" / role
            run(role + "-generate", [compiler, "--target", "rust", "--target", "ts", "--skip-zk", "--rust-runtime-root", ROOT,
                "--rust-functional-ledger8", "--rust-require-recording", spec["path"], generated])
            run(role + "-independent-ir", [scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk", spec["path"], independent])
            ir = generated / "contract/compact-rust-ir.json"
            golden = FIXTURES / "proof-goldens" / role
            require(ir.read_bytes() == (independent / "contract/compact-rust-ir.json").read_bytes()
                    == (golden / "source-ir.json").read_bytes(), "whole independent/golden IR differs")
            data = json.loads(ir.read_bytes())
            require([c["name"] for c in data["stateful_circuits"]] == spec["entries"]
                    and [c["name"] for c in data["circuits"]] == spec.get("helpers", [])
                    and [c["name"] for c in data["witnesses"]] == spec.get("witnesses", []), "complete IR membership differs")
            info = json.loads((generated / "compiler/contract-info.json").read_bytes())
            require([c["name"] for c in info["circuits"]] == spec["entries"], "export roster differs")
            caps = json.loads((generated / "contract/rust-capabilities.json").read_bytes())
            require(caps["schema_version"] == 3 and [c["name"] for c in caps["circuits"]] == spec["entries"]
                    and all(c["recorded"] and c["observed_call"] and c["proof_required"] is True for c in caps["circuits"]), "capability roster differs")
            programs = generated / "contract/checked-programs"
            names = {f"program-{i}.json" for i in range(len(spec["entries"]))}
            require({p.name for p in programs.iterdir()} == {p.name for p in (golden / "checked-programs").iterdir()} == names
                    and all((programs / n).read_bytes() == (golden / "checked-programs" / n).read_bytes() for n in names), "whole raw Programs differ")
            map_path = generated / "contract/index.js.map"
            mapping = json.loads(map_path.read_bytes())
            targets = [Path(os.path.normpath(map_path.parent / mapping.get("sourceRoot", "") / name)).resolve(strict=True) for name in mapping["sources"]]
            require(targets == [(ROOT / spec["path"]).resolve(strict=True)] and digest(targets[0]) == spec["sha256"], "source map binding differs")
            require({c["entry"] for c in cases if c["source"] == role} == set(spec["entries"]), "case export membership differs")
            raw[role] = {str(p.relative_to(generated)): digest(p) for p in generated.rglob("*") if p.is_file()}
            for name in raw[role]:
                bind(generated / name)
            report["sources"][role] = {"source_sha256": spec["sha256"], "entries": spec["entries"], "manifest": source_manifest_snapshot(generated, True), "raw_files": raw[role]}
            manifest = generated / "contract/Cargo.toml"
            package = tomllib.loads(manifest.read_text())["package"]
            allowed.add((package["name"], package["version"]))
            deps.append(role + "={package=" + json.dumps(package["name"]) + ",path=" + json.dumps(str(manifest.parent)) + ',features=["ledger-transaction"]}')
            shutil.copyfile(ROOT / "Cargo.lock", manifest.with_name("Cargo.lock"))
            run(role + "-resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
            lock(manifest.with_name("Cargo.lock"))
            run(role + "-strict", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest, "--all-targets", "--all-features", "--", "-D", "warnings"])
        modules = out / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (out / "package.json").write_text('{"type":"module"}\n')
        bind(out / "package.json")
        ts = out / "ts-results.json"
        run("fresh-ts", [node, FIXTURES / "capture.mjs", "--generated", out / "generated", "--runtime", runtime, "--version", "0.16.102", "--output", ts])
        observed = json.loads(ts.read_bytes())
        require(observed["status"] == "passed" and observed["format"] == "outer-vector-product-widening-ts/v1"
                and [r["id"] for r in observed["rows"]] == [r["id"] for r in cases], "fresh TS case roster differs")
        require(observed["runtime"]["path"] == str(runtime) and observed["runtime"]["version"] == "0.16.102"
                and observed["node"]["path"] == str(node) and observed["node"]["sha256"] == digest(node), "TS runtime/Node identity differs")
        require(observed["capture"] == {"path": str((FIXTURES / "capture.mjs").resolve()), "sha256": digest(FIXTURES / "capture.mjs")}, "TS capture identity differs")
        require(set(observed["wrappers"]) == {s["role"] for s in definitions}, "TS wrapper roster differs")
        for role, wrapper in observed["wrappers"].items():
            path = (out / "generated" / role / "contract/index.js").resolve(strict=True)
            require(wrapper == {"path": str(path), "sha256": digest(path)}, "TS selected another wrapper")
        bound.update(observed["runtime"]["files"])
        report["ts_runtime"] = observed["runtime"]
        bind(ts)
        unchanged()
        consumer = out / "consumer"
        consumer.mkdir()
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "consumer.rs")
        manifest = consumer / "Cargo.toml"
        manifest.write_text('[workspace]\n[package]\nname="outer-vector-product-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + "\n".join(deps) + "\n")
        for path in (consumer / "consumer.rs", manifest):
            bind(path)
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        env["COMPACT_OUTER_VECTOR_PRODUCT_TS"] = str(ts)
        run("consumer-resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
        lock(consumer / "Cargo.lock")
        stdout = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--lib"])
        require("test result: ok. 26 passed; 0 failed; 0 ignored;" in stdout, "native case count differs")
        run("consumer-strict", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest, "--lib", "--tests", "--", "-D", "warnings"])
        stdout = run("independent-recorded", ["cargo", "test", "--locked", "--offline", "-p", "compact-rust-backend", "--lib", "recorded::effect_plan::tests::outer_vector_products::"])
        require("test result: ok. 5 passed; 0 failed; 0 ignored;" in stdout, "independent recorded controls did not execute")

        boundary = out / "boundary-consumer"
        (boundary / "src/bin").mkdir(parents=True)
        for name in ("consumer.rs", "Cargo.toml", "Cargo.lock"):
            shutil.copyfile(consumer / name, boundary / name)
        for path in (FIXTURES / "boundary").glob("*.rs"):
            shutil.copyfile(path, boundary / "src/bin" / path.name)
        for path in boundary.rglob("*"):
            if path.is_file() and path.name != "Cargo.lock":
                bind(path)
        manifest = boundary / "Cargo.toml"
        run("boundary-resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
        require((boundary / "Cargo.lock").read_bytes() == (consumer / "Cargo.lock").read_bytes(), "boundary dependency graph changed")
        lock(boundary / "Cargo.lock")
        controls = json.loads((FIXTURES / "boundary/expected.json").read_bytes())
        require(set(controls) == {"leaf_bound", "second_leaf_bound", "inner_arity", "product_arity", "outer_dimension", "inner_vector_kind", "array_not_vector", "result", "program", "witness"}, "static control roster differs")
        report["static"] = {}
        for name, code in controls.items():
            text = (boundary / "src/bin" / (name + ".rs")).read_text()
            stdout = run(name + "-refusal", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", name, "--message-format=json"], expected_exit=101)
            report["static"][name] = compile_failure(subprocess.CompletedProcess([], 101, stdout=stdout), name, code, text)
            run(name + "-executed-twin", ["cargo", "run", "--locked", "--offline", "--manifest-path", manifest, "--bin", name + "_twin"])

        # The historical table remains literal; this table describes actual
        # current lane behavior, including the now-supported original source.
        table = json.loads((historical / "scope-controls.json").read_bytes())
        require(set(table) == {"top_level_nested_tuple", "vector_inside_nested_product"}, "current nested scope roster differs")
        legacy = ROOT / "tools/compact-rust-backend/tests/stateful_vector_product_elements"
        sources = [(name, historical / "unsupported" / (name + ".compact"), spec) for name, spec in table.items()]
        sources.append(("vector_inside_component_product", legacy / "unsupported/vector_inside_component_product.compact",
                        json.loads((legacy / "scope-controls.json").read_bytes())["vector_inside_component_product"]))
        report["current_source_lanes"] = {}
        for name, source, spec in sources:
            require(digest(source) == spec["source_sha256"], "scope source differs")
            bind(source)
            for lane in ("ts", "rust", "ir"):
                dest = out / "source-scope" / (name + "-" + lane)
                command = ([scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk"] if lane == "ir"
                           else [compiler, "--target", lane, "--skip-zk"] + (["--rust-runtime-root", ROOT, "--rust-functional-ledger8", "--rust-require-recording"] if lane == "rust" else []))
                label = name + "-" + lane + "-scope"
                expected = spec["lanes"][lane]
                stdout = run(label, [*command, source, dest], expected_exit=expected["exit"])
                if expected["exit"]:
                    require(stdout == expected["stdout"] and (out / (label + ".stderr")).read_bytes().decode() == expected["stderr"], "scope refusal differs")
                    require(not (dest / "contract/lib.rs").exists() and not (dest / "contract/compact-rust-ir.json").exists(), "refused source published Rust/IR")
                else:
                    require((dest / "contract" / ("compact-rust-ir.json" if lane == "ir" else "lib.rs" if lane == "rust" else "index.js")).is_file(), "scope artifact missing")
            report["current_source_lanes"][name] = spec
        report["boundaries"] = qualify(out, ROOT, compiler, env, run, lock, bind, digest, require)
        require(len(report["commands"]) == 91, "complete command roster differs")
        for role, files in raw.items():
            generated = out / "generated" / role
            require({str(p.relative_to(generated)) for p in generated.rglob("*") if p.is_file()} == set(files) | {"contract/Cargo.lock"}, "whole generated membership changed")
        unchanged()
        report.update(status="passed", current_ts_cases=26, rust_routes=104, error_routes=28, vm_replays=38,
                      marked_refusals=10, executed_positive_twins=10, private_native_ir_controls=19,
                      independent_recorded_methods=5, model_observation=False, full_integration_acceptance="OPEN")
    except BaseException as error:
        report.update(status="failed", error=repr(error))
        raise
    finally:
        report.update(seconds=time.monotonic() - started, bound_build_inputs=bound)
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in ("status", "head", "current_ts_cases", "rust_routes", "private_native_ir_controls")}))


if __name__ == "__main__":
    main()
