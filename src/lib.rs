use std::collections::BTreeMap;

use facet::{
    ArrayDef, Def, EnumType, Facet, IntegerSize, ListDef, NumberBits, NumericType, OptionDef,
    PointerType, PrimitiveType, ScalarAffinity, ScalarDef, SequenceType, Shape, Signedness,
    SliceDef, SmartPointerDef, StructKind, StructType, TextualType, Type, UserType,
};
use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, VariantFormat};

#[derive(Debug)]
pub struct Registry {
    containers: BTreeMap<String, ContainerFormat>,
    current: Vec<String>,
}

impl Registry {
    fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
            current: Vec::new(),
        }
    }

    fn push(&mut self, name: String, format: ContainerFormat) {
        self.containers.insert(name.clone(), format);
        self.current.push(name);
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
            // Check if we're currently inside a container that needs updating
            if let Some(current_format) = registry.get_mut() {
                match current_format {
                    ContainerFormat::NewTypeStruct(inner_format) => {
                        if inner_format.is_unknown() {
                            // Update the NewTypeStruct format to reference the inner type
                            **inner_format = Format::TypeName(shape.type_identifier.to_string());
                        }
                    }
                    ContainerFormat::TupleStruct(formats) => {
                        // Add TypeName format to the tuple
                        formats.push(Format::TypeName(shape.type_identifier.to_string()));
                    }
                    ContainerFormat::Struct(nameds) => {
                        // Update the last named field with the TypeName format
                        if let Some(last_named) = nameds.last_mut() {
                            if last_named.value.is_unknown() {
                                last_named.value =
                                    Format::TypeName(shape.type_identifier.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            format_struct(shape.type_identifier, struct_def, registry);
            return;
        }
        Type::User(UserType::Enum(enum_def)) => {
            format_enum(shape.type_identifier, enum_def, registry);
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
        Def::Map(_map_def) => todo!("Map"),
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
                    PrimitiveType::Numeric(NumericType::Float) => {}
                    PrimitiveType::Boolean => {}
                    PrimitiveType::Textual(TextualType::Str) => {}
                    p => {
                        println!("Unknown primitive type: {p:?}");
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

#[allow(clippy::too_many_lines)]
fn format_scalar(scalar_def: ScalarDef, registry: &mut Registry) {
    match scalar_def.affinity {
        ScalarAffinity::Number(number_affinity) => {
            match number_affinity.bits {
                NumberBits::Integer { size, sign } => {
                    let bits = match size {
                        IntegerSize::Fixed(bits) => bits,
                        IntegerSize::PointerSized => core::mem::size_of::<usize>() * 8,
                    };

                    match sign {
                        Signedness::Unsigned => {
                            let uint_format = match bits {
                                8 => Format::U8,
                                16 => Format::U16,
                                32 => Format::U32,
                                64 => Format::U64,
                                128 => Format::U128,
                                _ => unimplemented!(),
                            };
                            if let Some(format) = registry.get_mut() {
                                match format {
                                    ContainerFormat::UnitStruct => {}
                                    ContainerFormat::NewTypeStruct(format) => {
                                        *format = Box::new(uint_format);
                                    }
                                    ContainerFormat::TupleStruct(formats) => {
                                        formats.push(uint_format);
                                    }
                                    ContainerFormat::Struct(nameds) => {
                                        // Update the last field's value (it was added with Format::unknown())
                                        if let Some(last) = nameds.last_mut() {
                                            last.value = uint_format;
                                        }
                                    }
                                    ContainerFormat::Enum(_btree_map) => todo!(),
                                }
                            }
                        }
                        Signedness::Signed => {
                            let int_format = match bits {
                                8 => Format::I8,
                                16 => Format::I16,
                                32 => Format::I32,
                                64 => Format::I64,
                                128 => Format::I128,
                                _ => unimplemented!(),
                            };
                            if let Some(format) = registry.get_mut() {
                                match format {
                                    ContainerFormat::UnitStruct => {}
                                    ContainerFormat::NewTypeStruct(format) => {
                                        *format = Box::new(int_format);
                                    }
                                    ContainerFormat::TupleStruct(formats) => {
                                        formats.push(int_format);
                                    }
                                    ContainerFormat::Struct(nameds) => {
                                        // Update the last entry's value (which was added as unknown)
                                        if let Some(last) = nameds.last_mut() {
                                            if last.value.is_unknown() {
                                                last.value = int_format;
                                            }
                                        }
                                    }
                                    ContainerFormat::Enum(_btree_map) => todo!(),
                                }
                            }
                        }
                    }
                }
                NumberBits::Float { .. } => {}
                _ => unimplemented!(),
            }
        }
        ScalarAffinity::String(_) => {
            if let Some(format) = registry.get_mut() {
                match format {
                    ContainerFormat::UnitStruct => {}
                    ContainerFormat::NewTypeStruct(format) => {
                        *format = Box::new(Format::Str);
                    }
                    ContainerFormat::TupleStruct(formats) => {
                        formats.push(Format::Str);
                    }
                    ContainerFormat::Struct(nameds) => {
                        if let Some(last) = nameds.last_mut() {
                            if last.value.is_unknown() {
                                last.value = Format::Str;
                            }
                        }
                    }
                    ContainerFormat::Enum(_btree_map) => todo!(),
                }
            }
        }
        ScalarAffinity::Boolean(_) => {
            if let Some(format) = registry.get_mut() {
                match format {
                    ContainerFormat::UnitStruct => {}
                    ContainerFormat::NewTypeStruct(format) => {
                        *format = Box::new(Format::Bool);
                    }
                    ContainerFormat::TupleStruct(formats) => {
                        formats.push(Format::Bool);
                    }
                    ContainerFormat::Struct(nameds) => {
                        if let Some(last) = nameds.last_mut() {
                            if last.value.is_unknown() {
                                last.value = Format::Bool;
                            }
                        }
                    }
                    ContainerFormat::Enum(_btree_map) => todo!(),
                }
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_lines)]
fn format_struct(name: &str, struct_type: &StructType, registry: &mut Registry) {
    println!("Struct {name}");
    match struct_type.kind {
        StructKind::Unit => {
            registry.push(name.to_string(), ContainerFormat::UnitStruct);
            registry.pop();
        }
        StructKind::TupleStruct => {
            println!("TupleStruct {struct_type:?}");
            if struct_type.fields.len() == 1 {
                // Handle newtype struct
                let field = struct_type.fields[0];
                let field_shape = field.shape();

                // Create the newtype container
                let container = ContainerFormat::NewTypeStruct(Box::new(Format::unknown()));
                registry.push(name.to_string(), container);

                // Process the inner field
                format(field_shape, registry);

                registry.pop();
            } else {
                // Handle tuple struct with multiple fields
                let container = ContainerFormat::TupleStruct(vec![]);
                registry.push(name.to_string(), container);
                for field in struct_type.fields {
                    format(field.shape(), registry);
                }
                registry.pop();
            }
        }
        StructKind::Struct => {
            let container = ContainerFormat::Struct(vec![]);
            registry.push(name.to_string(), container);
            for field in struct_type.fields {
                let field_shape = field.shape();

                // Check if the field is an Option first
                if field_shape.type_identifier == "Option" {
                    if let Def::Option(option_def) = field_shape.def {
                        // Handle Option types directly
                        let inner_shape = option_def.t();
                        let option_format = match inner_shape.def {
                            Def::Scalar(scalar_def) => match scalar_def.affinity {
                                ScalarAffinity::Number(number_affinity) => {
                                    match number_affinity.bits {
                                        NumberBits::Integer { size, sign } => {
                                            let bits = match size {
                                                IntegerSize::Fixed(bits) => bits,
                                                IntegerSize::PointerSized => {
                                                    core::mem::size_of::<usize>() * 8
                                                }
                                            };
                                            match sign {
                                                Signedness::Unsigned => match bits {
                                                    8 => Format::Option(Box::new(Format::U8)),
                                                    16 => Format::Option(Box::new(Format::U16)),
                                                    32 => Format::Option(Box::new(Format::U32)),
                                                    64 => Format::Option(Box::new(Format::U64)),
                                                    128 => Format::Option(Box::new(Format::U128)),
                                                    _ => unimplemented!(),
                                                },
                                                Signedness::Signed => match bits {
                                                    8 => Format::Option(Box::new(Format::I8)),
                                                    16 => Format::Option(Box::new(Format::I16)),
                                                    32 => Format::Option(Box::new(Format::I32)),
                                                    64 => Format::Option(Box::new(Format::I64)),
                                                    128 => Format::Option(Box::new(Format::I128)),
                                                    _ => unimplemented!(),
                                                },
                                            }
                                        }
                                        _ => Format::Option(Box::new(Format::Unit)),
                                    }
                                }
                                ScalarAffinity::Boolean(_) => {
                                    Format::Option(Box::new(Format::Bool))
                                }
                                ScalarAffinity::String(_) => Format::Option(Box::new(Format::Str)),
                                _ => Format::Option(Box::new(Format::Unit)),
                            },
                            _ => {
                                // For user-defined types, use TypeName
                                Format::Option(Box::new(Format::TypeName(
                                    inner_shape.type_identifier.to_string(),
                                )))
                            }
                        };

                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: option_format,
                            });
                        }

                        // If the inner type is a user-defined type, we need to process it too
                        if !matches!(inner_shape.def, Def::Scalar(_)) {
                            format(inner_shape, registry);
                        }
                    } else {
                        // Fallback: add unknown format and let format() handle it
                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: Format::unknown(),
                            });
                        }
                        format(field_shape, registry);
                    }
                }
                // Check if the field is a tuple
                else if let Type::User(UserType::Struct(inner_struct)) = &field_shape.ty {
                    if inner_struct.kind == StructKind::Tuple {
                        // Handle tuple field specially
                        let mut tuple_formats = vec![];
                        for tuple_field in inner_struct.fields {
                            let tuple_field_shape = tuple_field.shape();
                            match tuple_field_shape.def {
                                Def::Scalar(scalar_def) => match scalar_def.affinity {
                                    ScalarAffinity::Number(number_affinity) => {
                                        match number_affinity.bits {
                                            NumberBits::Integer { size, sign } => {
                                                let bits = match size {
                                                    IntegerSize::Fixed(bits) => bits,
                                                    IntegerSize::PointerSized => {
                                                        core::mem::size_of::<usize>() * 8
                                                    }
                                                };
                                                let format = match sign {
                                                    Signedness::Unsigned => match bits {
                                                        8 => Format::U8,
                                                        16 => Format::U16,
                                                        32 => Format::U32,
                                                        64 => Format::U64,
                                                        128 => Format::U128,
                                                        _ => unimplemented!(),
                                                    },
                                                    Signedness::Signed => match bits {
                                                        8 => Format::I8,
                                                        16 => Format::I16,
                                                        32 => Format::I32,
                                                        64 => Format::I64,
                                                        128 => Format::I128,
                                                        _ => unimplemented!(),
                                                    },
                                                };
                                                tuple_formats.push(format);
                                            }
                                            _ => {
                                                tuple_formats.push(Format::Unit);
                                            }
                                        }
                                    }
                                    ScalarAffinity::Boolean(_) => {
                                        tuple_formats.push(Format::Bool);
                                    }
                                    ScalarAffinity::String(_) => {
                                        tuple_formats.push(Format::Str);
                                    }
                                    _ => {
                                        tuple_formats.push(Format::Unit);
                                    }
                                },
                                _ => {
                                    tuple_formats.push(Format::Unit);
                                }
                            }
                        }

                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: Format::Tuple(tuple_formats),
                            });
                        }
                    } else {
                        // Regular user-defined struct
                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: Format::TypeName(field_shape.type_identifier.to_string()),
                            });
                        }
                        // Process the inner type to add it to the registry
                        format(field_shape, registry);
                    }
                } else {
                    // For non-struct types, add unknown format and let format() fill it
                    if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                        nameds.push(Named {
                            name: field.name.to_string(),
                            value: Format::unknown(),
                        });
                    }
                    format(field_shape, registry);
                }
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

fn format_list(list_def: ListDef, registry: &mut Registry) {
    // Get the inner type of the list
    let inner_shape = list_def.t();

    // Determine the format for the inner type
    let inner_format = match inner_shape.def {
        Def::Scalar(scalar_def) => match scalar_def.affinity {
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
                            _ => unimplemented!(),
                        },
                        Signedness::Signed => match bits {
                            8 => Format::I8,
                            16 => Format::I16,
                            32 => Format::I32,
                            64 => Format::I64,
                            128 => Format::I128,
                            _ => unimplemented!(),
                        },
                    }
                }
                _ => Format::Unit,
            },
            ScalarAffinity::Boolean(_) => Format::Bool,
            ScalarAffinity::String(_) => Format::Str,
            _ => Format::Unit,
        },
        _ => {
            // For user-defined types, use TypeName
            Format::TypeName(inner_shape.type_identifier.to_string())
        }
    };

    let seq_format = Format::Seq(Box::new(inner_format));

    // Update the current container with the sequence format
    if let Some(current_format) = registry.get_mut() {
        match current_format {
            ContainerFormat::NewTypeStruct(format) => {
                *format = Box::new(seq_format);
            }
            ContainerFormat::TupleStruct(formats) => {
                formats.push(seq_format);
            }
            ContainerFormat::Struct(nameds) => {
                if let Some(last) = nameds.last_mut() {
                    if last.value.is_unknown() {
                        last.value = seq_format;
                    }
                }
            }
            _ => {}
        }
    }

    // If the inner type is a user-defined type, we need to process it too
    if !matches!(inner_shape.def, Def::Scalar(_)) {
        format(inner_shape, registry);
    }
}

