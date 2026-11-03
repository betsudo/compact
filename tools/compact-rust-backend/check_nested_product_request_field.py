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

"""Maintain the complete same-type nested request-field regression.

Strictly lint three complete unedited SDKs and their actual consumer test code.
Execute four typed routes and two VM replays per source. Retain the original
three marked type errors; both Vector-leaf fixtures also check inner dimension
and consumed-Program refusal, with matching positive controls.
This metadata-only prerequisite has no new proof or TS execution claim.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib

from check_constant_vector_projection import compile_failure
from check_passport_current_profile import bounded_command
from check_stateful_product_vector_widening import (
    digest, inventory, reject_compiler_overrides, require, source_manifest_snapshot,
)

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "tools/compact-rust-backend/tests/nested_product_request_field"
SOURCES = ROOT / "examples/rust_backend/nested_product_request_field"
SOURCE = SOURCES / "nested_product_request_field.compact"
SOURCE_SHA256 = "21dcb591417df0414440f9c8f94b6369c62ff41a580ad5bea9ddbbccaa58f367"
LEAF_SOURCES = ROOT / "examples/rust_backend/vector_leaf_request_field"
COHORT = [
    dict(name="original", source=SOURCE, sha256=SOURCE_SHA256, fixtures=FIXTURES,
         static=("bound", "arity", "dimension")),
    dict(
        name="vector_leaf",
        source=LEAF_SOURCES / "component_vector.compact",
        sha256="d1dd73a5670d0c1c2e6f90f17223b4cf85905b703663798bb55831ce2c6a36f9",
        fixtures=FIXTURES / "vector_leaf",
        static=("bound", "arity", "dimension", "inner_dimension", "double_execute"),
    ),
    dict(
        name="nested_vector_leaf",
        source=LEAF_SOURCES / "nested_vector.compact",
        sha256="5fc3805aef9edeea3c3d572b2b60e836f76ed9e64e4942d117d3ecc7d1f859b9",
        fixtures=FIXTURES / "nested_vector_leaf",
        static=("bound", "arity", "dimension", "inner_dimension", "double_execute"),
    ),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "output must be absent")
    env = dict(os.environ)
    config = reject_compiler_overrides(dict(env))
    env.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "use pinned Rust1.99.0")
    target = Path(env.get("CARGO_TARGET_DIR", str(output / "cargo-target")))
    target = (ROOT / target).resolve()
    require(all(not target.is_relative_to((output / "cases" / spec["name"] / "generated").resolve())
                for spec in COHORT), "Cargo target must be outside every immutable generated SDK")
    env["CARGO_TARGET_DIR"] = str(target)
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "clean source required")
    build = json.loads(args.compiler_receipt.read_text())
    require(build.get("format") == "compact-original-existing-compiler-recipe/v1", "original recipe receipt format required")
    require(build["status"] == "passed" and (build["head"], build["tree"]) == (head, tree),
            "original compiler receipt must bind this exact clean HEAD/tree")
    compiler, scheme = [Path(build[key]["path"]).resolve(strict=True)
                        for key in ("compactc", "compactc-scheme")]
    require(compiler.parent == scheme.parent, "one original compiler package required")
    require(Path(build["store"]).resolve(strict=True) == compiler.parent.parent, "receipt store differs from compiler package")
    for key, path in (("compactc", compiler), ("compactc-scheme", scheme)):
        require(digest(path) == build[key]["sha256"], "original compiler changed")
    require(set(SOURCES.glob("*.compact")) == {SOURCE}
            and set(LEAF_SOURCES.glob("*.compact")) == {spec["source"] for spec in COHORT[1:]},
            "complete declared source cohort changed")
    require(all(digest(spec["source"]) == spec["sha256"] for spec in COHORT),
            "complete declared source bytes changed")
    inputs = [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "flake.nix", "flake.lock")]
    for directory in (ROOT / "compiler", ROOT / "runtime-rs", ROOT / "runtime-rs-macros",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES, SOURCES, LEAF_SOURCES):
        inputs.extend(p for p in directory.rglob("*") if p.suffix in (".ss", ".rs", ".json", ".toml", ".compact"))
    inputs.extend(Path(__file__).parent.glob("*.py"))
    pins = {str(p): digest(p) for p in inputs}
    pins.update({str(p): digest(p) for p in (compiler, scheme, args.compiler_receipt.resolve())})
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    pins[str(rustup)] = digest(rustup)
    rust_tools = {}
    for name in ("cargo", "rustc", "rustfmt", "clippy-driver", "cargo-clippy"):
        p = Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name], text=True).strip()).resolve(strict=True)
        rust_tools[name] = str(p)
        pins[str(p)] = digest(p)
    env["PATH"] = str(Path(rust_tools["cargo"]).parent) + os.pathsep + env["PATH"]
    env["RUSTC"] = rust_tools["rustc"]
    output.mkdir(parents=True)
    report = dict(status="incomplete", head=head, tree=tree, commands=[], inputs=pins,
                  cargo_configuration_sha256=config, rust_tools=rust_tools,
                  environment={k: env.get(k) for k in ("RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_TARGET_DIR",
                                                       "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS")},
                  scope=__doc__, bound_build_inputs={}, resolved_locks={})
    bound = report["bound_build_inputs"]
    root_packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    root_versions = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in root_packages}
    allowed_local = {("nested-product-request-field-consumer", "0.1.0")}
    started = time.monotonic()

    def bind(paths):
        for path in paths:
            sha = digest(path)
            require(str(path) not in bound or bound[str(path)] == sha, "build input changed before reuse")
            bound[str(path)] = sha

    def unchanged():
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source changed")
        require(reject_compiler_overrides(dict(os.environ)) == config, "Cargo configuration changed")
        for path, sha in {**pins, **bound}.items():
            require(Path(path).is_file() and digest(path) == sha, "bound input changed: " + path)

    def run(label, command, expected=0):
        unchanged()
        began = time.monotonic()
        command = list(map(str, command))
        try:
            p = bounded_command(command, env)
        except subprocess.TimeoutExpired as error:
            for suffix, value in (("stdout", error.stdout), ("stderr", error.stderr)):
                (output / (label + "." + suffix)).write_bytes(value if isinstance(value, bytes) else (value or "").encode())
            report["commands"].append(dict(label=label, command=command, timeout=True))
            raise
        row = dict(label=label, command=command, exit=p.returncode, seconds=time.monotonic() - began)
        for suffix in ("stdout", "stderr"):
            path = output / (label + "." + suffix)
            path.write_text(getattr(p, suffix))
            row[suffix + "_sha256"] = digest(path)
        report["commands"].append(row)
        unchanged()
        require(p.returncode == expected, label + " failed: retained raw diagnostics")
        return p

    def resolve(manifest, label):
        lock = manifest.with_name("Cargo.lock")
        shutil.copyfile(ROOT / "Cargo.lock", lock)
        bind([manifest])
        run(label, ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"])
        packages = tomllib.loads(lock.read_text())["package"]
        for package in packages:
            key = (package["name"], package["version"], package.get("source"))
            require(key in root_versions and package.get("checksum") == root_versions[key]
                    or key[2] is None and key[:2] in allowed_local,
                    "resolved package differs from repository lock")
        bind([lock])
        report["resolved_locks"][str(lock)] = dict(sha256=digest(lock), packages=packages)

    def qualify(spec):
        case = {"source": str(spec["source"].relative_to(ROOT)), "source_sha256": spec["sha256"]}
        report["cases"][spec["name"]] = case
        def execute(label, command, expected=0):
            return run(spec["name"] + "-" + label, command, expected)
        generated = output / "cases" / spec["name"] / "generated"
        execute("generate-sdk", [compiler, "--target", "rust", "--skip-zk", "--rust-functional-ledger8",
            "--rust-require-recording", "--rust-runtime-root", ROOT, spec["source"].relative_to(ROOT), generated])
        independent = output / "cases" / spec["name"] / "independent"
        execute("independent-ir", [scheme, "--emit-rust-ir", "--skip-ts", "--skip-zk", spec["source"].relative_to(ROOT), independent])
        require((generated / "contract/compact-rust-ir.json").read_bytes()
                == (independent / "contract/compact-rust-ir.json").read_bytes(), "entrypoint IR differs")
        case["generated_manifest"] = source_manifest_snapshot(generated, True)
        require(set(inventory(generated / "contract/checked-programs")) == {"program-0.json"}, "expected one complete Program")
        caps = json.loads((generated / "contract/rust-capabilities.json").read_text())
        require(caps["schema_version"] == 3 and len(caps["circuits"]) == 1, "capability roster changed")
        cap = caps["circuits"][0]
        require(cap["name"] == "run" and cap["recorded"] is True and cap["observed_call"] is True
                and cap["proof_required"] is True, "required APIs missing")
        before = inventory(generated)
        case["generated_before_cargo"] = before
        bind(generated / name for name in before)
        manifest = generated / "contract/Cargo.toml"
        package = tomllib.loads(manifest.read_text())["package"]["name"]
        allowed_local.add((package, "0.1.0"))
        resolve(manifest, spec["name"] + "-sdk-lock")
        execute("sdk-strict", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest,
                           "--all-targets", "--all-features", "--", "-D", "warnings"])
        consumer = output / "cases" / spec["name"] / "consumer"
        (consumer / "src/bin").mkdir(parents=True)
        shutil.copyfile(spec["fixtures"] / "consumer.rs", consumer / "consumer.rs")
        cm = consumer / "Cargo.toml"
        cm.write_text('[workspace]\n[package]\nname="nested-product-request-field-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\ncandidate={package=' + json.dumps(package) + ',path=' + json.dumps(str(manifest.parent)) + ',features=["ledger-transaction"]}\n')
        bind(p for p in consumer.rglob("*.rs"))
        resolve(cm, spec["name"] + "-consumer-lock")
        result = execute("four-route-test", ["cargo", "test", "--locked", "--offline", "--manifest-path", cm, "--lib"])
        require("test result: ok. 1 passed; 0 failed; 0 ignored;" in result.stdout, "consumer method did not execute")
        execute("consumer-strict", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", cm,
                                "--lib", "--tests", "--", "-D", "warnings"])
        positives = ("positive", "once") if "double_execute" in spec["static"] else ("positive",)
        for name in (*spec["static"], *positives):
            shutil.copyfile(spec["fixtures"] / (name + ".rs"), consumer / "src/bin" / (name + ".rs"))
        bind(p for p in (consumer / "src/bin").glob("*.rs"))
        case["typed_refusals"] = {}
        for name in spec["static"]:
            code = (consumer / "src/bin" / (name + ".rs")).read_text()
            p = execute("wrong-" + name, ["cargo", "check", "--locked", "--offline", "--manifest-path", cm,
                                     "--bin", name, "--message-format=json"], 101)
            error = "E0382" if name == "double_execute" else "E0308"
            case["typed_refusals"][name] = compile_failure(p, name, error, code)
            positive = "once" if name == "double_execute" else "positive"
            execute("positive-" + name, ["cargo", "check", "--locked", "--offline",
                                        "--manifest-path", cm, "--bin", positive])
        require(inventory(generated) == {**before, "contract/Cargo.lock": digest(manifest.with_name("Cargo.lock"))},
                "generated SDK changed beyond its own Cargo.lock")
        unchanged()
        case.update(rust_methods=1, rust_routes=4, recorded_replays=2)

    try:
        report["cases"] = {}
        for spec in COHORT:
            qualify(spec)
        require(len(report["commands"]) == 47, "complete three-source gate command count changed")
        for key in ("generated_manifest", "generated_before_cargo", "typed_refusals"):
            report[key] = report["cases"]["original"][key]
        report.update(status="passed", rust_methods=3, rust_routes=12, recorded_replays=6,
                      proof_execution=False, full_integration_acceptance="OPEN")
    except BaseException as error:
        report.update(status="failed", error=repr(error))
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ("status", "head", "rust_routes", "recorded_replays")}))


if __name__ == "__main__":
    main()
