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

"""Build six generated contracts together using only the runtime archives."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

from consumer_safety import check_consumer_safety
from check_release_packages import LOCKS, compatibility_record


ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = (
    ("counter", ROOT / "examples/rust_backend/counter.compact"),
    ("cell-boolean", ROOT / "examples/rust_backend/cell_boolean.compact"),
    ("set-boolean", ROOT / "examples/rust_backend/set_boolean.compact"),
    ("map-boolean-field", ROOT / "examples/rust_backend/map_boolean_field.compact"),
    ("list-field", ROOT / "examples/rust_backend/list_field.compact"),
    ("merkle-path-witness", ROOT / "examples/rust_backend/merkle_path_witness.compact"),
)
PACKAGE_NAMES = ("midnight-compact-runtime-macros", "midnight-compact-runtime")
MACROS, RUNTIME = PACKAGE_NAMES


def run(arguments: list[str], *, cwd: Path = ROOT, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(arguments, cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(
            f"{' '.join(arguments)} failed with {result.returncode}:\n"
            f"{result.stdout[-4000:]}\n{result.stderr[-4000:]}"
        )
    return result.stdout


def sha256(path: Path) -> str:
    with path.open("rb") as content:
        return hashlib.file_digest(content, "sha256").hexdigest()


def install_archive(vendor: Path, entry: dict, macro_version: str) -> Path:
    name = entry["name"]
    version = entry["version"]
    if name not in PACKAGE_NAMES or not isinstance(version, str):
        raise RuntimeError("release manifest contains an unexpected package")
    archive = ROOT / entry["archive"]
    if not archive.is_file() or sha256(archive) != entry["sha256"]:
        raise RuntimeError(f"{archive}: missing or different from release manifest")
    directory = vendor / f"{name}-{version}"
    if directory.exists():
        raise RuntimeError(f"{directory}: already present in upstream vendor source")
    with tarfile.open(archive, "r:gz") as package:
        members = package.getmembers()
        if not members:
            raise RuntimeError(f"{archive}: empty package")
        for member in members:
            destination = (vendor / member.name).resolve()
            if not destination.is_relative_to(directory.resolve()) or not member.isfile():
                raise RuntimeError(f"{archive}: unsafe member {member.name}")
        package.extractall(vendor, filter="data")
    manifest = tomllib.loads((directory / "Cargo.toml").read_text())
    if manifest["package"]["name"] != name or manifest["package"]["version"] != version:
        raise RuntimeError(f"{archive}: package identity mismatch")
    if name == RUNTIME:
        dependency = manifest["dependencies"][MACROS]
        if dependency != {"version": f"={macro_version}"}:
            raise RuntimeError(f"{archive}: macro dependency is not version-only")
    files = {
        path.relative_to(directory).as_posix(): sha256(path)
        for path in directory.rglob("*") if path.is_file()
    }
    (directory / ".cargo-checksum.json").write_text(
        json.dumps({"files": files, "package": entry["sha256"]}, sort_keys=True)
    )
    return directory


def archived_source_hashes(archives: dict[str, Path]) -> dict[str, str]:
    """Identify the exact archive files covered by the compiler's source fingerprint."""
    hashes = {}
    for name, source in ((MACROS, "runtime-rs-macros"), (RUNTIME, "runtime-rs")):
        archive = archives[name]
        hashes[f"{source}/Cargo.toml"] = sha256(archive / "Cargo.toml.orig")
        for path in (archive / "src").rglob("*"):
            if path.is_file():
                hashes[f"{source}/{path.relative_to(archive).as_posix()}"] = sha256(path)
    hashes["runtime-rs/compatibility.json"] = sha256(archives[RUNTIME] / "compatibility.json")
    return hashes


def file_record(path: Path) -> dict:
    return {"type": "file", "size": path.stat().st_size, "hash": sha256(path)}


def contract_record(contract: Path) -> dict:
    record = {"type": "directory"}
    for path in contract.iterdir():
        if path.is_symlink():
            raise RuntimeError(f"generated contract contains a symlink: {path}")
        record[path.name] = contract_record(path) if path.is_dir() else file_record(path)
    return record


