//! Exact Compact standard-library shapes for Ledger 9 unshielded kernel operands.
use crate::ir::{StructField, Type};

fn bytes32() -> Type {
    Type::Bytes { length: 32 }
}

fn address(name: &str) -> Type {
    Type::Struct {
        name: name.into(),
        fields: vec![StructField {
            name: "bytes".into(),
            ty: bytes32(),
        }],
    }
}

pub(super) fn token_type() -> Type {
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: bytes32(),
            },
            StructField {
                name: "right".into(),
                ty: bytes32(),
            },
        ],
    }
}

pub(super) fn recipient_type() -> Type {
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: address("ContractAddress"),
            },
            StructField {
                name: "right".into(),
                ty: address("UserAddress"),
            },
        ],
    }
}
