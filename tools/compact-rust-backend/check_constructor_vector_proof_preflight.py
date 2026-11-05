#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Build isolated constructor proof consumers and execute pre-key native checks.

This command never generates keys or proves. Compiler producer and proof consumer
identities remain separate. Keep complete SDKs and their bundled runtimes intact.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib

import check_functional_source_sdk as sdk
import constructor_proof_dependencies as dependencies
import did_proof_gate
import local_parity_gate as common

ROOT = sdk.ROOT
FIXTURE = ROOT / "tools/compact-rust-backend/tests/constructor_vector_operands"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_assets(directory, expected):
    complete = dict(expected["assets"])
    complete["compiler/contract-manifest.json"] = expected["manifest_sha256"]
    require(sdk.inventory(directory) == complete,
            f"complete producer SDK membership or bytes differ: {directory}")
    return complete


def stage_sdk(original, destination, expected):
    complete = check_assets(original, expected)
    shutil.copytree(original, destination)
    sdk.equal_inventory(destination, complete)
    return complete


def check_source(path, digest):
    require(sdk.digest(path) == digest, f"complete source bytes differ: {path}")


def identity_controls(output, original, expected, source, source_digest):
    """Real staging-boundary refusals before a mutated SDK can reach Cargo."""
    host = output / "HOST-identity-controls"
    host.mkdir()
    controls = {}
    for label in ("extra-build-script", "changed-manifest", "changed-ir", "swapped-programs"):
        root = host / label
        shutil.copytree(original, root)
        if label == "extra-build-script":
            (root / "contract/build.rs").write_text('fn main() { panic!("must never execute"); }\n')
        elif label == "changed-manifest":
            path = root / "compiler/contract-manifest.json"
            path.write_bytes(path.read_bytes() + b"\n")
        elif label == "changed-ir":
            path = root / "contract/compact-rust-ir.json"
            path.write_bytes(path.read_bytes() + b"\n")
        else:
            paths = sorted((root / "contract/checked-programs").glob("*.json"))
            require(len(paths) == 3, "complete Program roster required")
            a, b = paths[0].read_bytes(), paths[1].read_bytes()
            require(a != b, "distinct Program controls required")
            paths[0].write_bytes(b)
            paths[1].write_bytes(a)
        destination = host / (label + "-must-not-stage")
        try:
            stage_sdk(root, destination, expected)
        except ValueError as error:
            require("complete producer SDK membership or bytes differ" in str(error), "unrelated refusal")
        else:
            raise ValueError(f"mutated SDK reached staging: {label}")
        require(not destination.exists(), "mutated SDK must refuse before staging/Cargo")
        controls[label] = dict(refused_before_staging=True, files=sdk.inventory(root))
    changed = host / "changed-source.compact"
    changed.write_bytes(source.read_bytes() + b"\n// changed source identity\n")
    try:
        check_source(changed, source_digest)
    except ValueError as error:
        require("complete source bytes differ" in str(error), "unrelated source refusal")
    else:
        raise ValueError("changed complete source accepted")
    controls["changed-source"] = dict(refused_before_staging=True, sha256=sdk.digest(changed))
    return controls