fn format_slice(slice_def: SliceDef, registry: &mut Registry) {
    format(slice_def.t(), registry);
}

fn format_array(array_def: ArrayDef, registry: &mut Registry) {
    format(array_def.t(), registry);
}

fn format_enum(name: &str, enum_type: &EnumType, registry: &mut Registry) {
    let mut variants = BTreeMap::new();

    for (variant, index) in enum_type.variants.iter().zip(0u32..) {
        let variant_format = if variant.data.fields.is_empty() {
            // Unit variant
            VariantFormat::Unit
        } else if variant.data.fields.len() == 1 {
            // Newtype variant - create a temporary container to process the field
            let field = variant.data.fields[0];
            let field_shape = field.shape();

            // Create a temporary NewTypeStruct container to determine the format
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
                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: Format::TypeName(field_shape.type_identifier.to_string()),
                            });
                        }
                        // Process the inner type to add it to the registry
                        format(field_shape, registry);
                    } else {
                        // For non-struct types, add unknown format and let format() fill it
                        if let Some(ContainerFormat::Struct(nameds)) = registry.get_mut() {
                            nameds.push(Named {
                                name: field.name.to_string(),
                                value: Format::unknown(),
                            });
                        }
                        format(field_shape, registry);
                    }
                }

                // Extract the formats from the temporary container
                let variant_format = if let Some(ContainerFormat::Struct(nameds)) =
                    registry.containers.get(&format!("temp_{}", variant.name))
                {
                    VariantFormat::Struct(nameds.clone())
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
            index,
            Named {
                name: variant.name.to_string(),
                value: variant_format,
            },
        );
    }

    let container = ContainerFormat::Enum(variants);
    registry.push(name.to_string(), container);
    registry.pop();
}

