#!/usr/bin/env python3
"""Record the exact original unshielded-token Rust admission boundary.

The full pinned contract is intentionally compiled unchanged. The current
backend refuses its incUnshieldedInputs Kernel operation before publication;
this gate prevents a partial artifact from being mistaken for parity.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "test-center/compact/unshielded-tokens.compact"
ORIGINAL = (ROOT / "tools/compact-rust-backend/fixtures/"
            "unshielded-tokens-original-672bd0f/original-unshielded-tokens.ts")
SOURCE_SHA = "581c2485dad88e71550f612921735a9bbf0b9b00b04b5f6bb680e2651c73e556"
ORIGINAL_SHA = "6fc3737182878c3a31bcba5eae1dec70377097901791fc681adca294f576c913"
DIAGNOSTIC = ("Exception: <standard library>:\n"
              "  Rust backend does not yet support Kernel operation incUnshieldedInputs\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    output = args.output_dir.resolve()
    if output.exists():
        parser.error("output directory must be absent")
    if sha(SOURCE) != SOURCE_SHA or sha(ORIGINAL) != ORIGINAL_SHA:
        raise ValueError("pinned original source or assertions changed")
    output.mkdir(parents=True)
    command = [str(compiler), "--target", "rust", "--target", "ts",
               "--feature-zkir-v3", "--skip-zk", "--rust-runtime-root",
               str(ROOT), str(SOURCE), str(output / "generated")]
    process = subprocess.run(command, cwd=ROOT, capture_output=True)
    (output / "compile.stdout").write_bytes(process.stdout)
    (output / "compile.stderr").write_bytes(process.stderr)
    (output / "compile.exit").write_text(f"{process.returncode}\n")
    if (process.returncode != 255 or process.stdout or
            process.stderr.decode(errors="replace") != DIAGNOSTIC or
            (output / "generated").exists()):
        raise RuntimeError("original source Rust admission changed; inspect retained compile logs")
    result = {
        "status": "safe-refusal",
        "upstream": "672bd0f226f47a7d036c5c074ce71fbe1af1925e",
        "source_sha256": SOURCE_SHA,
        "original_assertions_sha256": ORIGINAL_SHA,
        "compiler": str(compiler),
        "compiler_sha256": sha(compiler),
        "command": command,
        "exit": process.returncode,
        "stdout_sha256": sha(output / "compile.stdout"),
        "stderr_sha256": sha(output / "compile.stderr"),
        "diagnostic": DIAGNOSTIC,
        "output_published": False,
    }
    (output / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
