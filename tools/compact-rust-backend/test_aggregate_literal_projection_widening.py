#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Admission controls for the aggregate projection qualification gate."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_aggregate_literal_projection_widening as gate
import check_functional_source_sdk as configuration_owner


class CargoAdmissionTests(unittest.TestCase):
    def test_lint_suppression_and_wrapper_environment_refuse(self):
        for key, value in [
                ("CARGO_BUILD_RUSTFLAGS", "--cap-lints=allow"),
                ("CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS", "--cap-lints=allow"),
                ("CARGO_BUILD_RUSTC_WRAPPER", "/bin/true"),
                ("CARGO_PROFILE_PROOF_CHECKS_DEBUG_ASSERTIONS", "false")]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.reject_compiler_overrides({key: value})

    def test_configuration_wrappers_refuse_and_membership_is_bound(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = root / ".cargo/config.toml"
            config.parent.mkdir()
            environment = {"CARGO_HOME": str(root / "cargo-home")}
            with patch.object(configuration_owner, "ROOT", root):
                before = gate.reject_compiler_overrides(environment)
                self.assertIsNone(before[str(config)])
                config.write_text('[build]\nrustc-wrapper="/bin/true"\n')
                with self.assertRaisesRegex(ValueError, "compiler/lint"):
                    gate.reject_compiler_overrides(environment)
                config.write_text('[net]\noffline=true\n')
                after = gate.reject_compiler_overrides(environment)
                self.assertNotEqual(after, before)
                self.assertEqual(after[str(config)], gate.digest(config))


if __name__ == "__main__":
    unittest.main()

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
