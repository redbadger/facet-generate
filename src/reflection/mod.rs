pub mod format;
#[cfg(test)]
pub mod regression_tests;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    string::ToString,
    sync::LazyLock,
};

use facet::{
    ArrayDef, Def, EnumType, Facet, Field, FieldAttribute, ListDef, MapDef, NumericType, OptionDef,
    PointerDef, PointerType, PrimitiveType, SequenceType, SetDef, Shape, ShapeAttribute, SliceDef,
    StructKind, StructType, TextualType, Type, UserType, Variant, VariantAttribute,
};
use regex::Regex;

use crate::Registry;
use format::{
    ContainerFormat, Format, FormatHolder, Named, Namespace, QualifiedTypeName, VariantFormat,
};

const SUPPORTED_GENERIC_TYPES: [&str; 10] = [
    "Arc", "Rc", "Box", "Option", "Vec", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "DateTime",
];

/// A namespace context with its source information
#[derive(Debug, Clone, PartialEq)]
struct NamespaceContext {
    namespace: Namespace,
    explicit: bool, // true = explicitly set, false = inherited
}

impl NamespaceContext {
    /// Creates a cleared namespace context (explicit root)
    fn cleared() -> Self {
        Self {
            namespace: Namespace::Root,
            explicit: true,
        }
    }

    /// Creates an explicitly set namespace context
    fn explicit(namespace: Namespace) -> Self {
        Self {
            namespace,
            explicit: true,
        }
    }

    /// Checks if this context is explicit
    fn is_explicit(&self) -> bool {
        self.explicit
    }

    /// Checks if this context was cleared (explicit root)
    fn is_cleared(&self) -> bool {
        self.explicit && matches!(self.namespace, Namespace::Root)
    }
}

/// Action to take with namespace context
#[derive(Debug, Clone, PartialEq)]
enum NamespaceAction {
    /// Set a specific namespace context
    SetContext(NamespaceContext),
    /// Inherit the current context unchanged (push current context again)
    Inherit,
}

impl NamespaceAction {
    /// Returns true if this action represents an explicit namespace setting
    fn is_explicit(&self) -> bool {
        match self {
            NamespaceAction::SetContext(ctx) => ctx.is_explicit(),
            NamespaceAction::Inherit => false, // Inheriting is not explicit
        }
    }

