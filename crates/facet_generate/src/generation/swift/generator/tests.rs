//! Unit tests for [`SwiftCodeGenerator`] — import generation and qualified-name
//! resolution.
//!
//! Tests build small [`Registry`] values by hand (rather than via the
//! `reflect!` macro) so that module and external-package configurations can
//! be controlled precisely.
//!
//! # Coverage
//!
//! | Area | What is tested |
//! |------|----------------|
//! | Serde imports | Bincode encoding triggers `import Serde`; `Encoding::None` does not |
//! | External definitions | External namespaces appear as `import` statements |
//! | Encoding config | Encoding propagates through to generated output |
//! | Feature helpers | Complex types (e.g. `Seq`) trigger trait helper emission when encoding is active |

use std::collections::BTreeMap;

use crate::{
    generation::{CodeGeneratorConfig, Encoding},
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName},
};

use super::*;

fn generate(config: &CodeGeneratorConfig, encoding: Encoding, registry: &Registry) -> String {
    let generator = SwiftCodeGenerator::new(config).with_encoding(encoding);
    let mut output = Vec::new();
    generator.output(&mut output, registry).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn test_no_encoding_does_not_import_serde() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "id".to_string(),
        doc: Doc::new(),
        value: Format::U32,
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, Encoding::None, &registry);

    assert!(
        !output.contains("import Serde"),
        "Should not import Serde when encoding is None: {output}"
    );
}

#[test]
fn test_bincode_encoding_has_serde_import() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "name".to_string(),
        doc: Doc::new(),
        value: Format::Str,
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, Encoding::Bincode, &registry);

    assert!(
        output.contains("import Serde"),
        "Bincode encoding should import Serde: {output}"
    );
}

#[test]
fn test_preamble_includes_external_definition_imports() {
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("another_target".to_string(), vec!["Child".to_string()]);

    let config = CodeGeneratorConfig::new("MyPackage".to_string())
        .with_external_definitions(external_definitions);

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::UnitStruct(Doc::new()),
    );

    let output = generate(&config, Encoding::Bincode, &registry);

    assert!(
        output.contains("import AnotherTarget"),
        "Should import UpperCamelCase external definitions: {output}"
    );
    assert!(
        output.contains("import Serde"),
        "Should always import Serde when encoding is set: {output}"
    );
}

#[test]
fn test_trait_helpers_emitted_for_complex_types() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Seq(Box::new(Format::Str)),
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, Encoding::Bincode, &registry);

    assert!(
        output.contains("func serializeArray<T, S: Serializer>"),
        "Should emit generic array serialization helper: {output}"
    );
    assert!(
        output.contains("func deserializeArray<T, D: Deserializer>"),
        "Should emit generic array deserialization helper: {output}"
    );
}

#[test]
fn test_no_trait_helpers_without_encoding() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Seq(Box::new(Format::Str)),
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, Encoding::None, &registry);

    assert!(
        !output.contains("func serializeArray"),
        "No encoding means no trait helpers: {output}"
    );
    assert!(
        !output.contains("func deserializeArray"),
        "No encoding means no trait helpers: {output}"
    );
}
