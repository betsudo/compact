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

"""Shared finite functional proof lifecycle; keys cached, proofs always execute."""
import argparse
import os
import json
from pathlib import Path
import re
import resource
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

# Explicit roots also support isolated local qualification of this lifecycle.
BOOT = argparse.ArgumentParser(add_help=False)
BOOT.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
BOOT_ARGS, _ = BOOT.parse_known_args()
sys.path.insert(0, str(BOOT_ARGS.repo_root.resolve(strict=True) / "tools/compact-rust-backend"))
import forge_circuits as forge
from check_functional_proof import (ROOT, RUNNER, execution_environment, scheme_executable,
    snapshot, source_premises, workspace_packages)
from check_functional_effects import generated_manifest_snapshot
STAGED = Path(__file__).resolve().parent
PACKAGE = "compact-functional-core-proof-smoke"
DRIVER = Path(__file__)
EXTRA_PREMISES = []
SPECS = {}

def overlay(runner, generated, spec):
    raise RuntimeError("configure an explicit core/domain proof overlay")


STATIC_FIXTURE_FILES = frozenset({"dust/spend.zkir.sha256","dust/spend.prover.sha256",
    "dust/spend.verifier.sha256","dust/spend.bzkir","dust/spend.zkir",
    "dust/spend.bzkir.sha256","dust/spend.verifier"})

def fee_fixture_environment(environment):
    """Optional real Ledger test inputs, never fabricated fee/proof fallback."""
    raw=environment.get("MIDNIGHT_LEDGER_TEST_STATIC_DIR")
    if raw is None:
        return None
    forge.require(bool(raw),"empty explicit ledger static fixture directory refused")
    directory=Path(raw).resolve(strict=True)
    forge.require(directory.is_dir(),"ledger static fixture must be an existing directory")
    entries=list(directory.rglob("*"))
    forge.require(not any(path.is_symlink() for path in entries),"symlinked ledger static fixture members refused")
    files={str(path.relative_to(directory)):forge.digest(path) for path in entries if path.is_file()}
    forge.require(set(files)==STATIC_FIXTURE_FILES,"ledger static fixture membership differs from seven expected files")
    environment["MIDNIGHT_LEDGER_TEST_STATIC_DIR"]=str(directory)
    return {"path":str(directory),"sha256":files}