def consumer_manifest(package):
    # Reuse the exact pinned proof cohort/feature/profile recipe. Drop unrelated
    # witnessed-example controls and change only this consumer's SDK identity.
    original = (sdk.EXAMPLE / "consumer/Cargo.toml").read_text()
    prefix = original.split("witness_result_negative=[]", 1)[0]
    old = 'package="compact-contract-field-witness-proof"'
    require(prefix.count(old) == 1, "SDK package recipe changed")
    prefix = prefix.replace(old, f'package="{package}"')
    return prefix + '\n[[bin]]\nname="constructor-vector-proof"\npath="proof/prove.rs"\nrequired-features=["proof"]\n\n[profile.proof-checks]' + original.split("[profile.proof-checks]", 1)[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--producer-receipt", type=Path, required=True)
    parser.add_argument("--producer-root", type=Path, required=True)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--model-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink() and not output.is_relative_to(ROOT),
            "fresh output outside checkout required")
    output.mkdir(parents=True, mode=0o700)
    report = dict(format="constructor-vector-proof-preflight/v1", status="running", commands=[],
                  sources={}, key_generation=False, proof_execution=False,
                  scope="selected compiler-produced complete SDKs; native constructor/Program and build-graph admission only; not cryptographic, integration or constructor-cost acceptance")
    receipt = output / "receipt.json"
    def save():
        receipt.write_text(json.dumps(report, indent=2) + "\n")
    save()
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    env.setdefault("CARGO_BUILD_JOBS", "2")
    frozen = {}
    frozen_roots = {}
    dependency_sources = {}
    def unchanged():
        for name, digest in frozen.items():
            require(sdk.digest(name) == digest, f"frozen input changed: {name}")
        for name, expected in frozen_roots.items():
            sdk.equal_inventory(Path(name), expected)
        dependencies.unchanged(dependency_sources)
        require(common.git_head() == report["qualifier_head"] and not common.dirty_paths(),
                "proof qualifier source changed")
        require(sdk.cargo_configuration(env) == report["cargo_configuration"], "Cargo configuration changed")
    def run(label, command, *, succeeds=True):
        unchanged()
        start = time.monotonic()
        try:
            result = sdk.bounded_command(command, env)
        except subprocess.TimeoutExpired as error:
            for name, data in (("stdout", error.stdout), ("stderr", error.stderr)):
                (output / f"{label}.{name}").write_text(data.decode(errors="replace") if isinstance(data, bytes) else data or "")
            report["commands"].append(dict(label=label, command=list(map(str,command)), timed_out=True))
            save()
            raise
        row = dict(label=label, command=list(map(str,command)), exit=result.returncode,
                   seconds=time.monotonic()-start)
        for name in ("stdout", "stderr"):
            path = output / f"{label}.{name}"
            path.write_text(getattr(result, name))
            row[name] = str(path)
            row[name+"_sha256"] = sdk.digest(path)
        report["commands"].append(row)
        save()
        require((result.returncode == 0) if succeeds else (result.returncode != 0),
                f"command result differs: {label}: {result.stderr[-2000:]}")
        return result
    try:
        require(not common.dirty_paths(), "clean committed qualifier required")
        require(not env.get("COMPACTC_SCHEME") and "COMPACT_RUST_RUNTIME_DIR" not in env,
                "original compiler and default runtime required")
        report["qualifier_head"] = common.git_head()
        report["qualifier_tree"] = subprocess.check_output(["git","rev-parse","HEAD^{tree}"],cwd=ROOT,text=True).strip()
        report["cargo_configuration"] = sdk.cargo_configuration(env)
        target = Path(env.get("CARGO_TARGET_DIR") or output / "cargo-target").resolve()
        require(not target.is_relative_to(ROOT) and not target.is_relative_to(output) or target == output / "cargo-target",
                "target must be outside source and evidence inputs")
        env["CARGO_TARGET_DIR"] = str(target)
        report["cargo_target"] = str(target)
        producer = json.loads(args.producer_receipt.read_text())
        compiler = json.loads(args.compiler_receipt.read_text())
        models = json.loads(args.model_receipt.read_text())
        require(producer["status"] == compiler["status"] == models["status"] == "passed", "accepted predecessors required")
        require(producer["head"] == compiler["head"] == models["producer_source"], "producer identities differ")
        report["producer_head"] = producer["head"]
        report["model_observer_head"] = models["qualifier_source"]
        require(subprocess.run(["git","merge-base","--is-ancestor",producer["head"],report["qualifier_head"]],cwd=ROOT).returncode == 0,
                "producer is not an ancestor")
        paths = [args.producer_receipt, args.compiler_receipt, args.model_receipt, Path(__file__),
                 Path(sdk.__file__), Path(dependencies.__file__), Path(did_proof_gate.__file__), Path(common.__file__),
                 ROOT / "Cargo.toml", ROOT / "Cargo.lock", sdk.EXAMPLE / "consumer/Cargo.toml"]
        paths += sdk.tracked_source_files(FIXTURE)
        paths += sdk.tracked_source_files(ROOT / "testkit-rs")
        for relative, digest in producer["inputs"].items():
            if relative.startswith(("compiler/", "runtime-rs/", "runtime-rs-macros/", "tools/compact-rust-backend/src/")):
                p = ROOT / relative
                require(sdk.digest(p) == digest, f"compiler producer source differs: {relative}")
                paths.append(p)
        frozen.update({str(p.resolve()):sdk.digest(p) for p in paths})
        python = Path(sys.executable).resolve()
        frozen[str(python)] = sdk.digest(python)
        frozen[dependencies.GIT] = sdk.digest(dependencies.GIT)
        report["controller_tools"] = dict(python=str(python), python_version=sys.version,
                                          git=dependencies.GIT)
        binary = Path(models["binary"]["path"])
        require(sdk.digest(binary) == models["binary"]["sha256"], "selected source-built material runner changed")
        frozen[str(binary)] = sdk.digest(binary)
        compiler_path = Path(compiler["compiler"])
        require(sdk.digest(compiler_path) == compiler["compiler_sha256"], "producer compiler changed")
        frozen[str(compiler_path)] = sdk.digest(compiler_path)
        prereq = did_proof_gate.prerequisites(env)
        report["prerequisites"] = prereq
        report["dust_fixture_hashes"] = did_proof_gate.dust_fixture_inventory(Path(prereq["MIDNIGHT_LEDGER_TEST_STATIC_DIR"]))
        frozen.update(report["dust_fixture_hashes"])
        frozen[prereq["zkir"]] = sdk.digest(prereq["zkir"])
        report["versions"] = {}
        tools = report["tools"] = {}
        for tool in ("cargo", "rustc", "rustfmt", "cargo-clippy", "clippy-driver"):
            path = subprocess.check_output(["rustup","which",tool],env=env,cwd=ROOT,text=True).strip()
            frozen[path] = sdk.digest(path)
            tools[tool] = dict(path=path, sha256=frozen[path])
        require(len({str(Path(t["path"]).parent) for t in tools.values()}) == 1,
                "one resolved Rust toolchain directory required")
        env["PATH"] = str(Path(tools["cargo"]["path"]).parent) + os.pathsep + env["PATH"]
        cargo = tools["cargo"]["path"]
        for tool, command in (("compiler",[compiler_path,"--version"]),("zkir",[prereq["zkir"],"--version"]),("cargo",[cargo,"--version"]),("rustc",[tools["rustc"]["path"],"-Vv"])):
            report["versions"][tool] = run(tool+"-version", command).stdout.strip()
        require(report["versions"]["compiler"] == compiler["version"], "compiler version differs")
        material = json.loads(run("verify-material",[binary,"--prepare-proof-material","--verify-only","--parameter-k","6"]).stdout)
        require(material.get("mode") == "verify-only" and material.get("verification") == "all files checked against locked upstream SHA256 tables"
                and Path(material["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve()
                and 6 in [p["k"] for p in material["parameters"]], "official k6 material verification required")
        report["verified_material"] = material
        for asset in material["assets"]+material["parameters"]:
            p = Path(env["MIDNIGHT_PP"]) / asset["name"]
            frozen[str(p)] = sdk.digest(p)
        specs = json.loads((FIXTURE / "sources.json").read_text())
        cases = json.loads((FIXTURE / "cases.json").read_text())
        require(len(specs)==13 and len([c for c in cases if c["error"] is None])==15, "complete source/case membership differs")
        cohort = {(p["name"],p["version"],p.get("source")):p.get("checksum") for p in tomllib.loads((ROOT/"Cargo.lock").read_text())["package"]}
        for spec in specs:
            role=spec["role"]
            require(spec["sha256"] == producer["sources"][role]["source_sha256"], "source identity differs")
            check_source(ROOT/spec["path"], spec["sha256"])
            for entry in spec["entries"]:
                row=[m for m in models["inventory"]["circuits"] if m["source"]==role and m["circuit"]==entry]
                require(len(row)==1 and row[0]["k"]==6, "exact intact model observation differs")
            original=args.producer_root/role
            expected_manifest=producer["sources"][role]["manifest"]
            complete=check_assets(original,expected_manifest)
            if not report["sources"]:
                report["identity_controls"]=identity_controls(output,original,expected_manifest,ROOT/spec["path"],spec["sha256"])
            frozen_roots[str(original)]=complete
            frozen.update({str(original/rel):digest for rel,digest in complete.items()})
            lane=output/role;lane.mkdir()
            generated=lane/"generated"
            raw=stage_sdk(original,generated,expected_manifest)
            frozen_roots[str(generated)]=raw
            frozen.update({str(generated/rel):digest for rel,digest in raw.items()})
            consumer=lane/"consumer";(consumer/"proof").mkdir(parents=True)
            manifest=consumer/"Cargo.toml"
            package=tomllib.loads((generated/"contract/Cargo.toml").read_text())["package"]["name"]
            manifest.write_text(consumer_manifest(package))
            shutil.copyfile(FIXTURE/"proof.rs",consumer/"proof/prove.rs")
            shutil.copyfile(FIXTURE/"constructors"/(spec["constructor_consumer"]+".rs"),consumer/"proof/constructor.rs")
            recipe=sdk.source_testkit(ROOT/"testkit-rs",lane/"testkit-rs")
            shutil.copyfile(ROOT/"Cargo.lock",consumer/"Cargo.lock")
            metadata=json.loads(run(role+"-metadata",[cargo,"metadata","--offline","--format-version","1","--features","proof","--manifest-path",manifest]).stdout)
            graph=sdk.dependency_graph(metadata,consumer/"Cargo.lock",lane,cohort)
            dependencies.authenticate(metadata,cohort,dependency_sources)
            frozen.update({str(consumer/rel):digest for rel,digest in sdk.inventory(consumer).items()})
            frozen.update({str(lane/"testkit-rs"/rel):digest for rel,digest in sdk.inventory(lane/"testkit-rs").items()})
            frozen_roots[str(consumer)]=sdk.inventory(consumer)
            frozen_roots[str(lane/"testkit-rs")]=sdk.inventory(lane/"testkit-rs")
            build=run(role+"-build",[cargo,"build","--locked","--offline","--profile","proof-checks","--features","proof","--bin","constructor-vector-proof","--manifest-path",manifest,"--message-format","json"])
            messages=[json.loads(line) for line in build.stdout.splitlines() if line.strip()]
            artifacts=[m for m in messages if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["name"]=="constructor-vector-proof"]
            require(len(artifacts)==1, "one actual proof binary required")
            actual=Path(artifacts[0]["executable"]).resolve()
            require(actual.is_relative_to(target), "proof binary escaped target")
            (lane/"bin").mkdir()
            executable=common.stable_copy(actual,lane/"bin/constructor-vector-proof")
            frozen[executable["snapshot"]]=executable["sha256"]
            run(role+"-clippy",[cargo,"clippy","--locked","--offline","--profile","proof-checks","--features","proof","--bin","constructor-vector-proof","--manifest-path",manifest,"--","-D","warnings"])
            native=json.loads(run(role+"-native",[executable["snapshot"],"--preflight",generated,role,FIXTURE/"cases.json",lane/"material-not-created",lane/"proofs-not-created"]).stdout)
            expected={c["id"] for c in cases if c["role"]==role and c["error"] is None}
            require(native["status"]=="passed" and {r["case"] for r in native["results"]}==expected, "all selected cases must execute")
            report["sources"][role]=dict(raw_sdk=raw,testkit_recipe=recipe,dependency_graph=graph,binary=executable,native_preflight=native)
            save()
        unchanged()
        require(len(report["sources"])==13, "all sources must pass")
        report.update(status="passed", selected_cases=15, input_bytes_unchanged=True)
    except BaseException as error:
        report.update(status="failed",error=repr(error))
        raise
    finally:
        report["frozen_inputs"] = frozen
        report["frozen_directories"] = frozen_roots
        report["dependency_sources"] = dependency_sources
        save()
        print(json.dumps({k:report[k] for k in ("format","status","qualifier_head","producer_head","key_generation","proof_execution","selected_cases","error") if k in report},indent=2))


if __name__ == "__main__":
    main()
