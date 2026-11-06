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

"""Proof-profile selection must preserve workload and failure propagation."""
import os
import io
from contextlib import redirect_stdout
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest
import tomllib
from unittest.mock import patch

import check_compactc_target as cli
import proof_commands

ROOT = Path(__file__).resolve().parents[2]


class ProofCommands(unittest.TestCase):
    def test_repository_proof_profile_preserves_checked_workspace_code(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        profile = manifest["profile"]["proof-checks"]
        self.assertEqual(profile["inherits"], "dev")
        self.assertIs(profile["debug-assertions"], True)
        self.assertIs(profile["overflow-checks"], True)
        # Dependency optimization must not become a workspace-wide release build
        # or acquire package-specific overrides that disable the inherited checks.
        self.assertNotIn("opt-level", profile)
        self.assertEqual(profile["package"], {"*": {"opt-level": 3}})

    def test_workflow_preparation_uses_the_same_checked_proof_profile(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        preparation = [shlex.split(line.strip()) for line in workflow.splitlines()
                       if line.lstrip().startswith("cargo run ")
                       and "--prepare-proof-material" in line]
        self.assertEqual(len(preparation), 1, "expected one direct proof-material preparation")
        command = preparation[0]
        boundary = command.index("--")
        options = command[:boundary]
        self.assertEqual(options[options.index("--profile") + 1], "proof-checks")
        self.assertEqual(options[options.index("-p") + 1], "compact-rust-proof-smoke")
        self.assertIn("--locked", options)
        self.assertIn("--offline", options)
        self.assertNotIn("--release", options)
        self.assertNotIn("-r", options)
        self.assertEqual(options.count("--profile"), 1)
        self.assertEqual(command[boundary + 1], "--prepare-proof-material")

    def test_capability_preflight_precedes_proofs_and_retains_later_live_gate(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        native = (ROOT / "tools/compact-rust-backend/check_native_ci.sh").read_text()
        command = ("env -u LD_LIBRARY_PATH nix develop .#compiler --command env "
                   "COMPACTC=compactc python3 -X faulthandler -u "
                   "tools/compact-rust-backend/check_rejections.py")
        self.assertEqual(workflow.count(command), 1)
        first = workflow.index(command)
        native_job = workflow.split('  build:\n', 1)[1].split('  e2e-tests:\n', 1)[0]
        self.assertLess(native_job.index("- name: Import authenticated same-run compiler closure"),
                        native_job.index(command))
        compiler_tests = workflow.split('  compiler-tests:\n', 1)[1].split('  build:\n', 1)[0]
        self.assertIn('- name: Run compiler tests', compiler_tests)
        self.assertIn('needs: compiler-linux', compiler_tests)
        self.assertLess(first, workflow.index("--prepare-proof-material"))
        live = command.replace("COMPACTC=compactc", 'COMPACTC="${COMPACTC:-compactc}"')
        self.assertEqual(native.count(live), 1)
        self.assertLess(native.index("--consumer --proof"), native.index(live))

    def test_workspace_clippy_runs_before_proofs_and_remains_in_the_final_gate(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        toolchain = workflow.index("      - name: Verify consumer toolchain inside Nix\n")
        early = workflow.index("      - name: Check workspace Clippy before proof qualification\n")
        prepare = workflow.index("      - name: Prepare verified built-in proof material\n")
        final = workflow.index("      - name: Check and test native Rust backend\n")
        self.assertLess(toolchain, early)
        self.assertLess(early, prepare)
        self.assertLess(prepare, final)
        expected = "cargo clippy --workspace --exclude compact --all-targets --all-features --locked -- -D warnings"
        commands = [line.strip() for line in workflow.splitlines()
                    if line.lstrip().startswith("cargo clippy --workspace ")]
        self.assertEqual(commands, [expected])
        early_step = workflow[early:prepare]
        self.assertEqual(early_step.count(expected), 1)
        self.assertNotIn("continue-on-error", early_step)
        self.assertNotIn("MIDNIGHT_PP", early_step)
        final_step = workflow[final:workflow.index("      - name: Rehearse Rust runtime packages\n", final)]
        invocation = "run: bash tools/compact-rust-backend/check_native_ci.sh"
        self.assertEqual(final_step.count(invocation), 1)
        self.assertEqual(workflow.count(invocation), 1)
        self.assertNotIn("continue-on-error", final_step)
        native = (ROOT / "tools/compact-rust-backend/check_native_ci.sh").read_text()
        self.assertIn("set -e -o pipefail", native)
        self.assertEqual(native.count(expected), 1)
        self.assertIn("check_compactc_target.py --consumer --proof", native)
        self.assertIn("cargo test --workspace --exclude compact --locked", native)

    def test_passport_failure_receipts_do_not_wait_for_runtime_package_success(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        gate = workflow.index("      - name: Check standalone original Passport pure consumers\n")
        retain = workflow.index("      - name: Retain failed Passport qualification diagnostics\n")
        prepare = workflow.index("      - name: Prepare verified built-in proof material\n")
        self.assertLess(gate, retain)
        self.assertLess(retain, prepare)
        qualification = workflow[gate:retain]
        failure_upload = workflow[retain:prepare]
        self.assertIn("id: passport_current", qualification)
        self.assertNotIn("continue-on-error", qualification)
        self.assertIn("if: failure() && steps.passport_current.outcome == 'failure'", failure_upload)
        self.assertNotIn("runtime-packages", failure_upload)
        self.assertIn("compact-passport-current/", failure_upload)
        self.assertIn("!${{ runner.temp }}/compact-passport-current/cargo-target/**", failure_upload)
        self.assertIn("if-no-files-found: ignore", failure_upload)
        self.assertIn("passport-current-failure-${{ github.sha }}-attempt-${{ github.run_attempt }}", failure_upload)
        # Success still retains the original required package artifact/proof path.
        self.assertIn("if: always() && steps.runtime-packages.outcome == 'success'", workflow)
        self.assertIn("compact-passport-current/formatted-generated-lib.rs", workflow)

    def test_did_zero_proof_keeps_default_valid_and_uses_authenticated_compiler(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        valid = workflow.index("      - name: Qualify selected original DID functional proof\n")
        zero = workflow.index("      - name: Qualify original DID zero timestamp proof\n")
        retain = workflow.index("      - name: Retain failed DID zero timestamp proof diagnostics\n")
        native_job = workflow.split('  build:\n', 1)[1].split('  e2e-tests:\n', 1)[0]
        self.assertLess(native_job.index("- name: Import authenticated same-run compiler closure"),
                        native_job.index("- name: Qualify original DID zero timestamp proof"))
        self.assertLess(workflow.index("- name: Prepare verified built-in proof material"), valid)
        self.assertLess(valid, zero)
        self.assertLess(zero, retain)
        valid_step = workflow[valid:zero]
        self.assertIn('--output "$RUNNER_TEMP/compact-did-functional-proof"', valid_step)
        self.assertNotIn("--case", valid_step)  # Historical valid default is retained.
        zero_step = workflow[zero:retain]
        self.assertIn("id: did_zero_timestamp_proof", zero_step)
        self.assertNotIn("continue-on-error", zero_step)
        self.assertNotIn("if:", zero_step)
        command = shlex.split(zero_step.split("run: |\n", 1)[1].strip())
        self.assertEqual(command[:7], ["env", "-u", "LD_LIBRARY_PATH", "nix", "develop", ".#compiler", "--command"])
        self.assertIn("tools/compact-rust-backend/check_did_functional_lifecycle_proof.py", command)
        self.assertEqual(command[command.index("--compiler") + 1], "$COMPACTC")
        self.assertEqual(command[command.index("--case") + 1], "did/timestamp-zero")
        self.assertEqual(command[command.index("--output") + 1], "$RUNNER_TEMP/compact-did-zero-timestamp-proof")

    def test_did_zero_proof_retains_failure_without_waiting_for_packages(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        start = workflow.index("      - name: Retain failed DID zero timestamp proof diagnostics\n")
        end = workflow.index("      - name: Qualify selected Unit mutation initializer proofs\n", start)
        failure = workflow[start:end]
        self.assertIn("if: failure() && steps.did_zero_timestamp_proof.outcome == 'failure'", failure)
        self.assertNotIn("runtime-packages", failure)
        self.assertIn("did-zero-timestamp-proof-failure-${{ github.sha }}-attempt-${{ github.run_attempt }}", failure)
        self.assertIn("if-no-files-found: ignore", failure)
        success = workflow[workflow.index("      - name: Retain Rust runtime packages and manifest\n"):]
        for suffix in ["receipt.json", "logs/", "*.stdout", "*.stderr", "*material*.json", "generated/"]:
            path = "${{ runner.temp }}/compact-did-zero-timestamp-proof/" + suffix
            self.assertIn(path, failure)
            self.assertIn(path, success)
        self.assertIn("if: always() && steps.runtime-packages.outcome == 'success'", success)
        self.assertIn("compact-did-functional-proof/generated/", success)

    def test_did_zero_proof_shell_forwards_case_and_propagates_failure(self):
        workflow = (ROOT / ".github/workflows/build-compiler.yml").read_text()
        step = workflow.split("      - name: Qualify original DID zero timestamp proof\n", 1)[1]
        script = step.split("      - name:", 1)[0].split("run: |\n", 1)[1]
        with tempfile.TemporaryDirectory(prefix="did proof boundary ") as directory:
            root = Path(directory)
            nix = root / "nix"
            nix.write_text('#!/bin/sh\ntest -z "${LD_LIBRARY_PATH+x}" || exit 48\n'
                           'printf "%s\\0" "$@" > "$TRACE"\nexit 47\n')
            nix.chmod(0o755)
            trace = root / "trace"
            compiler = root / "compiler with spaces/bin/compactc"
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"],
                       TRACE=str(trace), COMPACTC=str(compiler), RUNNER_TEMP=str(root),
                       LD_LIBRARY_PATH="must be removed")
            result = subprocess.run(["bash", "-e", "-o", "pipefail", "-c", script],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 47, result.stderr)
            argv = trace.read_text().rstrip("\0").split("\0")
            self.assertEqual(argv[argv.index("--case") + 1], "did/timestamp-zero")
            self.assertEqual(argv[argv.index("--compiler") + 1], str(compiler))
            self.assertEqual(argv[argv.index("--output") + 1], str(root / "compact-did-zero-timestamp-proof"))

    def test_native_script_stops_after_a_failed_proof_command(self):
        # Exercise the actual shell boundary without generating or caching proofs.
        with tempfile.TemporaryDirectory(prefix="native failure ") as temporary:
            root = Path(temporary)
            trace = root / "commands"
            for name in ("df", "du", "cargo", "nix"):
                tool = root / name
                status = 47 if name == "nix" else 0
                tool.write_text('#!/bin/sh\nprintf "%s\\0" "' + name + '" "$@" >> "$TRACE"\n'
                                + 'printf "\\n" >> "$TRACE"\n'
                                + f"exit {status}\n")
                tool.chmod(0o755)
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"],
                       TRACE=str(trace), GITHUB_WORKSPACE=str(root),
                       CARGO_TARGET_DIR=str(root), RUNNER_TEMP=str(root))
            env.pop("COMPACTC", None)
            result = subprocess.run(["bash", ROOT / "tools/compact-rust-backend/check_native_ci.sh"],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 47, result.stderr)
            commands = [line.rstrip("\0").split("\0") for line in trace.read_text().splitlines()]
            self.assertEqual(len(commands), 4, commands)
            self.assertEqual(commands[2], ["cargo", "fmt", "--all", "--", "--check"])
            self.assertEqual(commands[3][:5], ["nix", "develop", ".#compiler", "--command", "env"])
            self.assertIn("COMPACTC=compactc", commands[3])
            self.assertIn("COMPACT_RUST_WALLET_HANDOFF=" + str(root / "compact-counter-wallet-handoff.bin"),
                          commands[3])
            self.assertIn("COMPACT_RUST_DEPLOY_HANDOFF=" + str(root / "compact-counter-deploy-handoff.bin"),
                          commands[3])
            self.assertEqual(commands[3][-3:], ["tools/compact-rust-backend/check_compactc_target.py",
                                               "--consumer", "--proof"])

    def test_existing_run_options_and_all_executable_arguments_survive(self):
        for prefix in (["cargo", "run"], ["cargo", "+1.99.0", "run"]):
            for package in ("-p", "--package"):
                original = [*prefix, "--quiet", "--locked", "--offline", package,
                            "compact-rust-proof-smoke", "--", "--coracle-start",
                            "/private/output with spaces", "--profile", "consumer-argument"]
                boundary = original.index("--")
                self.assertEqual(proof_commands.command(original, {}),
                    original[:boundary] + ["--profile", "proof-checks"] + original[boundary:])

    def test_nonproof_work_and_package_like_executable_arguments_are_untouched(self):
        for original in (
            ["cargo", "test", "-p", "compact-rust-proof-smoke"],
            ["cargo", "check", "--manifest-path", "consumer/Cargo.toml"],
            ["cargo", "run", "-p", "compact-rust-backend", "--", "-p", "compact-rust-proof-smoke"],
            ["zkir", "compile", "current.zkir", "current.prover", "current.verifier"],
        ):
            self.assertEqual(proof_commands.command(original,
                {"COMPACT_RUST_PROOF_PROFILE": "invalid"}), original)

    def test_dev_baseline_and_preparation_without_executable_separator(self):
        original = ["cargo", "run", "-p", "compact-rust-proof-smoke"]
        self.assertEqual(proof_commands.command(original, {"COMPACT_RUST_PROOF_PROFILE": "dev"}),
                         original + ["--profile", "dev"])
        preparation = original + ["--", "--prepare-proof-material", "--verify-only"]
        self.assertEqual(proof_commands.command(preparation, {}),
                         original + ["--profile", "proof-checks", *preparation[len(original):]])

    def test_unknown_or_conflicting_profiles_refuse_instead_of_dropping_checks(self):
        original = ["cargo", "run", "-p", "compact-rust-proof-smoke"]
        for profile in ("", "release", "unknown"):
            with self.subTest(profile=profile), self.assertRaises(ValueError):
                proof_commands.command(original, {"COMPACT_RUST_PROOF_PROFILE": profile})
        for option in (["--release"], ["-r"], ["--profile", "dev"], ["--profile=proof-checks"]):
            with self.subTest(option=option), self.assertRaises(ValueError):
                proof_commands.command(original + option, {})

    def test_runner_retains_cwd_environment_and_original_subprocess_failure(self):
        original = ("cargo", "run", "-p", "compact-rust-proof-smoke", "--", "--coracle-start", "output")
        environment = {"COMPACT_RUST_PROOF_PROFILE": "dev", "UNRELATED_VARIABLE": "retained"}
        failure = subprocess.CalledProcessError(23, original)
        with patch.object(cli.subprocess, "run", side_effect=failure) as run:
            with self.assertRaises(subprocess.CalledProcessError) as raised:
                cli.run(*original, cwd=Path("fixture"), env=environment)
        self.assertIs(raised.exception, failure)
        run.assert_called_once_with(proof_commands.command(original, environment),
                                    cwd=Path("fixture"), env=environment, check=True)

    def test_runner_uses_process_environment_when_no_override_is_passed(self):
        original = ("cargo", "run", "-p", "compact-rust-proof-smoke", "--", "--effect-plan-jubjub", "output")
        with patch.dict(os.environ, {"COMPACT_RUST_PROOF_PROFILE": "dev"}), \
                patch.object(cli.subprocess, "run") as run:
            cli.run(*original)
        self.assertEqual(run.call_args.args[0][-5:],
                         ["--profile", "dev", "--", "--effect-plan-jubjub", "output"])
        self.assertIsNone(run.call_args.kwargs["env"])

    def test_scenario_timing_reports_failure_without_hiding_it(self):
        command = (sys.executable, str(ROOT / "tools/compact-rust-backend/check_compactc_target.py"),
                   "--coracle-start", "--proof")
        failure = subprocess.CalledProcessError(23, command)
        output = io.StringIO()
        with patch.dict(os.environ, {"COMPACT_CI_TIMINGS": "1"}), \
                patch.object(cli.subprocess, "run", side_effect=failure), \
                redirect_stdout(output):
            with self.assertRaises(subprocess.CalledProcessError) as raised:
                cli.run(*command)
        self.assertIs(raised.exception, failure)
        self.assertIn("CI scenario start: --coracle-start", output.getvalue())
        self.assertIn("CI scenario failed: --coracle-start; elapsed=", output.getvalue())


if __name__ == "__main__":
    unittest.main()
