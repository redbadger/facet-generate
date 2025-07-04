pub mod format;

use std::{
    collections::{BTreeMap, HashSet},
    string::ToString,
};

use facet::{
    ArrayDef, Def, EnumType, Facet, Field, FieldAttribute, ListDef, MapDef, NumericType, OptionDef,
    PointerDef, PointerType, PrimitiveType, SequenceType, Shape, ShapeAttribute, SliceDef,
    StructKind, StructType, TextualType, Type, UserType, Variant, VariantAttribute,
};
use format::{ContainerFormat, Format, FormatHolder, Named, QualifiedTypeName, VariantFormat};

use crate::reflection::format::Namespace;

/// A map of container formats.
pub type Registry = BTreeMap<QualifiedTypeName, ContainerFormat>;

#[derive(Debug)]
struct State {
    pub containers: Registry,
    current: Vec<QualifiedTypeName>,
    processed: HashSet<QualifiedTypeName>,
    name_mappings: BTreeMap<QualifiedTypeName, QualifiedTypeName>,
}

impl State {
    fn new() -> Self {
        Self {
            containers: Registry::new(),
            current: Vec::new(),
            processed: HashSet::new(),
            name_mappings: BTreeMap::new(),
        }
    }

    fn push(&mut self, name: QualifiedTypeName, container: ContainerFormat) {
        self.containers.insert(name.clone(), container);
        self.current.push(name);
    }

    fn push_temporary(&mut self, name: String, container: ContainerFormat) -> QualifiedTypeName {
        let name = QualifiedTypeName {
            namespace: Namespace::Named("__temp__".to_string()),
            name,
        };
        self.push(name.clone(), container);
        name
    }

    fn register_type_mapping(&mut self, original: QualifiedTypeName, renamed: QualifiedTypeName) {
        self.name_mappings.insert(original, renamed);
    }

    fn is_processed(&self, name: &QualifiedTypeName) -> bool {
        self.processed.contains(name)
    }

    fn mark_processed(&mut self, name: QualifiedTypeName) {
        self.processed.insert(name);
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
    let mut state = State::new();
    format(T::SHAPE, &mut state);
    state.containers
}

fn format<'shape>(shape: &'shape Shape<'shape>, registry: &mut State) {
    process_type(shape, None, registry);
}

fn process_type<'shape>(
    shape: &'shape Shape<'shape>,
    namespace: Option<&str>,
    registry: &mut State,
) {
    // First check for special cases in the def system (like Option)
    if let Def::Option(option_def) = shape.def {
        format_option(option_def, namespace, registry);
        return;
    }

    // Try type system first
    if try_format_from_type_system(shape, namespace, registry) {
        return;
    }

    // Fall back to def system
    format_from_def_system(shape, namespace, registry);
}

fn try_format_from_type_system(
    shape: &Shape,
    namespace: Option<&str>,
    registry: &mut State,
) -> bool {
    match &shape.ty {
        Type::User(UserType::Struct(struct_def)) => {
            handle_user_struct(shape, struct_def, namespace, registry);
            true
        }
        Type::User(UserType::Enum(enum_def)) => {
            format_enum(enum_def, shape, namespace, registry);
            true
        }
        Type::Sequence(sequence_type) => {
            handle_sequence_type(shape, sequence_type, namespace, registry)
        }
        _ => false,
    }
}

fn handle_user_struct(
    shape: &Shape,
    struct_def: &StructType,
    namespace: Option<&str>,
    registry: &mut State,
) {
    let type_name = get_name(shape);

    // Update container with the struct format
    let format = if shape.type_identifier == "()" {
        Format::Unit
    } else {
        Format::TypeName(type_name.clone())
    };
    update_container_format_if_unknown(format, registry);
    format_struct(struct_def, shape, namespace, registry);
}

