use std::collections::{BTreeMap, HashSet};

use facet::{
    ArrayDef, Def, EnumType, Facet, Field, FieldAttribute, IntegerSize, ListDef, MapDef,
    NumberBits, NumericType, OptionDef, PointerType, PrimitiveType, ScalarAffinity, ScalarDef,
    SequenceType, Shape, ShapeAttribute, Signedness, SliceDef, SmartPointerDef, StructKind,
    StructType, TextualType, Type, UserType, VariantAttribute,
};
use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, VariantFormat};

#[derive(Debug)]
pub struct Registry {
    containers: BTreeMap<String, ContainerFormat>,
    current: Vec<String>,
    processed: HashSet<String>,
    name_mappings: BTreeMap<String, String>,
}

impl Registry {
    fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
            current: Vec::new(),
            processed: HashSet::new(),
            name_mappings: BTreeMap::new(),
        }
    }

    fn push(&mut self, name: String, container: ContainerFormat) {
        self.containers.insert(name.clone(), container);
        self.current.push(name);
    }

    fn register_type_mapping(&mut self, original: String, renamed: String) {
        self.name_mappings.insert(original, renamed);
    }

    fn get_mapped_name(&self, name: &str) -> Option<String> {
        self.name_mappings.get(name).cloned()
    }

    fn is_processed(&self, name: &str) -> bool {
        self.processed.contains(name)
    }

    fn mark_processed(&mut self, name: &str) {
        self.processed.insert(name.to_string());
    }

    fn pop(&mut self) {
        self.current.pop();
    }

    fn get_mut(&mut self) -> Option<&mut ContainerFormat> {
        if let Some(name) = self.current.last() {
            self.containers.get_mut(name)
        } else {
            None
        }
    }

    #[must_use]
    pub fn consume(self) -> BTreeMap<String, ContainerFormat> {
        self.containers
    }
}

/// Build a registry of container formats.
#[must_use]
pub fn reflect<'a, T: Facet<'a>>() -> Registry {
    let mut registry = Registry::new();
    format(T::SHAPE, &mut registry);
    registry
}

