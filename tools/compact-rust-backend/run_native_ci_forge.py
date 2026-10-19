#!/usr/bin/env python3

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

"""Explicit local/CI experiment: one original native owner, no reusable publication."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
from types import SimpleNamespace

import qualify_linux_native as native

ROOT = Path(__file__).resolve().parents[2]
PUBLIC = "github:betsudo/forge/d64f52db7239b0ca4891fb0316dca6adb54a5db5"


def execute(command, env, *, capture=False):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, check=True,
                            capture_output=capture, text=capture)
    return result.stdout.strip() if capture else None


def owner_commands(prepared, output, helper):
    plan = output / "native-plan"
    common = [sys.executable, "-B", ROOT / "tools/compact-rust-backend/forge_ci.py"]
    return [
        [*common, "plan", "--compiler", prepared["compiler"], "--forge", prepared["forge"],
         "--helper", helper, "--proof-runner", prepared["proof"], "--params-dir", prepared["params"],
         "--output", plan, "--generation", "linux-native-owner-experiment",
         "--native-script", ROOT / "tools/compact-rust-backend/check_native_ci.sh",
         "--native-policy", prepared["policy_path"], "--native-helper", prepared["native_helper"]],
        [*common, "run", "--plan", plan, "--snapshot", output / "absent-snapshot",
         "--uncached-zkir", prepared["zkir"], "--max-bytes", 268435456,
         "--snapshot-out", output / "stopped-private-snapshot"],
    ]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    for name in ("qualification-dir", "proof-runner", "helper"):
        parser.add_argument("--" + name, type=Path)
    args = parser.parse_args(argv)
    selected = (args.qualification_dir, args.proof_runner, args.helper)
    native.require(not any(selected) or all(selected), "qualified input requires directory, proof runner and helper together")
    native.require(platform.system() == "Linux" and platform.machine() == "x86_64", "actual x86_64 Linux required")
    compiler = args.compiler.resolve(strict=True)
    output = args.output.resolve()
    native.require(not output.exists() and not output.is_relative_to(ROOT), "fresh external experiment directory required")
    output.mkdir(parents=True)
    wrapper_sha = native.adapter.digest(Path(__file__))
    env = dict(os.environ, MIDNIGHT_PP=str(output / "parameters"))
    env.pop("COMPACTC_SCHEME", None)
    env.pop("COMPACT_ZKIR_SHIM", None)
    env["COMPACT_ZKIR_MODE"] = "uncached"
    if args.qualification_dir:
        # Existing workflow already performed provisioning and its five finite laps.
        # Reuse the same admission/owner code, never repeat either expensive phase.
        return run_qualified(compiler, args.proof_runner.resolve(strict=True),
            args.qualification_dir.resolve(strict=True), args.helper.resolve(strict=True), output,
            ROOT / "tools/compact-rust-backend/forge_native_qualification.json", env, wrapper_sha)
    metadata = json.loads(execute(["nix", "flake", "metadata", PUBLIC, "--json"], env, capture=True))
    forge_source = Path(metadata["path"])
    rust_expr = ('let f = builtins.getFlake "' + PUBLIC + '"; '
        'pkgs = import f.inputs.nixpkgs { system = "x86_64-linux"; overlays = [(import f.inputs.rust-overlay)]; }; '
        'in pkgs.rust-bin.stable."1.98.1".minimal.override { extensions = ["rustfmt" "clippy"]; }')
    rust = Path(execute(["nix", "build", "--impure", "--expr", rust_expr, "--no-link", "--print-out-paths", "--max-jobs", 1, "--cores", 2], env, capture=True))
    forge = Path(execute(["nix", "build", PUBLIC, "--no-link", "--print-out-paths", "--max-jobs", 1, "--cores", 2], env, capture=True))
    derivation = output / "forge-derivation.json"
    derivation.write_text(execute(["nix", "derivation", "show", forge], env, capture=True))
    zkir = Path(execute(["nix", "build", "--impure", "--expr", '(builtins.getFlake (toString ./.)).inputs.zkir.packages.x86_64-linux.zkir', "--no-link", "--print-out-paths"], env, capture=True)) / "bin/zkir"
    execute(["cargo", "build", "--profile", "proof-checks", "--locked", "-p", "compact-rust-proof-smoke"], env)
    canonical = Path(env.get("CARGO_TARGET_DIR", ROOT / "target")) / "proof-checks/compact-rust-proof-smoke"
    proof = output / "proof-smoke"
    shutil.copyfile(canonical, proof)
    proof.chmod(0o500)
    native.require(native.adapter.digest(canonical) == native.adapter.digest(proof), "stable proof runner copy mismatch")
    cargo_metadata = output / "cargo-metadata.json"
    cargo_metadata.write_text(execute(["cargo", "metadata", "--locked", "--format-version", 1], env, capture=True))
    static_env = output / "ledger.env"
    execute([sys.executable, "-B", ROOT / "tools/compact-rust-backend/resolve_ledger_test_static.py", "--metadata", cargo_metadata,
             "--lock", ROOT / "Cargo.lock", "--provenance", output / "ledger-provenance.json", "--github-env", static_env], env)
    for row in static_env.read_text().splitlines():
        key, value = row.split("=", 1)
        native.require(key == "MIDNIGHT_LEDGER_TEST_STATIC_DIR", "unexpected ledger environment entry")
        env[key] = value
    config_path = ROOT / "tools/compact-rust-backend/forge_native_qualification.json"
    config = native.configuration(native.read(config_path))
    requested = [item for k in sorted(config["parameters"], key=int) for item in ("--parameter-k", k)]
    for label, flags in (("prepare", []), ("verify", ["--verify-only"])):
        receipt = execute([proof, "--prepare-proof-material", *flags, *requested], env, capture=True)
        (output / (label + "-material.json")).write_text(receipt)
    qdir = output / "finite-current"
    execute([sys.executable, "-B", ROOT / "tools/compact-rust-backend/qualify_linux_forge.py",
        "--compiler", compiler, "--proof-runner", proof, "--forge", forge / "bin/zkir-forge", "--zkir", zkir,
        "--forge-source", forge_source, "--rustc", rust / "bin/rustc", "--forge-derivation", derivation,
        "--source-manifest", ROOT / "tools/compact-rust-backend/forge_linux_source.json",
        "--params-dir", output / "parameters", "--output", qdir], env)
    admitted = output / "admitted"
    admitted.mkdir()
    helper = forge / "bin/midnight-zkir-forge-ci"
    return run_qualified(compiler, proof, qdir, helper, admitted, config_path, env, wrapper_sha)


def run_qualified(compiler, proof, qdir, helper, admitted, config_path, env, wrapper_sha):
    """Admit fresh finite receipts and execute one unchanged native owner only."""
    config = native.configuration(native.read(config_path))
    prepared = native.prepare_native(SimpleNamespace(qualification_dir=qdir, config=config_path, helper=helper, output=admitted))
    native.require(prepared["compiler"] == compiler and prepared["proof"] == proof,
                   "finite qualification is not bound to current job tools")
    # Only one full owner; finite qualifier above remains its original five small laps.
    for command in owner_commands(prepared, admitted, helper):
        prepared["live"]()
        execute(command, {**prepared["env"], **{k: env[k] for k in ("MIDNIGHT_LEDGER_TEST_STATIC_DIR",)}})
        prepared["live"]()
    plan = admitted / "native-plan"
    native.require(native.read(plan / "run.json")["passed"] is True, "native owner incomplete")
    command = native.read(plan / "native-script.command.json")
    native.require(command["exit_code"] == 0, "original native owner failed")
    native.acquisitions(native.printed_acquisitions(command), config, "forge-miss")
    native.rust_cohorts(command["stdout"] + "\n" + command["stderr"])
    native.finite.checked_lap(native.read(plan / "uncached-report.json"), None)
    delta = native.read(plan / "cache-report.json")["delta"]
    count = sum(config["expected_cached_k"].values())
    native.require(delta["requests"] == count and delta["misses"] == count and delta["hits"] == 0
                   and delta["errors"] == 0, "single-owner acquisition accounting changed")
    snapshot = native.read(plan / "snapshot.command.json")
    native.require(snapshot["exit_code"] == 0 and native.adapter.read_json(snapshot["stdout"])["bytes"] <= config["max_bytes"],
                   "private snapshot exceeds capacity")
    prepared["live"]()
    native.require(native.adapter.digest(Path(__file__)) == wrapper_sha, "owner wrapper changed during execution")
    print("Single original native owner completed; private stopped snapshot only, no reusable publication")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
