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

"""Current complete constructor equality SDKs and independent TS/native controls.

No keys/proofs or constructor/deployment cryptography are performed here.
Dependency lock identities are checked; this is not a transitive build attestation.
"""
import argparse
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
from check_passport_current_profile import cargo_target
import check_passport_current_profile as process_owner
import constructor_proof_dependencies as proof_dependencies
from check_stateful_product_vector_widening import digest, inventory, reject_compiler_overrides, require

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "constructor_equality"
ROLES = ("boolean_parameter", "field_literal_eq", "field_pair_eq", "field_pair_ne",
         "bool_pair_eq", "uint_pair_eq", "bytes_pair_eq", "nested_eq")
NEGATIVES = ("wrong_type", "wrong_length", "wrong_right")
BOUNDARY_ROLES = ("mixed_uint_eq", "mixed_uint_ne", "tuple_eq", "tuple_ne",
                  "vector_eq", "vector_ne", "struct_eq", "struct_ne")
BOUNDARY_NEGATIVES = ("wrong_uint_width", "wrong_outer_length", "wrong_tuple_component",
                      "wrong_inner_length", "wrong_nominal_struct")


TUPLE_ROLES = ('aliased_vector_variable', 'asymmetric_eq', 'direct_tuple_variable', 'direct_vector_variable', 'flat_eq', 'flat_ne', 'hash_tuple_argument', 'mixed_literal_eq', 'nested_eq', 'owned_capture_eq', 'reversed_eq', 'struct_literal_field', 'tuple_to_vector', 'uint_to_field_tuple', 'vector_to_tuple', 'widened_vector_capture')
TUPLE_SUPPLEMENTAL = ('aliased_vector_variable', 'direct_tuple_variable', 'direct_vector_variable', 'hash_tuple_argument', 'struct_literal_field', 'tuple_to_vector', 'uint_to_field_tuple', 'vector_to_tuple', 'widened_vector_capture')
TUPLE_NEGATIVES = ("wrong_input_width", "wrong_tuple_width", "wrong_capture_length", "wrong_vector_component", "wrong_field_pair")