fn handle_sequence_type(
    shape: &Shape,
    sequence_type: &SequenceType,
    namespace: Option<&str>,
    registry: &mut State,
) -> bool {
    match sequence_type {
        SequenceType::Slice(_slice_type) => {
            // For slices, use the Def::Slice if available
            if let Def::Slice(slice_def) = shape.def {
                format_slice(slice_def, namespace, registry);
                true
            } else {
                false
            }
        }
        SequenceType::Array(_array_type) => {
            // For arrays, use the Def::Array if available
            if let Def::Array(array_def) = shape.def {
                format_array(array_def, namespace, registry);
                true
            } else {
                false
            }
        }
    }
}

fn format_from_def_system(shape: &Shape, namespace: Option<&str>, registry: &mut State) {
    match shape.def {
        Def::Scalar => format_scalar(shape, registry),
        Def::Map(map_def) => format_map(map_def, namespace, registry),
        Def::List(list_def) => format_list(list_def, namespace, registry),
        Def::Slice(slice_def) => format_slice(slice_def, namespace, registry),
        Def::Array(array_def) => format_array(array_def, namespace, registry),
        Def::Option(option_def) => format_option(option_def, namespace, registry),
        Def::Pointer(PointerDef {
            pointee: Some(inner_shape),
            ..
        }) => {
            handle_pointer(inner_shape(), namespace, registry);
        }
        Def::Undefined => {
            handle_undefined_def(shape, namespace, registry);
        }
        _ => {}
    }
}

fn handle_pointer(inner_shape: &Shape, namespace: Option<&str>, registry: &mut State) {
    // For Pointer, we need to update the current container with the inner type's format
    let inner_format = get_inner_format(inner_shape);

    // Update the current container with the Pointer's inner format
    update_container_format_if_unknown(inner_format, registry);

    // Also process the inner type if it's a user-defined type
    process_nested_types(inner_shape, namespace, registry);
}

fn handle_undefined_def(shape: &Shape, namespace: Option<&str>, registry: &mut State) {
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
            process_type((pt.target)(), namespace, registry);
        }
        _ => {}
    }
}

fn type_to_format(shape: &Shape) -> Format {
    match &shape.ty {
        Type::Primitive(primitive) => match primitive {
            PrimitiveType::Boolean => Format::Bool,
            PrimitiveType::Numeric(numeric_type) => match numeric_type {
                NumericType::Float => {
                    // Determine float type based on size or type identifier
                    match shape.type_identifier {
                        "f32" => Format::F32,
                        "f64" => Format::F64,
                        _ => unimplemented!("Unsupported float type: {}", shape.type_identifier),
                    }
                }
                NumericType::Integer { signed } => {
                    // Get the size from the layout
                    let size_bytes = shape
                        .layout
                        .sized_layout()
                        .expect("Layout must be sized")
                        .size();
                    let size_bits = size_bytes * 8;

                    match (*signed, size_bits) {
                        (false, 8) => Format::U8,
                        (false, 16) => Format::U16,
                        (false, 32) => Format::U32,
                        (false, 64) => Format::U64,
                        (false, 128) => Format::U128,
                        (true, 8) => Format::I8,
                        (true, 16) => Format::I16,
                        (true, 32) => Format::I32,
                        (true, 64) => Format::I64,
                        (true, 128) => Format::I128,
                        _ => unimplemented!(
                            "Unsupported integer type: {} bits, signed: {}",
                            size_bits,
                            signed
                        ),
                    }
                }
            },
            PrimitiveType::Textual(textual_type) => match textual_type {
                TextualType::Str => Format::Str,
                TextualType::Char => Format::Char,
            },
            PrimitiveType::Never => unimplemented!("Never type not supported"),
        },
        Type::User(UserType::Opaque) => {
            // Handle opaque types like String based on type identifier
            match shape.type_identifier {
                "String" => Format::Str,
                _ => unimplemented!("Unsupported opaque type: {}", shape.type_identifier),
            }
        }
        _ => unimplemented!("Unsupported type for scalar format: {:?}", shape.ty),
    }
}

fn format_scalar(shape: &Shape, registry: &mut State) {
    let format = type_to_format(shape);
    update_container_format(format, registry);
}

