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
"""Fail closed on the 13 source-bound native Linux Vector gate receipts."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/compact-rust-backend"))
from check_upstream_ledger9_vector_casts import runtime_identity, runtime_recipe_identity  # noqa: E402

PINNED = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
CASES = {
    "bytes_to_vector": 11,
    "vector_casts": 24,
    "vector_comparison": 6,
    "vector_default": 3,
    "vector_destructuring": 3,
    "vector_fold": 10,
    "vector_implicit_casts": 6,
    "vector_indexing": 12,
    "vector_length_refusal": 2,
    "vector_loops": 9,
    "vector_slice": 30,
    "vector_spread": 23,
    "vector_witness_return": 3,
}
REFUSALS = {
    "bytes_to_vector": "original_refusal_on_both_targets",
    "vector_casts": "original_refusals_on_both_targets",
    "vector_fold": "refusals",
    "vector_indexing": "refusals",
    "vector_length_refusal": "refusals",
    "vector_slice": "original_refusals_on_both_targets",
    "vector_spread": "scalar_type_refused_on_both_targets",
}
GENERATED = {
    "bytes_to_vector": ("to_vector", "to_vector_default", "to_vector_wide_element"),
    "vector_casts": ("empty", "to_bytes", "to_bytes_default",
                     "to_bytes_narrow_element", "to_tuple", "to_vector"),
    "vector_comparison": ("equality", "inequality"),
    "vector_default": (),
    "vector_destructuring": (),
    "vector_fold": ("empty_tuple", "semantics"),
    "vector_implicit_casts": ("to_tuple", "widening"),
    "vector_indexing": ("basic", "nested"),
    "vector_loops": ("empty_iteration", "iteration", "spread_iteration"),
    "vector_slice": ("computed_index", "default", "empty", "from_parameter",
                     "mixed_types", "nested", "radix_bounds", "semantics"),
    "vector_spread": ("concatenation", "default", "empty", "mixed_types",
                      "nested", "with_literals", "with_slice"),
    "vector_witness_return": (),
}
ARTIFACTS = {
    "typescript_sha256": "contract/index.js",
    "rust_sha256": "contract/lib.rs",
    "rust_ir_sha256": "contract/compact-rust-ir.json",
    "contract_info_sha256": "compiler/contract-info.json",
    "manifest_sha256": "compiler/contract-manifest.json",
}


def require(value: bool, message: str) -> None:
    if not value:
        raise ValueError(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_receipt(path: Path) -> dict:
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f"missing, symlinked or oversized receipt: {path}")
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"non-object receipt: {path}")
    return value


def digest_fields(value: object) -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            if key.endswith("_sha256"):
                require(isinstance(item, str) and re.fullmatch(r"[0-9a-f]{64}", item) is not None,
                        f"invalid artifact digest: {key}")
            digest_fields(item)
    elif isinstance(value, list):
        for item in value:
            digest_fields(item)


def validate_generated(case: str, receipt: dict, output: Path) -> None:
    if case == "vector_length_refusal":
        require("generated" not in receipt, "refusal unexpectedly claims generated artifacts")
        return
    generated = receipt.get("generated")
    require(isinstance(generated, dict), f"{case}: generated inventory missing")
    names = GENERATED[case]
    require(set(generated) == (set(names) if names else {"typescript", "both"}),
            f"{case}: generated fixture inventory differs")
    entries = generated.items() if names else (("", generated),)
    for name, modes in entries:
        require(isinstance(modes, dict) and set(modes) == {"typescript", "both"},
                f"{case}/{name}: generated target modes differ")
        for mode, row in modes.items():
            require(isinstance(row, dict), f"{case}/{name}/{mode}: no artifacts")
            destination = output / "generated" / mode
            if name:
                destination /= name
            required = {"typescript_sha256"}
            if mode == "both":
                required.update(("rust_sha256", "rust_ir_sha256"))
            require(required <= row.keys(), f"{case}/{name}/{mode}: target artifact missing")
            for key, relative in ARTIFACTS.items():
                if key in row:
                    path = destination / relative
                    require(path.is_file() and not path.is_symlink() and
                            row[key] == sha256(path),
                            f"{case}/{name}/{mode}: {relative} differs")
            if "zkir_sha256" in row:
                zkir = list((destination / "zkir").glob("*.zkir"))
                require(len(zkir) == 1 and zkir[0].is_file() and
                        row["zkir_sha256"] == sha256(zkir[0]),
                        f"{case}/{name}/{mode}: raw ZKIR differs")
    if case == "vector_slice":
        local = receipt.get("local_field_to_uint")
        require(isinstance(local, dict) and set(local) == {"typescript", "both"},
                "local Field-to-Uint slice evidence missing")
        for mode, row in local.items():
            destination = output / "generated" / mode / "field_to_uint"
            for key, relative in ARTIFACTS.items():
                if key in row:
                    path = destination / relative
                    require(path.is_file() and row[key] == sha256(path),
                            "local Field-to-Uint slice artifact differs")


def validate(case: str, path: Path, *, source_sha: str, tree: str,
             compiler: Path, recipe_path: Path, runtime: Path,
             runtime_snapshot: dict, runtime_recipe: dict) -> dict:
    require(case in CASES and path.parent.name == case, "unknown or misplaced Vector gate")
    receipt = load_receipt(path)
    require(receipt.get("status", "passed") == "passed", f"{case}: gate status failed")
    require(receipt.get("head") == source_sha and receipt.get("tree") == tree
            and receipt.get("source_clean") is True, f"{case}: source identity differs")
    require(receipt.get("upstream_commit") == PINNED, f"{case}: pinned upstream differs")
    require(receipt.get("platform") == "Linux x86_64", f"{case}: not native x86_64 Linux")
    originals = receipt.get("original_files")
    count = len(originals) if isinstance(originals, dict) else originals
    require(count == CASES[case], f"{case}: original inventory count differs")
    scheme = compiler.with_name("compactc-scheme")
    require(receipt.get("compiler_sha256") == sha256(compiler) and
            receipt.get("scheme_sha256") == sha256(scheme),
            f"{case}: compiler artifact differs")
    recipe_receipt = load_receipt(recipe_path)
    require(recipe_receipt.get("status") == "passed" and
            recipe_receipt.get("head") == source_sha and recipe_receipt.get("tree") == tree and
            receipt.get("compiler_recipe") == {
                "path": str(recipe_path), "sha256": sha256(recipe_path),
                "head": source_sha, "tree": tree,
                "store": str(compiler.parent.parent),
            }, f"{case}: compiler recipe differs")
    if case != "vector_length_refusal":
        recorded_runtime_recipe = receipt.get("runtime_recipe", {})
        require(receipt.get("runtime") == runtime_snapshot and
                isinstance(recorded_runtime_recipe, dict) and
                recorded_runtime_recipe.get("store") == runtime_recipe["store"] and
                recorded_runtime_recipe.get("path") == runtime_recipe["path"] and
                recorded_runtime_recipe.get("command", [])[1:] ==
                    runtime_recipe["command"][1:],
                f"{case}: exact-head TypeScript runtime differs")
    validate_generated(case, receipt, path.parent)
    if case in REFUSALS:
        refusal = receipt.get(REFUSALS[case])
        require(bool(refusal), f"{case}: original refusal evidence missing")
    if "cargo_target" in receipt:
        expected = os.environ.get("VECTOR_CARGO_TARGET")
        require(expected and receipt["cargo_target"] == expected,
                f"{case}: shared Cargo target differs")
    digest_fields(receipt)
    return {"case": case, "head": source_sha, "tree": tree,
            "receipt_sha256": sha256(path), "original_files": count,
            "status": "passed"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--case", choices=CASES)
    selection.add_argument("--all", type=Path)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--compiler-recipe", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()
    require(re.fullmatch(r"[0-9a-f]{40}", args.source_sha) is not None,
            "invalid source SHA")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip()
    require(head == args.source_sha and not subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT), "source changed during Vector qualification")
    compiler = args.compiler.resolve(strict=True)
    recipe_path = args.compiler_recipe.resolve(strict=True)
    runtime = args.runtime.resolve(strict=True)
    nix = shutil.which("nix")
    require(nix is not None, "Nix required for exact-head runtime receipt")
    runtime_snapshot = runtime_identity(runtime)
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    options = dict(source_sha=head, tree=tree, compiler=compiler, recipe_path=recipe_path,
                   runtime=runtime, runtime_snapshot=runtime_snapshot,
                   runtime_recipe=runtime_recipe)
    if args.case:
        require(args.receipt is not None and args.summary is None, "single gate needs receipt")
        print(json.dumps(validate(args.case, args.receipt, **options), sort_keys=True))
    else:
        require(args.receipt is None and args.summary is not None, "group needs summary path")
        directory = args.all.resolve(strict=True)
        require(not args.summary.exists() and args.summary.parent == directory,
                "summary must be new in the evidence directory")
        rows = [validate(case, directory / case / "receipt.json", **options)
                for case in CASES]
        summary = {"status": "passed", "head": head, "tree": tree,
                   "upstream_commit": PINNED, "platform": "Linux x86_64",
                   "original_files": sum(row["original_files"] for row in rows),
                   "cases": rows, "compiler_recipe_sha256": sha256(recipe_path),
                   "runtime_package_sha256": sha256(runtime / "package.json")}
        require(summary["original_files"] == 142 and len(rows) == 13,
                "Vector family inventory incomplete")
        args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        print(json.dumps({"status": "passed", "head": head, "cases": len(rows)}))


if __name__ == "__main__":
    main()