def project_contract(
    artifact: Path, destination: Path, compatibility: dict, source_hashes: dict[str, str],
) -> Path:
    contract = artifact / "contract"
    if (contract / "runtime-rs").exists() or (contract / "runtime-rs-macros").exists():
        raise RuntimeError(f"{artifact}: shared-source generation copied runtime sources")
    manifest = json.loads((artifact / "compiler/contract-manifest.json").read_text())
    compatibility_path = artifact / "compiler/rust-compatibility.json"
    if (
        manifest["contract"] != contract_record(contract)
        or manifest["compiler"]["rust-compatibility.json"] != file_record(compatibility_path)
    ):
        raise RuntimeError(f"{artifact}: generated files differ from their recorded hashes")
    metadata = json.loads(compatibility_path.read_text())
    if (
        metadata["schema_version"] != 1
        or metadata["distribution"] != "shared-source"
        or metadata["registry_publication_verified"] is not False
        or metadata["compatibility"] != compatibility
        or metadata["source_sha256"] != source_hashes
    ):
        raise RuntimeError(f"{artifact}: generated runtime source differs from the release archives")

    original = (contract / "Cargo.toml").read_text()
    generated = tomllib.loads(original)
    dependency = generated["dependencies"][RUNTIME]
    if dependency != {"path": str(ROOT.resolve() / "runtime-rs"), "package": RUNTIME}:
        raise RuntimeError(f"{artifact}: generated manifest has an unexpected runtime dependency")

    # Preserve the original compiler output and change just the runtime edge in
    # the consumer copy. Cargo then resolves the same sources via the archives.
    version = f"={compatibility['runtime_version']}"
    lines = original.splitlines(keepends=True)
    runtime_lines = [index for index, line in enumerate(lines) if line.startswith(f"{RUNTIME} = ")]
    if len(runtime_lines) != 1:
        raise RuntimeError(f"{artifact}: cannot identify the single generated runtime dependency")
    lines[runtime_lines[0]] = f'{RUNTIME} = {{ version = "{version}", package = "{RUNTIME}" }}\n'
    projected = "".join(lines)
    generated["dependencies"][RUNTIME] = {"version": version, "package": RUNTIME}
    if tomllib.loads(projected) != generated:
        raise RuntimeError(f"{artifact}: archive projection changed another manifest setting")
    shutil.copytree(contract, destination)
    (destination / "Cargo.toml").write_text(projected)
    return destination