fn format_struct(
    struct_type: &StructType,
    shape: &Shape,
    _parent_namespace: Option<&str>,
    registry: &mut State,
) {
    let struct_name = get_name(shape);

    // Check if already processed using the full namespaced name
    if registry.is_processed(&struct_name) {
        // This is a mutual recursion case - only update if there's an unknown format that needs updating
        let format = Format::TypeName(struct_name.clone());
        update_container_format_for_mutual_recursion(format, registry);
        return;
    }

    // Register name mapping if it's different from original
    if struct_name.name != shape.type_identifier {
        let name = QualifiedTypeName {
            namespace: struct_name.namespace.clone(),
            name: shape.type_identifier.to_string(),
        };
        registry.register_type_mapping(name, struct_name.clone());
    }

    // Extract namespace from this struct if it has one
    let current_namespace = extract_namespace_from_shape(shape);

    registry.mark_processed(struct_name.clone());

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
                    process_type(field_shape, current_namespace.as_deref(), registry);
                    return;
                }

                // Handle regular newtype struct
                let container = ContainerFormat::NewTypeStruct(Box::default());
                registry.push(struct_name, container);

                // Process the inner field
                process_type(field_shape, current_namespace.as_deref(), registry);

                registry.pop();
            } else {
                // Handle tuple struct with multiple fields
                let container = ContainerFormat::TupleStruct(vec![]);
                registry.push(struct_name, container);
                for field in struct_type.fields {
                    process_type(field.shape(), current_namespace.as_deref(), registry);
                }
                registry.pop();
            }
        }
        StructKind::Struct => {
            let container = ContainerFormat::Struct(vec![]);
            registry.push(struct_name, container);
            for field in struct_type.fields {
                handle_struct_field(field, current_namespace.as_deref(), registry);
            }
            registry.pop();
        }
        StructKind::Tuple => {
            // This handles standalone tuple types, but for tuple fields in structs,
            // they are handled in the StructKind::Struct case above
        }
    }
}

fn handle_struct_field(field: &Field, namespace: Option<&str>, registry: &mut State) {
    let field_shape = field.shape();

    // Check for field-level attributes first
    let has_bytes_attribute = field.attributes.iter().any(|attr| match attr {
        FieldAttribute::Arbitrary(attr_str) => *attr_str == "bytes",
    });

    if has_bytes_attribute && try_handle_bytes_attribute(field, field_shape, registry) {
        return;
    }

    if try_handle_option_field(field, field_shape, namespace, registry) {
        return;
    }

    if try_handle_tuple_struct_field(field, field_shape, namespace, registry) {
        return;
    }

    // Default behavior: determine the proper format and add it

    // For user-defined types (structs/enums), get the renamed name before mutable borrow
    // But skip primitives like String, which are also Type::User but should use scalar format
    let field_format = if let Type::User(user_type) = &field_shape.ty {
        match user_type {
            UserType::Struct(_) | UserType::Enum(_) => {
                let renamed_name = get_name(field_shape);

                Format::TypeName(renamed_name)
            }
            _ => get_inner_format(field_shape),
        }
    } else {
        get_inner_format(field_shape)
    };

    if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
        named_formats.push(Named {
            name: field.name.to_string(),
            value: field_format,
        });
    }
    process_type(field_shape, namespace, registry);
}

fn try_handle_bytes_attribute(field: &Field, field_shape: &Shape, registry: &mut State) -> bool {
    // Handle bytes attribute for Vec<u8>
    if field_shape.type_identifier == "Vec" {
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
                return true;
            }
        }
    }

    // Handle bytes attribute for &[u8] slices
    if field_shape.type_identifier == "&[_]" {
        // Check if it's a smart pointer to a slice of u8
        if let Def::Pointer(PointerDef {
            pointee: Some(target_shape_fn),
            ..
        }) = field_shape.def
        {
            let target_shape = target_shape_fn();
            if let Def::Slice(slice_def) = target_shape.def {
                let element_shape = slice_def.t();
                if element_shape.type_identifier == "u8" {
                    if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                        named_formats.push(Named {
                            name: field.name.to_string(),
                            value: Format::Bytes,
                        });
                    }
                    return true;
                }
            }
        }
    }

    false
}