    /// Returns true if this action should move a type to a specific namespace
    fn should_move_to_namespace(&self, target_namespace: &str) -> bool {
        match self {
            NamespaceAction::SetContext(ctx) if ctx.is_explicit() => {
                match &ctx.namespace {
                    Namespace::Named(type_ns) => {
                        // Type has explicit namespace - only move if it matches the target namespace
                        type_ns == target_namespace
                    }
                    Namespace::Root => {
                        // Type explicitly wants to be in root - don't move to named namespace
                        target_namespace.is_empty() // This would be unusual but handle it
                    }
                }
            }
            NamespaceAction::SetContext(_) | NamespaceAction::Inherit => {
                // No type-level explicit annotation - field-level can override
                true
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct RegistryBuilder {
    pub registry: Registry,
    current: Vec<QualifiedTypeName>,
    processed: HashSet<QualifiedTypeName>,
    name_mappings: BTreeMap<QualifiedTypeName, QualifiedTypeName>,
    generic_type_params: HashMap<String, String>,
    processing_nested: bool,
    namespace_context_stack: Vec<NamespaceContext>,
    type_namespace_sources: HashMap<QualifiedTypeName, bool>, // true = explicit, false = inherited
}

impl RegistryBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the registry from the current state.
    /// # Panics
    /// Will panic with a suitable error message if the registry is invalid,
    /// usually due to incomplete reflection.
    #[must_use]
    pub fn build(self) -> Registry {
        for (type_name, format) in &self.registry {
            if let Err(err) = format.visit(&mut |_| Ok(())) {
                panic!("There was a problem reflecting type '{type_name:?}': {err}");
            }
        }

        self.registry
    }

    /// Reflect a type into the registry.
    #[must_use]
    pub fn add_type<'a, T: Facet<'a>>(mut self) -> Self {
        self.format(T::SHAPE);
        self
    }
}

impl RegistryBuilder {
    fn push(&mut self, name: QualifiedTypeName, container: ContainerFormat) {
        self.registry.insert(name.clone(), container);
        self.current.push(name);
    }

    fn push_with_type_check(
        &mut self,
        name: QualifiedTypeName,
        container: ContainerFormat,
        shape: &Shape,
    ) {
        let is_explicit = extract_namespace_from_shape(shape).is_explicit();

        // Check for conflicts: type name in multiple namespaces with mixed explicit/inherited sources
        self.check_namespace_ambiguity(&name, is_explicit);

        // Record this type name -> namespace source mapping
        self.type_namespace_sources
            .insert(name.clone(), is_explicit);

        self.registry.insert(name.clone(), container);
        self.current.push(name);
    }

    fn push_temporary(
        &mut self,
        name: String,
        container: ContainerFormat,
        parent_context: Option<&Shape>,
    ) -> QualifiedTypeName {
        let temp_name = if let Some(parent) = parent_context {
            let parent_name = parent.type_identifier.replace(['<', '>', ' ', ','], "_");
            format!("{name}__in__{parent_name}")
        } else {
            name
        };
        let qualified_name = QualifiedTypeName {
            namespace: Namespace::Named("__temp__".to_string()),
            name: temp_name,
        };

        self.push(qualified_name.clone(), container);
        qualified_name
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
            self.registry.get_mut(name)
        } else {
            None
        }
    }

    fn format_with_namespace_override(&mut self, shape: &Shape, namespace: &str) {
        let base_name = shape.type_identifier.to_string();
        let namespaced_key =
            QualifiedTypeName::namespaced(namespace.to_string(), base_name.clone());

        if !self.registry.contains_key(&namespaced_key) {
            // Store the previous namespace context
            let context = NamespaceContext::explicit(Namespace::Named(namespace.to_string()));
            self.push_namespace(NamespaceAction::SetContext(context));

            // Process the type with the namespace context so nested types inherit the namespace
            self.format(shape);

            // Restore the previous namespace context
            self.pop_namespace();

            // Check if the type has a conflicting type-level explicit namespace
            // If type-level explicit matches what we want, or if no type-level explicit, then move is OK
            let original_key = self.get_name_with_mappings(shape);

            if original_key != namespaced_key {
                let type_level_namespace = extract_namespace_from_shape(shape);

                let should_move = type_level_namespace.should_move_to_namespace(namespace);

                if should_move {
                    if let Some(format) = self.registry.remove(&original_key) {
                        self.registry.insert(namespaced_key.clone(), format);
                    }
                }
            }
        }
    }

    fn format(&mut self, shape: &Shape) {
        assert!(
            self.is_supported_generic_type(shape),
            "Unsupported generic type: {shape}, the type may have already been used with different parameters"
        );

        // First check for special cases in the def system (like Option)
        if let Def::Option(option_def) = shape.def {
            self.format_option(option_def);
            return;
        }

        // Try type system first
        if self.try_format_from_type_system(shape) {
            return;
        }

        // Fall back to def system
        self.format_from_def_system(shape);
    }

    fn is_supported_generic_type(&mut self, shape: &Shape) -> bool {
        if shape.type_params.is_empty()
            || shape.type_identifier.starts_with('&')
            || SUPPORTED_GENERIC_TYPES.contains(&shape.type_identifier)
        {
            return true;
        }

        let current = format!("{:?}", shape.type_params);
        let previous = self
            .generic_type_params
            .entry(shape.type_identifier.to_string())
            .or_insert(current.clone());
        previous == &current
    }

    fn try_format_from_type_system(&mut self, shape: &Shape) -> bool {
        match &shape.ty {
            Type::User(UserType::Struct(struct_def)) => {
                self.handle_user_struct(shape, struct_def);
                true
            }
            Type::User(UserType::Enum(enum_def)) => {
                self.format_enum(enum_def, shape);
                true
            }
            Type::Sequence(sequence_type) => self.handle_sequence_type(shape, sequence_type),
            _ => false,
        }
    }

    fn handle_user_struct(&mut self, shape: &Shape, struct_def: &StructType) {
        let type_name = self.get_name_with_mappings(shape);

        // Update container with the struct format only if not processing nested types
        if !self.processing_nested {
            let format = if shape.type_identifier == "()" {
                Format::Unit
            } else {
                Format::TypeName(type_name.clone())
            };

            self.update_container_format(format, UpdateMode::IfUnknown);
        }

        self.format_struct(struct_def, shape);
    }

    fn handle_sequence_type(&mut self, shape: &Shape, sequence_type: &SequenceType) -> bool {
        match sequence_type {
            SequenceType::Slice(slice_type) => {
                // For slices, use the Def::Slice if available
                if let Def::Slice(slice_def) = shape.def {
                    self.format_slice(slice_def);
                    true
                } else {
                    // Fallback: create a slice format from the sequence type info
                    let target_shape = slice_type.t;
                    let inner_format = get_inner_format(target_shape);
                    let slice_format = Format::Seq(Box::new(inner_format));
                    self.update_container_format(slice_format, UpdateMode::Force);
                    self.process_nested_types(target_shape);
                    true
                }
            }
            SequenceType::Array(array_type) => {
                // For arrays, use the Def::Array if available
                if let Def::Array(array_def) = shape.def {
                    self.format_array(array_def);
                    true
                } else {
                    // Fallback: create an array format from the sequence type info
                    let target_shape = array_type.t;
                    let inner_format = get_inner_format(target_shape);
                    let array_format = Format::Seq(Box::new(inner_format)); // Arrays are also sequences
                    self.update_container_format(array_format, UpdateMode::Force);
                    self.process_nested_types(target_shape);
                    true
                }
            }
        }
    }

    fn format_from_def_system(&mut self, shape: &Shape) {
        match shape.def {
            Def::Scalar => self.format_scalar(shape),
            Def::Map(map_def) => self.format_map(map_def),
            Def::List(list_def) => self.format_list(list_def),
            Def::Slice(slice_def) => self.format_slice(slice_def),
            Def::Array(array_def) => self.format_array(array_def),
            Def::Set(set_def) => self.format_set(set_def),
            Def::Option(option_def) => self.format_option(option_def),
            Def::Pointer(PointerDef {
                pointee: Some(inner_shape),
                ..
            }) => {
                self.handle_pointer(inner_shape());
            }
            Def::Pointer(PointerDef { pointee: None, .. }) => {
                self.handle_opaque_pointee();
            }
            Def::Undefined => {
                self.handle_undefined_def(shape);
            }
            _ => (),
        }
    }

    fn handle_pointer(&mut self, inner_shape: &Shape) {
        // For Pointer, we need to update the current container with the inner type's format
        let inner_format = get_format_for_shape(inner_shape);

        // Update the current container with the Pointer's inner format
        self.update_container_format(inner_format, UpdateMode::IfUnknown);

        // Also process the inner type if it's a user-defined type
        self.process_nested_types(inner_shape);
    }

    fn handle_undefined_def(&mut self, shape: &Shape) {
        // Handle the case when not yet migrated to the Type enum
        // For primitives, we can try to infer the type
        match &shape.ty {
            Type::Primitive(primitive) => match primitive {
                PrimitiveType::Boolean => {
                    let format = Format::Bool;
                    self.update_container_format(format, UpdateMode::Force);
                }
                PrimitiveType::Numeric(NumericType::Float) => {
                    let format = Format::F32; // or F64, but F32 is more common
                    self.update_container_format(format, UpdateMode::Force);
                }
                PrimitiveType::Textual(TextualType::Str) => {
                    let format = Format::Str;
                    self.update_container_format(format, UpdateMode::Force);
                }
                p => {
                    unimplemented!("Unknown primitive type: {p:?}");
                }
            },
            Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) => {
                self.format((pt.target)());
            }
            _ => {}
        }
    }

    fn format_scalar(&mut self, shape: &Shape) {
        let format = type_to_format(shape);
        self.update_container_format(format, UpdateMode::Force);
    }

    fn format_struct(&mut self, struct_type: &StructType, shape: &Shape) {
        let struct_name = self.get_name_with_mappings(shape);

        // Check if already processed using the full namespaced name
        if self.is_processed(&struct_name) {
            // This is a mutual recursion case - only update if there's an unknown format that needs updating
            let format = Format::TypeName(struct_name.clone());
            self.update_container_format(format, UpdateMode::MutualRecursion);
            return;
        }

        // Register name mapping if it's different from original
        if struct_name.name != shape.type_identifier {
            let name = QualifiedTypeName {
                namespace: struct_name.namespace.clone(),
                name: shape.type_identifier.to_string(),
            };
            self.register_type_mapping(name, struct_name.clone());
        }

        self.mark_processed(struct_name.clone());

        // Extract namespace from this enum if it has one
        let type_level_namespace = extract_namespace_from_shape(shape);

        // Handle namespace context for the enum processing
        self.push_namespace(type_level_namespace);

        match struct_type.kind {
            StructKind::Unit => {
                self.push_with_type_check(
                    struct_name.clone(),
                    ContainerFormat::UnitStruct(shape.into()),
                    shape,
                );
                self.pop();
            }
            StructKind::TupleStruct => {
                if struct_type.fields.len() == 1 {
                    let field = struct_type.fields[0];
                    let field_shape = field.shape();

                    // Check if this is a transparent struct
                    let is_transparent = is_transparent_struct(shape);

                    if is_transparent {
                        // For transparent structs, don't create a container - just process the inner type
                        // This will register the transparent struct with its inner type's format
                        self.format(field_shape);
                        self.pop_namespace();
                        return;
                    }

                    // Handle regular newtype struct
                    let container = ContainerFormat::NewTypeStruct(Box::default(), shape.into());
                    self.push_with_type_check(struct_name.clone(), container, shape);

                    // Process the inner field
                    self.format(field_shape);

                    self.pop();
                } else {
                    // Handle tuple struct with multiple fields
                    let container = ContainerFormat::TupleStruct(vec![], shape.into());
                    self.push_with_type_check(struct_name.clone(), container, shape);
                    for field in struct_type.fields {
                        let skip = field.attributes.iter().any(|attr| match attr {
                            FieldAttribute::Arbitrary(attr_str) => *attr_str == "skip",
                        });
                        if skip {
                            continue;
                        }
                        self.format(field.shape());
                    }
                    self.pop();
                }
            }
            StructKind::Struct => {
                let container = ContainerFormat::Struct(vec![], shape.into());
                self.push_with_type_check(struct_name.clone(), container, shape);
                for field in struct_type.fields {
                    let skip = field.attributes.iter().any(|attr| match attr {
                        FieldAttribute::Arbitrary(attr_str) => *attr_str == "skip",
                    });
                    if skip {
                        continue;
                    }
                    self.handle_struct_field(field);
                }
                self.pop();
            }
            StructKind::Tuple => {
                // This handles standalone tuple types, but for tuple fields in structs,
                // they are handled in the StructKind::Struct case above
            }
        }

        self.pop_namespace();
    }

    fn handle_struct_field(&mut self, field: &Field) {
        let field_shape = field.shape();

        // Check for field-level attributes first
        let has_bytes_attribute = field.attributes.iter().any(|attr| match attr {
            FieldAttribute::Arbitrary(attr_str) => *attr_str == "bytes",
        });

        if has_bytes_attribute && self.try_handle_bytes_attribute(field) {
            return;
        }

        if self.try_handle_option_field(field) {
            return;
        }

        if self.try_handle_tuple_struct_field(field) {
            return;
        }

        // Check for field-level namespace annotation
        let field_namespace = extract_namespace_from_field_attributes(field);

        self.push_namespace(field_namespace.clone());

        // Now determine the proper format with the field-level context in place
        let field_format = self.get_user_type_format(field_shape);

        // Process the type under the field-level namespace context
        if let NamespaceAction::SetContext(ctx) = &field_namespace {
            if ctx.is_explicit() {
                if let Namespace::Named(name) = &ctx.namespace {
                    self.format_with_namespace_override(field_shape, name);
                } else {
                    self.format(field_shape);
                }
            } else {
                self.format(field_shape);
            }
        } else {
            self.format(field_shape);
        }

        self.pop_namespace();

        if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
            let format = Named {
                name: field.name.to_string(),
                doc: field.into(),
                value: field_format,
            };
            named_formats.push(format);
        }
    }