def check_graph(
    metadata: dict, consumer: Path, contracts: dict[str, Path],
    archives: dict[str, Path], entries: list[dict],
) -> None:
    packages = metadata["packages"]
    lock_packages = tomllib.loads((consumer / "Cargo.lock").read_text())["package"]
    package_ids = {}
    for entry in entries:
        name = entry["name"]
        found = [package for package in packages if package["name"] == name]
        if len(found) != 1:
            raise RuntimeError(f"expected one {name} in consumer graph, got {len(found)}")
        package = found[0]
        if (
            not (package["source"] or "").startswith("registry+")
            or package["version"] != entry["version"]
            or Path(package["manifest_path"]).resolve() != (archives[name] / "Cargo.toml").resolve()
        ):
            raise RuntimeError(f"{name} did not resolve to the exact archived registry package")
        lock_entry = [locked for locked in lock_packages if locked["name"] == name]
        if len(lock_entry) != 1 or any(
            lock_entry[0].get(key) != expected for key, expected in (
                ("version", entry["version"]), ("source", package["source"]), ("checksum", entry["sha256"]),
            )
        ):
            raise RuntimeError(f"{name} does not match the archive checksum and source in Cargo.lock")
        package_ids[name] = package["id"]

    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    runtime_id = package_ids[RUNTIME]
    if package_ids[MACROS] not in {dependency["pkg"] for dependency in nodes[runtime_id]["deps"]}:
        raise RuntimeError("archived runtime did not resolve the archived macros")
    for name, contract in contracts.items():
        package_name = f"compact-contract-{name}"
        found = [package for package in packages if package["name"] == package_name]
        if len(found) != 1 or Path(found[0]["manifest_path"]).resolve() != contract / "Cargo.toml":
            raise RuntimeError(f"expected one generated {package_name} from the archive consumer")
        direct = {dependency["pkg"] for dependency in nodes[found[0]["id"]]["deps"]}
        if runtime_id not in direct:
            raise RuntimeError(f"{package_name} did not resolve the shared archived runtime")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path,
                        help="release manifest produced by check_release_packages.py")
    parser.add_argument("--compiler", default="compactc", help="packaged compactc executable")
    args = parser.parse_args()
    release = json.loads(args.manifest.read_text())
    if release.get("schema") != 1 or release.get("source", {}).get("commit") != run(
        ["git", "rev-parse", "HEAD"]
    ).strip():
        raise RuntimeError("release manifest does not describe this source commit")
    entries = release.get("packages")
    if not isinstance(entries, list) or [entry.get("name") for entry in entries] != list(PACKAGE_NAMES):
        raise RuntimeError("release manifest must contain macro then runtime archives")
    if release["source"].get("tree") != run(["git", "rev-parse", "HEAD^{tree}"]).strip():
        raise RuntimeError("release manifest does not describe this source tree")
    compatibility = compatibility_record()
    if release["compatibility"] != compatibility or [
        entry["version"] for entry in entries
    ] != [compatibility["macros_version"], compatibility["runtime_version"]]:
        raise RuntimeError("release manifest does not describe this runtime compatibility")
    if release["locks"] != {path: sha256(ROOT / path) for path in LOCKS}:
        raise RuntimeError("release manifest does not describe the current dependency locks")

    (ROOT / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rust-archive-consumer-", dir=ROOT / "target") as scratch:
        temporary = Path(scratch)
        vendor = temporary / "vendor"
        run(["cargo", "vendor", "--offline", "--locked", "--versioned-dirs",
             "--manifest-path", str(ROOT / "runtime-rs/Cargo.toml"), str(vendor)])
        archives = {
            entry["name"]: install_archive(vendor, entry, compatibility["macros_version"])
            for entry in entries
        }
        source_hashes = archived_source_hashes(archives)

        contracts = {}
        for name, source in CONTRACTS:
            artifact = temporary / name
            run([args.compiler, "--target", "rust", "--rust-runtime-root", str(ROOT),
                 "--skip-zk", str(source), str(artifact)])
            contracts[name] = project_contract(
                artifact, temporary / "archive-contracts" / name, compatibility, source_hashes,
            )

        consumer = temporary / "consumer"
        (consumer / "tests").mkdir(parents=True)
        (consumer / ".cargo").mkdir()
        (consumer / "Cargo.toml").write_text(
            '[package]\nname = "compact-rust-archive-consumer"\nversion = "0.1.0"\n'
            'edition = "2024"\n\n[workspace]\n\n[dependencies]\n'
            + "".join(
                f'compact-contract-{name} = {{ path = {json.dumps(str(contract), ensure_ascii=False)}, '
                'features = ["ledger-transaction"] }\n'
                for name, contract in contracts.items()
            )
        )
        (consumer / ".cargo/config.toml").write_text(
            '[source.crates-io]\nreplace-with = "archive-vendor"\n'
            f'[source.archive-vendor]\ndirectory = {json.dumps(str(vendor), ensure_ascii=False)}\n'
        )
        (consumer / "tests/counter.rs").write_text(
            'use compact_contract_counter as counter;\n'
            'use compact_contract_cell_boolean as cell;\n'
            'use compact_contract_set_boolean as set;\n'
            'use compact_contract_map_boolean_field as map;\n'
            'use compact_contract_list_field as list;\n'
            'use compact_contract_merkle_path_witness as merkle;\n'
            'use compact_contract_counter::runtime::context::ConstructorContext;\n'
            'use compact_contract_counter::runtime::ledger::{ContractAddress, StateValue, read_cell};\n'
            '#[test]\nfn archived_runtime_executes_generated_counter() {\n'
            '    let state = counter::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    assert_eq!(counter::ledger_contract::PublicStateView::from(&state).round().unwrap().value(), 0);\n'
            '    let context = state.into_circuit_context(ContractAddress::default());\n'
            '    let call = counter::ledger_contract::Contract::default().increment(context).unwrap();\n'
            '    assert_eq!(counter::ledger_contract::PublicStateView::from(&call).round().unwrap().value(), 1);\n'
            '    let read = counter::ledger_contract::Contract::default().read_round(call.context).unwrap();\n'
            '    assert_eq!(read.result.value(), 1);\n}\n'
            '#[test]\nfn archived_runtime_executes_generated_cell() {\n'
            '    let address: counter::runtime::ledger::ContractAddress =\n'
            '        cell::runtime::ledger::ContractAddress::default();\n'
            '    let state = cell::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    let context = state.into_circuit_context(address);\n'
            '    let call = cell::ledger_contract::Contract::default().set_flag(context).unwrap();\n'
            '    assert!(cell::ledger_contract::PublicStateView::from(&call).flag().unwrap());\n'
            '    let StateValue::Array(fields) = call.context.query.state.get_ref() else {\n'
            '        panic!("expected generated Cell field array");\n'
            '    };\n'
            '    assert!(read_cell::<bool, _>(&fields.get(0).unwrap()).unwrap());\n}\n'
            '#[test]\nfn archived_runtime_executes_generated_set() {\n'
            '    let state = set::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    let initial = set::ledger_contract::PublicStateView::from(&state);\n'
            '    assert!(initial.seen().unwrap().is_empty());\n'
            '    let context = state.into_circuit_context(ContractAddress::default());\n'
            '    let call = set::ledger_contract::Contract::default().add(context, true).unwrap();\n'
            '    let view = set::ledger_contract::PublicStateView::from(&call);\n'
            '    assert!(view.seen().unwrap().member(true));\n'
            '    assert_eq!(view.seen().unwrap().size().unwrap().value(), 1);\n}\n'
            '#[test]\nfn archived_runtime_executes_generated_map() {\n'
            '    let state = map::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    let initial = map::ledger_contract::PublicStateView::from(&state);\n'
            '    assert!(initial.table().unwrap().is_empty());\n'
            '    let context = state.into_circuit_context(ContractAddress::default());\n'
            '    let call = map::ledger_contract::Contract::default().put('
            'context, true, map::runtime::Field::from(7_u64)).unwrap();\n'
            '    let view = map::ledger_contract::PublicStateView::from(&call);\n'
            '    assert_eq!(view.table().unwrap().lookup(true).unwrap(), '
            'map::runtime::Field::from(7_u64));\n}\n'
            '#[test]\nfn archived_runtime_executes_generated_list() {\n'
            '    let state = list::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    let initial = list::ledger_contract::PublicStateView::from(&state);\n'
            '    assert_eq!(initial.items().unwrap().head().unwrap(), None);\n'
            '    assert_eq!(initial.items().unwrap().length().unwrap().value(), 0);\n'
            '    let context = state.into_circuit_context(ContractAddress::default());\n'
            '    let call = list::ledger_contract::prepend(context, list::runtime::Field::from(7_u64)).unwrap();\n'
            '    let view = list::ledger_contract::PublicStateView::from(&call);\n'
            '    assert_eq!(view.items().unwrap().head().unwrap(), Some(list::runtime::Field::from(7_u64)));\n'
            '    assert_eq!(view.items().unwrap().length().unwrap().value(), 1);\n}\n'
            '#[test]\nfn archived_runtime_executes_generated_merkle_views() {\n'
            '    let state = merkle::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();\n'
            '    let initial = merkle::ledger_contract::PublicStateView::from(&state);\n'
            '    assert_eq!(initial.t().unwrap().first_free().unwrap().value(), 0);\n'
            '    assert_eq!(initial.h().unwrap().first_free().unwrap().value(), 0);\n'
            '    let context = state.into_circuit_context(ContractAddress::default());\n'
            '    let call = merkle::ledger_contract::append('
            'context, merkle::runtime::BoundedUint::<255>::new(7).unwrap()).unwrap();\n'
            '    let view = merkle::ledger_contract::PublicStateView::from(&call);\n'
            '    assert_eq!(view.t().unwrap().first_free().unwrap().value(), 1);\n'
            '    let next = merkle::ledger_contract::append_h('
            'call.context, merkle::runtime::BoundedUint::<255>::new(7).unwrap()).unwrap();\n'
            '    let historic = merkle::ledger_contract::PublicStateView::from(&next);\n'
            '    assert_eq!(historic.h().unwrap().first_free().unwrap().value(), 1);\n'
            '    assert_eq!(historic.h().unwrap().history().unwrap().len(), 2);\n}\n'
        )
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target/compactc-consumer"))
        environment.setdefault("CARGO_INCREMENTAL", "0")
        environment.setdefault("CARGO_BUILD_JOBS", "2")
        metadata = json.loads(run(["cargo", "metadata", "--offline", "--format-version", "1"],
                                  cwd=consumer, env=environment))
        check_graph(metadata, consumer, contracts, archives, entries)
        run(["cargo", "test", "--locked", "--offline", "--quiet"], cwd=consumer, env=environment)
        check_consumer_safety(consumer, environment)
        compile_rejections = {
            "wrong_set_key": (
                'use compact_contract_set_boolean as set;\n'
                'fn main() {\n'
                '    let state = set::ledger_contract::initial_state('
                'set::runtime::context::ConstructorContext::new(())).unwrap();\n'
                '    let view = set::ledger_contract::PublicStateView::from(&state);\n'
                '    view.seen().unwrap().member(set::runtime::Field::from(42_u64));\n}\n'
            ),
            "wrong_map_key": (
                'use compact_contract_map_boolean_field as map;\n'
                'fn main() {\n'
                '    let state = map::ledger_contract::initial_state('
                'map::runtime::context::ConstructorContext::new(())).unwrap();\n'
                '    let view = map::ledger_contract::PublicStateView::from(&state);\n'
                '    view.table().unwrap().member(map::runtime::Field::from(42_u64));\n}\n'
            ),
            "wrong_list_element": (
                'use compact_contract_list_field as list;\n'
                'fn main() {\n'
                '    let state = list::ledger_contract::initial_state('
                'list::runtime::context::ConstructorContext::new(())).unwrap();\n'
                '    let view = list::ledger_contract::PublicStateView::from(&state);\n'
                '    let _: Option<bool> = view.items().unwrap().head().unwrap();\n}\n'
            ),
            "wrong_merkle_leaf": (
                'use compact_contract_merkle_path_witness as merkle;\n'
                'fn main() {\n'
                '    let state = merkle::ledger_contract::initial_state('
                'merkle::runtime::context::ConstructorContext::new(())).unwrap();\n'
                '    let view = merkle::ledger_contract::PublicStateView::from(&state);\n'
                '    let _ = view.t().unwrap().path_for_leaf(0, true);\n}\n'
            ),
        }
        for name, source in compile_rejections.items():
            negative = consumer / f"tests/{name}.rs"
            negative.write_text(source)
            rejection = subprocess.run(
                ["cargo", "check", "--locked", "--offline", "--test", name, "--quiet"],
                cwd=consumer, env=environment, capture_output=True, text=True,
            )
            negative.unlink()
            if rejection.returncode == 0 or "mismatched types" not in rejection.stderr:
                raise RuntimeError(f"{name} was not rejected by the archive-only consumer: {rejection.stderr[-1200:]}")
        unavailable_methods = {
            "plain_merkle_history": "view.t().unwrap().history()",
            "local_merkle_check_root": "view.t().unwrap().check_root(merkle::runtime::Field::from(0_u64))",
            "local_historic_merkle_is_full": "view.h().unwrap().is_full()",
        }
        for name, expression in unavailable_methods.items():
            negative = consumer / f"tests/{name}.rs"
            negative.write_text(
                'use compact_contract_merkle_path_witness as merkle;\n'
                'fn main() {\n'
                '    let state = merkle::ledger_contract::initial_state('
                'merkle::runtime::context::ConstructorContext::new(())).unwrap();\n'
                '    let view = merkle::ledger_contract::PublicStateView::from(&state);\n'
                f'    let _ = {expression};\n}}\n'
            )
            rejection = subprocess.run(
                ["cargo", "check", "--locked", "--offline", "--test", name, "--quiet"],
                cwd=consumer, env=environment, capture_output=True, text=True,
            )
            negative.unlink()
            if rejection.returncode == 0 or "no method named" not in rejection.stderr:
                raise RuntimeError(f"{name} was not rejected by the archive-only consumer: {rejection.stderr[-1200:]}")
    print("archive-only Counter + Cell + Set + Map + List + Merkle consumer passed with one shared runtime; public registry publication remains unverified")


if __name__ == "__main__":
    main()