fn try_handle_option_field(
    field: &Field,
    field_shape: &Shape,
    namespace: Option<&str>,
    registry: &mut State,
) -> bool {
    // Check if the field is an Option
    if field_shape.type_identifier == "Option" {
        if let Def::Option(option_def) = field_shape.def {
            // Handle Option types directly
            let inner_shape = option_def.t();
            let inner_format = get_inner_format(inner_shape);
            let option_format = Format::Option(Box::new(inner_format));

            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: option_format,
                });
            }

            // If the inner type is a user-defined type, we need to process it too
            if !matches!(inner_shape.def, Def::Scalar) {
                process_type(inner_shape, namespace, registry);
            }
            return true;
        }
    }
    false
}

fn try_handle_tuple_struct_field(
    field: &Field,
    field_shape: &Shape,
    _namespace: Option<&str>,
    registry: &mut State,
) -> bool {
    // Check if the field is a tuple struct
    if let Type::User(UserType::Struct(inner_struct)) = &field_shape.ty {
        if inner_struct.kind == StructKind::Tuple {
            // Handle tuple field specially
            let mut tuple_formats = vec![];
            for tuple_field in inner_struct.fields {
                let tuple_field_shape = tuple_field.shape();
                let field_format = get_inner_format(tuple_field_shape);
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
            return true;
        }

        // Check if the referenced struct is transparent
        let is_referenced_transparent = inner_struct.kind == StructKind::TupleStruct
            && inner_struct.fields.len() == 1
            && is_transparent_struct(field_shape);

        if is_referenced_transparent {
            // For transparent struct references, use the inner type directly with namespace context
            // Extract namespace from the transparent struct itself, not the parent
            let transparent_namespace = extract_namespace_from_shape(field_shape);

            let inner_field = inner_struct.fields[0];
            let inner_field_shape = inner_field.shape();

            // Check if the inner type is a user-defined type that needs namespace-aware naming
            let inner_format = if let Type::User(UserType::Struct(_) | UserType::Enum(_)) =
                &inner_field_shape.ty
            {
                let namespaced_name = get_name(inner_field_shape);
                Format::TypeName(namespaced_name)
            } else {
                get_inner_format(inner_field_shape)
            };

            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: inner_format,
                });
            }

            // Process the inner type with the transparent struct's namespace context
            process_type(
                inner_field_shape,
                transparent_namespace.as_deref(),
                registry,
            );

            return true;
        }
    }

    false
}

fn format_enum(
    enum_type: &EnumType,
    shape: &Shape,
    parent_namespace: Option<&str>,
    registry: &mut State,
) {
    let enum_name = get_name(shape);

    // Check if already processed using the full namespaced name
    if registry.is_processed(&enum_name) {
        return;
    }

    // Register name mapping if it's different from original
    if enum_name.name != shape.type_identifier {
        let name = QualifiedTypeName {
            namespace: enum_name.namespace.clone(),
            name: shape.type_identifier.to_string(),
        };
        registry.register_type_mapping(name, enum_name.clone());
    }

    registry.mark_processed(enum_name.clone());

    let variants = process_enum_variants(enum_type, shape, parent_namespace, registry);

    let container = ContainerFormat::Enum(variants);
    registry.push(enum_name, container);
    registry.pop();
}

fn process_enum_variants(
    enum_type: &EnumType,
    shape: &Shape,
    parent_namespace: Option<&str>,
    registry: &mut State,
) -> BTreeMap<u32, Named<VariantFormat>> {
    let mut variants = BTreeMap::new();
    let mut variant_index = 0u32;

    for variant in enum_type.variants {
        let skip = variant.attributes.iter().any(|attr| match attr {
            VariantAttribute::Arbitrary(attr_str) => *attr_str == "skip",
        });
        if skip {
            continue;
        }

        let variant_format = process_single_variant(variant, shape, parent_namespace, registry);

        variants.insert(
            variant_index,
            Named {
                name: variant.name.to_string(),
                value: variant_format,
            },
        );
        variant_index += 1;
    }

    variants
}

