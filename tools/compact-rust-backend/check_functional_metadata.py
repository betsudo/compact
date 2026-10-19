#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Compare embedded/sidecar metadata and exercise local relocation and refusals."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_functional_effects import (ROOT, SOURCES, digest, generated_manifest_snapshot,
                                      scheme_compiler)


def main():
    if not __debug__:
        raise RuntimeError("qualification requires Python without -O/-OO")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    # Nested generated crates would inherit the repository Cargo workspace,
    # invalidating the independent relocation/dependency-resolution control.
    if output.is_relative_to(ROOT):
        parser.error("output must be outside the repository to avoid Cargo workspace inheritance")
    if output.exists() or output.is_symlink():
        parser.error("output must be absent")
    output.mkdir(parents=True)
    environment = dict(os.environ)
    binaries = {key: path.resolve(strict=True) for key, path in
                (("baseline", args.baseline), ("candidate", args.candidate))}
    schemes = {key: scheme_compiler(path, environment) for key, path in binaries.items()}
    if len(set(schemes.values())) != 1:
        parser.error("baseline and candidate must use the same actual Scheme compiler")
    scheme = next(iter(schemes.values()))
    environment["COMPACTC_SCHEME"] = str(scheme)
    premises = [*binaries.values(), scheme, ROOT / "Cargo.lock", ROOT / "Cargo.toml",
                Path(__file__).with_name("check_compactc_target.py"),
                ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs/compatibility.json",
                ROOT / "runtime-rs-macros/Cargo.toml", Path(__file__).resolve(),
                Path(__file__).with_name("check_functional_effects.py"), *SOURCES.values()]
    premises += sorted((ROOT / "runtime-rs/src").rglob("*.rs"))
    premises += sorted((ROOT / "runtime-rs-macros/src").rglob("*.rs"))
    hashes = {str(path): digest(path) for path in premises}
    report = {"passed": False, "premises": hashes, "phases": [], "sources": {}}
    started = time.monotonic()

    def run(label, argv, expected_success=True):
        before = time.monotonic()
        result = subprocess.run(list(map(str, argv)), cwd=ROOT, env=environment,
                                capture_output=True, text=True)
        (output / (label + ".log")).write_text(result.stdout + result.stderr)
        report["phases"].append({"name": label, "argv": list(map(str, argv)),
            "exit_code": result.returncode, "wall_seconds": time.monotonic() - before})
        if (result.returncode == 0) != expected_success:
            raise ValueError("unexpected command outcome: " + label)
        return result.stdout if expected_success else result.stdout + result.stderr


    def resolve_locked(label, manifest, runtime_root):
        shutil.copyfile(ROOT / "Cargo.lock", manifest.parent / "Cargo.lock")
        metadata = json.loads(run(label + "-resolve", ["cargo", "metadata", "--offline",
            "--format-version", "1", "--manifest-path", manifest]))
        root_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        identities = {(row["name"], row["version"], row.get("source"), row.get("checksum"))
                      for row in root_lock["package"]}
        local = {}
        for row in metadata["packages"]:
            if row.get("source") is None:
                local[row["name"]] = str(Path(row["manifest_path"]).resolve())
                if Path(row["manifest_path"]).resolve() not in {
                        manifest.resolve(), (runtime_root / "runtime-rs/Cargo.toml").resolve(),
                        (runtime_root / "runtime-rs-macros/Cargo.toml").resolve()}:
                    raise ValueError("unexpected relocated local dependency")
        child_lock = tomllib.loads((manifest.parent / "Cargo.lock").read_text())
        for row in child_lock["package"]:
            identity = (row["name"], row["version"], row.get("source"), row.get("checksum"))
            if row.get("source") is not None and identity not in identities:
                raise ValueError("relocated external dependency identity changed")
            if row.get("source") is None and row["name"] not in local:
                raise ValueError("unresolved relocated local dependency")
        child_lock_hash = digest(manifest.parent / "Cargo.lock")
        report[label + "_local_dependencies"] = local
        report[label + "_lock_sha256"] = child_lock_hash
        return child_lock_hash

    try:
        total = 0
        for alias, source in SOURCES.items():
            paths = {}
            for owner, compiler in binaries.items():
                for mode in ("default", "functional"):
                    destination = output / f"{alias}-{owner}-{mode}"
                    argv = [compiler, "--target", "rust", "--skip-zk", "--rust-require-recording",
                            "--rust-runtime-root", ROOT]
                    if mode == "functional":
                        argv.append("--rust-functional-ledger8")
                    run(destination.name, [*argv, source, destination])
                    generated_manifest_snapshot(destination)
                    paths[owner, mode] = destination
            def tree_bytes(directory, *, functional=False):
                return {str(path.relative_to(directory)): path.read_bytes()
                        for path in directory.rglob("*") if path.is_file()
                        and not (functional and (
                            path.relative_to(directory).as_posix() in {
                                "contract/lib.rs", "compiler/contract-manifest.json"}
                            or path.relative_to(directory).as_posix().startswith("contract/checked-programs/")))}
            if tree_bytes(paths["baseline", "default"]) != tree_bytes(paths["candidate", "default"]):
                raise ValueError("default generated output tree changed: " + alias)
            if tree_bytes(paths["baseline", "functional"], functional=True) != tree_bytes(paths["candidate", "functional"], functional=True):
                raise ValueError("functional output tree changed outside controlled metadata assets: " + alias)
            old_lib = (paths["baseline", "functional"] / "contract/lib.rs").read_text()
            original = {}
            original_modules = {}
            old_constants = list(re.finditer(r'pub const CHECKED_PROGRAM:\s*&str\s*=\s*("(?:\\.|[^"\\])*")\s*;', old_lib))
            for match in old_constants:
                literal = match.group(1)
                raw = json.loads(literal)
                root = json.loads(raw)["root"]
                if root in original:
                    raise ValueError("duplicate embedded metadata root")
                original[root] = raw.encode()
                module = re.findall(r"pub mod ((?:r#)?\w+)\s*\{", old_lib[:match.start()])[-1]
                if module in original_modules:
                    raise ValueError("duplicate original public metadata module")
                original_modules[module] = raw.encode()
            destination = paths["candidate", "functional"]
            sidecars = sorted((destination / "contract/checked-programs").glob("program-*.json"))
            actual = {}
            for sidecar in sidecars:
                raw = sidecar.read_bytes()
                root = json.loads(raw)["root"]
                if root in actual:
                    raise ValueError("duplicate sidecar root")
                actual[root] = raw
            candidate_lib = (destination / "contract/lib.rs").read_text()
            includes = list(re.finditer(r'pub const CHECKED_PROGRAM:\s*&str\s*=\s*(?:(?:::)?[\w:]+::)?include_str!\s*\(\s*("[^"\n]+")\s*\)\s*;', candidate_lib))
            if len(old_constants) != len(includes):
                raise ValueError("public metadata declaration counts differ: " + alias)
            candidate_modules = {}
            for match in includes:
                relative = Path(json.loads(match.group(1)))
                if relative.is_absolute() or ".." in relative.parts:
                    raise ValueError("metadata include escapes generated crate")
                module = re.findall(r"pub mod ((?:r#)?\w+)\s*\{", candidate_lib[:match.start()])[-1]
                if module in candidate_modules:
                    raise ValueError("duplicate public metadata module")
                candidate_modules[module] = (destination / "contract" / relative).read_bytes()
            def without_metadata(text, matches):
                for match in reversed(matches):
                    text = text[:match.start()] + "pub const CHECKED_PROGRAM: &str = METADATA;" + text[match.end():]
                return text
            if without_metadata(old_lib, old_constants) != without_metadata(candidate_lib, includes):
                raise ValueError("generated execution/API source changed beyond metadata constants: " + alias)
            if original_modules != candidate_modules:
                raise ValueError("public module metadata mapping changed: " + alias)
            if not original or original != actual:
                raise ValueError("metadata bytes/export accounting changed: " + alias)
            total += len(actual)
            report["sources"][alias] = {"exports": len(actual), "roots": sorted(actual),
                "snapshot": generated_manifest_snapshot(destination)}

        # The copy is the only mutation fixture. Original generated trees stay intact.
        original = output / "let_fixture-candidate-functional"
        relocated = output / "relocated"
        shutil.copytree(original, relocated)
        original_saved = original.with_name(original.name + "-unavailable")
        original.rename(original_saved)
        try:
            manifest = relocated / "contract/Cargo.toml"
            listed = run("relocated-package-list", ["cargo", "package", "--list", "--allow-dirty",
                "--manifest-path", manifest, "--offline"])
            sidecar = next((relocated / "contract/checked-programs").glob("program-*.json"))
            relative = sidecar.relative_to(relocated / "contract").as_posix()
            if relative not in listed.splitlines():
                raise ValueError("sidecar missing from Cargo package file list")
            child_lock_hash = resolve_locked("relocated", manifest, ROOT)
            run("relocated-check", ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest])
            pristine = sidecar.read_bytes()
            sidecar.unlink()
            try:
                generated_manifest_snapshot(relocated)
                raise ValueError("missing sidecar was accepted")
            except FileNotFoundError:
                pass
            missing = run("missing-sidecar-check", ["cargo", "check", "--locked", "--offline",
                "--manifest-path", manifest], expected_success=False)
            if "checked-programs" not in missing or "couldn't read" not in missing:
                raise ValueError("missing-sidecar failure was unrelated")
            sidecar.write_bytes(pristine)
            changed = json.loads(pristine)
            changed["purpose"] = "tampered-valid-json"
            sidecar.write_text(json.dumps(changed))
            try:
                generated_manifest_snapshot(relocated)
            except AssertionError:
                pass
            else:
                raise ValueError("valid JSON tamper was accepted by manifest validator")
            sidecar.write_bytes(pristine)
            generated_manifest_snapshot(relocated)
            extra = sidecar.with_name("program-undeclared.json")
            extra.write_bytes(pristine)
            try:
                generated_manifest_snapshot(relocated)
            except ValueError as error:
                if "assets differ" not in str(error):
                    raise
            else:
                raise ValueError("undeclared sidecar was accepted")
            finally:
                extra.unlink()
            run("restored-sidecar-check", ["cargo", "check", "--locked", "--offline",
                "--manifest-path", manifest])
            if digest(manifest.parent / "Cargo.lock") != child_lock_hash:
                raise ValueError("relocated lock changed during validation")
        finally:
            original_saved.rename(original)
        bundled = output / "bundled-original"
        run("bundled-generate", [binaries["candidate"], "--target", "rust", "--skip-zk",
            "--rust-require-recording", "--rust-functional-ledger8", SOURCES["let_fixture"], bundled])
        bundled_snapshot = generated_manifest_snapshot(bundled)
        bundled_copy = output / "bundled-relocated"
        shutil.copytree(bundled, bundled_copy)
        bundled_saved = bundled.with_name("bundled-original-unavailable")
        bundled.rename(bundled_saved)
        try:
            manifest = bundled_copy / "contract/Cargo.toml"
            lock_hash = resolve_locked("bundled", manifest, manifest.parent)
            run("bundled-relocated-check", ["cargo", "check", "--locked", "--offline",
                "--manifest-path", manifest])
            if digest(manifest.parent / "Cargo.lock") != lock_hash:
                raise ValueError("bundled relocated lock changed")
            if generated_manifest_snapshot(bundled_copy) != bundled_snapshot:
                raise ValueError("bundled relocated assets changed")
        finally:
            bundled_saved.rename(bundled)
        report["bundled_snapshot"] = bundled_snapshot
        for alias in SOURCES:
            destination = output / f"{alias}-candidate-functional"
            if generated_manifest_snapshot(destination) != report["sources"][alias]["snapshot"]:
                raise ValueError("candidate assets changed during validation")
        if hashes != {str(path): digest(path) for path in premises}:
            raise ValueError("source/compiler/runtime premises changed")
        report.update(passed=True, source_count=len(SOURCES), export_count=total,
            controls=["entire default output tree bytes unchanged",
                      "functional output tree unchanged outside controlled metadata assets", "all metadata exact bytes unchanged",
                      "relocated Cargo check", "sidecar Cargo package inclusion",
                      "missing sidecar compile refusal", "valid JSON tamper manifest refusal",
                      "undeclared sidecar refusal", "restored sidecar compilation",
                      "public module/include mapping unchanged"],
            limitation="ASCII fixture metadata only (Rust literals decoded as JSON); include_str is packaging, not authentication; manifest catches uncoordinated edits")
    except Exception as error:
        report["error"] = str(error)
        print(error, flush=True)
    finally:
        report["wall_seconds"] = time.monotonic() - started
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
