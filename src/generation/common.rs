// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::string::ToString;

use crate::reflection::format::Format;

pub(crate) fn mangle_type(format: &Format) -> String {
    match format {
        Format::TypeName(qualified_name) => qualified_name.to_legacy_string(ToString::to_string),
        Format::Unit => "unit".into(),
        Format::Bool => "bool".into(),
        Format::I8 => "i8".into(),
        Format::I16 => "i16".into(),
        Format::I32 => "i32".into(),
        Format::I64 => "i64".into(),
        Format::I128 => "i128".into(),
        Format::U8 => "u8".into(),
        Format::U16 => "u16".into(),
        Format::U32 => "u32".into(),
        Format::U64 => "u64".into(),
        Format::U128 => "u128".into(),
        Format::F32 => "f32".into(),
        Format::F64 => "f64".into(),
        Format::Char => "char".into(),
        Format::Str => "str".into(),
        Format::Bytes => "bytes".into(),

        Format::Option(format) => format!("option_{}", mangle_type(format)),
        Format::Seq(format) => format!("vector_{}", mangle_type(format)),
        Format::Set(format) => format!("set_{}", mangle_type(format)),
        Format::Map { key, value } => format!("map_{}_to_{}", mangle_type(key), mangle_type(value)),
        Format::Tuple(formats) => format!(
            "tuple{}_{}",
            formats.len(),
            formats
                .iter()
                .map(mangle_type)
                .collect::<Vec<_>>()
                .join("_")
        ),
        Format::TupleArray { content, size } => {
            format!("array{}_{}_array", size, mangle_type(content))
        }
        Format::Variable(_) => panic!("unexpected value"),
    }
}