fn format_option(option_def: OptionDef, registry: &mut Registry) {
    // Get the inner type of the Option
    let inner_shape = option_def.t();

    // We need to determine what format to use for the Option based on the inner type
    let option_format = match inner_shape.def {
        Def::Scalar(scalar_def) => {
            // Handle scalar types like bool, u8, i32, etc.
            match scalar_def.affinity {
                ScalarAffinity::Number(number_affinity) => match number_affinity.bits {
                    NumberBits::Integer { size, sign } => {
                        let bits = match size {
                            IntegerSize::Fixed(bits) => bits,
                            IntegerSize::PointerSized => core::mem::size_of::<usize>() * 8,
                        };
                        match sign {
                            Signedness::Unsigned => match bits {
                                8 => Format::Option(Box::new(Format::U8)),
                                16 => Format::Option(Box::new(Format::U16)),
                                32 => Format::Option(Box::new(Format::U32)),
                                64 => Format::Option(Box::new(Format::U64)),
                                128 => Format::Option(Box::new(Format::U128)),
                                _ => unimplemented!(),
                            },
                            Signedness::Signed => match bits {
                                8 => Format::Option(Box::new(Format::I8)),
                                16 => Format::Option(Box::new(Format::I16)),
                                32 => Format::Option(Box::new(Format::I32)),
                                64 => Format::Option(Box::new(Format::I64)),
                                128 => Format::Option(Box::new(Format::I128)),
                                _ => unimplemented!(),
                            },
                        }
                    }
                    _ => Format::Option(Box::new(Format::Unit)),
                },
                ScalarAffinity::Boolean(_) => Format::Option(Box::new(Format::Bool)),
                ScalarAffinity::String(_) => Format::Option(Box::new(Format::Str)),
                _ => Format::Option(Box::new(Format::Unit)),
            }
        }
        _ => {
            // For user-defined types, use TypeName
            Format::Option(Box::new(Format::TypeName(
                inner_shape.type_identifier.to_string(),
            )))
        }
    };

    // Update the current container with the option format
    if let Some(current_format) = registry.get_mut() {
        match current_format {
            ContainerFormat::Struct(nameds) => {
                if let Some(last) = nameds.last_mut() {
                    if last.value.is_unknown() {
                        last.value = option_format;
                    }
                }
            }
            ContainerFormat::TupleStruct(formats) => {
                formats.push(option_format);
            }
            ContainerFormat::NewTypeStruct(format) => {
                *format = Box::new(option_format);
            }
            _ => {}
        }
    }

    // If the inner type is a user-defined type, we need to process it too
    if !matches!(inner_shape.def, Def::Scalar(_)) {
        format(inner_shape, registry);
    }
}

#[cfg(test)]
mod tests;
