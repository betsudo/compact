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

//! Reject ambiguous duplicate object fields before interpreting provider JSON.
use super::error::{Result, require};
use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeSet, fmt, fs, io::Read, path::Path};

struct Strict;
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = Strict;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("unambiguous JSON")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Strict, A::Error> {
                let mut keys = BTreeSet::new();
                while let Some(key) = a.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(de::Error::custom("duplicate JSON field"));
                    }
                    a.next_value::<Strict>()?;
                }
                Ok(Strict)
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Strict, A::Error> {
                while a.next_element::<Strict>()?.is_some() {}
                Ok(Strict)
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
            fn visit_str<E: de::Error>(self, _: &str) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Strict, E> {
                Ok(Strict)
            }
        }
        d.deserialize_any(Fields)
    }
}
pub fn parse(bytes: &[u8]) -> Result<Value> {
    let mut reader = serde_json::Deserializer::from_slice(bytes);
    Strict::deserialize(&mut reader)?;
    reader.end()?;
    Ok(serde_json::from_slice(bytes)?)
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    require(
        metadata.is_file(),
        "regular input file required; symlinks refused",
    )?;
    require(metadata.len() <= limit, "input exceeds artifact size bound")?;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    require(
        bytes.len() as u64 <= limit,
        "input grew beyond artifact size bound",
    )?;
    Ok(bytes)
}
pub fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| super::error::CacheError::Refused(format!("missing string field {key}")))
}
pub fn number(value: &Value, key: &str) -> Result<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| super::error::CacheError::Refused(format!("missing integer field {key}")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn duplicate_nested_identity_is_refused() {
        assert!(super::parse(br#"{"cache":{"hit":true,"hit":false}}"#).is_err());
        assert!(super::parse(br#"{"cache":{"hit":true},"items":[null,1.5]}"#).is_ok());
    }
}