fn process_single_variant(
    variant: &Variant,
    shape: &Shape,
    parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    if variant.data.fields.is_empty() {
        // Unit variant
        VariantFormat::Unit
    } else if variant.data.fields.len() == 1 {
        // Check if it's a struct variant (named field) or tuple variant (numeric field name)
        let field = variant.data.fields[0];
        let is_struct_variant = !field.name.chars().all(|c| c.is_ascii_digit());

        if is_struct_variant {
            process_struct_variant(variant, shape, parent_namespace, registry)
        } else {
            process_newtype_variant(variant, shape, parent_namespace, registry)
        }
    } else {
        process_multi_field_variant(variant, shape, parent_namespace, registry)
    }
}

fn process_newtype_variant(
    variant: &Variant,
    shape: &Shape,
    parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    let field = variant.data.fields[0];
    let field_shape = field.shape();

    if field_shape.type_identifier == "()" {
        VariantFormat::NewType(Box::new(Format::Unit))
    } else if let Type::User(UserType::Struct(_) | UserType::Enum(_)) = &field_shape.ty {
        // For user-defined struct/enum types, create a TypeName reference and process the type
        let current_namespace = extract_namespace_from_shape(shape);
        process_type(field_shape, current_namespace.as_deref(), registry);
        let namespaced_name = get_name(field_shape);
        VariantFormat::NewType(Box::new(Format::TypeName(namespaced_name)))
    } else {
        // For other types, use the temporary container approach
        process_newtype_variant_with_temp_container(
            variant,
            field_shape,
            shape,
            parent_namespace,
            registry,
        )
    }
}

fn process_newtype_variant_with_temp_container(
    variant: &Variant,
    field_shape: &Shape,
    shape: &Shape,
    _parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    let temp = registry.push_temporary(
        variant.name.to_string(),
        ContainerFormat::NewTypeStruct(Box::default()),
    );

    // Process the field to determine its format
    let current_namespace = extract_namespace_from_shape(shape);
    process_type(field_shape, current_namespace.as_deref(), registry);

    // Extract the format from the temporary container
    let variant_format = if let Some(ContainerFormat::NewTypeStruct(inner_format)) =
        registry.containers.get(&temp)
    {
        VariantFormat::NewType(inner_format.clone())
    } else {
        VariantFormat::Unit
    };

    // Clean up the temporary container
    registry.containers.remove(&temp);
    registry.pop();

    variant_format
}

fn process_multi_field_variant(
    variant: &Variant,
    shape: &Shape,
    parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    // Check if it's a struct variant (named fields) or tuple variant
    let first_field = variant.data.fields[0];
    let is_struct_variant = !first_field.name.chars().all(|c| c.is_ascii_digit());

    if is_struct_variant {
        process_struct_variant(variant, shape, parent_namespace, registry)
    } else {
        process_tuple_variant(variant, shape, parent_namespace, registry)
    }
}

fn process_struct_variant(
    variant: &Variant,
    shape: &Shape,
    _parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    let temp = registry.push_temporary(variant.name.to_string(), ContainerFormat::Struct(vec![]));

    // Process all fields with their names
    for field in variant.data.fields {
        let field_shape = field.shape();

        // Check if the field is a user-defined struct
        if let Type::User(UserType::Struct(_)) = &field_shape.ty {
            // Compute the value before the mutable borrow
            let value = if field_shape.type_identifier == "()" {
                Format::Unit
            } else {
                let _current_namespace = extract_namespace_from_shape(shape);
                let namespaced_name = get_name(field_shape);
                Format::TypeName(namespaced_name)
            };

            // Add Named TypeName format to the struct
            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value,
                });
            }
            // Process the inner type to add it to the registry (skip for unit type)
            if field_shape.type_identifier != "()" {
                let current_namespace = extract_namespace_from_shape(shape);
                process_type(field_shape, current_namespace.as_deref(), registry);
            }
        } else {
            // For non-struct types, add unknown format and let format() fill it
            if let Some(ContainerFormat::Struct(named_formats)) = registry.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    value: Format::unknown(),
                });
            }
            let current_namespace = extract_namespace_from_shape(shape);
            process_type(field_shape, current_namespace.as_deref(), registry);
        }
    }

    // Extract the formats from the temporary container
    let variant_format =
        if let Some(ContainerFormat::Struct(named_formats)) = registry.containers.get(&temp) {
            VariantFormat::Struct(named_formats.clone())
        } else {
            VariantFormat::Unit
        };

    // Clean up the temporary container
    registry.containers.remove(&temp);
    registry.pop();

    variant_format
}