def main(*, boundaries=False, tuple_guards=False):
    # Preserve both old finite cohorts and their default CLI/imports.
    require(not (boundaries and tuple_guards), "one finite cohort required")
    extended = boundaries or tuple_guards
    fixtures = (FIXTURES.with_name("constructor_equality_tuple_guards") if tuple_guards else
                FIXTURES.with_name("constructor_equality_boundaries") if boundaries else FIXTURES)
    sources = fixtures if tuple_guards else fixtures / "sources" if boundaries else fixtures
    roles = TUPLE_ROLES if tuple_guards else BOUNDARY_ROLES if boundaries else ROLES
    negatives = TUPLE_NEGATIVES if tuple_guards else BOUNDARY_NEGATIVES if boundaries else NEGATIVES
    case_count = 105 if tuple_guards else 30 if boundaries else 16
    def source_for(role):
        return sources / ("supplemental" if tuple_guards and role in TUPLE_SUPPLEMENTAL else "") / (role + ".compact")
    require(__debug__, "qualification requires Python without optimization")
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
    recipe = json.loads(args.compiler_receipt.read_bytes())
    require(recipe["status"] == "passed" and recipe["head"] == head and recipe["tree"] == tree,
            "current clean original compiler recipe required")
    if tuple_guards:
        specs = json.loads((fixtures / "sources.json").read_bytes())
        require(tuple(row["role"] for row in specs) == roles and all(
            row["path"] == str(source_for(row["role"]).relative_to(ROOT)) and
            row["sha256"] == digest(source_for(row["role"])) and
            row["entries"] == ["read", "advance", "advance_other"] for row in specs),
            "complete tuple source/export roster differs")
    compiler, scheme = [Path(recipe[k]["path"]).resolve(strict=True) for k in ("compactc", "compactc-scheme")]
    require(compiler.parent == scheme.parent, "one original compiler package required")
    require(all(digest(p) == recipe[k]["sha256"] for k, p in (("compactc", compiler), ("compactc-scheme", scheme))),
            "compiler identity differs")
    env = dict(os.environ)
    config = reject_compiler_overrides(dict(env))
    env.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "Rust1.99.0 required")
    env["CARGO_TARGET_DIR"] = str(cargo_target(env, out))
    env.pop("LD_LIBRARY_PATH", None)
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    rust = {name: Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name],
            text=True).strip()).resolve(strict=True) for name in ("cargo", "rustc")}
    require(subprocess.check_output([str(rust["rustc"]), "--version"], text=True).startswith("rustc 1.99.0 "),
            "actual Rust compiler version differs")
    require(subprocess.check_output([str(rust["cargo"]), "--version"], text=True).startswith("cargo 1.99.0 "),
            "actual Cargo version differs")
    env.update(RUSTC=str(rust["rustc"]), CARGO=str(rust["cargo"]))
    env["PATH"] = str(rust["cargo"].parent) + os.pathsep + env["PATH"]
    runtime = args.ts_runtime.resolve(strict=True)
    node = Path(shutil.which("node")).resolve(strict=True)
    paths = [Path(__file__), args.compiler_receipt, compiler, scheme, node, rustup, *rust.values(), ROOT / "Cargo.lock"]
    paths.extend(fixtures.rglob("*"))
    paths.extend(FIXTURES.rglob("*"))  # Historical frontend/native refusal sources.
    paths.extend(Path(__file__).parent.glob("*.py"))
    for directory in ("compiler", "runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src"):
        paths.extend((ROOT / directory).rglob("*"))
    pins = {str(p): digest(p) for p in paths if p.is_file()}
    out.mkdir(parents=True)
    report = dict(status="running", head=head, tree=tree, inputs=pins, commands=[], sources={},
                  scope=__doc__, proof_execution=False)
    bound = {}
    dependency_sources = {}
    started = time.monotonic()

    def unchanged():
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source changed")
        require(reject_compiler_overrides(dict(os.environ)) == config, "Cargo configuration changed")
        require(all(Path(p).is_file() and digest(p) == h for p, h in {**pins, **bound}.items()), "input changed")
        if extended:
            proof_dependencies.unchanged(dependency_sources)

    def run(label, argv, success=True, extra=None):
        unchanged()
        argv = list(map(str, argv))
        if argv[0] == "cargo":
            argv[0] = str(rust["cargo"])
        start = time.monotonic()
        try:
            if extended:
                result = process_owner.bounded_command(argv, env if extra is None else dict(env, **extra))
            else:
                result = subprocess.run(argv, cwd=ROOT, env=env if extra is None else dict(env, **extra),
                                        capture_output=True, text=True, timeout=600)
        except subprocess.TimeoutExpired as error:
            for stream in ("stdout", "stderr"):
                data = getattr(error, stream)
                (out / (label + "." + stream)).write_bytes(data.encode() if isinstance(data, str) else data or b"")
            report["commands"].append(dict(label=label, argv=argv, exit=None, timeout=process_owner.COMMAND_TIMEOUT_SECONDS if extended else 600,
                                            seconds=time.monotonic() - start))
            raise
        row = dict(label=label, argv=argv, exit=result.returncode, seconds=time.monotonic() - start)
        if extended:
            row["timeout_seconds"] = process_owner.COMMAND_TIMEOUT_SECONDS
        for stream in ("stdout", "stderr"):
            p = out / (label + "." + stream)
            p.write_text(getattr(result, stream))
            row[stream + "_sha256"] = digest(p)
        report["commands"].append(row)
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        unchanged()
        require(not success or result.returncode == 0, label + " failed; retained raw streams")
        return result

    try:
        generated = out / "generated"
        snapshots, dependencies = {}, []
        for role in roles:
            destination = generated / role
            run(role + "-generate", [compiler, "--target", "rust", "--target", "ts", "--skip-zk",
                "--rust-functional-ledger8", "--rust-require-recording", "--rust-runtime-root", ROOT,
                source_for(role), destination])
            manifest = generated_manifest_snapshot(destination)
            snapshots[role] = inventory(destination)
            report["sources"][role] = dict(source_sha256=digest(source_for(role)), manifest=manifest)
            package = tomllib.loads((destination / "contract/Cargo.toml").read_text())["package"]["name"]
            dependencies.append(role + "={package=" + json.dumps(package) + ",path="
                                + json.dumps(str(destination / "contract")) + ',features=["ledger-transaction"]}')
        # Current frontend and native refusals remain distinct from positive behavior.
        report["refusals"] = {}
        for role, message in (("witness_condition", "constructor can initialize"),
                              ("field_arithmetic_condition", "constructor can initialize"),
                              ("undisclosed_field_eq", "potential witness-value disclosure"),
                              ("mismatched_equality", "incompatible types Field and Boolean")):
            result = run(role + "-refusal", [compiler, "--target", "rust", "--skip-zk",
                "--rust-functional-ledger8", "--rust-require-recording", FIXTURES / (role + ".compact"),
                out / "refused" / role], success=False)
            require(result.returncode != 0 and message in result.stderr, "wrong refusal: " + role)
            report["refusals"][role] = message
        if tuple_guards:
            from constructor_tuple_scope import RUST_REFUSALS
            for role, message in RUST_REFUSALS.items():
                result = run(role + "-known-alias-refusal", [compiler, "--target", "rust", "--skip-zk",
                    "--rust-functional-ledger8", "--rust-require-recording", "--rust-runtime-root", ROOT,
                    fixtures / "supplemental" / (role + ".compact"), out / "refused" / role], success=False)
                require(result.returncode == (255 if role == "aliased_tuple_variable" else 1) and message in result.stderr,
                        "wrong known alias transport refusal: " + role)
                report["refusals"][role] = message
        modules = out / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        capture = out / "typescript.json"
        run("typescript", [node, fixtures / "capture.mjs", generated, runtime, fixtures / "cases.json", capture])
        observed = json.loads(capture.read_bytes())
        require(observed["node"]["path"] == str(node) and observed["node"]["sha256"] == digest(node), "Node identity differs")
        bound.update(observed["runtime"])
        bound[str(capture)] = digest(capture)
        reference = out / "reference.json"
        reference.write_text(json.dumps(observed["rows"]) + "\n")
        bound[str(reference)] = digest(reference)
        consumer = out / "consumer"
        (consumer / "src/bin").mkdir(parents=True)
        shutil.copyfile(fixtures / "consumer.rs", consumer / "consumer.rs")
        if extended:
            shutil.copyfile(fixtures / "cases.json", consumer / "cases.json")
        text = '[workspace]\n[package]\nname="constructor-equality-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\nmidnight-compact-runtime={path=' + json.dumps(str(ROOT / "runtime-rs")) + '}\n'
        bins = tuple(n for name in negatives for n in (name, name + "_twin")) if extended else negatives
        text += "\n".join(dependencies) + "\n[features]\n" + "\n".join(n + "=[]" for n in bins) + "\n"
        for name in bins:
            shutil.copyfile(fixtures / (name + ".rs"), consumer / ("src/bin/" + name + ".rs"))
            text += '\n[[bin]]\nname="' + name + '"\npath="src/bin/' + name + '.rs"\nrequired-features=["' + name + '"]\n'
        manifest = consumer / "Cargo.toml"
        manifest.write_text(text)
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        metadata = json.loads(run("resolve", ["cargo", "metadata", "--offline", "--manifest-path", manifest, "--format-version", "1"]).stdout)
        locked = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
        for package in metadata["packages"]:
            key = package["name"], package["version"], package.get("source")
            if key[2] is None:
                path = Path(package["manifest_path"]).resolve(strict=True)
                require(path.is_relative_to(generated) or path.parent in (consumer, ROOT / "runtime-rs", ROOT / "runtime-rs-macros"), "unexpected local dependency")
            else:
                require(key in locked, "dependency not in original lock")
        for package in tomllib.loads((consumer / "Cargo.lock").read_text())["package"]:
            if package.get("source"):
                key = package["name"], package["version"], package["source"]
                require(key in locked and package.get("checksum") == locked[key], "lock identity/checksum differs")
        if extended:
            proof_dependencies.authenticate(metadata, locked, dependency_sources)
            report["dependency_sources"] = dependency_sources
        bound.update({str(consumer / p): h for p, h in inventory(consumer).items()})
        env["COMPACT_CONSTRUCTOR_EQUALITY_TS"] = str(reference)
        argv = ["cargo", "test", "--offline", "--locked", "--manifest-path", manifest, "--lib"]
        result = run("consumer", [*argv, "--", "--nocapture"] if boundaries else argv)
        require(re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result.stdout) == [("1", "0", "0")], "consumer did not execute")
        if extended:
            require(("tuple-guard-cases:105 calls:1260 replays:630" if tuple_guards else
                     "boundary-cases:30 calls:360 replays:180") in result.stdout, "finite execution counts differ")
        for name in negatives:
            if extended:
                twin = name + "_twin"
                run(twin, ["cargo", "check", "--offline", "--locked", "--manifest-path", manifest,
                    "--bin", twin, "--features", twin])
            result = run(name, ["cargo", "check", "--offline", "--locked", "--manifest-path", manifest,
                "--bin", name, "--features", name, "--message-format=json"], success=False)
            compile_failure(result, name, "E0308", (consumer / ("src/bin/" + name + ".rs")).read_text())
        changed = json.loads(reference.read_bytes())
        changed[0]["initial"] = "8"
        wrong = out / "wrong-reference.json"
        wrong.write_text(json.dumps(changed))
        if extended:
            bound[str(wrong)] = digest(wrong)
        result = run("changed-reference", argv, success=False, extra={"COMPACT_CONSTRUCTOR_EQUALITY_TS": str(wrong)})
        require(result.returncode != 0 and "test result: FAILED. 0 passed; 1 failed;" in result.stdout, "changed reference was accepted")
        if extended:
            restored = run("restored-reference", argv)
            require(re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", restored.stdout)
                    == [("1", "0", "0")], "restored consumer did not execute")
        require(all(inventory(generated / role) == before for role, before in snapshots.items()), "generated SDK changed")
        unchanged()
        report.update(status="passed", cases=case_count,
                      cohort="implicit-tuple-guards" if tuple_guards else "operand-boundaries" if boundaries else "historical-equality",
                      generated_snapshots=snapshots)
    except BaseException as error:
        report.update(status="failed", error=repr(error))
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