fn format<'shape>(shape: &'shape Shape<'shape>, registry: &mut Registry) {
    // First check the type system (Type)
    match &shape.ty {
        Type::User(UserType::Struct(struct_def)) => {
            // Get renamed name before mutable borrow to avoid borrowing conflicts
            let type_name = get_name(shape, registry);

            // Check if we're currently inside a container that needs updating
            if let Some(current_format) = registry.get_mut() {
                match current_format {
                    ContainerFormat::NewTypeStruct(inner_format) => {
                        if inner_format.is_unknown() {
                            // Special case for unit type
                            if shape.type_identifier == "()" {
                                **inner_format = Format::Unit;
                            } else {
                                // Update the NewTypeStruct format to reference the inner type
                                **inner_format = Format::TypeName(type_name.clone());
                            }
                        }
                    }
                    ContainerFormat::TupleStruct(formats) => {
                        // Special case for unit type
                        if shape.type_identifier == "()" {
                            formats.push(Format::Unit);
                        } else {
                            // Add TypeName format to the tuple
                            formats.push(Format::TypeName(type_name.clone()));
                        }
                    }
                    ContainerFormat::Struct(named_formats) => {
                        // Update the last named field with the TypeName format
                        if let Some(last_named) = named_formats.last_mut() {
                            if last_named.value.is_unknown() {
                                // Special case for unit type
                                if shape.type_identifier == "()" {
                                    last_named.value = Format::Unit;
                                } else {
                                    last_named.value = Format::TypeName(type_name.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            format_struct(struct_def, shape, registry);
            return;
        }
        Type::User(UserType::Enum(enum_def)) => {
            format_enum(enum_def, shape, registry);
            return;
        }
        Type::Sequence(sequence_type) => {
            match sequence_type {
                SequenceType::Slice(_slice_type) => {
                    // For slices, use the Def::Slice if available
                    if let Def::Slice(slice_def) = shape.def {
                        format_slice(slice_def, registry);
                        return;
                    }
                }
                SequenceType::Array(_array_type) => {
                    // For arrays, use the Def::Array if available
                    if let Def::Array(array_def) = shape.def {
                        format_array(array_def, registry);
                        return;
                    }
                }
            }
        }
        _ => {} // Continue to check the def system
    }

    // Then check the def system (Def)
    match shape.def {
        Def::Scalar(scalar_def) => format_scalar(scalar_def, registry),
        Def::Map(map_def) => format_map(map_def, registry),
        Def::List(list_def) => format_list(list_def, registry),
        Def::Slice(slice_def) => format_slice(slice_def, registry),
        Def::Array(array_def) => format_array(array_def, registry),
        Def::Option(option_def) => format_option(option_def, registry),
        Def::SmartPointer(SmartPointerDef {
            pointee: Some(inner_shape),
            ..
        }) => format(inner_shape(), registry),
        Def::Undefined => {
            // Handle the case when not yet migrated to the Type enum
            // For primitives, we can try to infer the type
            match &shape.ty {
                Type::Primitive(primitive) => match primitive {
                    PrimitiveType::Boolean
                    | PrimitiveType::Numeric(NumericType::Float)
                    | PrimitiveType::Textual(TextualType::Str) => {}
                    p => {
                        unimplemented!("Unknown primitive type: {p:?}");
                    }
                },
                Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) => {
                    format((pt.target)(), registry);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn scalar_def_to_format(scalar_def: ScalarDef) -> Format {
    match scalar_def.affinity {
        ScalarAffinity::Number(number_affinity) => match number_affinity.bits {
            NumberBits::Integer { size, sign } => {
                let bits = match size {
                    IntegerSize::Fixed(bits) => bits,
                    IntegerSize::PointerSized => core::mem::size_of::<usize>() * 8,
                };

                match sign {
                    Signedness::Unsigned => match bits {
                        8 => Format::U8,
                        16 => Format::U16,
                        32 => Format::U32,
                        64 => Format::U64,
                        128 => Format::U128,
                        _ => unimplemented!("Unsupported integer size: {}", bits),
                    },
                    Signedness::Signed => match bits {
                        8 => Format::I8,
                        16 => Format::I16,
                        32 => Format::I32,
                        64 => Format::I64,
                        128 => Format::I128,
                        _ => unimplemented!("Unsupported integer size: {}", bits),
                    },
                }
            }
            NumberBits::Float {
                sign_bits: _,
                exponent_bits,
                mantissa_bits,
                has_explicit_first_mantissa_bit: _,
            } => {
                // IEEE 754 standard:
                // f32: 8 exponent bits, 23 mantissa bits
                // f64: 11 exponent bits, 52 mantissa bits
                match (exponent_bits, mantissa_bits) {
                    (8, 23) => Format::F32,
                    (11, 52) => Format::F64,
                    _ => unimplemented!(
                        "Unsupported float format: {} exponent bits, {} mantissa bits",
                        exponent_bits,
                        mantissa_bits
                    ),
                }
            }
            NumberBits::Fixed {
                sign_bits: _,
                integer_bits: _,
                fraction_bits: _,
            } => todo!(),
            NumberBits::Decimal {
                sign_bits: _,
                integer_bits: _,
                scale_bits: _,
            } => todo!(),
            _ => todo!(),
        },
        ScalarAffinity::Boolean(_) => Format::Bool,
        ScalarAffinity::String(_) => Format::Str,
        ScalarAffinity::ComplexNumber(_complex_number_affinity) => todo!("ComplexNumber"),
        ScalarAffinity::Empty(_empty_affinity) => todo!("Empty"),
        ScalarAffinity::SocketAddr(_socket_addr_affinity) => todo!("SocketAddr"),
        ScalarAffinity::IpAddr(_ip_addr_affinity) => todo!("IpAddr"),
        ScalarAffinity::Url(_url_affinity) => todo!("Url"),
        ScalarAffinity::UUID(_uuid_affinity) => todo!("UUID"),
        ScalarAffinity::ULID(_ulid_affinity) => todo!("ULID"),
        ScalarAffinity::Time(_time_affinity) => todo!("Time"),
        ScalarAffinity::Opaque(_opaque_affinity) => todo!("Opaque"),
        ScalarAffinity::Other(_other_affinity) => todo!("Other"),
        ScalarAffinity::Char(_char_affinity) => Format::Char,
        ScalarAffinity::Path(_path_affinity) => todo!("Path"),
        _ => todo!(),
    }
}

fn format_scalar(scalar_def: ScalarDef, registry: &mut Registry) {
    let format = scalar_def_to_format(scalar_def);
    update_container_format(format, registry);
}

fn format_struct(struct_type: &StructType, shape: &Shape, registry: &mut Registry) {
    // Check if already processed
    if registry.is_processed(shape.type_identifier) {
        return;
    }

    let struct_name = get_name(shape, registry);

    // Register name mapping if it's different from original
    if struct_name != shape.type_identifier {
        registry.register_type_mapping(shape.type_identifier.to_string(), struct_name.clone());
    }

    registry.mark_processed(shape.type_identifier);

    match struct_type.kind {
        StructKind::Unit => {
            registry.push(struct_name, ContainerFormat::UnitStruct);
            registry.pop();
        }
        StructKind::TupleStruct => {
            if struct_type.fields.len() == 1 {
                let field = struct_type.fields[0];
                let field_shape = field.shape();

                // Check if this is a transparent struct
                let is_transparent = is_transparent_struct(shape);

                if is_transparent {
                    // For transparent structs, don't create a container - just process the inner type
                    // This will register the transparent struct's name with its inner type's format
                    format(field_shape, registry);
                    return;
                }

                // Handle regular newtype struct
                let container = ContainerFormat::NewTypeStruct(Box::new(Format::unknown()));
                registry.push(struct_name, container);

                // Process the inner field
                format(field_shape, registry);

                registry.pop();
            } else {
                // Handle tuple struct with multiple fields
                let container = ContainerFormat::TupleStruct(vec![]);
                registry.push(struct_name, container);
                for field in struct_type.fields {
                    format(field.shape(), registry);
                }
                registry.pop();
            }
        }
        StructKind::Struct => {
            let container = ContainerFormat::Struct(vec![]);
            registry.push(struct_name, container);
            for field in struct_type.fields {
                handle_struct_field(field, registry);
            }
            registry.pop();
        }
        StructKind::Tuple => {
            // This handles standalone tuple types, but for tuple fields in structs,
            // they are handled in the StructKind::Struct case above
        }
        _ => todo!(),
    }
}

#[allow(clippy::too_many_lines)]
fn handle_struct_field(field: &Field, registry: &mut Registry) {
    let field_shape = field.shape();

    // Check for field-level attributes first
    let has_bytes_attribute = field.attributes.iter().any(|attr| match attr {
        FieldAttribute::Arbitrary(attr_str) => *attr_str == "bytes",
        _ => false,
    });

    // Handle bytes attribute for Vec<u8>
    if has_bytes_attribute && field_shape.type_identifier == "Vec" {
        // Check if it's actually Vec<u8> by examining the definition
        if let Def::List(list_def) = field_shape.def {
            let inner_shape = list_def.t();
            if inner_shape.type_identifier == "u8" {
                if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                    named_formats.push(Named {
                        name: field.name.to_string(),
                        value: Format::Bytes,
                    });
                }
                return;
            }
        }
    }

    // Handle bytes attribute for &[u8] slices
    if has_bytes_attribute && field_shape.type_identifier == "&_" {
        // Check if it's a wide pointer (slice) to u8
        if let Type::Pointer(PointerType::Reference(ref_type)) = &field_shape.ty {
            if ref_type.wide {
                let target_shape = (ref_type.target)();
                // Check if target is a slice and get its element type
                if target_shape.type_identifier == "[_]" {
                    if let Def::Slice(slice_def) = target_shape.def {
                        let element_shape = slice_def.t();
                        if element_shape.type_identifier == "u8" {
                            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut()
                            {
                                named_formats.push(Named {
                                    name: field.name.to_string(),
                                    value: Format::Bytes,
                                });
                            }
                            return;
                        }
                    }
                }
            }
        }
    }

    // Check if the field is an Option
    if field_shape.type_identifier == "Option" {
        if let Def::Option(option_def) = field_shape.def {
            // Handle Option types directly
            let inner_shape = option_def.t();
            let inner_format = get_inner_format(inner_shape, registry);
            let option_format = Format::Option(Box::new(inner_format));

            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: option_format,
                });
            }

            // If the inner type is a user-defined type, we need to process it too
            if !matches!(inner_shape.def, Def::Scalar(_)) {
                format(inner_shape, registry);
            }
            return;
        }
    }

    // Check if the field is a tuple struct
    if let Type::User(UserType::Struct(inner_struct)) = &field_shape.ty {
        if inner_struct.kind == StructKind::Tuple {
            // Handle tuple field specially
            let mut tuple_formats = vec![];
            for tuple_field in inner_struct.fields {
                let tuple_field_shape = tuple_field.shape();
                let field_format = get_inner_format(tuple_field_shape, registry);
                tuple_formats.push(field_format);
            }

            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                let tuple_format = if tuple_formats.is_empty() {
                    Format::Unit
                } else {
                    Format::Tuple(tuple_formats)
                };
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: tuple_format,
                });
            }
            return;
        }

        // Check if the referenced struct is transparent
        let is_referenced_transparent = inner_struct.kind == StructKind::TupleStruct
            && inner_struct.fields.len() == 1
            && is_transparent_struct(field_shape);

        if is_referenced_transparent {
            // For transparent struct references, use the inner type directly
            let inner_field = inner_struct.fields[0];
            let inner_field_shape = inner_field.shape();
            let inner_format = get_inner_format(inner_field_shape, registry);

            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: inner_format,
                });
            }
            return;
        }
    }

    // Default behavior: determine the proper format and add it

    // For user-defined types (structs/enums), get the renamed name before mutable borrow
    // But skip primitives like String, which are also Type::User but should use scalar format
    let field_format = if let Type::User(user_type) = &field_shape.ty {
        match user_type {
            UserType::Struct(_) | UserType::Enum(_) => {
                let renamed_name = get_name(field_shape, registry);

                Format::TypeName(renamed_name)
            }
            _ => Format::unknown(),
        }
    } else {
        Format::unknown()
    };

    if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
        named_formats.push(Named {
            name: field.name.to_string(),
            value: field_format,
        });
    }
    format(field_shape, registry);
}