fn process_tuple_variant(
    variant: &Variant,
    shape: &Shape,
    _parent_namespace: Option<&str>,
    registry: &mut State,
) -> VariantFormat {
    let temp = registry.push_temporary(
        variant.name.to_string(),
        ContainerFormat::TupleStruct(vec![]),
    );

    // Process all fields
    for field in variant.data.fields {
        // Use the current enum's namespace context for its variant fields
        let current_namespace = extract_namespace_from_shape(shape);
        process_type(field.shape(), current_namespace.as_deref(), registry);
    }

    // Extract the formats from the temporary container
    let variant_format =
        if let Some(ContainerFormat::TupleStruct(formats)) = registry.containers.get(&temp) {
            VariantFormat::Tuple(formats.clone())
        } else {
            VariantFormat::Unit
        };

    // Clean up the temporary container
    registry.containers.remove(&temp);
    registry.pop();

    variant_format
}

fn format_list(list_def: ListDef, namespace: Option<&str>, registry: &mut State) {
    // Get the inner type of the list
    let inner_shape = list_def.t();

    // Get the format for the inner type recursively
    let inner_format = get_inner_format(inner_shape);
    let seq_format = Format::Seq(Box::new(inner_format));

    // Update the current container with the sequence format
    update_container_format(seq_format, registry);

    // Process any user-defined types in the nested structure
    process_nested_types(inner_shape, namespace, registry);
}

fn format_map(map_def: MapDef, namespace: Option<&str>, registry: &mut State) {
    // Get the key and value types of the map
    let key_shape = map_def.k();
    let value_shape = map_def.v();

    // Get the formats for key and value types
    let key_format = get_inner_format(key_shape);
    let value_format = get_inner_format(value_shape);

    let map_format = Format::Map {
        key: Box::new(key_format),
        value: Box::new(value_format),
    };

    // Update the current container with the map format
    update_container_format(map_format, registry);

    // Process any user-defined types in the nested structure
    process_nested_types(key_shape, namespace, registry);
    process_nested_types(value_shape, namespace, registry);
}

fn format_slice(slice_def: SliceDef, namespace: Option<&str>, registry: &mut State) {
    process_type(slice_def.t(), namespace, registry);
}

fn format_array(array_def: ArrayDef, namespace: Option<&str>, registry: &mut State) {
    // Get the inner type and size of the array
    let inner_shape = array_def.t();
    let array_size = array_def.n;

    // Determine the format for the inner type
    let inner_format = get_inner_format(inner_shape);

    let array_format = Format::TupleArray {
        content: Box::new(inner_format),
        size: array_size,
    };

    // Update the current container with the array format
    update_container_format(array_format, registry);

    // If the inner type is a user-defined type, we need to process it too
    if !matches!(inner_shape.def, Def::Scalar) {
        process_type(inner_shape, namespace, registry);
    }
}

fn format_option(option_def: OptionDef, namespace: Option<&str>, registry: &mut State) {
    // Get the inner type of the Option
    let inner_shape = option_def.t();

    // We need to determine what format to use for the Option based on the inner type
    let inner_format = get_inner_format(inner_shape);
    let option_format = Format::Option(Box::new(inner_format));

    // Update the current container with the option format
    update_container_format(option_format, registry);

    // Also process nested types
    process_nested_types(inner_shape, namespace, registry);
}

