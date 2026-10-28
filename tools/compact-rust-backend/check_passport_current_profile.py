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

"""Execute the unchanged Passport pure API from an isolated generated SDK.

The historical oracle/provenance owner is retained, not repinned. Recording,
SNARK proofs and ledger application are not applicable to this pure library.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "examples/rust_backend/vc_passport_adoption"
FIXTURE = ROOT / "tests-rust-backend/vc-passport-adoption"
ORACLE = FIXTURE / "oracle"
GUIDE = ROOT / "doc/rust/guides/examples/passport/consumer/src/lib.rs"
CONTROLS = Path(__file__).with_name("tests") / "passport_current_profile"
SOURCE_MANIFEST_SHA256 = "f122c9c890e841d50f2f2980df4229de584333a9eb6c9a4525794f8ce8a77f78"
FAMILIES = ("ts", "age", "request", "private-parts", "protocol", "roots", "bindings",
            "signed-flow", "authorization", "protocol-roundtrip", "complete-flow", "codec")
NEGATIVES = ("wrong_bytes", "wrong_shape")
LEDGER_REVISION = "b5d3e965a18e4cac17fac4f38446fc55c524439a"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def original_source_files():
    manifest = SOURCE / "source-manifest.json"
    if digest(manifest) != SOURCE_MANIFEST_SHA256:
        raise ValueError("original Passport source manifest differs")
    pins = json.loads(manifest.read_text())["files"]
    if len(pins) != 17 or {p.relative_to(SOURCE).as_posix() for p in SOURCE.rglob("*.compact")} != {p["path"] for p in pins}:
        raise ValueError("original Passport source membership differs")
    for pin in pins:
        if digest(SOURCE / pin["path"]) != pin["sha256"]:
            raise ValueError(f"original Passport source differs: {pin['path']}")
    return pins


def require_clean_source(receipt):
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if receipt.get("source_head", head) != head:
        raise RuntimeError("source HEAD changed during Passport qualification")
    receipt["source_head"] = head
    receipt["source_dirty"] = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT))
    if receipt["source_dirty"]:
        raise RuntimeError("clean source checkout required")


def compare_capture(actual, expected, profile):
    """Keep actual profile identity; compare every other original behavior field."""
    if actual.get("profile") != profile or set(actual) != set(expected):
        raise ValueError("Passport capture profile or field membership differs")
    for key in expected:
        if key != "profile" and actual[key] != expected[key]:
            raise ValueError(f"Passport capture behavior differs: {key}")


def validate_dependencies(metadata, lock_path, allowed, qualified):
    """Existing consumer cohort policy, with every local manifest isolated."""
    for package in metadata["packages"]:
        identity = (package["name"], package["version"], package.get("source"))
        if package.get("source") is None:
            if Path(package["manifest_path"]).resolve() not in allowed:
                raise ValueError(f"unexpected local Passport dependency: {identity}")
        elif identity not in qualified:
            raise ValueError(f"Passport dependency differs from qualified cohort: {identity}")
    for package in tomllib.loads(lock_path.read_text())["package"]:
        if package.get("source"):
            identity = (package["name"], package["version"], package["source"])
            if identity not in qualified or package.get("checksum") != qualified[identity]:
                raise ValueError(f"Passport lock identity/checksum differs: {identity}")
    return digest(lock_path)


def typed_refusal(process):
    codes = []
    for line in process.stdout.splitlines():
        row = json.loads(line)
        message = row.get("message", {})
        if row.get("reason") == "compiler-message" and message.get("level") == "error":
            codes.append((message.get("code") or {}).get("code"))
    if process.returncode == 0 or codes != ["E0308"]:
        raise ValueError(f"Passport ordinary misuse did not independently refuse with E0308: {codes}")
    return codes


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    output = args.output.absolute()
    if output.exists() or output.is_symlink():
        raise ValueError("Passport qualification destination must be absent")
    output.mkdir(parents=True)
    receipt = {"format": "compact-passport-current-profile/v1", "status": "incomplete", "commands": [],
               "scope": "original pure Passport current-profile execution and isolated SDK consumer",
               "proof_applicability": "not applicable: original source has no stateful exports",
               "historical_provenance": "original provenance.rs retains its historical generated-library hash; source/oracle pins are checked separately here",
               "limits": "sampled original behavior, not exhaustive protocol/branch coverage, network or security attestation"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", PYTHONDONTWRITEBYTECODE="1")

    def run(label, command, *, success=True):
        start = time.monotonic()
        process = subprocess.run([str(a) for a in command], cwd=ROOT, env=env, text=True, capture_output=True)
        (output / f"{label}.stdout").write_text(process.stdout)
        (output / f"{label}.stderr").write_text(process.stderr)
        receipt["commands"].append({"label": label, "command": [str(a) for a in command],
                                    "exit": process.returncode, "seconds": time.monotonic() - start})
        if success and process.returncode:
            raise RuntimeError(f"Passport command failed: {label}: {process.stderr[-2000:]}")
        return process

    try:
        require_clean_source(receipt)
        compiler = args.compiler.resolve(strict=True)
        runtime = args.ts_runtime.resolve(strict=True)
        if not shutil.which("node"):
            raise RuntimeError("Node.js is required for live Passport comparisons")
        node = Path(shutil.which("node")).resolve(strict=True)
        if json.loads((runtime / "package.json").read_text())["version"] != json.loads((ROOT / "runtime/package.json").read_text())["version"]:
            raise ValueError("Passport TS runtime version differs from selected source")
        version = re.search(r"make-version 'compiler (\d+) (\d+) (\d+)", (ROOT / "compiler/compiler-version.ss").read_text())
        if version is None:
            raise ValueError("selected source compiler version is unavailable")
        selected_version = ".".join(version.groups())
        if run("compiler-version", [compiler, "--version"]).stdout.strip() != selected_version:
            raise ValueError("Passport compiler version differs from selected source")
        manifest = SOURCE / "source-manifest.json"
        pins = original_source_files()
        provenance = json.loads((ORACLE / "provenance.json").read_text())
        ledger = tomllib.loads((ROOT / "runtime-rs/Cargo.toml").read_text())["dependencies"]["midnight-ledger"]
        if ledger["version"] != "=8.1.3" or ledger["rev"] != LEDGER_REVISION:
            raise ValueError("Passport qualification requires the declared Ledger8.1.3 runtime cohort")
        if {p.stem for p in CONTROLS.glob("*.rs")} != set(NEGATIVES):
            raise ValueError("Passport typed refusal source membership differs")
        for name, expected in provenance["oracle_files"].items():
            if digest(ORACLE / name) != expected:
                raise ValueError(f"original Passport oracle differs: {name}")
        for name, record in provenance["codec_sources"].items():
            if digest(ORACLE / "upstream-codec-source" / name) != record["sha256"]:
                raise ValueError(f"original Passport codec source differs: {name}")
        paths = [Path(__file__), ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "compiler/compiler-version.ss", manifest, GUIDE,
                 ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml",
                 ROOT / "runtime/package.json", *SOURCE.rglob("*.compact"), *ORACLE.rglob("*"),
                 *(FIXTURE / "tests").glob("*.rs"), *CONTROLS.glob("*.rs")]
        paths += [p for name in ("runtime-rs", "runtime-rs-macros") for p in (ROOT / name / "src").rglob("*.rs")]
        receipt["source_hashes"] = {str(p): digest(p) for p in paths if p.is_file()}
        receipt["tools"] = {str(p): digest(p) for p in (compiler, node)}
        receipt["ts_runtime_hashes"] = {str(p): digest(p) for p in [runtime / "package.json", *(runtime / "dist").rglob("*")] if p.is_file()}
        source = output / "passport-source"
        source.mkdir()
        shutil.copyfile(manifest, source / "source-manifest.json")
        for pin in pins:
            original = SOURCE / pin["path"]
            if digest(original) != pin["sha256"]:
                raise ValueError(f"original Passport source differs: {pin['path']}")
            target = source / pin["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, target)
        generated = output / "generated"
        run("generate", [compiler, "--target", "rust", "--target", "ts", "--skip-zk",
                         source / "src/digital-passport-credential.compact", generated])
        contract = generated / "contract"
        ir = json.loads((contract / "compact-rust-ir.json").read_text())
        coverage = json.loads((ORACLE / "coverage.json").read_text())
        exports = {row["export"] for row in coverage["exports"]}
        if (len(ir["circuits"]) != 89 or ir["stateful_circuits"] or ir["ledger_fields"] or ir["witnesses"]
                or len(exports) != 75 or set(re.findall(r"pub fn ([A-Za-z_][A-Za-z_0-9]*)", (contract / "lib.rs").read_text())) != exports
                or json.loads((contract / "rust-capabilities.json").read_text())["circuits"]):
            raise ValueError("Passport pure export/IR/capability inventory differs")
        receipt["inventory"] = {"internal_pure_circuits": 89, "public_pure_exports": 75, "stateful_exports": 0}
        for name in ("runtime-rs", "runtime-rs-macros"):
            for original in (ROOT / name / "src").rglob("*.rs"):
                bundled = contract / name / original.relative_to(ROOT / name)
                if bundled.read_bytes() != original.read_bytes():
                    raise ValueError(f"bundled Passport runtime source differs: {bundled}")
        receipt["raw_generated_rust_sha256"] = digest(contract / "lib.rs")
        shutil.copyfile(contract / "lib.rs", output / "raw-generated-lib.rs")
        run("format-generated", ["rustfmt", "--edition", "2024", contract / "lib.rs"])
        receipt["formatted_generated_rust_sha256"] = digest(contract / "lib.rs")
        generated_hashes = {str(p): digest(p) for p in generated.rglob("*") if p.is_file()}
        link = contract / "node_modules/@midnight-ntwrk/compact-runtime"
        link.parent.mkdir(parents=True)
        link.symlink_to(runtime, target_is_directory=True)
        expression = ("const m=await import(" + json.dumps((contract / "index.js").as_uri())
                      + ");process.stdout.write(JSON.stringify(Object.keys(m.pureCircuits).sort()));")
        surface = json.loads(run("typescript-exports", [node, "--input-type=module", "-e", expression]).stdout)
        if surface != sorted(exports):
            raise ValueError("Passport live TypeScript public export inventory differs")
        profile = f"current-{selected_version}-ledger8.1.3"
        receipt["typescript"] = []
        for family in FAMILIES:
            capture = output / f"{family}.json"
            script = ORACLE / ("capture.mjs" if family == "ts" else f"capture-{family}.mjs")
            command = [node, script, contract / "index.js"]
            if family == "age":
                command.append(ORACLE / "age-inputs.json")
            run(f"typescript-{family}", [*command, profile, capture])
            actual = json.loads(capture.read_text())
            compare_capture(actual, json.loads((ORACLE / f"upstream-{family}-capture.json").read_text()), profile)
            receipt["typescript"].append({"family": family, "cases": len(actual.get("rows", [])), "sha256": digest(capture)})
        if not link.is_symlink() or link.resolve() != runtime:
            raise ValueError("Passport TS runtime link changed during execution")
        if sum(row["cases"] for row in receipt["typescript"]) != 202:
            raise ValueError("Passport live TypeScript case inventory differs")
        consumer = output / "consumer"
        shutil.copytree(FIXTURE / "oracle", consumer / "oracle")
        (consumer / "tests").mkdir()
        for original in (FIXTURE / "tests").glob("*.rs"):
            if original.name != "provenance.rs":
                shutil.copyfile(original, consumer / "tests" / original.name)
        shutil.copyfile(GUIDE, consumer / "tests/documented_guide.rs")
        (consumer / "lib.rs").write_text("pub use passport::{pure_circuits, runtime, types};\n")
        lines = ['[workspace]', '[package]', 'name="compact-rust-vc-passport-adoption-fixture"',
                 'version="0.1.0"', 'edition="2024"', '[lib]', 'path="lib.rs"', '[dependencies]',
                 'passport={package="compact-contract-digital-passport-credential",path="../generated/contract"}',
                 'midnight-compact-runtime={path="../generated/contract/runtime-rs"}',
                 '[dev-dependencies]', 'hex="0.4"', 'serde_json="1"', 'sha2="0.10"', '[features]']
        for name in NEGATIVES:
            lines.append(f"{name}=[]")
        for name in NEGATIVES:
            shutil.copyfile(CONTROLS / f"{name}.rs", consumer / f"{name}.rs")
            lines += ['[[bin]]', f'name="{name}"', f'path="{name}.rs"', f'required-features=["{name}"]']
        consumer_manifest = consumer / "Cargo.toml"
        consumer_manifest.write_text("\n".join(lines) + "\n")
        lock = consumer / "Cargo.lock"
        shutil.copyfile(ROOT / "Cargo.lock", lock)
        metadata = json.loads(run("resolve", ["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", consumer_manifest]).stdout)
        qualified = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
        allowed = {consumer_manifest.resolve(), (contract / "Cargo.toml").resolve(),
                   (contract / "runtime-rs/Cargo.toml").resolve(), (contract / "runtime-rs-macros/Cargo.toml").resolve()}
        if not all(path.is_relative_to(output.resolve()) for path in allowed):
            raise ValueError("Passport local manifest escapes isolated SDK")
        receipt["consumer_lock_sha256"] = validate_dependencies(metadata, lock, allowed, qualified)
        # Clippy on a consumer does not lint its SDK dependency as a selected
        # package. Select that unchanged generated crate independently as well.
        sdk_lock = contract / "Cargo.lock"
        if sdk_lock.exists():
            raise ValueError("unexpected generated Passport SDK lock")
        shutil.copyfile(ROOT / "Cargo.lock", sdk_lock)
        sdk_metadata = json.loads(run("sdk-resolve", ["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", contract / "Cargo.toml"]).stdout)
        receipt["sdk_lock_sha256"] = validate_dependencies(sdk_metadata, sdk_lock, allowed, qualified)
        receipt["sdk_lock_added_by_qualification"] = True
        generated_hashes[str(sdk_lock)] = receipt["sdk_lock_sha256"]
        run("sdk-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", contract / "Cargo.toml", "--all-targets", "--", "-D", "warnings"])
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", consumer_manifest, "--", "--show-output"])
        summaries = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result.stdout)
        if sum(int(p) for p, _, _ in summaries) != 18 or any(f != "0" or i != "0" for _, f, i in summaries):
            raise ValueError(f"Passport consumer execution inventory differs: {summaries}")
        receipt["rust_test_methods"] = 18
        run("clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", consumer_manifest, "--all-targets", "--", "-D", "warnings"])
        receipt["compile_fail"] = {}
        for name in NEGATIVES:
            process = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", consumer_manifest,
                                 "--features", name, "--bin", name, "--message-format=json"], success=False)
            receipt["compile_fail"][name] = typed_refusal(process)
        if digest(lock) != receipt["consumer_lock_sha256"]:
            raise ValueError("Passport consumer lock changed during execution")
        current_generated = {str(p): digest(p) for p in generated.rglob("*") if p.is_file() and "node_modules" not in p.parts}
        if current_generated != generated_hashes:
            raise ValueError("Passport generated SDK membership or bytes changed during execution")
        receipt["generated_hashes"] = generated_hashes
        for group in ("source_hashes", "tools", "ts_runtime_hashes"):
            for path, expected in receipt[group].items():
                if digest(path) != expected:
                    raise ValueError(f"Passport qualification input changed: {path}")
        require_clean_source(receipt)
        receipt["status"] = "passed"
    except Exception as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
        raise
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")}))


if __name__ == "__main__":
    main()