#[allow(clippy::too_many_lines)]
fn format_enum(enum_type: &EnumType, shape: &Shape, registry: &mut Registry) {
    // Check if already processed
    if registry.is_processed(shape.type_identifier) {
        return;
    }

    let enum_name = get_name(shape, registry);

    // Register name mapping if it's different from original
    if enum_name != shape.type_identifier {
        registry.register_type_mapping(shape.type_identifier.to_string(), enum_name.clone());
    }

    registry.mark_processed(shape.type_identifier);

    let mut variants = BTreeMap::new();

    let mut variant_index = 0u32;
    for variant in enum_type.variants {
        let skip = variant.attributes.iter().any(|attr| match attr {
            VariantAttribute::Arbitrary(attr_str) => *attr_str == "skip",
            _ => false,
        });
        if skip {
            continue;
        }
        let variant_format = if variant.data.fields.is_empty() {
            // Unit variant
            VariantFormat::Unit
        } else if variant.data.fields.len() == 1 {
            // Newtype variant - handle user-defined types explicitly
            let field = variant.data.fields[0];
            let field_shape = field.shape();

            if field_shape.type_identifier == "()" {
                VariantFormat::NewType(Box::new(Format::Unit))
            } else if let Type::User(UserType::Struct(_) | UserType::Enum(_)) = &field_shape.ty {
                // For user-defined struct/enum types,
                // create a TypeName reference and process the type
                format(field_shape, registry);
                VariantFormat::NewType(Box::new(Format::TypeName(
                    field_shape.type_identifier.to_string(),
                )))
            } else {
                // For other types, use the temporary container approach
                let temp_container = ContainerFormat::NewTypeStruct(Box::new(Format::unknown()));
                registry.push(format!("temp_{}", variant.name), temp_container);

                // Process the field to determine its format
                format(field_shape, registry);

                // Extract the format from the temporary container
                let variant_format = if let Some(ContainerFormat::NewTypeStruct(inner_format)) =
                    registry.containers.get(&format!("temp_{}", variant.name))
                {
                    VariantFormat::NewType(inner_format.clone())
                } else {
                    VariantFormat::Unit
                };

                // Clean up the temporary container
                registry
                    .containers
                    .remove(&format!("temp_{}", variant.name));
                registry.pop();

                variant_format
            }
        } else {
            // Multiple fields - check if it's a struct variant (named fields) or tuple variant
            let first_field = variant.data.fields[0];
            let is_struct_variant = !first_field.name.chars().all(|c| c.is_ascii_digit());

            if is_struct_variant {
                // Struct variant with named fields
                let temp_container = ContainerFormat::Struct(vec![]);
                registry.push(format!("temp_{}", variant.name), temp_container);

                // Process all fields with their names
                for field in variant.data.fields {
                    let field_shape = field.shape();

                    // Check if the field is a user-defined struct
                    if let Type::User(UserType::Struct(_)) = &field_shape.ty {
                        // Add Named TypeName format to the struct
                        if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                            // Special case for unit type
                            let value = if field_shape.type_identifier == "()" {
                                Format::Unit
                            } else {
                                Format::TypeName(field_shape.type_identifier.to_string())
                            };
                            named_formats.push(Named {
                                name: field.name.to_string(),
                                value,
                            });
                        }
                        // Process the inner type to add it to the registry (skip for unit type)
                        if field_shape.type_identifier != "()" {
                            format(field_shape, registry);
                        }
                    } else {
                        // For non-struct types, add unknown format and let format() fill it
                        if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                            named_formats.push(Named {
                                name: field.name.to_string(),
                                value: Format::unknown(),
                            });
                        }
                        format(field_shape, registry);
                    }
                }

                // Extract the formats from the temporary container
                let variant_format = if let Some(ContainerFormat::Struct(named_formats)) =
                    registry.containers.get(&format!("temp_{}", variant.name))
                {
                    VariantFormat::Struct(named_formats.clone())
                } else {
                    VariantFormat::Unit
                };

                // Clean up the temporary container
                registry
                    .containers
                    .remove(&format!("temp_{}", variant.name));
                registry.pop();

                variant_format
            } else {
                // Tuple variant (multiple unnamed fields)
                let temp_container = ContainerFormat::TupleStruct(vec![]);
                registry.push(format!("temp_{}", variant.name), temp_container);

                // Process all fields
                for field in variant.data.fields {
                    format(field.shape(), registry);
                }

                // Extract the formats from the temporary container
                let variant_format = if let Some(ContainerFormat::TupleStruct(formats)) =
                    registry.containers.get(&format!("temp_{}", variant.name))
                {
                    VariantFormat::Tuple(formats.clone())
                } else {
                    VariantFormat::Unit
                };

                // Clean up the temporary container
                registry
                    .containers
                    .remove(&format!("temp_{}", variant.name));
                registry.pop();

                variant_format
            }
        };

        variants.insert(
            variant_index,
            Named {
                name: variant.name.to_string(),
                value: variant_format,
            },
        );
        variant_index += 1;
    }

    let container = ContainerFormat::Enum(variants);
    registry.push(enum_name, container);
    registry.pop();
}

