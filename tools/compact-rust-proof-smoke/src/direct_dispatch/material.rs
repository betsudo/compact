// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Direct command argument parsing and execution for material proof scenarios.
//!
//! Each handler retains its original argument checks and worker policy.

use std::error::Error;
use std::ffi::{OsStr, OsString};

use crate::proof_material;

const USAGE: &str = "usage: compact-rust-proof-smoke --prepare-proof-material [--verify-only] [--parameter-k K ...]";

#[derive(Debug, PartialEq, Eq)]
struct Options {
    verify_only: bool,
    parameter_k: std::collections::BTreeSet<u8>,
}

fn parse(arguments: &mut impl Iterator<Item = OsString>) -> Result<Options, &'static str> {
    let mut options = Options {
        verify_only: false,
        parameter_k: std::collections::BTreeSet::new(),
    };
    while let Some(argument) = arguments.next() {
        if argument == OsStr::new("--verify-only") && !options.verify_only {
            options.verify_only = true;
        } else if argument == OsStr::new("--parameter-k") {
            let value = arguments.next().ok_or(USAGE)?;
            let value = value.to_str().ok_or(USAGE)?;
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(USAGE);
            }
            options
                .parameter_k
                .insert(value.parse::<u8>().map_err(|_| USAGE)?);
        } else {
            return Err(USAGE);
        }
    }
    Ok(options)
}

pub(super) fn prepare_proof_material(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn Error>> {
    let options = parse(arguments)?;
    proof_material::run(options.verify_only, options.parameter_k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_verify_only_and_requested_parameters_are_explicit() {
        assert_eq!(
            parse(&mut std::iter::empty()).unwrap(),
            Options {
                verify_only: false,
                parameter_k: Default::default(),
            }
        );
        let mut arguments = [
            "--parameter-k",
            "9",
            "--verify-only",
            "--parameter-k",
            "8",
            "--parameter-k",
            "9",
        ]
        .into_iter()
        .map(OsString::from);
        assert_eq!(
            parse(&mut arguments).unwrap(),
            Options {
                verify_only: true,
                parameter_k: [8, 9].into_iter().collect(),
            }
        );
    }

    #[test]
    fn malformed_parameters_stop_at_the_first_failure() {
        for supplied in [
            vec!["--parameter-k"],
            vec!["--parameter-k", "abc"],
            vec!["--parameter-k", "256"],
            vec!["--parameter-k", "-1"],
            vec!["--parameter-k", "+1"],
            vec!["--invalid"],
            vec!["--verify-only", "--verify-only"],
        ] {
            let mut arguments = supplied.into_iter().map(OsString::from);
            assert_eq!(parse(&mut arguments), Err(USAGE));
        }
        let mut arguments = ["--parameter-k", "bad", "unconsumed"]
            .into_iter()
            .map(OsString::from);
        assert_eq!(parse(&mut arguments), Err(USAGE));
        assert_eq!(arguments.next(), Some(OsString::from("unconsumed")));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_parameter_is_rejected_without_consuming_the_tail() {
        use std::os::unix::ffi::OsStringExt;
        let mut arguments = [
            OsString::from("--parameter-k"),
            OsString::from_vec(vec![0xff]),
            OsString::from("unconsumed"),
        ]
        .into_iter();
        assert_eq!(parse(&mut arguments), Err(USAGE));
        assert_eq!(arguments.next(), Some(OsString::from("unconsumed")));
    }
}
