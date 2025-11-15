// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::string::ToString;

use crate::reflection::format::Format;

pub(crate) fn mangle_type(format: &Format) -> String {
    match format {
        Format::TypeName(qualified_name) => qualified_name.format(ToString::to_string, "_"),
        Format::Unit => "unit".to_string(),
        Format::Bool => "bool".to_string(),
        Format::I8 => "i8".to_string(),
        Format::I16 => "i16".to_string(),
        Format::I32 => "i32".to_string(),
        Format::I64 => "i64".to_string(),
        Format::I128 => "i128".to_string(),
        Format::U8 => "u8".to_string(),
        Format::U16 => "u16".to_string(),
        Format::U32 => "u32".to_string(),
        Format::U64 => "u64".to_string(),
        Format::U128 => "u128".to_string(),
        Format::F32 => "f32".to_string(),
        Format::F64 => "f64".to_string(),
        Format::Char => "char".to_string(),
        Format::Str => "str".to_string(),
        Format::Bytes => "bytes".to_string(),

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