fn format_list(list_def: ListDef, registry: &mut Registry) {
    // Get the inner type of the list
    let inner_shape = list_def.t();

    // Get the format for the inner type recursively
    let inner_format = get_inner_format(inner_shape, registry);
    let seq_format = Format::Seq(Box::new(inner_format));

    // Update the current container with the sequence format
    update_container_format(seq_format, registry);

    // Process any user-defined types in the nested structure
    process_nested_types(inner_shape, registry);
}

fn format_map(map_def: MapDef, registry: &mut Registry) {
    // Get the key and value types of the map
    let key_shape = map_def.k();
    let value_shape = map_def.v();

    // Get the formats for key and value types
    let key_format = get_inner_format(key_shape, registry);
    let value_format = get_inner_format(value_shape, registry);

    let map_format = Format::Map {
        key: Box::new(key_format),
        value: Box::new(value_format),
    };

    // Update the current container with the map format
    update_container_format(map_format, registry);

    // Process any user-defined types in the nested structure
    process_nested_types(key_shape, registry);
    process_nested_types(value_shape, registry);
}

fn format_slice(slice_def: SliceDef, registry: &mut Registry) {
    format(slice_def.t(), registry);
}

fn format_array(array_def: ArrayDef, registry: &mut Registry) {
    // Get the inner type and size of the array
    let inner_shape = array_def.t();
    let array_size = array_def.n;

    // Determine the format for the inner type
    let inner_format = get_inner_format(inner_shape, registry);

    let array_format = Format::TupleArray {
        content: Box::new(inner_format),
        size: array_size,
    };

    // Update the current container with the array format
    update_container_format(array_format, registry);

    // If the inner type is a user-defined type, we need to process it too
    if !matches!(inner_shape.def, Def::Scalar(_)) {
        format(inner_shape, registry);
    }
}

