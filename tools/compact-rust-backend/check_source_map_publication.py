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

"""Whole Counter source-map publication checks; no runtime/proof acceptance.

The 28 genuine commands exercise the original paired frontend, combined CLI and
current-source standalone Cargo CLI. Three separately labelled configured test
producers exercise optional/malformed manifest policy, not source authority.
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

from check_passport_current_profile import bounded_command, cargo_target
from check_stateful_product_vector_widening import reject_compiler_overrides

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "examples/rust_backend/counter.compact"
SOURCE_SHA256 = "613747a7d09bb2e73df0f43e751879e54c82a10d08379d4763e1ffb96a37b94e"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def capture_command(command, environment, cwd, stdout, stderr):
    # Redirect before exec so the bounded owner never decodes the child's bytes.
    # exec preserves its process group and the existing timeout/kill policy.
    wrapped = [sys.executable, "-c",
               "import os,sys;os.chdir(sys.argv[1]);"
               "out=open(sys.argv[2],'xb');err=open(sys.argv[3],'xb');"
               "os.dup2(out.fileno(),1);os.dup2(err.fileno(),2);"
               "os.execvpe(sys.argv[4],sys.argv[4:],os.environ)",
               str(cwd), str(stdout), str(stderr), *map(str, command)]
    result = bounded_command(wrapped, environment)
    require(not result.stdout and not result.stderr, "capture controller failed before exec")
    return subprocess.CompletedProcess(command, result.returncode, stdout.read_bytes(), stderr.read_bytes())


def cargo_executable(stdout, manifest):
    artifacts = [message for line in stdout.splitlines() if line
                 for message in [json.loads(line)]
                 if message.get("reason") == "compiler-artifact"
                 and message.get("target", {}).get("name") == "compact-rustc"
                 and "bin" in message.get("target", {}).get("kind", [])
                 and Path(message["manifest_path"]).resolve() == manifest.resolve()
                 and message.get("executable")]
    require(len(artifacts) == 1, "Cargo must report exactly one current standalone executable")
    return Path(artifacts[0]["executable"]).resolve(strict=True)


def tree(path, compiler=False):
    data = {"type": "directory"}
    for item in sorted(path.iterdir()):
        if compiler and item.name == "contract-manifest.json":
            continue
        require(not item.is_symlink(), "generated link is not a regular asset")
        if item.is_dir():
            data[item.name] = tree(item)
        else:
            require(item.is_file(), "non-regular generated asset")
            data[item.name] = {"type": "file", "hash": digest(item), "size": item.stat().st_size}
    return data


def inspect(output, source, explicit):
    path = output / "contract/index.js.map"
    data = json.loads(path.read_text())
    require(data["version"] == 3 and data["file"] == "index.js", "map format differs")
    if explicit is None:
        targets = [Path(os.path.normpath(path.parent / data.get("sourceRoot", "") / name))
                   for name in data["sources"]]
        require(len(targets) == 1 and targets[0].is_file()
                and targets[0].resolve() == source.resolve()
                and digest(targets[0]) == SOURCE_SHA256, "map lost its exact physical Counter source")
    else:
        require(data["sourceRoot"] == explicit, "explicit sourceRoot changed")
    manifest = json.loads((output / "compiler/contract-manifest.json").read_text())
    for name in ("compiler", "contract", "zkir", "keys"):
        directory = output / name
        require(manifest.get(name) == tree(directory, name == "compiler") if directory.is_dir()
                else name not in manifest, "manifest does not match actual publication: " + name)
    info = json.loads((output / "compiler/contract-info.json").read_text())
    require([c["name"] for c in info["circuits"]] == ["increment", "read_round"], "whole source exports changed")
    return {"map": data, "map_sha256": digest(path), "manifest_sha256": digest(output / "compiler/contract-manifest.json"),
            "source_binding": "exact physical source" if explicit is None else "N/A: explicit deployment root",
            "raw_files": {str(p.relative_to(output)): digest(p) for p in output.rglob("*") if p.is_file()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    out = args.output.absolute()
    require(not out.exists() and not out.is_symlink(), "output must be absent")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head = git("rev-parse", "HEAD")
    require(not git("status", "--porcelain"), "clean source required")
    receipt = json.loads(args.compiler_receipt.read_text())
    require(receipt["status"] == "passed" and receipt["head"] == head
            and receipt["tree"] == git("rev-parse", "HEAD^{tree}"), "original recipe must bind current clean source")
    compiler = Path(receipt["compactc"]["path"]).resolve(strict=True)
    scheme = Path(receipt["compactc-scheme"]["path"]).resolve(strict=True)
    require(compiler.parent == scheme.parent and digest(compiler) == receipt["compactc"]["sha256"]
            and digest(scheme) == receipt["compactc-scheme"]["sha256"], "compiler package identity changed")
    require(digest(SOURCE) == SOURCE_SHA256, "complete Counter source changed")
    require(os.name == "posix", "this configured-producer gate requires the current macOS/Linux CI hosts")
    python = Path(sys.executable).resolve(strict=True)
    require(not any(c.isspace() for c in str(python)), "test producer requires a whitespace-free interpreter path")
    env = dict(os.environ)
    config = reject_compiler_overrides(dict(env))
    env.update(RUSTUP_TOOLCHAIN="1.99.0", CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    env["CARGO_TARGET_DIR"] = str(cargo_target(env, out))
    env.pop("LD_LIBRARY_PATH", None)
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    tools = {name: Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name], text=True).strip()).resolve(strict=True)
             for name in ("cargo", "rustc")}
    env["RUSTC"] = str(tools["rustc"])
    env["PATH"] = str(tools["cargo"].parent) + os.pathsep + env["PATH"]
    pins = {str(p): digest(p) for p in [SOURCE, Path(__file__), args.compiler_receipt.resolve(), compiler, scheme, python, rustup, *tools.values(), ROOT / "Cargo.lock", ROOT / "Cargo.toml", ROOT / "flake.nix", ROOT / "flake.lock"]}
    for directory in (ROOT / "compiler", ROOT / "tools/compact-rust-backend/src", ROOT / "runtime-rs", ROOT / "runtime-rs-macros"):
        pins.update({str(p): digest(p) for p in directory.rglob("*") if p.is_file() and p.suffix in (".rs", ".ss", ".json", ".toml")})
    pins.update({str(p): digest(p) for p in Path(__file__).parent.glob("*.py")})
    out.mkdir(parents=True)
    bound = {}
    report = {"status": "running", "head": head, "inputs": pins, "commands": [], "genuine": {}, "private_manifest_controls": {},
              "scope": __doc__, "proof_execution": False, "cargo_configuration_sha256": config}
    started = time.monotonic()

    def unchanged():
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source changed")
        require(reject_compiler_overrides(dict(os.environ)) == config, "Cargo configuration changed")
        require(all(Path(p).is_file() and digest(p) == h for p, h in {**pins, **bound}.items()), "bound input changed")

    def run(label, command, cwd=ROOT, extra=None, expected=0):
        unchanged()
        environment = dict(env)
        environment.update(extra or {})
        actual = list(map(str, command))
        report["current"] = label
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
        began = time.monotonic()
        try:
            result = capture_command(actual, environment, cwd, out / (label + ".stdout"), out / (label + ".stderr"))
        except subprocess.TimeoutExpired:
            row = {"label": label, "actual_argv": actual, "cwd": str(cwd), "timeout": True,
                   "seconds": time.monotonic() - began}
            for stream in ("stdout", "stderr"):
                path = out / (label + "." + stream)
                if path.is_file():
                    row[stream + "_sha256"] = digest(path)
            report["commands"].append(row)
            raise
        row = {"label": label, "actual_argv": actual, "cwd": str(cwd), "exit": result.returncode, "seconds": time.monotonic() - began}
        for stream in ("stdout", "stderr"):
            path = out / (label + "." + stream)
            row[stream + "_sha256"] = digest(path)
        report["commands"].append(row)
        unchanged()
        require(result.returncode == expected, label + " failed; raw diagnostics retained")
        return result

    try:
        build = run("build-current-standalone", ["cargo", "build", "--locked", "--offline", "--message-format=json-render-diagnostics", "-p", "compact-rust-backend", "--bin", "compact-rustc"])
        executable = cargo_executable(build.stdout, ROOT / "tools/compact-rust-backend/Cargo.toml")
        standalone = out / "tools/compact-rustc"
        standalone.parent.mkdir()
        shutil.copy2(executable, standalone)
        require(digest(executable) == digest(standalone), "reported Cargo executable changed while copying")
        bound[str(standalone)] = digest(standalone)
        report["standalone"] = {"path": str(standalone), "sha256": digest(standalone), "cargo_reported_executable": str(executable), "scope": "current maintained Cargo entrypoint; not a binary from the Nix package"}
        specs = []
        for name in ("plain", "Unicode β space", 'literal"quote β space'):
            cwd = out / "cwd" / name
            (cwd / "source").mkdir(parents=True)
            source = cwd / "source/counter.compact"
            shutil.copyfile(SOURCE, source)
            bound[str(source)] = digest(source)
            for lane in ("scheme", "combined", "standalone"):
                for skind in (("relative", "absolute") if name == "plain" else ("relative",)):
                    for okind in (("absolute", "relative", "parent") if name == "plain" else ("absolute",)):
                        label = f"{lane}-{name}-{skind}-{okind}"
                        target = str(out / "published" / label) if okind == "absolute" else ("published/" + label if okind == "relative" else "../parent-published/" + label)
                        specs.append((label, lane, cwd, source, str(source) if skind == "absolute" else "source/counter.compact", target, None))
            if name == "plain":
                for lane in ("scheme", "combined"):
                    for suffix, root in (("url", "https://example.test/sources/"), ("escaped", '/explicit/quoted"β\\source/\n\t')):
                        label = lane + "-explicit-" + suffix
                        specs.append((label, lane, cwd, source, "source/counter.compact", str(out / "published" / label), root))
        require(len(specs) == 28, "genuine case roster changed")
        seed = None
        for label, lane, cwd, source, operand, target, root in specs:
            destination = Path(target) if Path(target).is_absolute() else cwd / target
            destination.parent.mkdir(parents=True, exist_ok=True)
            require(not destination.exists(), "publication destination is not fresh")
            if lane == "scheme":
                command = [scheme, "--emit-rust-ir", "--feature-zkir-v3", "--skip-zk"]
            elif lane == "combined":
                command = [compiler, "--target", "rust", "--target", "ts", "--skip-zk", "--rust-runtime-root", ROOT, "--rust-functional-ledger9", "--rust-require-recording"]
            else:
                command = [standalone]
            if root is not None:
                command += ["--sourceRoot", root]
            run(label, [*command, operand, target], cwd, {"COMPACTC": str(compiler)} if lane == "standalone" else None)
            report["genuine"][label] = inspect(destination, source, root)
            if len(report["genuine"]) > 1:
                first = next(iter(report["genuine"].values()))
                current = report["genuine"][label]
                require({k: v for k, v in current["map"].items() if k not in ("sourceRoot", "sources")}
                        == {k: v for k, v in first["map"].items() if k not in ("sourceRoot", "sources")},
                        "non-path map values changed across identical whole Counter sources")
                require(current["raw_files"]["contract/index.js"] == first["raw_files"]["contract/index.js"],
                        "generated Counter JS body changed across path forms")
            bound.update({str(destination / p): h for p, h in report["genuine"][label]["raw_files"].items()})
            if label == "scheme-plain-relative-absolute":
                seed = destination
        require(seed is not None, "missing complete genuine Counter seed")
        cwd = out / "cwd/plain"
        for mode in ("missing", "malformed", "directory"):
            producer = out / "tools" / (mode + "-producer.py")
            text = f"#!{python}\n" + "import pathlib,shutil,sys\n" + f"seed=pathlib.Path({str(seed)!r});output=pathlib.Path(sys.argv[-1])\n" + "for entry in seed.iterdir():\n if entry.is_dir():\n  shutil.copytree(entry,output/entry.name,dirs_exist_ok=True,ignore=lambda p,n:['contract-manifest.json'] if pathlib.Path(p).name=='compiler' else [])\n else: shutil.copyfile(entry,output/entry.name)\n"
            if mode == "malformed":
                text += "(output/'compiler/contract-manifest.json').write_text('[]')\n"
            if mode == "directory":
                text += "(output/'compiler/contract-manifest.json').mkdir()\n"
            producer.write_text(text)
            producer.chmod(0o755)
            bound[str(producer)] = digest(producer)
            destination = out / "private" / mode
            destination.parent.mkdir(exist_ok=True)
            if mode != "missing":
                destination.mkdir()
                (destination / "sentinel").write_bytes(b"prior publication must survive")
            result = run("private-" + mode, [standalone, "source/counter.compact", destination], cwd,
                         {"COMPACTC": str(producer)}, 0 if mode == "missing" else 1)
            if mode == "missing":
                require((destination / "contract/lib.rs").is_file() and not (destination / "compiler/contract-manifest.json").exists(), "optional manifest policy changed")
            else:
                require({p.name for p in destination.iterdir()} == {"sentinel"}
                        and (destination / "sentinel").read_bytes() == b"prior publication must survive", "failed publication replaced prior output")
                require((b"invalid contract manifest" if mode == "malformed" else b"not a regular file") in result.stderr, "wrong manifest refusal")
            report["private_manifest_controls"][mode] = {"status": "passed", "role": "configured test producer; no source/proof authority"}
        require(len(report["commands"]) == 32 and len(report["genuine"]) == 28, "command roster differs")
        unchanged()
        report.update(status="passed", genuine_commands=28, private_commands=3, full_integration_acceptance="OPEN")
    except BaseException as error:
        report.update(status="failed", error=repr(error))
        raise
    finally:
        report.update(seconds=time.monotonic() - started, bound_inputs=bound)
        (out / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in ("status", "head", "genuine_commands", "private_commands")}))


if __name__ == "__main__":
    main()
