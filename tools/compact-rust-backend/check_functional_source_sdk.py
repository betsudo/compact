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

"""Qualify the documented witnessed SDK from an isolated source dependency closure.

The complete source and raw bundled SDK are retained. Source-testkit packaging
changes exactly one copied dependency path. No registry availability, constructor
proof, deployment proof, wallet lifecycle or full vendoring claim is made.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import stat
import subprocess
import time
import tomllib

import did_proof_gate
import local_parity_gate as common

ROOT = Path(__file__).resolve().parents[2]
EXAMPLE = ROOT / "doc/rust/guides/examples/functional-witness"
ORIGINAL = ROOT / "tools/compact-rust-backend/tests/functional_effects/field_witness_proof.compact"
WITNESSED_K = 7
ZKIR_VERSION = "midnight-zkir 3.2.0"
TIMEOUT = 1200
DRAIN = 2
CONTROLS = {"witness_result": ("E0053", "Result<(u64, bool)"),
            "consumed_program": ("E0382", "let _ = program.execute_native(context()?"),
            "consumed_context": ("E0382", ".execute_native(owned,")}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def inventory(root):
    """Exact file membership and bytes; no symlink escape in the source closure."""
    result = {}
    for path in root.rglob("*"):
        if path.is_symlink():
            raise ValueError(f"symlink in isolated source closure: {path}")
        if path.is_file():
            result[str(path.relative_to(root))] = digest(path)
    return result


def equal_inventory(root, expected):
    if inventory(root) != expected:
        raise ValueError(f"isolated file membership or bytes changed: {root}")


def tracked_source_files(folder):
    """Refuse ignored build output or unowned source before copying its bytes."""
    relative = folder.relative_to(ROOT)
    names = subprocess.check_output(["git", "ls-files", "-z", "--", str(relative)], cwd=ROOT)
    tracked = {ROOT / name for name in names.decode().split("\0") if name}
    physical = {path for path in folder.rglob("*") if path.is_symlink() or not path.is_dir()}
    if not tracked or physical != tracked:
        raise ValueError(f"selected source folder has missing or untracked/ignored entries: {relative}")
    if any(path.is_symlink() or not path.is_file() for path in physical):
        raise ValueError(f"selected source folder requires regular tracked files: {relative}")
    return sorted(tracked)


def runtime_files(root):
    result = {}
    for name in ("runtime-rs", "runtime-rs-macros"):
        package = root / name
        paths = [package / n for n in ("Cargo.toml", "README.md", "LICENSE")]
        if name == "runtime-rs":
            paths.append(package / "compatibility.json")
        paths += [p for p in (package / "src").rglob("*") if p.is_file()]
        result.update({str(p.relative_to(root)): digest(p) for p in paths})
    return result


def source_testkit(source, destination):
    destination.mkdir()
    shutil.copytree(source / "src", destination / "src")
    for name in ("Cargo.toml", "README.md"):
        shutil.copyfile(source / name, destination / name)
    license_source = source / "LICENSE" if (source / "LICENSE").is_file() else ROOT / "LICENSE"
    shutil.copyfile(license_source, destination / "LICENSE")
    original = (destination / "Cargo.toml").read_bytes()
    old, new = b'path = "../runtime-rs"', b'path = "../generated/contract/runtime-rs"'
    if original.count(old) != 1:
        raise ValueError("copied testkit runtime dependency recipe no longer applies exactly once")
    (destination.parent / "original-testkit-Cargo.toml").write_bytes(original)
    (destination / "Cargo.toml").write_bytes(original.replace(old, new))
    return {"license_source": str(license_source), "license_sha256": digest(license_source),
            "original_sha256": hashlib.sha256(original).hexdigest(),
            "derived_sha256": digest(destination / "Cargo.toml"),
            "transformation": {"from": old.decode(), "to": new.decode(), "count": 1},
            "scope": "copied library source manifest; development fixtures are outside this dependency graph"}


def dependency_graph(metadata, lock, output, cohort):
    expected = {output / p for p in ("consumer/Cargo.toml", "testkit-rs/Cargo.toml",
                "generated/contract/Cargo.toml", "generated/contract/runtime-rs/Cargo.toml",
                "generated/contract/runtime-rs-macros/Cargo.toml")}
    local = set()
    runtimes = []
    for package in metadata["packages"]:
        if package["source"] is None:
            path = Path(package["manifest_path"]).resolve()
            local.add(path)
            if not path.is_relative_to(output):
                raise ValueError("local dependency escapes the isolated closure")
            if package["name"] == "midnight-compact-runtime":
                runtimes.append(path)
        elif (package["name"], package["version"], package["source"]) not in cohort:
            raise ValueError("dependency differs from the selected qualified cohort")
    if local != expected or runtimes != [output / "generated/contract/runtime-rs/Cargo.toml"]:
        raise ValueError("expected exactly five local packages and one bundled runtime")
    for package in tomllib.loads(lock.read_text())["package"]:
        if package.get("source"):
            key = package["name"], package["version"], package["source"]
            if key not in cohort or package.get("checksum") != cohort[key]:
                raise ValueError("registry/Git lock identity or checksum differs")
    return {"local_manifests": sorted(map(str, local)), "lock_sha256": digest(lock),
            "one_runtime": str(runtimes[0])}


def typed_refusal(process, code, path, line):
    errors = []
    for text in process.stdout.splitlines():
        row = json.loads(text)
        message = row.get("message", {})
        if row.get("reason") == "compiler-message" and message.get("level") == "error":
            errors.append(message)
    if process.returncode == 0 or len(errors) != 1 or (errors[0].get("code") or {}).get("code") != code:
        raise ValueError("ordinary misuse did not independently produce its intended single Rust error")
    if not any(s.get("is_primary") and Path(s["file_name"]).name == path.name
               and s["line_start"] <= line <= s["line_end"] for s in errors[0]["spans"]):
        raise ValueError("ordinary misuse error is outside its intended primary source span")
    return {"code": code, "path": str(path), "line": line}


def proof_artifact(stdout, consumer):
    candidates = []
    for line in stdout.splitlines():
        row = json.loads(line)
        target = row.get("target", {})
        if row.get("reason") == "compiler-artifact" and row.get("executable") and target.get("name") == "functional-sdk-proof":
            if Path(target["src_path"]).resolve() != (consumer / "proof/prove.rs").resolve():
                raise ValueError("proof executable was compiled from a different consumer source")
            profile = row.get("profile", {})
            if profile.get("debug_assertions") is not True or profile.get("overflow_checks") is not True:
                raise ValueError("proof executable lacks checked profile settings")
            candidates.append({"path": row["executable"], "package_id": row["package_id"],
                               "source": target["src_path"], "profile": profile,
                               "compiled_cache_reused": row.get("fresh")})
    if len(candidates) != 1:
        raise ValueError("expected one actual Cargo proof executable artifact")
    return candidates[0]


def validate_result(result):
    required = ("changed_binding_rejected", "changed_input_rejected", "changed_output_rejected",
                "changed_private_witness_rejected", "changed_proof_rejected",
                "changed_input_new_statement_valid", "same_geometry_controls", "replay_unchanged")
    if result.get("status") != "passed" or any(result.get(key) is not True for key in required):
        raise ValueError("proof/mutation/replay result omitted a required assertion")
    if result.get("strict_application") != "default" or result.get("replay_refusal") != "IntentAlreadyExists":
        raise ValueError("funded transaction did not use default strictness and exact replay refusal")
    if result.get("witness_private_state") != [7, 8] or result.get("skip_sequence") != [None] * 6:
        raise ValueError("selected witness transition or unconditional proof geometry differs")
    if result.get("preparation_refusals") != ["ReplayStateMismatch", "NonCanonicalPrivateOutput(0,0)"]:
        raise ValueError("malformed recordings must refuse at their exact preparation boundaries")
    if result.get("standalone_proof_bytes", 0) <= 0 or result.get("transaction_bytes", 0) <= 0:
        raise ValueError("proof and applied transaction artifacts must both be nonempty")
    if result.get("changed_input_new_statement_check") != "valid; witness return is unconstrained relative to input":
        raise ValueError("selected source witness relation control is missing")
    final = result.get("final_state_hex", "")
    if not isinstance(final, str) or not re.fullmatch(r"(?:[0-9a-f]{2})+", final):
        raise ValueError("retained final contract state must be nonempty canonical hex")
    data = result.get("final_data_hex", "")
    if not isinstance(data, str) or not re.fullmatch(r"(?:[0-9a-f]{2})+", data) or data != result.get("expected_data_hex"):
        raise ValueError("applied contract data differs from the independently executed expected data")
    if result.get("final_cell_le_hex") != (42).to_bytes(32, "little").hex():
        raise ValueError("applied Cell does not encode the selected Compact Field value 42")


def validate_ledger9_result(result, application):
    validate_result(result)
    version = result.get("transaction_proof_version")
    spends = result.get("funded_dust_spends")
    if type(version) is not int or version != 3 or type(spends) is not int or spends != 1:
        raise ValueError("expected a transaction-bound v3 proof and one proved Dust spend")
    for filename, field in (
        ("standalone.proof", "standalone_proof_bytes"),
        ("transaction-bound.proof", "transaction_proof_bytes"),
        ("applied-transaction.bin", "transaction_bytes"),
    ):
        size = result.get(field)
        if type(size) is not int or size <= 0 or (application / filename).stat().st_size != size:
            raise ValueError("retained proof/transaction length differs from actual result")
    if digest(application / "standalone.proof") == digest(application / "transaction-bound.proof"):
        raise ValueError("standalone and transaction-bound proofs must be distinct")


def clippy_library_command(cargo, manifest, package):
    # These packages are dependencies outside the consumer workspace. Cargo
    # clippy's workspace wrapper does not lint them, even with --package.
    # The caller selects the pinned clippy-driver as RUSTC for this command only.
    return [str(cargo), "rustc", "--locked", "--offline", "--manifest-path", str(manifest),
            "--package", package, "--lib", "--message-format", "json", "--", "-D", "warnings"]


def library_lint_root(consumer_manifest, destination):
    # Cargo cannot select features for a nonmember with --package. A separate
    # lint-only root enables the existing consumer's proof feature as a normal
    # dependency, without editing any qualified manifest or source.
    destination.mkdir()
    name = tomllib.loads(consumer_manifest.read_text())["package"]["name"]
    relative = os.path.relpath(consumer_manifest.parent, destination)
    manifest = destination / "Cargo.toml"
    manifest.write_text('[workspace]\n[package]\nname="isolated-sdk-library-lints"\n'
                        'version="0.0.0"\nedition="2024"\n[lib]\npath="lib.rs"\n'
                        '[dependencies]\nconsumer={package='+json.dumps(name)+',path='+json.dumps(relative)
                        +',features=["proof"]}\n')
    (destination / "lib.rs").write_text("// Private lint root; never a generated contract or execution input.\n")
    shutil.copyfile(consumer_manifest.with_name("Cargo.lock"), destination / "Cargo.lock")
    return manifest


def check_library_lint_graph(reference, selected, lint_manifest):
    roots = [p for p in selected["packages"] if Path(p["manifest_path"]) == lint_manifest]
    if len(roots) != 1 or roots[0]["name"] != "isolated-sdk-library-lints":
        raise ValueError("one owned library lint root required")
    root = roots[0]["id"]
    if selected["workspace_members"] != [root] or selected["resolve"]["root"] != root:
        raise ValueError("library lint workspace ownership differs")
    packages = {p["id"]: p for p in selected["packages"] if p["id"] != root}
    nodes = {n["id"]: n for n in selected["resolve"]["nodes"] if n["id"] != root}
    if (packages != {p["id"]: p for p in reference["packages"]}
            or nodes != {n["id"]: n for n in reference["resolve"]["nodes"]}):
        raise ValueError("library lint dependency identities or features differ from qualified consumer")
    own = next(n for n in selected["resolve"]["nodes"] if n["id"] == root)
    if own["dependencies"] != [reference["resolve"]["root"]]:
        raise ValueError("library lint root must depend only on the qualified consumer")
    return {"root": root, "unchanged_packages": len(packages), "unchanged_feature_nodes": len(nodes)}


def clippy_library_artifact(stdout, metadata, package, *, required_feature=None):
    owners = [p for p in metadata["packages"] if p["name"] == package]
    if len(owners) != 1:
        raise ValueError("one selected Clippy package required")
    owner = owners[0]
    targets = [t for t in owner["targets"] if t["kind"] == ["lib"]]
    if len(targets) != 1:
        raise ValueError("one selected Clippy library required")
    source = Path(targets[0]["src_path"]).resolve(strict=True)
    artifacts = []
    for line in stdout.splitlines():
        row = json.loads(line)
        if (row.get("reason") == "compiler-artifact" and row.get("package_id") == owner["id"]
                and row.get("target", {}).get("kind") == ["lib"]
                and Path(row["target"]["src_path"]).resolve(strict=True) == source):
            artifacts.append(row)
    if len(artifacts) != 1 or artifacts[0].get("fresh") is not False:
        raise ValueError("selected Clippy library must actually compile, not reuse a check artifact")
    features = artifacts[0].get("features", [])
    if required_feature and required_feature not in features:
        raise ValueError("selected Clippy library omitted its required proof feature")
    return {"package_id": owner["id"], "source": str(source), "fresh": False, "features": features}


def installed_runtime_source_root(compiler):
    """Admit the installed prefixes used by the CLI, without its checkout fallback."""
    for prefix in (compiler.parent, compiler.parent.parent):
        candidate = prefix / "share/compactc"
        try:
            mode = candidate.stat().st_mode
        except (FileNotFoundError, NotADirectoryError):
            continue
        if not stat.S_ISDIR(mode):
            raise ValueError("installed runtime source root is not a directory")
        return candidate.resolve(strict=True)
    raise ValueError("selected compiler lacks an installed runtime source bundle")


def cargo_configuration(environment):
    # This is selected configuration admission, not a hostile-host attestation.
    forbidden = {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
                 "RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP", "CARGO_BUILD_RUSTC",
                 "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_RUSTC_WRAPPER",
                 "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_WARNINGS", "CLIPPY_ARGS"}
    def refused(key):
        return (key in forbidden or key.startswith("CARGO_TARGET_") and key.endswith(("_RUSTFLAGS", "_RUNNER"))
                or key.startswith("CARGO_PROFILE_") and not key.endswith("_DEBUG"))
    if any(value and refused(key) for key, value in environment.items()):
        raise ValueError("unsupported Cargo compiler, profile or lint override")
    paths = [p / ".cargo" / n for p in (ROOT, *ROOT.parents) for n in ("config", "config.toml")]
    home = Path(environment.get("CARGO_HOME", str(Path.home() / ".cargo")))
    paths += [home / n for n in ("config", "config.toml")]
    result = {}
    for path in paths:
        result[str(path)] = digest(path) if path.is_file() else None
        if not path.is_file():
            continue
        config = tomllib.loads(path.read_text())
        if any(key in config.get("build", {}) for key in ("rustflags", "rustc", "rustc-wrapper", "rustc-workspace-wrapper", "warnings")):
            raise ValueError("unsupported Cargo compiler/lint configuration")
        if config.get("profile") or any(refused(key) for key in config.get("env", {})) or any(
                key in value for value in config.get("target", {}).values() for key in ("rustflags", "runner")):
            raise ValueError("unsupported Cargo target/profile/environment configuration")
    return result


def bounded_command(command, environment):
    process = subprocess.Popen([str(a) for a in command], cwd=ROOT, env=environment,
                               text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=os.name == "posix")
    def stop():
        try:
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGKILL)
            elif process.poll() is None:
                process.kill()
        except ProcessLookupError:
            pass
    try:
        try:
            stdout, stderr = process.communicate(timeout=TIMEOUT)
        except subprocess.TimeoutExpired as error:
            stop()
            try:
                stdout, stderr = process.communicate(timeout=DRAIN)
            except subprocess.TimeoutExpired as drain:
                stdout, stderr = drain.stdout, drain.stderr
                process.stdout.close()
                process.stderr.close()
                process.wait(timeout=DRAIN)
            raise subprocess.TimeoutExpired(command, TIMEOUT, output=stdout, stderr=stderr) from error
    except BaseException:
        stop()
        process.stdout.close()
        process.stderr.close()
        process.wait(timeout=DRAIN)
        raise
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--material-runner", type=Path, required=True,
                        help="selected source-built compact-rust-proof-smoke executable")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    output = args.output.resolve()
    if output.exists() or args.output.is_symlink() or output.is_relative_to(ROOT):
        raise ValueError("qualification output must be absent and outside the source checkout")
    output.mkdir(parents=True, mode=0o700)
    receipt = {"format": "compact-functional-source-sdk/v2", "status": "incomplete", "commands": [],
               "scope": "one complete witnessed Cell source; raw default bundled functional SDK and explicit optional testkit source package",
               "limits": "selected source/host/configuration and official fixture qualification; not exhaustive semantics, hostile-host attestation, deployment/constructor proof, wallet lifecycle or fully vendored distribution"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", PYTHONDONTWRITEBYTECODE="1", CARGO_INCREMENTAL="0")
    env.setdefault("CARGO_BUILD_JOBS", "4")
    def run(label, command, *, success=True, lint_library=False):
        started = time.monotonic()
        command_env = env
        if lint_library:
            # Inherited compiler/wrapper overrides were refused above. This is
            # an explicit, hash-bound linter selection, never used for execution
            # or proving builds and never written into env or Cargo config.
            command_env = dict(env, RUSTC=tools["clippy-driver"]["path"])
        try:
            result = bounded_command(command, command_env)
        except subprocess.TimeoutExpired as error:
            for stream, value in (("stdout", error.stdout), ("stderr", error.stderr)):
                (output / f"{label}.{stream}").write_text(value.decode(errors="replace") if isinstance(value, bytes) else value or "")
            receipt["commands"].append({"label": label, "command": list(map(str, command)), "timed_out": True, "seconds": time.monotonic()-started})
            raise RuntimeError(f"isolated SDK command timed out: {label}") from error
        for stream in ("stdout", "stderr"):
            (output / f"{label}.{stream}").write_text(getattr(result, stream))
        receipt["commands"].append({"label": label, "command": list(map(str, command)), "exit": result.returncode, "seconds": time.monotonic()-started})
        if lint_library:
            receipt["commands"][-1]["selected_clippy_driver"] = dict(tools["clippy-driver"])
        if success and result.returncode:
            raise RuntimeError(f"isolated SDK command failed: {label}: {result.stderr[-2000:]}")
        return result
    try:
        receipt["source_head"] = common.git_head()
        receipt["source_dirty"] = common.dirty_paths()
        if receipt["source_dirty"] or env.get("COMPACTC_SCHEME"):
            raise ValueError("clean selected source and original packaged compiler without Scheme override required")
        if "MIDNIGHT_PROOF_SERVER" in env:
            raise ValueError("witnessed SDK requires local proving without MIDNIGHT_PROOF_SERVER")
        # Even an empty variable is an override to the CLI: it selects the cwd.
        # This gate qualifies installed default selection, not a caller's root.
        if "COMPACT_RUST_RUNTIME_DIR" in env:
            raise ValueError("default installed SDK qualification forbids COMPACT_RUST_RUNTIME_DIR")
        # Fail before generation or Cargo work when selected material/static
        # locations or the native ZKIR executable are missing.
        prerequisites = receipt["prerequisites"] = did_proof_gate.prerequisites(env, zkir_name="zkir-v3")
        receipt["dust_fixture_hashes"] = did_proof_gate.dust_fixture_inventory(
            Path(prerequisites["MIDNIGHT_LEDGER_TEST_STATIC_DIR"]))
        receipt["cargo_configuration_sha256"] = cargo_configuration(env)
        target = Path(env.get("CARGO_TARGET_DIR") or output / "cargo-target").resolve()
        if target.is_relative_to(ROOT) and not target.is_relative_to(ROOT / "target") or target.is_relative_to(output) and target != output / "cargo-target":
            raise ValueError("Cargo target must not overwrite source or qualification evidence")
        env["CARGO_TARGET_DIR"] = str(target)
        receipt["cargo_target_dir"] = str(target)
        receipt["build_inputs"] = common.build_input_receipt(env)
        tools = receipt["tools"] = {}
        for name, supplied in (("compiler", args.compiler), ("material-runner", args.material_runner), ("zkir", Path(prerequisites["zkir"]))):
            path = supplied.resolve(strict=True)
            if not path.is_file():
                raise ValueError(f"missing selected executable: {name}")
            tools[name] = {"path": str(path), "sha256": digest(path)}
        compiler = Path(tools["compiler"]["path"])
        installed_runtime = installed_runtime_source_root(compiler)
        scheme = compiler.with_name("compactc-scheme").resolve(strict=True)
        tools["scheme"] = {"path": str(scheme), "sha256": digest(scheme)}
        for name in ("cargo", "rustc", "rustfmt", "cargo-clippy", "clippy-driver"):
            path = Path(run("rustup-path-"+name, ["rustup", "which", "--toolchain", "1.99.0", name]).stdout.strip()).resolve(strict=True)
            tools[name] = {"path": str(path), "sha256": digest(path)}
        env["PATH"] = str(Path(tools["cargo"]["path"]).parent) + os.pathsep + env["PATH"]
        for name, command in (("compiler", [compiler, "--version"]), ("zkir", [tools["zkir"]["path"], "--version"]),
                              ("cargo", ["cargo", "--version"]), ("rustc", ["rustc", "-Vv"]), ("clippy", ["cargo", "clippy", "--version"])):
            receipt.setdefault("versions", {})[name] = run(name+"-version", command).stdout.strip()
        if receipt["versions"]["zkir"] != ZKIR_VERSION:
            raise ValueError("expected the released Ledger 9 ZKIR v3 compiler")
        version = re.search(r"make-version 'compiler (\d+) (\d+) (\d+)", (ROOT / "compiler/compiler-version.ss").read_text())
        if version is None or receipt["versions"]["compiler"] != ".".join(version.groups()):
            raise ValueError("compiler version differs from selected source")
        paths = [Path(__file__), Path(common.__file__), Path(did_proof_gate.__file__), ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "compiler/compiler-version.ss", ROOT / "LICENSE", ORIGINAL]
        source_folders = (EXAMPLE, ROOT / "testkit-rs", ROOT / "tools/compact-rust-proof-smoke/src")
        for folder in source_folders:
            paths += tracked_source_files(folder)
        expected_runtime = runtime_files(ROOT)
        installed_hashes = runtime_files(installed_runtime)
        if installed_hashes != expected_runtime:
            raise ValueError("installed runtime source bundle differs from selected source")
        receipt["runtime_source_selection"] = {
            "mode": "installed-default", "override_present": False,
            "root": str(installed_runtime),
            "basis": "admitted CLI installed prefix and absent override; not a consumed-file syscall trace",
        }
        receipt["installed_runtime_hashes"] = {
            str(installed_runtime / path): value for path, value in installed_hashes.items()}
        paths += [ROOT / p for p in expected_runtime]
        receipt["source_hashes"] = {str(p): digest(p) for p in paths}
        receipt["runtime_bundle_hashes"] = expected_runtime
        # Verify the selected k7 fixture and built-in material content
        # before generating source or compiling the external consumer.
        material = json.loads(run("verify-material", [tools["material-runner"]["path"], "--prepare-proof-material", "--verify-only", "--parameter-k", str(WITNESSED_K)]).stdout)
        if material.get("mode") != "verify-only" or material.get("verification") != "all files checked against locked upstream SHA256 tables" or Path(material["cache_directory"]).resolve() != Path(env["MIDNIGHT_PP"]).resolve() or WITNESSED_K not in [p["k"] for p in material["parameters"]]:
            raise ValueError("selected SRS and built-ins require fresh upstream hash verification")
        receipt["verified_material"] = material
        material_names = [p["name"] for p in material["assets"]+material["parameters"]]
        receipt["material_hashes"] = {str(Path(env["MIDNIGHT_PP"]) / n): digest(Path(env["MIDNIGHT_PP"]) / n) for n in material_names}
        shutil.copytree(EXAMPLE / "consumer", output / "consumer")
        shutil.copytree(EXAMPLE / "source", output / "source")
        source = output / "source/field_witness_proof.compact"
        if source.read_bytes() != ORIGINAL.read_bytes():
            raise ValueError("documented source differs from complete maintained original")
        consumer = output / "consumer"
        manifest = consumer / "Cargo.toml"
        run("handwritten-format", [tools["rustfmt"]["path"], "--edition", "2024", "--check",
                                   *sorted(consumer.rglob("*.rs"))])
        profile = tomllib.loads(manifest.read_text())["profile"]["proof-checks"]
        if profile.get("debug-assertions") is not True or profile.get("overflow-checks") is not True:
            raise ValueError("owned proof checks require assertions and checked overflow")
        generated = output / "generated"
        run("compile-complete-source", [compiler, "--target", "rust", "--target", "ts", "--skip-zk", "--rust-functional-ledger9", "--rust-require-recording", source, generated])
        contract = generated / "contract"
        for name in ("runtime-rs", "runtime-rs-macros"):
            equal_inventory(contract / name, {p[len(name)+1:]: v for p,v in expected_runtime.items() if p.startswith(name+"/")})
        receipt["raw_generated_hashes"] = inventory(generated)
        receipt["testkit_recipe"] = source_testkit(ROOT / "testkit-rs", output / "testkit-rs")
        receipt["staged_testkit_hashes"] = inventory(output / "testkit-rs")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        cohort = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
        ledger = tomllib.loads((contract / "runtime-rs/Cargo.toml").read_text())["dependencies"]["midnight-ledger-v9"]
        if ledger != {"version": "=1.0.1", "optional": True}:
            raise ValueError("SDK ledger cohort differs")
        metadata = json.loads(run("metadata", ["cargo", "metadata", "--offline", "--format-version", "1", "--features", "proof", "--manifest-path", manifest]).stdout)
        receipt["dependency_graph"] = dependency_graph(metadata, consumer / "Cargo.lock", output, cohort)
        receipt["staged_consumer_hashes"] = inventory(consumer)
        cargo = ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest, "--test", "witnessed"]
        native = run("native-tests", cargo)
        if not re.search(r"test result: ok\. 3 passed; 0 failed; 0 ignored", native.stdout):
            raise ValueError("three actual documented ContractLab methods must execute")
        receipt["native_methods"] = 3
        receipt["typed_controls"] = {}
        for name, (code, needle) in CONTROLS.items():
            for polarity in ("positive", "negative"):
                unit = name+"_"+polarity
                process = run(unit, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest, "--bin", unit, "--features", unit, "--message-format", "json"], success=polarity=="positive")
                if polarity == "negative":
                    path = consumer / "controls" / (unit+".rs")
                    lines = [i for i,s in enumerate(path.read_text().splitlines(),1) if needle in s]
                    if not lines:
                        raise ValueError("missing intended ordinary misuse location")
                    receipt["typed_controls"][name] = typed_refusal(process, code, path, lines[-1])
        # Keep the admitted graph and raw manifests: testkit's repository-only
        # development fixtures must not enter this isolated library package.
        lint_manifest = library_lint_root(manifest, output / "library-lints")
        lint_metadata = json.loads(run("library-lint-metadata", [tools["cargo"]["path"], "metadata",
            "--offline", "--format-version", "1", "--manifest-path", lint_manifest]).stdout)
        receipt["clippy_graph"] = check_library_lint_graph(metadata, lint_metadata, lint_manifest)
        receipt["clippy_root_hashes"] = inventory(lint_manifest.parent)
        receipt["clippy_libraries"] = {}
        for name, package, feature in (("consumer", "isolated-functional-sdk-consumer", "proof"),
                                       ("raw-sdk", "compact-contract-field-witness-proof", "ledger-transaction"),
                                       ("testkit", "midnight-compact-testkit", "proof")):
            lint = run(name+"-clippy", clippy_library_command(tools["cargo"]["path"], lint_manifest, package),
                       lint_library=True)
            receipt["clippy_libraries"][name] = clippy_library_artifact(lint.stdout, metadata, package,
                                                                      required_feature=feature)
        (generated / "keys").mkdir()
        keygen = run("fresh-direct-keygen", [tools["zkir"]["path"], "compile", generated / "zkir/apply_offset.zkir", generated / "keys/apply_offset.prover", generated / "keys/apply_offset.verifier"])
        match = re.search(r"\(k=(\d+), rows=(\d+)\)", keygen.stdout+keygen.stderr)
        if not match or (int(match[1]), int(match[2])) != (WITNESSED_K, 68):
            raise ValueError("selected witnessed SDK fixture requires k=7 and 68 rows")
        receipt["keygen"] = {"mode": "fresh direct native", "k": int(match[1]), "rows": int(match[2])}
        receipt["generated_hashes"] = inventory(generated)
        for name, expected in receipt["raw_generated_hashes"].items():
            if receipt["generated_hashes"].get(name) != expected:
                raise ValueError("original raw generated SDK changed during key generation")
        additions = {"keys/apply_offset.prover", "keys/apply_offset.verifier", "zkir/apply_offset.bzkir"}
        if set(receipt["generated_hashes"]) != set(receipt["raw_generated_hashes"]) | additions:
            raise ValueError("unexpected generated SDK/material file membership")
        build = run("build-proof", ["cargo", "build", "--locked", "--offline", "--profile", "proof-checks", "--features", "proof", "--bin", "functional-sdk-proof", "--manifest-path", manifest, "--message-format", "json"])
        run("proof-clippy", ["cargo", "clippy", "--locked", "--offline", "--profile", "proof-checks", "--features", "proof", "--all-targets", "--manifest-path", manifest, "--", "-D", "warnings"])
        receipt["compiled_proof"] = proof_artifact(build.stdout, consumer)
        binary = Path(receipt["compiled_proof"]["path"]).resolve(strict=True)
        if not binary.is_relative_to(target):
            raise ValueError("actual proof executable escaped the selected Cargo target")
        (output / "bin").mkdir()
        receipt["proof_binary"] = common.stable_copy(binary, output / "bin/functional-sdk-proof")
        run("fresh-proof-and-strict-application", [receipt["proof_binary"]["snapshot"], generated, output / "application"])
        application = output / "application"
        receipt["application_hashes"] = inventory(application)
        run("independent-standalone-verification", [receipt["proof_binary"]["snapshot"], "--verify", generated, application])
        equal_inventory(application, receipt["application_hashes"])
        result = json.loads((application / "result.json").read_text())
        validate_ledger9_result(result, application)
        receipt["proof_result"] = result
        equal_inventory(generated, receipt["generated_hashes"])
        for relative, expected in inventory(EXAMPLE / "consumer").items():
            if digest(consumer / relative) != expected:
                raise ValueError("documented external consumer source changed")
        for relative, expected in inventory(ROOT / "testkit-rs/src").items():
            if digest(output / "testkit-rs/src" / relative) != expected:
                raise ValueError("copied testkit library source changed")
        equal_inventory(consumer, receipt["staged_consumer_hashes"])
        equal_inventory(output / "testkit-rs", receipt["staged_testkit_hashes"])
        equal_inventory(output / "library-lints", receipt["clippy_root_hashes"])
        for folder in source_folders:
            tracked_source_files(folder)
        receipt["consumer_hashes"] = inventory(consumer)
        receipt["testkit_hashes"] = inventory(output / "testkit-rs")
        if installed_runtime_source_root(compiler) != installed_runtime:
            raise ValueError("installed runtime source selection changed")
        for group in (receipt["source_hashes"], receipt["installed_runtime_hashes"],
                      receipt["dust_fixture_hashes"], receipt["material_hashes"]):
            for name, expected in group.items():
                if digest(name) != expected:
                    raise ValueError(f"selected source/material input changed: {name}")
        for tool in tools.values():
            if digest(tool["path"]) != tool["sha256"]:
                raise ValueError("selected executable changed")
        if cargo_configuration(env) != receipt["cargo_configuration_sha256"] or common.git_head() != receipt["source_head"] or common.dirty_paths():
            raise ValueError("source or selected configuration changed during qualification")
        common.finish_build_input_receipt(receipt, env)
        receipt["status"] = "passed"
    except Exception as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
        raise
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