fn get_name(shape: &Shape) -> QualifiedTypeName {
    // Check type_tag first (is this where facet rename is stored?)
    if let Some(type_tag) = shape.type_tag {
        return QualifiedTypeName::root(type_tag.to_string());
    }

    // Check attributes for namespace and name
    let mut shape_namespace = None;
    let mut name = None;

    for attr in shape.attributes {
        if let ShapeAttribute::Arbitrary(attr_str) = attr {
            // Check for namespace attribute
            if let Some(stripped) = attr_str.strip_prefix("namespace = \"") {
                if let Some(end_idx) = stripped.find('"') {
                    shape_namespace = Some(stripped[..end_idx].to_string());
                }
            } else if let Some(stripped) = attr_str.strip_prefix("namespace = ") {
                shape_namespace = Some(stripped.trim_matches('"').to_string());
            }
            // Check for rename attribute in the format "name = \"NewName\""
            else if let Some(stripped) = attr_str.strip_prefix("name = \"") {
                if let Some(end_idx) = stripped.find('"') {
                    name = Some(stripped[..end_idx].to_string());
                }
            }
            // Check for rename attribute without quotes "name = NewName"
            else if let Some(stripped) = attr_str.strip_prefix("name = ") {
                name = Some(stripped.trim_matches('"').to_string());
            }
        }
    }

    // Determine the base name
    let base_name = if let Some(custom_name) = name {
        custom_name
    } else {
        shape.type_identifier.to_string()
    };

    // Apply namespace - only use explicit namespace annotations, no inheritance
    if let Some(ns) = shape_namespace {
        QualifiedTypeName::namespaced(ns, base_name)
    } else {
        QualifiedTypeName::root(base_name)
    }
}