fn format_option(option_def: OptionDef, registry: &mut Registry) {
    // Get the inner type of the Option
    let inner_shape = option_def.t();

    // We need to determine what format to use for the Option based on the inner type
    let inner_format = get_inner_format(inner_shape, registry);
    let option_format = Format::Option(Box::new(inner_format));

    // Update the current container with the option format
    update_container_format(option_format, registry);

    // If the inner type is a user-defined type, we need to process it too (skip for unit type)
    if should_process_nested_type(inner_shape) {
        format(inner_shape, registry);
    }
}

fn get_name(shape: &Shape, registry: &Registry) -> String {
    // Check type_tag first (is this where facet rename is stored?)
    if let Some(type_tag) = shape.type_tag {
        return type_tag.to_string();
    }

    // Check attributes for name.
    // We want to use #[facet(rename = "NewName")], but that doesn't come through (yet?).
    // So, for now, we use an arbitrary attribute with "#[facet(name = "\NewName\")]"
    for attr in shape.attributes {
        if let ShapeAttribute::Arbitrary(attr_str) = attr {
            // Check for rename attribute in the format "name = \"NewName\""
            if let Some(stripped) = attr_str.strip_prefix("name = \"") {
                if let Some(end_idx) = stripped.find('"') {
                    return stripped[..end_idx].to_string();
                }
            }
            // Check for rename attribute without quotes "name = NewName"
            if let Some(stripped) = attr_str.strip_prefix("name = ") {
                return stripped.trim_matches('"').to_string();
            }
        }
    }

    // Check registry for mapped names
    if let Some(name) = registry.get_mapped_name(shape.type_identifier) {
        return name;
    }

    shape.type_identifier.to_string()
}

