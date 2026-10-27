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

//! Slot identity and element eligibility come from the immutable declaration registry.
use super::*;

pub(in super::super) fn element<'a>(
    declarations: &'a Declarations,
    slot: SetSlot,
    path: &str,
) -> Result<&'a Type, RecordingGap> {
    let field = declarations
        .field(slot.declaration)
        .ok_or_else(|| invalid(path, "unknown Set declaration"))?;
    if field.index != slot.index {
        return Err(invalid(path, "Set declaration index mismatch"));
    }
    let LedgerFieldKind::Set { ty } = &field.declaration else {
        return Err(invalid(path, "expected Set declaration"));
    };
    declarations.types().validate_at(ty, path)?;
    if !set_element_type(ty) {
        return Err(invalid(path, "unsupported Set element type"));
    }
    Ok(ty)
}
