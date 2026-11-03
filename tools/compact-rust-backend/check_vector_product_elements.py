#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http:#www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Focused complete Vector-component SDK execution. Proof/static gates are separate.

Requires a clean source and its original compiler recipe receipt. Every run writes
an absent evidence directory. Never edits a generated SDK or normalizes its IR.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib

from check_stateful_product_vector_widening import (
    digest, inventory, reject_compiler_overrides, source_manifest_snapshot,
)

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "stateful_vector_product_elements"
SOURCES = ROOT / "examples/rust_backend/stateful_vector_product_elements"


def require(condition, message):
    if not condition:
        raise ValueError(message)


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
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "flake.nix", ROOT / "flake.lock"]
    for directory in [ROOT / "compiler", ROOT / "runtime-rs", ROOT / "runtime-rs-macros",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES, SOURCES]:
        paths.extend(p for p in directory.rglob("*") if p.suffix in {".ss", ".rs", ".toml", ".json", ".mjs", ".compact"})
    paths.extend(Path(__file__).parent.glob("*.py"))
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
                  scope="Focused whole SDK execution/strict compilation, two exact frontend refusals and deeper-product scope controls; compile-fail, malformed IR, producer mutations, model/proofs/Linux/integration remain open")
    start = time.monotonic()

    def run(label, command, *, success=True):
        command = list(map(str, command))
        began = time.monotonic()
        try:
            p = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
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
        definitions = json.loads((FIXTURES / "sources.json").read_text())
        cases = json.loads((FIXTURES / "cases.json").read_text())
        refusals = json.loads((FIXTURES / "frontend-refusals.json").read_text())
        names = {s["source"] for s in definitions}
        require(len(names) == len(definitions) == 17 and names.isdisjoint(refusals)
                and names | set(refusals) == {p.stem for p in SOURCES.glob("*.compact")}, "whole source roster differs")
        require(len(cases) == len({r["id"] for r in cases}) == 20
                and {r["source"] for r in cases} == names, "case roster differs")
        for spec in definitions:
            require({r["entry"] for r in cases if r["source"] == spec["source"]}
                    == set(spec["entries"]), "an export has no declared execution case")
        generated = output / "generated"
        snapshots = {}
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
            dependencies.append(name + '={package=' + json.dumps(package) + ',path=' + json.dumps(str(path)) + ',features=["ledger-transaction"]}')
        manifest = consumer / "Cargo.toml"
        manifest.write_text('[workspace]\n[package]\nname="vector-product-elements-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + '\n'.join(dependencies) + '\n')
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_TS"] = str(ts)
        report["generated_inputs_before_build"] = {name: inventory(generated / name) for name in names}
        run("resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
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
            run(name + "-root-resolve", ["cargo", "metadata", "--offline", "--manifest-path", target / "Cargo.toml", "--format-version", "1"])
            run(name + "-root-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", target / "Cargo.toml", "--all-targets", "--all-features", "--", "-D", "warnings"])
            require(all(digest(target / path) == sha for path, sha in before.items()), "root Clippy edited emitted files")
            report["strict_sdk_roots"][name] = before
        report["frontend_refusals"] = {}
        for name, expected in refusals.items():
            source = SOURCES / (name + ".compact")
            require(digest(source) == expected["source_sha256"], "refusal source changed")
            rejected = run("refused-" + name, [compiler, "--target", "rust", "--skip-zk", "--rust-runtime-root", ROOT,
                           source.relative_to(ROOT), output / ("refused-" + name)], success=False)
            require(rejected.returncode == expected["exit"] != 0 and rejected.stdout == expected["stdout"]
                    and rejected.stderr == expected["stderr"], "exact frontend refusal changed")
            report["frontend_refusals"][name] = expected
        report["scope_controls"] = {}
        scope_controls = json.loads((FIXTURES / "scope-controls.json").read_text())
        require(set(scope_controls) == {p.stem for p in (FIXTURES / "unsupported").glob("*.compact")},
                "scope control roster differs")
        for name, expected in scope_controls.items():
            source = FIXTURES / "unsupported" / (name + ".compact")
            require(digest(source) == expected["source_sha256"], "scope control source changed")
            for lane, outcome in expected["lanes"].items():
                command = ([scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk"] if lane == "ir"
                           else [compiler, "--target", lane, "--skip-zk"])
                if lane == "rust":
                    command += ["--rust-runtime-root", ROOT, "--rust-functional-ledger8", "--rust-require-recording"]
                rejected = run("scope-" + name + "-" + lane,
                               command + [source.relative_to(ROOT), output / ("scope-" + name + "-" + lane)],
                               success=outcome["exit"] == 0)
                require(rejected.returncode == outcome["exit"], "scope control outcome changed")
                if outcome["exit"]:
                    require(rejected.stdout == outcome["stdout"] and rejected.stderr == outcome["stderr"],
                            "complete scope refusal changed")
            report["scope_controls"][name] = expected
        for name, before in snapshots.items():
            require(inventory(generated / name) == before, "generated artifact changed: " + name)
        for path, sha in pins.items():
            require(digest(ROOT / path) == sha, "source changed: " + path)
        for path, sha in tools.items():
            require(digest(path) == sha, "tool changed: " + path)
        require(reject_compiler_overrides(configuration_environment) == configuration, "Cargo configuration changed")
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source HEAD changed")
        pure_routes = sum(not next(s["stateful"] for s in definitions if s["source"] == row["source"]) for row in cases)
        report.update(status="passed", rust_tests=len(cases), stateful_routes=4*(len(cases)-pure_routes), pure_routes=pure_routes, generated_hashes=snapshots)
    except Exception as error:
        report.update(status="failed", error=str(error))
        raise
    finally:
        report["seconds"] = time.monotonic() - start
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(dict(status=report["status"], head=head, commands=len(report["commands"]), error=report.get("error"))))


if __name__ == "__main__":
    main()