fn is_transparent_struct(shape: &Shape) -> bool {
    shape.attributes.iter().any(|attr| match attr {
        ShapeAttribute::Transparent => true,
        ShapeAttribute::Arbitrary(attr_str) => *attr_str == "transparent",
        _ => false,
    })
}

fn update_container_format(format: Format, registry: &mut Registry) {
    if let Some(container_format) = registry.get_mut() {
        match container_format {
            ContainerFormat::UnitStruct => {}
            ContainerFormat::NewTypeStruct(inner_format) => {
                *inner_format = Box::new(format);
            }
            ContainerFormat::TupleStruct(formats) => {
                formats.push(format);
            }
            ContainerFormat::Struct(named_formats) => {
                if let Some(last) = named_formats.last_mut() {
                    if last.value.is_unknown() {
                        last.value = format;
                    }
                }
            }
            ContainerFormat::Enum(_) => todo!(),
        }
    }
}

fn should_process_nested_type(shape: &Shape) -> bool {
    !matches!(shape.def, Def::Scalar(_)) && shape.type_identifier != "()"
}

fn get_inner_format(shape: &Shape, registry: &Registry) -> Format {
    match shape.def {
        Def::Scalar(scalar_def) => scalar_def_to_format(scalar_def),
        Def::List(inner_list_def) => {
            // Recursively handle nested lists
            let inner_shape = inner_list_def.t();
            Format::Seq(Box::new(get_inner_format(inner_shape, registry)))
        }
        Def::Option(option_def) => {
            // Handle Option<T> -> OPTION: T
            let inner_shape = option_def.t();
            let inner_format = get_inner_format(inner_shape, registry);
            Format::Option(Box::new(inner_format))
        }
        Def::Map(map_def) => {
            // Handle Map<K, V> -> MAP: { KEY: K, VALUE: V }
            let key_shape = map_def.k();
            let value_shape = map_def.v();
            let key_format = get_inner_format(key_shape, registry);
            let value_format = get_inner_format(value_shape, registry);
            Format::Map {
                key: Box::new(key_format),
                value: Box::new(value_format),
            }
        }
        Def::Array(array_def) => {
            // Handle Array<T, N> -> TUPLEARRAY: { CONTENT: T, SIZE: N }
            let inner_shape = array_def.t();
            let inner_format = get_inner_format(inner_shape, registry);
            Format::TupleArray {
                content: Box::new(inner_format),
                size: array_def.n,
            }
        }
        Def::Undefined => {
            // Check if this is a tuple type by examining the type structure
            if let Type::User(UserType::Struct(struct_type)) = &shape.ty {
                if struct_type.kind == StructKind::Tuple && !struct_type.fields.is_empty() {
                    // Handle tuple types -> TUPLE: [field1, field2, ...]
                    let mut tuple_formats = vec![];
                    for field in struct_type.fields {
                        let field_shape = field.shape();
                        let field_format = get_inner_format(field_shape, registry);
                        tuple_formats.push(field_format);
                    }
                    return Format::Tuple(tuple_formats);
                }
            }

            // Special case for unit type
            // For user-defined types, use TypeName
            if shape.type_identifier == "()" {
                Format::Unit
            } else {
                // For user-defined types, use TypeName with renamed name if applicable
                let name = get_name(shape, registry);

                Format::TypeName(name)
            }
        }
        Def::Set(_set_def) => todo!(),
        Def::Slice(_slice_def) => todo!(),
        Def::SmartPointer(_smart_pointer_def) => todo!(),
        _ => {
            // For other types, use TypeName with renamed name if applicable
            let name = get_name(shape, registry);
            Format::TypeName(name)
        }
    }
}

fn process_nested_types(shape: &Shape, registry: &mut Registry) {
    match shape.def {
        Def::Scalar(_) => {
            // Scalar types don't need further processing
        }
        Def::List(inner_list_def) => {
            // Recursively process nested lists
            let inner_shape = inner_list_def.t();
            process_nested_types(inner_shape, registry);
        }
        Def::Option(option_def) => {
            // Recursively process options
            let inner_shape = option_def.t();
            if should_process_nested_type(inner_shape) {
                process_nested_types(inner_shape, registry);
            }
        }
        Def::Map(map_def) => {
            // Recursively process maps
            let key_shape = map_def.k();
            let value_shape = map_def.v();
            if should_process_nested_type(key_shape) {
                process_nested_types(key_shape, registry);
            }
            if should_process_nested_type(value_shape) {
                process_nested_types(value_shape, registry);
            }
        }
        _ => {
            // For other user-defined types, process them
            format(shape, registry);
        }
    }
}

#[cfg(test)]
mod tests;
