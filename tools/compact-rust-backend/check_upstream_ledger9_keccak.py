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

"""Execute the upstream Keccak assertions against both generated targets."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock

ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/keccak256"
CARGO = ("cargo", "+1.99.0")
PRAGMA = b"pragma language_version >= 0.25.0;\n"


def run(*command, **options):
    return subprocess.run(command, cwd=ROOT, check=True, **options)


def compile_fixture(compiler, fixture, output, probe):
    family = fixture.parent.name
    work = output / family
    work.mkdir()
    source = fixture
    command = (compiler, "--target", "rust", "--target", "ts", "--feature-zkir-v3",
               "--skip-zk", "--rust-runtime-root", ROOT)
    if probe:
        original = fixture.read_bytes()
        if original.count(PRAGMA) != 1:
            raise RuntimeError(f"{fixture} must contain the pinned language pragma exactly once")
        intact = subprocess.run((*command, fixture, work / "intact"), cwd=ROOT, capture_output=True)
        (work / "intact.stderr").write_bytes(intact.stderr)
        if intact.returncode == 0 or b"language version 0.23.105 mismatch" not in intact.stderr:
            raise RuntimeError(f"{fixture} did not refuse its intact language pragma")
        if (work / "intact").exists():
            raise RuntimeError(f"{fixture} published output on version refusal")
        source = work / fixture.name
        source.write_bytes(original.replace(PRAGMA, b"", 1))
    result = subprocess.run((*command, source, work / ".build"), cwd=ROOT, capture_output=True)
    (work / "compile.stdout").write_bytes(result.stdout)
    (work / "compile.stderr").write_bytes(result.stderr)
    (work / "compile.exit").write_text(f"{result.returncode}\n")
    if (fixture.parent / "compile.fail.test.ts").exists():
        return None
    if result.returncode != 0:
        raise RuntimeError(f"{family} compilation failed:\n{result.stderr.decode()}")
    return work / ".build/contract"


def build_consumer(contracts, output, env):
    directory = output / "rust-consumer"
    directory.mkdir()
    shutil.copyfile(FAMILY / "parity/consumer.rs", directory / "main.rs")
    manifest = directory / "Cargo.toml"
    lock = directory / "Cargo.lock"
    shutil.copyfile(ROOT / "Cargo.lock", lock)
    lines = ["[workspace]", "[package]", 'name = "upstream-ledger9-keccak-consumer"',
             'version = "0.0.0"', 'edition = "2024"', "[[bin]]",
             'name = "upstream-ledger9-keccak-consumer"', 'path = "main.rs"',
             "[dependencies]", 'hex = "0.4"', 'serde_json = "1"']
    for family, contract in contracts.items():
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        lines.append(f"{family} = {{ package = {json.dumps(package)}, path = {json.dumps(str(contract))} }}")
    manifest.write_text("\n".join(lines) + "\n")

    metadata = json.loads(run(*CARGO, "metadata", "--offline", "--format-version", "1",
                              "--manifest-path", manifest, env=env, stdout=subprocess.PIPE).stdout)
    pinned = {(row["name"], row["version"], row["source"]): row.get("checksum")
              for row in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"] if "source" in row}
    local = {manifest, ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml"}
    local.update(contract / "Cargo.toml" for contract in contracts.values())
    validate_consumer_lock(metadata, lock, local, pinned)
    result = subprocess.run((*CARGO, "build", "--offline", "--locked", "--message-format=json",
                             "--manifest-path", manifest), cwd=ROOT, env=env, stdout=subprocess.PIPE)
    (directory / "build.jsonl").write_bytes(result.stdout)
    messages = [json.loads(line) for line in result.stdout.splitlines()]
    if result.returncode != 0:
        diagnostics = [message["message"]["rendered"] for message in messages
                       if message["reason"] == "compiler-message" and message["message"]["rendered"]]
        raise RuntimeError("Rust consumer build failed:\n" + "".join(diagnostics))
    [binary] = [message["executable"] for message in messages
                if message["reason"] == "compiler-artifact" and message.get("executable")
                and message["target"]["name"] == "upstream-ledger9-keccak-consumer"]
    return Path(binary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--curves-utils", type=Path, required=True, help="Installed @noble/curves utils.js")
    parser.add_argument("--output", type=Path, required=True, help="Absent directory for generated outputs")
    parser.add_argument("--allow-pragma-probe", action="store_true",
                        help="On language 0.23.105, also compile a derivative without the 0.25.0 pragma")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    curves_utils = args.curves_utils.resolve(strict=True)
    output = args.output.resolve()
    built_package = json.loads((runtime / "package.json").read_text())
    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    if any(built_package[key] != source_package[key] for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match the checkout")
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")

    # A cold Scheme frontend can print its library compilation before the version.
    version_output = run(compiler, "--language-version", text=True, stdout=subprocess.PIPE).stdout
    language = version_output.strip().rpartition("\n")[2]
    if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", language) is None:
        parser.error(f"compiler did not report a language version: {version_output!r}")
    probe = args.allow_pragma_probe and language == "0.23.105"
    runtime_link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    (output / "package.json").write_text('{"type":"module"}\n')
    (output / "language.stdout").write_text(version_output)
    (output / "language.json").write_text(json.dumps({"compiler": language, "pragma_probe": probe}) + "\n")

    contracts = {}
    for fixture in sorted(FAMILY.glob("*/keccak256_*.compact")):
        generated = compile_fixture(compiler, fixture, output, probe)
        if generated is not None and (fixture.parent / "runtime.pass.test.ts").exists():
            contracts[fixture.parent.name] = generated

    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(output / "rust-target"))
    consumer = build_consumer(contracts, output, env)
    env.update(KECCAK_CORPUS_OUTPUT=str(output), KECCAK_CORPUS_CONSUMER=str(consumer),
               KECCAK_CURVES_UTILS=str(curves_utils))
    run("node", ROOT / "node_modules/vitest/vitest.mjs", "run",
        "--config", FAMILY / "parity/vitest.config.mts", env=env)


if __name__ == "__main__":
    main()
