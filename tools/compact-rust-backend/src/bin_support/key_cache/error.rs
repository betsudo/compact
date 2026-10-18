// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt, io};

#[derive(Debug)]
pub enum CacheError {
    Refused(String),
}
impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(message) => write!(f, "cache refused: {message}"),
        }
    }
}
impl std::error::Error for CacheError {}
impl From<io::Error> for CacheError {
    fn from(error: io::Error) -> Self {
        Self::Refused(error.to_string())
    }
}
impl From<serde_json::Error> for CacheError {
    fn from(error: serde_json::Error) -> Self {
        Self::Refused(format!("invalid cache JSON: {error}"))
    }
}
pub type Result<T> = std::result::Result<T, CacheError>;
pub fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(CacheError::Refused(message.into()))
    }
}
