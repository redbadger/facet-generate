// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::reflection::format::Format;

pub(crate) fn mangle_type(format: &Format) -> String {
    use Format::{
        Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Map, Option, QualifiedTypeName, Seq,
        Str, Tuple, TupleArray, TypeName, U8, U16, U32, U64, U128, Unit, Variable,
    };
    match format {
        TypeName(x) => x.to_string(),
        QualifiedTypeName(qualified_name) => qualified_name.to_legacy_string(),
        Unit => "unit".into(),
        Bool => "bool".into(),
        I8 => "i8".into(),
        I16 => "i16".into(),
        I32 => "i32".into(),
        I64 => "i64".into(),
        I128 => "i128".into(),
        U8 => "u8".into(),
        U16 => "u16".into(),
        U32 => "u32".into(),
        U64 => "u64".into(),
        U128 => "u128".into(),
        F32 => "f32".into(),
        F64 => "f64".into(),
        Char => "char".into(),
        Str => "str".into(),
        Bytes => "bytes".into(),

        Option(format) => format!("option_{}", mangle_type(format)),
        Seq(format) => format!("vector_{}", mangle_type(format)),
        Map { key, value } => format!("map_{}_to_{}", mangle_type(key), mangle_type(value)),
        Tuple(formats) => format!(
            "tuple{}_{}",
            formats.len(),
            formats
                .iter()
                .map(mangle_type)
                .collect::<Vec<_>>()
                .join("_")
        ),
        TupleArray { content, size } => format!("array{}_{}_array", size, mangle_type(content)),
        Variable(_) => panic!("unexpected value"),
    }
}
