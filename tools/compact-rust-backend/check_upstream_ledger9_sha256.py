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

"""Execute the upstream SHA-256 assertions against both generated targets."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock

ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/sha256"
CARGO = ("cargo", "+1.99.0")
PINNED = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"


def run(*command, **options):
    return subprocess.run(command, cwd=ROOT, check=True, **options)


def version_policy_controls(compiler, output):
    legacy = run(compiler, "--language-version", text=True,
                 stdout=subprocess.PIPE).stdout.strip().splitlines()[-1]
    ledger9 = run(compiler, "--feature-zkir-v3", "--language-version", text=True,
                  stdout=subprocess.PIPE).stdout.strip().splitlines()[-1]
    if (legacy, ledger9) != ("0.23.105", "0.28.0"):
        raise RuntimeError(f"unexpected language profiles: {legacy}, {ledger9}")
    controls = output / "version-controls"
    controls.mkdir()
    original = FAMILY / "multi_length/sha256_multi_length.compact"
    rows = []
    for name, source, flags, expected in [
        ("ledger8_original", original, (), b"language version 0.23.105 mismatch"),
        ("ledger9_future", controls / "future.compact", ("--feature-zkir-v3",),
         b"language version 0.28.0 mismatch"),
        ("ledger9_upper_bound", controls / "upper.compact", ("--feature-zkir-v3",),
         b"language version 0.28.0 mismatch"),
    ]:
        if name == "ledger9_future":
            source.write_text("pragma language_version >= 0.29.0;\n"
                              "export circuit id(x: Field): Field { return x; }\n")
        if name == "ledger9_upper_bound":
            source.write_text("pragma language_version < 0.25.0;\n"
                              "export circuit id(x: Field): Field { return x; }\n")
        product = controls / name
        result = subprocess.run((compiler, *flags, source, product), cwd=ROOT,
                                capture_output=True)
        (controls / f"{name}.stderr").write_bytes(result.stderr)
        if result.returncode == 0 or expected not in result.stderr or product.exists():
            raise RuntimeError(f"language refusal changed: {name}")
        rows.append({"name": name, "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                     "exit": result.returncode,
                     "diagnostic_sha256": hashlib.sha256(result.stderr).hexdigest(),
                     "product_absent": True})
    (controls / "receipt.json").write_text(json.dumps({"legacy": legacy,
                                                         "ledger9": ledger9,
                                                         "refusals": rows}, indent=2) + "\n")
    return ledger9


def compile_fixture(compiler, fixture, output):
    family = fixture.parent.name
    work = output / family
    work.mkdir()
    original = fixture.read_bytes()
    command = (compiler, "--target", "rust", "--target", "ts", "--feature-zkir-v3",
               "--rust-runtime-root", ROOT)
    relative = fixture.relative_to(ROOT)
    pinned = subprocess.check_output(["git", "show", f"{PINNED}:{relative}"], cwd=ROOT)
    if original != pinned:
        raise RuntimeError(f"{fixture} differs from pinned upstream source")
    (work / "source.json").write_text(json.dumps({
        "original": str(fixture),
        "original_sha256": hashlib.sha256(original).hexdigest(),
        "compiled": str(fixture),
        "compiled_sha256": hashlib.sha256(original).hexdigest(),
        "compiled_source_kind": "pinned original",
    }, indent=2) + "\n")
    result = subprocess.run((*command, fixture, work / ".build"), cwd=ROOT, capture_output=True)
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
    lines = ["[workspace]", "[package]", 'name = "upstream-ledger9-sha256-consumer"',
             'version = "0.0.0"', 'edition = "2024"', "[[bin]]",
             'name = "upstream-ledger9-sha256-consumer"', 'path = "main.rs"',
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
                and message["target"]["name"] == "upstream-ledger9-sha256-consumer"]
    return Path(binary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="Absent directory for generated outputs")
    parser.add_argument("--ts-only", action="store_true",
                        help="Run original TypeScript callbacks without a Cargo consumer")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    built_package = json.loads((runtime / "package.json").read_text())
    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    if any(built_package[key] != source_package[key] for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match the checkout")
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")

    runtime_link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    (output / "package.json").write_text('{"type":"module"}\n')
    language = version_policy_controls(compiler, output)
    (output / "language.json").write_text(json.dumps({"compiler": language,
                                                       "profile": "Ledger 9 ZKIR v3"}) + "\n")

    contracts = {}
    for fixture in sorted(FAMILY.glob("*/sha256_*.compact")):
        generated = compile_fixture(compiler, fixture, output)
        if generated is not None and (fixture.parent / "runtime.pass.test.ts").exists():
            contracts[fixture.parent.name] = generated

    env = os.environ.copy()
    if args.ts_only:
        consumer = Path("/bin/false")
    else:
        env.setdefault("CARGO_TARGET_DIR", str(output / "rust-target"))
        consumer = build_consumer(contracts, output, env)
    env.update(SHA256_CORPUS_OUTPUT=str(output), SHA256_CORPUS_CONSUMER=str(consumer))
    command = ["node", ROOT / "test-contracts/node_modules/vitest/vitest.mjs", "run",
               "--config", FAMILY / "parity/vitest.config.mts"]
    if args.ts_only:
        command.extend(["-t", "compile|TypeScript runtime"])
    run(*command, env=env)


if __name__ == "__main__":
    main()