fn extract_namespace_from_shape(shape: &Shape) -> Option<String> {
    for attr in shape.attributes {
        if let ShapeAttribute::Arbitrary(attr_str) = attr {
            if let Some(stripped) = attr_str.strip_prefix("namespace = \"") {
                if let Some(end_idx) = stripped.find('"') {
                    return Some(stripped[..end_idx].to_string());
                }
            } else if let Some(stripped) = attr_str.strip_prefix("namespace = ") {
                return Some(stripped.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn is_transparent_struct(shape: &Shape) -> bool {
    shape.attributes.iter().any(|attr| match attr {
        ShapeAttribute::Transparent => true,
        ShapeAttribute::Arbitrary(attr_str) => *attr_str == "transparent",
        _ => false,
    })
}

#[derive(Clone, Copy)]
enum UpdateMode {
    /// Unconditionally update the container format
    Force,
    /// Only update if the current format is unknown
    IfUnknown,
    /// Only update unknown formats, but don't add to `TupleStruct` (for mutual recursion)
    MutualRecursion,
}

fn update_container_format(format: Format, registry: &mut State) {
    update_container_format_with_mode(format, registry, UpdateMode::Force);
}

fn update_container_format_if_unknown(format: Format, registry: &mut State) {
    update_container_format_with_mode(format, registry, UpdateMode::IfUnknown);
}

fn update_container_format_for_mutual_recursion(format: Format, registry: &mut State) {
    update_container_format_with_mode(format, registry, UpdateMode::MutualRecursion);
}

fn update_container_format_with_mode(format: Format, registry: &mut State, mode: UpdateMode) {
    if let Some(container_format) = registry.get_mut() {
        match container_format {
            ContainerFormat::UnitStruct => {}
            ContainerFormat::NewTypeStruct(inner_format) => match mode {
                UpdateMode::Force => {
                    **inner_format = format;
                }
                UpdateMode::IfUnknown | UpdateMode::MutualRecursion => {
                    if inner_format.is_unknown() {
                        **inner_format = format;
                    }
                }
            },
            ContainerFormat::TupleStruct(formats) => {
                match mode {
                    UpdateMode::Force | UpdateMode::IfUnknown => {
                        formats.push(format);
                    }
                    UpdateMode::MutualRecursion => {
                        // For mutual recursion, don't add duplicate entries to TupleStruct
                        // They should already have the proper format from initial processing
                    }
                }
            }
            ContainerFormat::Struct(fields) => {
                if let Some(last_named) = fields.last_mut() {
                    match mode {
                        UpdateMode::Force => {
                            // Even in Force mode, struct fields are only updated if unknown
                            // This preserves the original behavior where struct fields are set
                            // when first processed and shouldn't be overwritten later
                            if last_named.value.is_unknown() {
                                last_named.value = format;
                            }
                        }
                        UpdateMode::IfUnknown | UpdateMode::MutualRecursion => {
                            if last_named.value.is_unknown() {
                                last_named.value = format;
                            }
                        }
                    }
                }
            }
            ContainerFormat::Enum(_) => {
                if matches!(mode, UpdateMode::Force) {
                    todo!("Enum container format update not implemented");
                }
            }
        }
    }
}

fn should_process_nested_type(shape: &Shape) -> bool {
    !matches!(shape.def, Def::Scalar) && shape.type_identifier != "()"
}

fn get_inner_format(shape: &Shape) -> Format {
    match shape.def {
        Def::Scalar => type_to_format(shape),
        Def::List(inner_list_def) => {
            // Recursively handle nested lists
            let inner_shape = inner_list_def.t();
            Format::Seq(Box::new(get_inner_format(inner_shape)))
        }
        Def::Option(option_def) => {
            // Handle Option<T> -> OPTION: T
            let inner_shape = option_def.t();
            let inner_format = get_inner_format(inner_shape);
            Format::Option(Box::new(inner_format))
        }
        Def::Map(map_def) => {
            // Handle Map<K, V> -> MAP: { KEY: K, VALUE: V }
            let key_shape = map_def.k();
            let value_shape = map_def.v();
            let key_format = get_inner_format(key_shape);
            let value_format = get_inner_format(value_shape);
            Format::Map {
                key: Box::new(key_format),
                value: Box::new(value_format),
            }
        }
        Def::Array(array_def) => {
            // Handle Array<T, N> -> TUPLEARRAY: { CONTENT: T, SIZE: N }
            let inner_shape = array_def.t();
            let inner_format = get_inner_format(inner_shape);
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
                        let field_format = get_inner_format(field_shape);
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
                let name = get_name(shape);

                Format::TypeName(name)
            }
        }
        Def::Set(_set_def) => todo!(),
        Def::Slice(slice_def) => {
            // Handle Slice<T> -> SEQ: T
            let inner_shape = slice_def.t();
            Format::Seq(Box::new(get_inner_format(inner_shape)))
        }
        Def::Pointer(pointer_def) => {
            // Handle Pointer (Box, Arc, etc.) by recursively processing the inner type
            if let Some(inner_shape) = pointer_def.pointee {
                get_inner_format(inner_shape())
            } else {
                // If no pointee, treat as unit
                Format::Unit
            }
        }
    }
}

fn process_nested_types(shape: &Shape, namespace: Option<&str>, registry: &mut State) {
    match shape.def {
        Def::Scalar => {
            // Scalar types don't need further processing
        }
        Def::List(inner_list_def) => {
            // Recursively process nested lists
            let inner_shape = inner_list_def.t();
            process_nested_types(inner_shape, namespace, registry);
        }
        Def::Option(option_def) => {
            // Recursively process options
            let inner_shape = option_def.t();
            if should_process_nested_type(inner_shape) {
                process_nested_types(inner_shape, namespace, registry);
            }
        }
        Def::Map(map_def) => {
            // Recursively process maps
            let key_shape = map_def.k();
            let value_shape = map_def.v();
            if should_process_nested_type(key_shape) {
                process_nested_types(key_shape, namespace, registry);
            }
            if should_process_nested_type(value_shape) {
                process_nested_types(value_shape, namespace, registry);
            }
        }
        _ => {
            // For other user-defined types, process them
            if should_process_nested_type(shape) {
                process_type(shape, namespace, registry);
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "namespace_tests.rs"]
mod namespace_tests;