    fn try_handle_bytes_attribute(&mut self, field: &Field) -> bool {
        let field_shape = field.shape();
        // Handle bytes attribute for Vec<u8>
        if field_shape.type_identifier == "Vec" {
            // Check if it's actually Vec<u8> by examining the definition
            if let Def::List(list_def) = field_shape.def {
                let inner_shape = list_def.t();
                if inner_shape.type_identifier == "u8" {
                    if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                        named_formats.push(Named {
                            name: field.name.to_string(),
                            doc: field.shape().into(),
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
                        if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                            named_formats.push(Named {
                                name: field.name.to_string(),
                                doc: field.into(),
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

    fn try_handle_option_field(&mut self, field: &Field) -> bool {
        let field_shape = field.shape();
        // Check if the field is an Option
        if field_shape.type_identifier == "Option" {
            if let Def::Option(option_def) = field_shape.def {
                // Handle Option types directly
                let inner_shape = option_def.t();
                // Handle pointer types specially
                let inner_format = get_format_for_shape(inner_shape);
                let option_format = Format::Option(Box::new(inner_format));

                if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                    named_formats.push(Named {
                        name: field.name.to_string(),
                        doc: field.into(),
                        value: option_format,
                    });
                }

                // If the inner type is a user-defined type, we need to process it too
                if !matches!(inner_shape.def, Def::Scalar) {
                    self.format(inner_shape);
                }
                return true;
            }
        }
        false
    }

    fn try_handle_tuple_struct_field(&mut self, field: &Field) -> bool {
        let field_shape = field.shape();
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

                if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                    let tuple_format = if tuple_formats.is_empty() {
                        Format::Unit
                    } else {
                        Format::Tuple(tuple_formats)
                    };
                    named_formats.push(Named {
                        name: field.name.to_string(),
                        doc: field.into(),
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
                    let namespaced_name = self.get_name_with_mappings(inner_field_shape);
                    Format::TypeName(namespaced_name)
                } else {
                    get_inner_format(inner_field_shape)
                };

                if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                    named_formats.push(Named {
                        name: field.name.to_string(),
                        doc: field.into(),
                        value: inner_format,
                    });
                }

                // Process the inner type with the transparent struct's namespace context
                self.push_namespace(transparent_namespace);
                self.format(inner_field_shape);
                self.pop_namespace();

                return true;
            }
        }

        false
    }

    fn format_enum(&mut self, enum_type: &EnumType, shape: &Shape) {
        let enum_name = self.get_name_with_mappings(shape);

        // Check if already processed using the full namespaced name
        if self.is_processed(&enum_name) {
            return;
        }

        // Register name mapping if it's different from original
        if enum_name.name != shape.type_identifier {
            let name = QualifiedTypeName {
                namespace: enum_name.namespace.clone(),
                name: shape.type_identifier.to_string(),
            };
            self.register_type_mapping(name, enum_name.clone());
        }

        self.mark_processed(enum_name.clone());

        // Extract namespace from this enum if it has one
        let type_level_namespace = extract_namespace_from_shape(shape);

        // Handle namespace context for the enum processing
        self.push_namespace(type_level_namespace);

        let variants = self.process_enum_variants(enum_type, shape);
        let container = ContainerFormat::Enum(variants, shape.into());
        self.push_with_type_check(enum_name.clone(), container, shape);
        self.pop();

        self.pop_namespace();
    }

    fn process_enum_variants(
        &mut self,
        enum_type: &EnumType,
        shape: &Shape,
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

            let variant_format = self.process_single_variant(variant, shape);

            variants.insert(
                variant_index,
                Named {
                    name: variant.name.to_string(),
                    doc: variant.into(),
                    value: variant_format,
                },
            );
            variant_index += 1;
        }

        variants
    }

    fn process_single_variant(&mut self, variant: &Variant, shape: &Shape) -> VariantFormat {
        if variant.data.fields.is_empty() {
            // Unit variant
            VariantFormat::Unit
        } else if variant.data.fields.len() == 1 {
            // Check if it's a struct variant (named field) or tuple variant (numeric field name)
            let field = variant.data.fields[0];
            let is_struct_variant = !field.name.chars().all(|c| c.is_ascii_digit());

            if is_struct_variant {
                self.process_struct_variant(variant, shape)
            } else {
                self.process_newtype_variant(variant, shape)
            }
        } else {
            self.process_multi_field_variant(variant, shape)
        }
    }

    fn process_newtype_variant(&mut self, variant: &Variant, shape: &Shape) -> VariantFormat {
        let field = variant.data.fields[0];
        let field_shape = field.shape();

        if field_shape.type_identifier == "()" {
            VariantFormat::NewType(Box::new(Format::Unit))
        } else if let Type::User(UserType::Struct(_) | UserType::Enum(_)) = &field_shape.ty {
            // Check for field-level namespace annotation
            let field_namespace = extract_namespace_from_field_attributes(&field);

            match field_namespace {
                NamespaceAction::SetContext(ctx) if ctx.is_explicit() => {
                    // Process the type under the field-specified namespace and create qualified name
                    let base_name = field_shape.type_identifier.to_string();
                    let qualified_name = match &ctx.namespace {
                        Namespace::Root => {
                            let context = NamespaceContext::explicit(Namespace::Root);
                            self.push_namespace(NamespaceAction::SetContext(context));
                            self.format(field_shape);
                            self.pop_namespace();
                            QualifiedTypeName::root(base_name)
                        }
                        Namespace::Named(name) => {
                            let context =
                                NamespaceContext::explicit(Namespace::Named(name.clone()));
                            self.push_namespace(NamespaceAction::SetContext(context));
                            self.format(field_shape);
                            self.pop_namespace();
                            QualifiedTypeName::namespaced(name.clone(), base_name)
                        }
                    };
                    VariantFormat::NewType(Box::new(Format::TypeName(qualified_name)))
                }
                NamespaceAction::SetContext(_) | NamespaceAction::Inherit => {
                    // For user-defined struct/enum types, create a TypeName reference and process the type
                    self.format(field_shape);
                    let namespaced_name = self.get_name_with_mappings(field_shape);
                    VariantFormat::NewType(Box::new(Format::TypeName(namespaced_name)))
                }
            }
        } else {
            // For other types, use the temporary container approach
            self.process_newtype_variant_with_temp_container(variant, field_shape, shape)
        }
    }

    fn process_newtype_variant_with_temp_container(
        &mut self,
        variant: &Variant,
        field_shape: &Shape,
        _shape: &Shape,
    ) -> VariantFormat {
        let field = variant.data.fields[0];

        // Check for field-level namespace annotation
        let field_namespace = extract_namespace_from_field_attributes(&field);

        self.push_namespace(field_namespace);

        // Build the format directly with the namespace context in place
        let inner_format = get_inner_format_with_context(field_shape, self.current_namespace());

        // Also ensure the type itself gets processed with the namespace context
        self.format(field_shape);

        self.pop_namespace();

        VariantFormat::NewType(Box::new(inner_format))
    }

    fn process_multi_field_variant(&mut self, variant: &Variant, shape: &Shape) -> VariantFormat {
        // Check if it's a struct variant (named fields) or tuple variant
        let first_field = variant.data.fields[0];
        let is_struct_variant = !first_field.name.chars().all(|c| c.is_ascii_digit());

        if is_struct_variant {
            self.process_struct_variant(variant, shape)
        } else {
            self.process_tuple_variant(variant, shape)
        }
    }

    fn process_struct_variant(&mut self, variant: &Variant, shape: &Shape) -> VariantFormat {
        let temp = self.push_temporary(
            variant.name.to_string(),
            ContainerFormat::Struct(vec![], shape.into()),
            Some(shape),
        );

        // Process all fields with their names
        for field in variant.data.fields {
            let field_shape = field.shape();

            // Check for field-level namespace annotation
            let field_namespace = extract_namespace_from_field_attributes(field);

            self.push_namespace(field_namespace.clone());

            // Determine the proper format with the field-level context in place
            let value = self.get_user_type_format(field_shape);

            // Process the type under the field-level namespace context
            if let NamespaceAction::SetContext(ctx) = &field_namespace {
                if ctx.is_explicit() {
                    if let Namespace::Named(name) = &ctx.namespace {
                        self.format_with_namespace_override(field_shape, name);
                    } else {
                        self.format(field_shape);
                    }
                } else {
                    self.format(field_shape);
                }
            } else {
                self.format(field_shape);
            }

            self.pop_namespace();

            if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut() {
                named_formats.push(Named {
                    name: field.name.to_string(),
                    doc: field.into(),
                    value,
                });
            }
        }

        // Extract the formats from the temporary container
        let variant_format =
            if let Some(ContainerFormat::Struct(named_formats, _doc)) = self.registry.get(&temp) {
                VariantFormat::Struct(named_formats.clone())
            } else {
                VariantFormat::Unit
            };

        // Clean up the temporary container
        let _removed = self.registry.remove(&temp);

        self.pop();

        variant_format
    }

    fn process_tuple_variant(&mut self, variant: &Variant, shape: &Shape) -> VariantFormat {
        let temp = self.push_temporary(
            variant.name.to_string(),
            ContainerFormat::TupleStruct(vec![], shape.into()),
            Some(shape),
        );

        // Process all fields
        for field in variant.data.fields {
            // Use the current enum's namespace context for its variant fields
            let transparent_namespace = extract_namespace_from_shape(shape);

            self.push_namespace(transparent_namespace);
            self.format(field.shape());
            self.pop_namespace();
        }

        // Extract the formats from the temporary container
        let variant_format =
            if let Some(ContainerFormat::TupleStruct(formats, _doc)) = self.registry.get(&temp) {
                VariantFormat::Tuple(formats.clone())
            } else {
                VariantFormat::Unit
            };

        // Clean up the temporary container
        let _removed = self.registry.remove(&temp);

        self.pop();

        variant_format
    }

    fn format_list(&mut self, list_def: ListDef) {
        // Get the inner type of the list
        let inner_shape = list_def.t();

        // Get the format for the inner type recursively
        let inner_format = get_inner_format(inner_shape);
        let seq_format = Format::Seq(Box::new(inner_format));

        // Update the current container with the sequence format
        self.update_container_format(seq_format, UpdateMode::Force);

        // Process any user-defined types in the nested structure
        self.process_nested_types(inner_shape);
    }

    fn format_map(&mut self, map_def: MapDef) {
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
        self.update_container_format(map_format, UpdateMode::Force);

        // Process any user-defined types in the nested structure
        self.process_nested_types(key_shape);
        self.process_nested_types(value_shape);
    }

    fn format_slice(&mut self, slice_def: SliceDef) {
        // Get the inner type of the slice
        let inner_shape = slice_def.t();

        // Get the format for the inner type
        let inner_format = get_format_for_shape(inner_shape);

        let slice_format = Format::Seq(Box::new(inner_format));

        // Update the current container with the slice format
        self.update_container_format(slice_format, UpdateMode::Force);

        // Process any user-defined types in the nested structure
        self.process_nested_types(inner_shape);
    }

    fn format_array(&mut self, array_def: ArrayDef) {
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
        self.update_container_format(array_format, UpdateMode::Force);

        // If the inner type is a user-defined type, we need to process it too
        if !matches!(inner_shape.def, Def::Scalar) {
            self.format(inner_shape);
        }
    }

    fn format_option(&mut self, option_def: OptionDef) {
        // Get the inner type of the Option
        let inner_shape = option_def.t();

        // We need to determine what format to use for the Option based on the inner type
        let inner_format = get_format_for_shape(inner_shape);
        let option_format = Format::Option(Box::new(inner_format));

        // Update the current container with the option format
        self.update_container_format(option_format, UpdateMode::Force);

        // Process any user-defined types in the nested structure
        self.process_nested_types(inner_shape);
    }

    fn format_set(&mut self, set_def: SetDef) {
        // Get the element type of the Set
        let element_shape = set_def.t();

        // Get the format for the element type recursively
        let element_format = get_inner_format(element_shape);

        // Sets are represented as sets in the format system
        let set_format = Format::Set(Box::new(element_format));

        // Update the current container with the set format
        self.update_container_format(set_format, UpdateMode::Force);

        // Process any user-defined types in the nested structure
        self.process_nested_types(element_shape);
    }

    fn handle_opaque_pointee(&mut self) {
        // For pointers that point to opaque types, treat as unit type for now
        let format = Format::Unit;
        self.update_container_format(format, UpdateMode::Force);
    }

    /// Push a namespace action onto the stack (always pushes something)
    fn push_namespace(&mut self, action: NamespaceAction) {
        let context = match action {
            NamespaceAction::SetContext(ctx) => ctx,
            NamespaceAction::Inherit => {
                // Push the current context again (or cleared context if none exists)
                self.namespace_context_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(NamespaceContext::cleared)
            }
        };
        self.namespace_context_stack.push(context);
    }

    /// Pop the current namespace context from the stack
    fn pop_namespace(&mut self) {
        self.namespace_context_stack.pop();
    }

    /// Get the current namespace context (top of stack)
    fn current_namespace_context(&self) -> Option<&NamespaceContext> {
        match self.namespace_context_stack.last() {
            Some(ctx) if ctx.is_cleared() => {
                // This is a "cleared context" marker, so return None
                None
            }
            Some(ctx) => Some(ctx),
            None => None,
        }
    }

    /// Get the current namespace (if any) for use with format functions
    fn current_namespace(&self) -> Option<&Namespace> {
        self.current_namespace_context().map(|ctx| &ctx.namespace)
    }

    /// Helper method to determine format for user-defined types with namespace context
    fn get_user_type_format(&mut self, field_shape: &Shape) -> Format {
        match &field_shape.ty {
            Type::User(UserType::Struct(_) | UserType::Enum(_)) => {
                if field_shape.type_identifier == "()" {
                    Format::Unit
                } else {
                    let renamed_name = self.get_name_with_mappings(field_shape);
                    Format::TypeName(renamed_name)
                }
            }
            Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) => {
                let target_shape = (pt.target)();
                get_inner_format_with_context(target_shape, self.current_namespace())
            }
            _ => get_inner_format_with_context(field_shape, self.current_namespace()),
        }
    }

    fn update_container_format(&mut self, format: Format, mode: UpdateMode) {
        if let Some(container_format) = self.get_mut() {
            match container_format {
                ContainerFormat::UnitStruct(_doc) => {}
                ContainerFormat::NewTypeStruct(inner_format, _doc) => match mode {
                    UpdateMode::Force => {
                        **inner_format = format;
                    }
                    UpdateMode::IfUnknown | UpdateMode::MutualRecursion => {
                        if inner_format.is_unknown() {
                            **inner_format = format;
                        }
                    }
                },
                ContainerFormat::TupleStruct(formats, _doc) => {
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
                ContainerFormat::Struct(fields, _doc) => {
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
                ContainerFormat::Enum(_, _doc) => {
                    if matches!(mode, UpdateMode::Force) {
                        todo!("Enum container format update not implemented");
                    }
                }
            }
        }
    }

    fn process_nested_types(&mut self, shape: &Shape) {
        self.processing_nested = true;
        match shape.def {
            Def::Scalar => {
                // Scalar types don't need further processing
            }
            Def::List(inner_list_def) => {
                // Recursively process nested lists
                let inner_shape = inner_list_def.t();
                self.process_nested_types(inner_shape);
            }
            Def::Option(option_def) => {
                // Recursively process options
                let inner_shape = option_def.t();
                if should_process_nested_type(inner_shape) {
                    self.process_nested_types(inner_shape);
                }
            }
            Def::Map(map_def) => {
                // Recursively process maps
                let key_shape = map_def.k();
                let value_shape = map_def.v();
                if should_process_nested_type(key_shape) {
                    self.process_nested_types(key_shape);
                }
                if should_process_nested_type(value_shape) {
                    self.process_nested_types(value_shape);
                }
            }
            Def::Slice(slice_def) => {
                // Recursively process slice inner types
                let inner_shape = slice_def.t();
                self.process_nested_types(inner_shape);
            }
            _ => {
                // For other user-defined types, process them
                if should_process_nested_type(shape) {
                    self.format(shape);
                }
            }
        }
        self.processing_nested = false;
    }

    fn get_name_with_mappings(&mut self, shape: &Shape) -> QualifiedTypeName {
        // First check if there's a mapping for this type
        let base_key = QualifiedTypeName::root(shape.type_identifier.to_string());
        if let Some(mapped_name) = self.name_mappings.get(&base_key) {
            return mapped_name.clone();
        }

        // Get the original name (which includes explicit namespace annotations)
        let original_name = get_name(shape);

        // Check if the type has an explicit namespace annotation (type-level explicit)
        let has_explicit_namespace = extract_namespace_from_shape(shape).is_explicit();

        if has_explicit_namespace {
            // Type-level explicit namespace annotation overrides everything (including field-level explicit)
            self.check_namespace_ambiguity(&original_name, true);
            return original_name;
        }

        // If no type-level explicit namespace annotation, apply current context if available
        if let Some(namespace_context) = self.current_namespace_context() {
            match &namespace_context.namespace {
                Namespace::Root => {
                    // Context is explicitly root, use root namespace
                    let root_name = QualifiedTypeName::root(original_name.name.clone());
                    self.check_namespace_ambiguity(&root_name, namespace_context.is_explicit());
                    return root_name;
                }
                Namespace::Named(context_ns) => {
                    // Apply the context namespace
                    let namespaced_name = QualifiedTypeName::namespaced(
                        context_ns.clone(),
                        original_name.name.clone(),
                    );

                    self.check_namespace_ambiguity(
                        &namespaced_name,
                        namespace_context.is_explicit(),
                    );

                    return namespaced_name;
                }
            }
        }

        // Fall back to the original name (which will be root if no annotation)
        original_name
    }

    fn check_namespace_ambiguity(&mut self, new_name: &QualifiedTypeName, is_explicit: bool) {
        // Check for conflicts: type name in multiple namespaces with mixed explicit/inherited sources
        for (existing_name, existing_is_explicit) in &self.type_namespace_sources {
            if existing_name.name == new_name.name && existing_name.namespace != new_name.namespace
            {
                // Error if at least one is inherited (same type appearing in multiple namespaces)
                let both_explicit = is_explicit && *existing_is_explicit;
                assert!(
                    both_explicit,
                    "Ambiguous namespace inheritance detected: Type '{}' appears in both namespace '{}' and namespace '{}'. \
                        This occurs when a type without explicit namespace annotation is inherited from multiple parents with different namespaces. \
                        Consider adding explicit #[facet(namespace = \"...\")] annotations to resolve the ambiguity.",
                    new_name.name, existing_name.namespace, new_name.namespace
                );
            }
        }

        // Record this type name -> namespace source mapping
        self.type_namespace_sources
            .insert(new_name.clone(), is_explicit);
    }
}

fn get_name(shape: &Shape) -> QualifiedTypeName {
    // Check type_tag first (is this where facet rename is stored?)
    if let Some(type_tag) = shape.type_tag {
        return QualifiedTypeName::root(type_tag.to_string());
    }

    let shape_namespace = extract_namespace_from_shape(shape);

    // Extract name attribute
    let mut name = None;
    for attr in shape.attributes {
        if let ShapeAttribute::Arbitrary(attr_str) = attr {
            // Check for rename attribute in the format "name = \"NewName\""
            if let Some(stripped) = attr_str.strip_prefix("name = \"") {
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
    match shape_namespace {
        NamespaceAction::SetContext(ctx) => match &ctx.namespace {
            Namespace::Root => QualifiedTypeName::root(base_name),
            Namespace::Named(name) => QualifiedTypeName::namespaced(name.clone(), base_name),
        },
        NamespaceAction::Inherit => QualifiedTypeName::root(base_name),
    }
}

fn get_format_for_shape(shape: &Shape) -> Format {
    let shape = match &shape.ty {
        Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) => (pt.target)(),
        _ => shape,
    };
    get_inner_format(shape)
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
        Type::User(UserType::Opaque) => match shape.type_identifier {
            "String" | "DateTime<Utc>" => Format::Str,
            _ => unimplemented!("Unsupported opaque type: {}", shape.type_identifier),
        },
        _ => unimplemented!("Unsupported type for scalar format: {:?}", shape.ty),
    }
}

fn extract_namespace_from_shape(shape: &Shape) -> NamespaceAction {
    for attr in shape.attributes {
        if let ShapeAttribute::Arbitrary(attr_str) = attr {
            match extract_namespace(attr_str) {
                Ok(None) => (),
                Ok(Some(namespace)) => return namespace,
                Err(e) => panic!("Failed to extract namespace: {e}"),
            }
        }
    }
    NamespaceAction::Inherit
}

fn extract_namespace_from_field_attributes(field: &Field) -> NamespaceAction {
    for attr in field.attributes {
        let FieldAttribute::Arbitrary(attr_str) = attr;
        match extract_namespace(attr_str) {
            Ok(None) => (),
            Ok(Some(namespace)) => return namespace,
            Err(e) => panic!("Failed to extract namespace: {e}"),
        }
    }
    NamespaceAction::Inherit
}

fn extract_namespace(attribute: &str) -> Result<Option<NamespaceAction>, String> {
    const INVALID_FORMAT: &str =
        r#"Attribute should be either `#[namespace = "my_namespace"]` or `#[namespace = None]`"#;
    const INVALID_IDENTIFIER: &str = "Invalid namespace identifier";
    static IS_NAMESPACE_ATTR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^namespace\s*=\s*(.*)$").expect("Invalid regex"));
    static HAS_QUOTES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"^"(.*)"$"#).expect("Invalid regex"));
    static VALID_IDENT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-zA-Z_]\w*$").expect("Invalid regex"));

    let Some(s) = IS_NAMESPACE_ATTR.captures(attribute) else {
        return Ok(None);
    };

    let s = &s[1];
    let Some(s) = HAS_QUOTES.captures(s) else {
        if s == "None" {
            return Ok(Some(NamespaceAction::SetContext(
                NamespaceContext::cleared(),
            )));
        }
        return Err(INVALID_FORMAT.to_string());
    };

    let s = &s[1];
    if !VALID_IDENT.is_match(s) {
        return Err(INVALID_IDENTIFIER.to_string());
    }

    Ok(Some(NamespaceAction::SetContext(
        NamespaceContext::explicit(Namespace::Named(s.to_string())),
    )))
}

fn is_transparent_struct(shape: &Shape) -> bool {
    shape
        .attributes
        .iter()
        .any(|attr| matches!(attr, ShapeAttribute::Transparent))
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

fn should_process_nested_type(shape: &Shape) -> bool {
    !matches!(shape.def, Def::Scalar) && shape.type_identifier != "()"
}

fn get_inner_format(shape: &Shape) -> Format {
    get_inner_format_with_context(shape, None)
}

fn get_inner_format_with_context(shape: &Shape, namespace_context: Option<&Namespace>) -> Format {
    match shape.def {
        Def::Scalar => type_to_format(shape),
        Def::List(inner_list_def) => {
            // Recursively handle nested lists
            let inner_shape = inner_list_def.t();
            Format::Seq(Box::new(get_inner_format_with_context(
                inner_shape,
                namespace_context,
            )))
        }
        Def::Option(option_def) => {
            // Handle Option<T> -> OPTION: T
            let inner_shape = option_def.t();
            let inner_format = get_inner_format_with_context(inner_shape, namespace_context);
            Format::Option(Box::new(inner_format))
        }
        Def::Map(map_def) => {
            // Handle Map<K, V> -> MAP: { KEY: K, VALUE: V }
            let key_shape = map_def.k();
            let value_shape = map_def.v();
            let key_format = get_inner_format_with_context(key_shape, namespace_context);
            let value_format = get_inner_format_with_context(value_shape, namespace_context);
            Format::Map {
                key: Box::new(key_format),
                value: Box::new(value_format),
            }
        }
        Def::Set(set_def) => {
            // Handle Set<T> -> SET: T
            let inner_shape = set_def.t();
            Format::Set(Box::new(get_inner_format_with_context(
                inner_shape,
                namespace_context,
            )))
        }
        Def::Array(array_def) => {
            // Handle Array<T, N> -> TUPLEARRAY: { CONTENT: T, SIZE: N }
            let inner_shape = array_def.t();
            let inner_format = get_inner_format_with_context(inner_shape, namespace_context);
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
                        let field_format =
                            get_inner_format_with_context(field_shape, namespace_context);
                        tuple_formats.push(field_format);
                    }
                    return Format::Tuple(tuple_formats);
                }
            }

            // Special case for unit type
            if shape.type_identifier == "()" {
                Format::Unit
            } else if let Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) =
                &shape.ty
            {
                // For pointer types like &'static str, get the format of the target type
                let target_shape = (pt.target)();
                get_inner_format_with_context(target_shape, namespace_context)
            } else {
                // For user-defined types, use TypeName with namespace context if available
                let original_name = get_name(shape);
                let name = if let Some(namespace) = namespace_context {
                    // If the type doesn't have its own namespace annotation and we're in a namespace context,
                    // apply the context namespace
                    if matches!(original_name.namespace, Namespace::Root) {
                        match namespace {
                            Namespace::Root => QualifiedTypeName::root(original_name.name.clone()),
                            Namespace::Named(name) => QualifiedTypeName::namespaced(
                                name.clone(),
                                original_name.name.clone(),
                            ),
                        }
                    } else {
                        original_name
                    }
                } else {
                    original_name
                };

                Format::TypeName(name)
            }
        }

        Def::Slice(slice_def) => {
            // Handle Slice<T> -> SEQ: T
            let inner_shape = slice_def.t();
            Format::Seq(Box::new(get_inner_format_with_context(
                inner_shape,
                namespace_context,
            )))
        }
        Def::Pointer(pointer_def) => {
            // Handle Pointer (Box, Arc, etc.) by recursively processing the inner type
            if let Some(inner_shape) = pointer_def.pointee {
                get_inner_format_with_context(inner_shape(), namespace_context)
            } else {
                // Fallback for pointers without a known pointee
                Format::Unit
            }
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "namespace_tests.rs"]
mod namespace_tests;

#[cfg(test)]
mod function_tests {
    use super::*;

    #[test]
    fn extract_namespace_tests() {
        const INVALID_FORMAT: &str = r#"Attribute should be either `#[namespace = "my_namespace"]` or `#[namespace = None]`"#;
        const INVALID_IDENTIFIER: &str = "Invalid namespace identifier";
        let tests = [
            ("not a namespace attribute", "some string", Ok(None)),
            (
                "error, no quotes around namespace",
                "namespace = my_namespace",
                Err(INVALID_FORMAT.to_string()),
            ),
            (
                "error, invalid namespace",
                r#"namespace = "%^t6!!""#,
                Err(INVALID_IDENTIFIER.to_string()),
            ),
            (
                "set namespace",
                r#"namespace = "my_namespace""#,
                Ok(Some(NamespaceAction::SetContext(
                    NamespaceContext::explicit(Namespace::Named("my_namespace".to_string())),
                ))),
            ),
            (
                "no space before equals",
                r#"namespace= "my_namespace""#,
                Ok(Some(NamespaceAction::SetContext(
                    NamespaceContext::explicit(Namespace::Named("my_namespace".to_string())),
                ))),
            ),
            (
                "no space after equals",
                r#"namespace ="my_namespace""#,
                Ok(Some(NamespaceAction::SetContext(
                    NamespaceContext::explicit(Namespace::Named("my_namespace".to_string())),
                ))),
            ),
            (
                "clear namespace",
                "namespace = None",
                Ok(Some(NamespaceAction::SetContext(
                    NamespaceContext::cleared(),
                ))),
            ),
            (
                "set namespace to the name None",
                r#"namespace = "None""#,
                Ok(Some(NamespaceAction::SetContext(
                    NamespaceContext::explicit(Namespace::Named("None".to_string())),
                ))),
            ),
        ];

        for (test, input, expected) in tests {
            assert_eq!(extract_namespace(input), expected, "{test}: {input}");
        }
    }
}