def case_main():
    if not __debug__:
        raise RuntimeError("qualification requires Python without -O/-OO")
    parser = argparse.ArgumentParser(description="Finite functional proof qualification", parents=[BOOT])
    parser.add_argument("--scenario", choices=sorted(SPECS), required=True)
    parser.add_argument("--compile-only", action="store_true")
    for flag in ("compiler", "params-dir", "target-dir", "forge", "uncached-zkir", "cache-dir", "report"):
        parser.add_argument("--" + flag, type=Path, required=True)
    args = parser.parse_args()
    spec = dict(SPECS[args.scenario], name=args.scenario)
    SOURCE = ROOT / spec["source"]
    source_inputs = spec.get("source_inputs", [(spec["source"], "primary")])
    for name in ("compiler", "params_dir", "forge", "uncached_zkir"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    args.cache_dir = args.cache_dir.resolve()
    args.report = args.report.resolve()
    forge.require(not args.report.exists(), "preserve existing scenario receipt; choose absent report")
    forge.require(not args.cache_dir.exists(), "use a new absent private Forge cache")
    args.target_dir = args.target_dir.resolve()
    environment = execution_environment()
    args.scheme = scheme_executable(args.compiler, environment)
    environment.update(CARGO_TARGET_DIR=str(args.target_dir), MIDNIGHT_PP=str(args.params_dir),
        COMPACT_ZKIR_REAL=str(args.uncached_zkir), COMPACTC_SCHEME=str(args.scheme),
        COMPACT_ZKIR_FORGE_BIN=str(args.forge), MIDNIGHT_ZKIR_FORGE_DIR=str(args.cache_dir))
    forge.validate_consumer()
    static_fixture=fee_fixture_environment(environment)
    def premises():
        forge.require(fee_fixture_environment(environment)==static_fixture,"ledger static fixture path/membership/bytes changed")
        values = source_premises(args)
        for path in [SOURCE, *(ROOT/source for source,_ in source_inputs), *EXTRA_PREMISES, *STAGED.glob("*.py"), *STAGED.glob("*.rs"), STAGED / "patch-premises.json",
                     ROOT / "tools/compact-rust-backend/check_functional_proof.py",
                     ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                     Path(__file__).resolve()]:
            values[str(path.resolve())] = forge.digest(path)
        return values
    initial = premises()
    runner_before = snapshot(RUNNER)
    canonical = args.target_dir / "proof-checks/compact-rust-proof-smoke"
    canonical_before = forge.digest(canonical) if canonical.exists() else None
    report = {"passed": False, "scope": "finite functional proof matrix; ordinary observed calls are comparisons only",
        "compiler_sha256": forge.digest(args.compiler), "premises_before": initial, "laps": [],
        "ledger_static_fixture_before":static_fixture,
        "cache_initially_absent": True, "profile": forge.PROFILE,
        "timing_limits": "fresh overlay build separate from proof case; no generalized speedup or peak-total-memory claim"}
    started = time.perf_counter()
    try:
        with tempfile.TemporaryDirectory(prefix="compact-functional-field-proof-") as temporary:
            temporary = Path(temporary)
            report["temporary_directory"] = str(temporary)
            for name, uncached in ((("compile-only", False),) if args.compile_only else (("forge-first", False), ("forge-repeat", False), ("uncached", True))):
                tick = time.perf_counter()
                lap = {"name": name, "commands": []}
                report["laps"].append(lap)
                def run(argv, phase, success=True):
                    before = time.perf_counter()
                    result = subprocess.run(list(map(str, argv)), cwd=ROOT, env=environment, capture_output=True, text=True)
                    lap["commands"].append({"phase": phase, "argv": list(map(str, argv)),
                        "exit_code": result.returncode, "wall_seconds": time.perf_counter() - before,
                        "stdout": result.stdout, "stderr": result.stderr})
                    forge.require((result.returncode == 0) == success, "unexpected command outcome: " + phase)
                    return result.stdout
                def stats():
                    return json.loads(run([args.forge, "--cache-dir", args.cache_dir, "stats"], "stats"))
                directory = temporary / name
                directory.mkdir()
                generated = directory / "generated"
                source_roots = {key:(generated if key=="primary" else directory/key) for _,key in source_inputs}
                for source_path,key in source_inputs:
                    run([args.compiler,"--target","rust","--rust-functional-ledger8","--skip-zk",
                        "--rust-runtime-root",ROOT,ROOT/source_path,source_roots[key]],"generate-"+key)
                lap["generated_manifest"] = {key:generated_manifest_snapshot(path) for key,path in source_roots.items()}
                before_sources = {key:snapshot(path) for key,path in source_roots.items()}
                artifacts_spec = spec.get("artifacts", [("primary",circuit) for circuit in spec["circuits"]])
                sources = [source_roots[key]/f"zkir/{circuit}.zkir" for key,circuit in artifacts_spec]
                lap["zkir_sha256"] = {key+"/"+circuit:forge.digest(path) for (key,circuit),path in zip(artifacts_spec,sources)}
                forge.require(all(old["zkir_sha256"] == lap["zkir_sha256"] for old in report["laps"][:-1]), "cross-lap current ZKIR differs")
                runner = directory / "tools/compact-rust-proof-smoke"
                runner.parent.mkdir()
                for relative, target in ((directory / "tests-rust-backend", ROOT / "tests-rust-backend"),
                    (directory / "runtime-rs", ROOT / "runtime-rs"),
                    (runner.parent / "compact-rust-backend", ROOT / "tools/compact-rust-backend")):
                    relative.symlink_to(target, target_is_directory=True)
                generated_name, private_allowed = overlay(runner, generated / "contract", spec)
                metadata = json.loads(run(["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", runner / "Cargo.toml"], "resolve"))
                root_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
                child_lock = tomllib.loads((runner / "Cargo.lock").read_text())["package"]
                identity = lambda row: (row["name"], row["version"], row.get("source"), row.get("checksum"))
                pinned = {identity(row) for row in root_lock}
                allowed = {str(path.resolve()) for path in workspace_packages().values()} | private_allowed
                for row in child_lock:
                    forge.require(row.get("source") is None or identity(row) in pinned, "external lock identity changed")
                for row in metadata["packages"]:
                    forge.require(row.get("source") is not None or str(Path(row["manifest_path"]).resolve()) in allowed, "unexpected local package origin")
                manifest = runner / "Cargo.toml"
                owners = {row["name"] for row in child_lock if row.get("source") is None and row["name"] in workspace_packages()} | ({generated_name} if isinstance(generated_name,str) else set(generated_name)) | set(spec.get("owner_aliases",[])) | ({spec["alias"]} if "alias" in spec else set())
                text = manifest.read_text()
                for owner in sorted(owners):
                    text += f'\n[profile.proof-checks.package.{json.dumps(owner)}]\nopt-level = 0\n'
                manifest.write_text(text)
                lap["owner_opt0_packages"] = sorted(owners)
                lap["private_manifest_sha256"] = forge.digest(manifest)
                lap["private_lock_sha256"] = forge.digest(runner / "Cargo.lock")
                # Intentional lock/profile changes are complete. Freeze BEFORE Cargo
                # can execute any build script; post-build bytes are not authority.
                overlay_before = snapshot(runner)
                private_sources = {path:forge.digest(Path(path).parent / "lib.rs") for path in private_allowed if Path(path).parent.name.endswith("-wrapper")}
                private_build = {path:forge.digest(Path(path)) for path in private_allowed}
                def unchanged_inputs():
                    forge.require(snapshot(runner)==overlay_before,"private overlay source/build inputs changed")
                    forge.require(all(forge.digest(Path(path).parent / "lib.rs")==value for path,value in private_sources.items()),"private frozen-authority wrapper changed")
                    forge.require(all(forge.digest(Path(path))==value for path,value in private_build.items()),"private dependency manifest changed")
                    forge.require({key:snapshot(path) for key,path in source_roots.items()}==before_sources,"generated source membership or bytes changed")
                    forge.require({key:generated_manifest_snapshot(path) for key,path in source_roots.items()}==lap["generated_manifest"],"generated manifest membership or bytes changed")
                    forge.require(premises()==initial,"live source/runtime/tool premises changed")
                lap["private_build_manifests"] = private_build
                lap["private_overlay_snapshot"] = overlay_before
                lap["private_wrapper_sources"] = private_sources
                unchanged_inputs()
                run(["cargo", "build", "--locked", "--offline", "--profile", "proof-checks", "-vv", "--manifest-path", manifest], "build")
                unchanged_inputs()
                lap["post_build_inputs_unchanged"] = True
                binary = args.target_dir / "proof-checks" / PACKAGE
                binary_hash = forge.digest(binary)
                if args.compile_only:
                    lap["proof_runner_sha256"] = binary_hash
                    lap["private_overlay_snapshot"] = overlay_before
                    lap["qualification"] = "compile-only; no key generation, proof or application executed"
                    lap["complete_wall_seconds"] = time.perf_counter()-tick
                    continue
                # Key generation intentionally adds artifacts. Keep the compiler's
                # strictly authenticated output immutable, and use a byte-identical
                # private execution copy with an exact addition allowlist.
                execution = directory / "execution-assets"
                execution.mkdir()
                execution_roots = {key:execution/key for key in source_roots}
                for key,path in source_roots.items():
                    shutil.copytree(path,execution_roots[key])
                    forge.require(snapshot(execution_roots[key])==before_sources[key],"execution copy differs from authenticated generation")
                    forge.require(generated_manifest_snapshot(execution_roots[key])==lap["generated_manifest"][key],"execution copy manifest differs")
                execution_before = snapshot(execution)
                additions = {root_key+"/"+path for root_key,circuit in artifacts_spec for path in
                    (f"zkir/{circuit}.bzkir",f"keys/{circuit}.prover",f"keys/{circuit}.verifier")}
                forge.require(not(set(execution_before)&additions),"proof artifact would replace authenticated source")
                sources = [execution_roots[key]/f"zkir/{circuit}.zkir" for key,circuit in artifacts_spec]
                case_start = time.perf_counter()
                if not uncached:
                    lap["stats_before"] = stats()
                lap["forge"] = []
                artifacts = []
                report.setdefault("parameters", {})
                for (root_key,circuit),source in zip(artifacts_spec,sources):
                    artifact_root=execution_roots[root_key]
                    prover,verifier=artifact_root/f"keys/{circuit}.prover",artifact_root/f"keys/{circuit}.verifier"
                    identity=root_key+"/"+circuit
                    if uncached:
                        prover.parent.mkdir(exist_ok=True)
                        run([args.uncached_zkir,"compile",source,prover,verifier],"uncached-keygen-"+circuit)
                    else:
                        prepared = forge.compile_circuit(source,prover,verifier,args.uncached_zkir,environment)
                        lap["forge"].append(prepared)
                        parameter = args.params_dir / f"bls_midnight_2p{prepared['k']}"
                        value = {"path":str(parameter),"sha256":prepared["parameter_digest"]}
                        forge.require(forge.digest(parameter)==value["sha256"],"current parameter mismatch")
                        if identity in report["parameters"]:
                            forge.require(report["parameters"][identity]==value,"cross-lap parameter changed")
                        report["parameters"][identity]=value
                    value=report["parameters"][identity]
                    forge.require(forge.digest(Path(value["path"]))==value["sha256"],"parameter changed")
                    artifacts.extend([source,source.with_suffix(".bzkir"),prover,verifier])
                lap["artifact_hashes"] = {str(p.relative_to(execution)):forge.digest(p) for p in artifacts}
                expected_execution = dict(execution_before)
                expected_execution.update({path:forge.digest(execution / path) for path in additions})
                forge.require(snapshot(execution)==expected_execution,"key generation changed files outside exact artifact allowlist")
                lap["execution_asset_snapshot"] = expected_execution
                proof_output = ""
                command_roots=spec.get("command_roots",["primary"])
                for command in spec["commands"]:
                    proof_output += run([binary,*command,*(execution_roots[key] for key in command_roots)],"proof-and-assertions")
                lap["actual_case_markers"] = {marker:proof_output.count(marker) for marker in spec["markers"]}
                forge.require(all(n==1 for n in lap["actual_case_markers"].values()),"exact proof case accounting changed")
                lap["actual_refusal_markers"]={marker:proof_output.count(marker) for marker in spec.get("refusal_markers",[])}
                forge.require(all(n==1 for n in lap["actual_refusal_markers"].values()),"original domain refusal control marker changed")
                for index,(root_key,circuit) in enumerate(spec.get("negative_artifacts",artifacts_spec)):
                    negative=directory/("corrupt-verifier-"+str(index))
                    shutil.copytree(execution,negative)
                    (negative/root_key/f"keys/{circuit}.verifier").write_bytes(b"invalid verifier")
                    command=spec["commands"][0] if len(spec["commands"])==1 else spec["commands"][index]
                    run([binary,*command,*(negative/key for key in command_roots)],"corrupt-verifier-"+root_key+"-"+circuit,success=False)
                    refusal=lap["commands"][-1]
                    forge.require(refusal["exit_code"]==1 and "expected header tag 'midnight:verifier-key[v6]:'" in refusal["stderr"],"corrupt verifier failed for unrelated reason")
                forge.require({str(p.relative_to(execution)):forge.digest(p) for p in artifacts}==lap["artifact_hashes"],"consumer changed artifacts")
                forge.require(snapshot(execution)==expected_execution,"consumer changed execution asset membership or bytes")
                forge.require(forge.digest(binary) == binary_hash, "proof executable changed")
                forge.require(forge.digest(manifest) == lap["private_manifest_sha256"] and forge.digest(runner / "Cargo.lock") == lap["private_lock_sha256"], "private build identity changed")
                unchanged_inputs()
                lap["post_proof_inputs_unchanged"] = True
                lap["private_build_manifests"] = private_build
                lap["private_overlay_snapshot"] = overlay_before
                lap["private_wrapper_sources"] = private_sources
                lap["proof_runner_sha256"] = binary_hash
                if not uncached:
                    lap["stats_after"] = stats()
                    before_totals = lap["stats_before"]["totals"]
                    after_totals = lap["stats_after"]["totals"]
                    expected_hit = name == "forge-repeat"
                    forge.require(all(row["cache"]["hit"] == expected_hit for row in lap["forge"]),
                                  "private first/repeat cache outcome changed")
                    forge.require(after_totals["hits"] - before_totals["hits"] == len(sources)*int(expected_hit)
                        and after_totals["misses"] - before_totals["misses"] == len(sources)*int(not expected_hit)
                        and after_totals["errors"] == before_totals["errors"],
                        "private cache accounting changed")
                lap["proof_case_wall_seconds"] = time.perf_counter() - case_start
                lap["complete_wall_seconds"] = time.perf_counter() - tick
            report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        try:
            report["premises_after"] = premises()
            report["premises_unchanged"] = report["premises_after"] == initial
        except Exception as error:
            report["premises_unchanged"] = False
            report["premises_error"] = str(error)
        try:
            report["ledger_static_fixture_after"]=fee_fixture_environment(environment)
            report["ledger_static_fixture_unchanged"]=report["ledger_static_fixture_after"]==static_fixture
        except Exception as error:
            report["ledger_static_fixture_unchanged"]=False
            report["ledger_static_fixture_error"]=str(error)
        report["parameter_unchanged"] = args.compile_only or bool(report.get("parameters")) and all(Path(row["path"]).is_file() and forge.digest(Path(row["path"]))==row["sha256"] for row in report["parameters"].values())
        report["canonical_runner_unchanged"] = snapshot(RUNNER) == runner_before
        report["canonical_binary_unchanged"] = (forge.digest(canonical) if canonical.exists() else None) == canonical_before
        report["temporary_cleanup"] = "temporary_directory" in report and not Path(report["temporary_directory"]).exists()
        report["passed"] = report["passed"] and all(report.get(k) for k in ("premises_unchanged", "parameter_unchanged", "canonical_runner_unchanged", "canonical_binary_unchanged", "temporary_cleanup", "ledger_static_fixture_unchanged"))
        report["complete_wall_seconds"] = time.perf_counter() - started
        report["largest_child_rss_bytes"] = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * (1 if sys.platform == "darwin" else 1024)
        report["memory_scope"] = "largest individual child process high-water; not concurrent aggregate"
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


def main():
    if "--scenario" in sys.argv:
        return case_main()
    if not __debug__:
        raise RuntimeError("qualification requires Python without -O/-OO")
    parser=argparse.ArgumentParser(description="Finite functional proof qualification",parents=[BOOT])
    parser.add_argument("--compile-only",action="store_true")
    for flag in ("compiler","params-dir","target-dir","forge","uncached-zkir","cache-dir","report"):
        parser.add_argument("--"+flag,type=Path,required=True)
    args=parser.parse_args()
    # Resolve in the invoking process before children change cwd to ROOT.
    for field in ("repo_root","compiler","params_dir","forge","uncached_zkir"):
        setattr(args,field,getattr(args,field).resolve(strict=True))
    for field in ("target_dir","cache_dir","report"):
        setattr(args,field,getattr(args,field).resolve())
    forge.require(not args.report.exists(),"preserve existing aggregate receipt; choose absent report")
    forge.require(not args.cache_dir.exists(),"qualification cache must be initially absent")
    args.report.parent.mkdir(parents=True,exist_ok=True)
    report={"passed":False,"scope":"finite cases on each cold/warm/uncached lap; proof verdicts never cached", "scenarios":[],"expected_cases_per_lap":sum(len(spec["markers"]) for spec in SPECS.values())}
    environment=execution_environment()
    args.scheme=scheme_executable(args.compiler.resolve(strict=True),environment)
    static_fixture=fee_fixture_environment(environment)
    report["ledger_static_fixture_before"]=static_fixture
    def full_premises():
        forge.require(fee_fixture_environment(environment)==static_fixture,"ledger static fixture path/membership/bytes changed")
        values=source_premises(args)
        paths=[ROOT/source for spec in SPECS.values() for source,_ in spec.get("source_inputs",[(spec["source"],"primary")])]+EXTRA_PREMISES+list(STAGED.glob("*.py"))+list(STAGED.glob("*.rs"))+[STAGED/"patch-premises.json",
            ROOT/"tools/compact-rust-backend/check_functional_effects.py",
            Path(__file__).resolve()]
        values.update({str(path.resolve()):forge.digest(path) for path in paths})
        return values
    report["premises_before"]=full_premises()
    report["qualification"]="compile-only" if args.compile_only else "real proof/verification/application cold/warm/uncached"
    start=time.perf_counter()
    try:
        for scenario,spec in SPECS.items():
            child_report=args.report.with_name(args.report.stem+"-"+scenario+".json")
            forge.require(not child_report.exists(),"preserve old receipts; choose absent report")
            command=[sys.executable,DRIVER,"--scenario",scenario,"--repo-root",args.repo_root]
            for key in ("compiler","params_dir","target_dir","forge","uncached_zkir"):
                command += ["--"+key.replace("_","-"),getattr(args,key)]
            command += ["--cache-dir",args.cache_dir/scenario,"--report",child_report]
            if args.compile_only:
                command += ["--compile-only"]
            result=subprocess.run(list(map(str,command)),cwd=ROOT,env=environment,capture_output=True,text=True)
            row={"scenario":scenario,"argv":list(map(str,command)),"exit_code":result.returncode,"stdout":result.stdout,"stderr":result.stderr}
            report["scenarios"].append(row)
            if child_report.exists():
                row["receipt_sha256"]=forge.digest(child_report)
                row["receipt"]=json.loads(child_report.read_text())
            forge.require(result.returncode==0 and row.get("receipt",{}).get("passed") is True,"core scenario failed: "+scenario)
        for index,name in enumerate(() if args.compile_only else ("forge-first","forge-repeat","uncached")):
            count=0
            for row in report["scenarios"]:
                lap=row["receipt"]["laps"][index]
                forge.require(lap["name"]==name,"lap identity changed")
                count+=sum(lap["actual_case_markers"].values())
            forge.require(count==report["expected_cases_per_lap"],"exact finite-case accounting changed")
        report["passed"]=True
    except Exception as error:
        report["error"]=f"{type(error).__name__}: {error}"
    finally:
        try:
            report["premises_after"]=full_premises()
            report["premises_unchanged"]=report["premises_before"]==report["premises_after"]
        except Exception as error:
            report["premises_unchanged"]=False
            report["premises_error"]=str(error)
        try:
            report["ledger_static_fixture_after"]=fee_fixture_environment(environment)
            report["ledger_static_fixture_unchanged"]=report["ledger_static_fixture_after"]==static_fixture
        except Exception as error:
            report["ledger_static_fixture_unchanged"]=False
            report["ledger_static_fixture_error"]=str(error)
        report["passed"]=report["passed"] and report["premises_unchanged"] and report["ledger_static_fixture_unchanged"]
        report["complete_wall_seconds"]=time.perf_counter()-start
        args.report.write_text(json.dumps(report,indent=2)+"\n")
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
