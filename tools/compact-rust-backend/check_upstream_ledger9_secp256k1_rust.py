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

"""Run the original secp256k1 known-answer inputs through generated Rust APIs."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock

ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/secp256k1/known-answer"
FIXTURES = ("ecdsa_verify", "bitcoin_kat", "ethereum_address")
CARGO = ("cargo", "+1.99.0")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="Output of the TypeScript known-answer runner")
    args = parser.parse_args()
    output = args.output.resolve(strict=True)
    contracts = {
        fixture: output / fixture / "ts-rust/.build/contract/Cargo.toml"
        for fixture in FIXTURES
    }
    consumer = output / "rust-consumer"
    consumer.mkdir()
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "consumer.rs")
    for fixture, destination in (("ecdsa_verify", "ecdsa-vectors.json"),
                                 ("ethereum_address", "address-vectors.json")):
        shutil.copyfile(output / fixture / "ts-rust/vectors.json", consumer / destination)
    lock = consumer / "Cargo.lock"
    shutil.copyfile(ROOT / "Cargo.lock", lock)
    manifest = consumer / "Cargo.toml"
    lines = ["[workspace]", "[package]", 'name = "upstream-ledger9-secp256k1-consumer"',
             'version = "0.0.0"', 'edition = "2024"', 'rust-version = "1.99"',
             "[[test]]", 'name = "known-answer"', 'path = "consumer.rs"',
             "[dependencies]", 'hex = "0.4"', 'serde = { version = "1", features = ["derive"] }',
             'serde_json = "1"']
    for fixture, path in contracts.items():
        package = tomllib.loads(path.read_text())["package"]["name"]
        lines.append(f"{fixture} = {{ package = {json.dumps(package)}, path = {json.dumps(str(path.parent))} }}")
    manifest.write_text("\n".join(lines) + "\n")

    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(output / "rust-target"))
    metadata_result = subprocess.run(
        (*CARGO, "metadata", "--offline", "--format-version", "1"),
        cwd=consumer, env=env, check=True, stdout=subprocess.PIPE,
    )
    (consumer / "metadata.json").write_bytes(metadata_result.stdout)
    metadata = json.loads(metadata_result.stdout)
    pinned = {(row["name"], row["version"], row["source"]): row.get("checksum")
              for row in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"] if "source" in row}
    local = {manifest, ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml", *contracts.values()}
    validate_consumer_lock(metadata, lock, local, pinned)
    subprocess.run(
        (*CARGO, "test", "--offline", "--locked", "--", "--nocapture"),
        cwd=consumer, env=env, check=True,
    )


if __name__ == "__main__":
    main()
