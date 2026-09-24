//! Tests that each language's public `requalify` gives a plugin the spelling
//! the emitter itself writes.
//!
//! Every test generates the modules of one small registry the way the
//! language's installer does, with a plugin that records the config it is
//! handed. It then requalifies a registry-spelled name with that config and
//! checks that the language's `render_type` and Bincode
//! `write_serialize_value` produce exactly what the generated file writes for
//! a field of that type, and that
//! [`is_enum`](super::CodeGeneratorConfig::is_enum) and
//! [`is_unit_enum`](super::CodeGeneratorConfig::is_unit_enum) recognise the
//! requalified name, in the config for the module and in the one for its
//! companion files.
//!
//! # Coverage
//!
//! | Language | Root module | Namespaced module | Idempotent |
//! |---|---|---|---|
//! | C# | dotted (`Example.Shared`) and undotted (`Example`) package; ROOT and sibling references | ROOT and same-namespace references | no |
//! | TypeScript | ROOT and sibling references | ROOT (through the root package import) and same-namespace references | no |
//! | Kotlin | ROOT and sibling references | ROOT and same-namespace references | no |
//! | Swift | ROOT and sibling references | ROOT (qualified with the root package) and same-namespace references | yes |

use std::{
    collections::BTreeMap,
    io,
    sync::{Arc, Mutex},
};

use super::{
    CodeGeneratorConfig,
    indent::{IndentConfig, IndentWrite, IndentedWriter},
    module,
    plugin::{CompanionFile, EmitContext, EmitterPlugin},
};
use crate::{
    Registry,
    reflection::format::{
        ContainerFormat, Doc, EnumTagging, Format, Named, QualifiedTypeName, VariantFormat,
    },
};

/// Records the config a plugin is handed while the module is emitted, and
/// the one it is handed for its companion files.
#[derive(Debug, Default)]
struct Capture {
    emitted: Mutex<Option<CodeGeneratorConfig>>,
    companion: Mutex<Option<CodeGeneratorConfig>>,
}

impl<L> EmitterPlugin<L> for Capture {
    fn after_type(&self, _w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        *self.emitted.lock().unwrap() = Some(ctx.config.clone());
        Ok(())
    }

    fn companion_files(&self, config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        *self.companion.lock().unwrap() = Some(config.clone());
        vec![]
    }
}

/// One generated module: its source, and the configs the plugin saw.
struct Generated {
    output: String,
    config: CodeGeneratorConfig,
    companion_config: Option<CodeGeneratorConfig>,
}

fn root(name: &str) -> QualifiedTypeName {
    QualifiedTypeName::root(name.to_string())
}

fn kit(name: &str) -> QualifiedTypeName {
    QualifiedTypeName::namespaced("kit".to_string(), name.to_string())
}

fn field(name: &str, type_name: QualifiedTypeName) -> Named<Format> {
    Named {
        name: name.to_string(),
        doc: Doc::new(),
        value: Format::TypeName(type_name),
    }
}

fn unit_enum(variants: &[&str]) -> ContainerFormat {
    let variants = variants
        .iter()
        .enumerate()
        .map(|(index, name)| {
            (
                u32::try_from(index).unwrap(),
                Named {
                    name: (*name).to_string(),
                    doc: Doc::new(),
                    value: VariantFormat::Unit,
                },
            )
        })
        .collect();
    ContainerFormat::Enum(variants, EnumTagging::External, Doc::new())
}

/// A root module and a `kit` namespace, each with a struct whose fields
/// reference the ROOT all-unit enum `Event` and the `kit` all-unit enum
/// `Status`; the root one references `kit`'s struct `Row` too.
fn registry() -> Registry {
    Registry::from([
        (root("Event"), unit_enum(&["Increment", "Decrement"])),
        (
            root("Holder"),
            ContainerFormat::Struct(
                vec![
                    field("event", root("Event")),
                    field("row", kit("Row")),
                    field("status", kit("Status")),
                ],
                Doc::new(),
            ),
        ),
        (
            kit("Row"),
            ContainerFormat::Struct(
                vec![
                    field("event", root("Event")),
                    field("status", kit("Status")),
                ],
                Doc::new(),
            ),
        ),
        (kit("Status"), unit_enum(&["Online"])),
    ])
}

/// Splits `registry` into modules under `package`, configures each one with
/// `configure` the way the language's installer does, and generates it with
/// `generate`, which returns the module's source.
fn generate_modules(
    package: &str,
    configure: impl Fn(&CodeGeneratorConfig) -> CodeGeneratorConfig,
    generate: impl Fn(&CodeGeneratorConfig, &Registry, Arc<Capture>) -> String,
) -> BTreeMap<String, Generated> {
    module::split(package, &registry())
        .into_iter()
        .map(|(module, module_registry)| {
            let config = configure(module.config());
            let capture = Arc::new(Capture::default());
            let output = generate(&config, &module_registry, capture.clone());
            let generated = Generated {
                output,
                config: capture.emitted.lock().unwrap().clone().unwrap(),
                companion_config: capture.companion.lock().unwrap().clone(),
            };
            (module.config().module_name().to_string(), generated)
        })
        .collect()
}

type WriteSerializeValue =
    fn(&mut dyn IndentWrite, &str, &Format, &CodeGeneratorConfig) -> io::Result<()>;

/// What `write_serialize_value` writes for `value_expr`, trimmed.
fn serialized(
    write_serialize_value: WriteSerializeValue,
    value_expr: &str,
    format: &Format,
    config: &CodeGeneratorConfig,
) -> String {
    let mut buf = Vec::new();
    write_serialize_value(
        &mut IndentedWriter::new(&mut buf, IndentConfig::Space(4)),
        value_expr,
        format,
        config,
    )
    .unwrap();
    String::from_utf8(buf).unwrap().trim().to_string()
}

type Requalify = fn(&CodeGeneratorConfig, &QualifiedTypeName) -> QualifiedTypeName;
type RenderType = fn(&Format, &CodeGeneratorConfig) -> String;

/// A language's public helpers, and how its emitter declares a field.
struct Helpers {
    requalify: Requalify,
    render_type: RenderType,
    write_serialize_value: WriteSerializeValue,
    /// The declaration the emitter writes for field `name` of type `ty`.
    declaration: fn(name: &str, ty: &str) -> String,
    /// The expression the emitter serializes field `name` from.
    value_expr: fn(name: &str) -> String,
}

impl Helpers {
    /// Requalifies `raw` with the config the plugin saw, checks that the
    /// generated module declares field `field` with the type `render_type`
    /// gives it and serializes it the way `write_serialize_value` does, and
    /// returns both.
    fn agree(&self, module: &Generated, field: &str, raw: &QualifiedTypeName) -> (String, String) {
        let config = &module.config;
        let format = Format::TypeName((self.requalify)(config, raw));

        let ty = (self.render_type)(&format, config);
        let declaration = (self.declaration)(field, &ty);
        assert!(
            module.output.contains(&declaration),
            "expected `{declaration}` in\n{}",
            module.output
        );

        let serialize = serialized(
            self.write_serialize_value,
            &(self.value_expr)(field),
            &format,
            config,
        );
        assert!(
            module.output.contains(&serialize),
            "expected `{serialize}` in\n{}",
            module.output
        );

        (ty, serialize)
    }

    /// What `write_serialize_value` writes for field `field` when handed the
    /// registry spelling of its type, `raw`, without requalifying it.
    fn unrequalified(&self, module: &Generated, field: &str, raw: &QualifiedTypeName) -> String {
        serialized(
            self.write_serialize_value,
            &(self.value_expr)(field),
            &Format::TypeName(raw.clone()),
            &module.config,
        )
    }

    /// Checks that the enum lookups answer a requalified name, both while
    /// the module is emitted and for its companion files.
    fn assert_enum_lookups(&self, module: &Generated, raw: &QualifiedTypeName, is_unit_enum: bool) {
        let configs = std::iter::once(&module.config).chain(&module.companion_config);
        for config in configs {
            let name = (self.requalify)(config, raw);
            assert_eq!(config.is_enum(&name), is_unit_enum, "is_enum({name:?})");
            assert_eq!(
                config.is_unit_enum(&name),
                is_unit_enum,
                "is_unit_enum({name:?})"
            );
        }
    }
}

/// Configures a module the way the TypeScript and Swift installers do: a
/// namespaced module keeps its name and learns the root package.
fn with_root_package(package: &str) -> impl Fn(&CodeGeneratorConfig) -> CodeGeneratorConfig {
    move |config| {
        let mut config = config.clone();
        if config.module_name() != package {
            config.parent = Some(package.to_string());
        }
        config
    }
}

#[cfg(feature = "csharp")]
mod csharp {
    use std::{collections::BTreeMap, sync::Arc};

    use super::{Generated, Helpers, generate_modules, kit, root};
    use crate::generation::{
        bincode::{self, BincodePlugin},
        csharp::{CSharpCodeGenerator, render_type, requalify},
    };

    const HELPERS: Helpers = Helpers {
        requalify,
        render_type,
        write_serialize_value: bincode::csharp::write_serialize_value,
        declaration: |name, ty| format!("private {ty} _{name};"),
        value_expr: |name| heck::AsUpperCamelCase(name).to_string(),
    };

    /// The modules the C# installer generates for package `package`.
    fn generate(package: &str) -> BTreeMap<String, Generated> {
        generate_modules(
            package,
            |config| config.clone().with_parent(package),
            |config, registry, capture| {
                let generator = CSharpCodeGenerator::new(config)
                    .with_plugins(vec![Arc::new(BincodePlugin), capture]);
                let mut output = Vec::new();
                generator.output(&mut output, registry).unwrap();
                generator.companion_files(registry).unwrap();
                String::from_utf8(output).unwrap()
            },
        )
    }

    #[test]
    fn requalify_agrees_with_a_dotted_root_module() {
        let modules = generate("Example.Shared");
        let module = &modules["Example.Shared"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "Example.Shared.Event".to_string(),
                "Example.Shared.EventBincode.Serialize(Event, serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "row", &kit("Row")),
            (
                "Example.Shared.Kit.Row".to_string(),
                "Row.Serialize(serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")),
            (
                "Example.Shared.Kit.Status".to_string(),
                "Example.Shared.Kit.StatusBincode.Serialize(Status, serializer);".to_string()
            )
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
        HELPERS.assert_enum_lookups(module, &kit("Row"), false);

        // The registry spelling misses the enum, and would call a method a
        // C# `enum` does not have.
        assert_eq!(
            HELPERS.unrequalified(module, "event", &root("Event")),
            "Event.Serialize(serializer);"
        );
    }

    #[test]
    fn requalify_agrees_with_an_undotted_root_module() {
        let modules = generate("Example");
        let module = &modules["Example"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "Event".to_string(),
                "EventBincode.Serialize(Event, serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")),
            (
                "Example.Kit.Status".to_string(),
                "Example.Kit.StatusBincode.Serialize(Status, serializer);".to_string()
            )
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
    }

    #[test]
    fn requalify_agrees_with_a_namespaced_module() {
        for package in ["Example.Shared", "Example"] {
            let modules = generate(package);
            let module = &modules["kit"];

            assert_eq!(
                HELPERS.agree(module, "event", &root("Event")),
                (
                    format!("{package}.Event"),
                    format!("{package}.EventBincode.Serialize(Event, serializer);")
                )
            );
            assert_eq!(
                HELPERS.agree(module, "status", &kit("Status")),
                (
                    "Status".to_string(),
                    "StatusBincode.Serialize(Status, serializer);".to_string()
                )
            );

            HELPERS.assert_enum_lookups(module, &root("Event"), true);
            HELPERS.assert_enum_lookups(module, &kit("Status"), true);

            assert_eq!(
                HELPERS.unrequalified(module, "event", &root("Event")),
                "Event.Serialize(serializer);"
            );
        }
    }

    #[test]
    fn requalify_is_not_idempotent() {
        let modules = generate("Example.Shared");
        let config = &modules["Example.Shared"].config;

        let once = requalify(config, &root("Event"));
        assert_eq!(
            render_type(
                &crate::reflection::format::Format::TypeName(requalify(config, &once)),
                config
            ),
            "Example.Shared.Example.Shared.Event"
        );
    }

    #[test]
    fn requalify_format_requalifies_every_nested_name() {
        use crate::reflection::format::{Format, QualifiedTypeName};

        let modules = generate("Example.Shared");
        let config = &modules["kit"].config;

        let mut format = Format::Map {
            key: Box::new(Format::TypeName(kit("Row"))),
            value: Box::new(Format::Seq(Box::new(Format::TypeName(root("Event"))))),
        };
        crate::generation::csharp::requalify_format(config, &mut format);

        assert_eq!(
            format,
            Format::Map {
                key: Box::new(Format::TypeName(root("Row"))),
                value: Box::new(Format::Seq(Box::new(Format::TypeName(
                    QualifiedTypeName::namespaced(
                        "Example.Shared".to_string(),
                        "Event".to_string()
                    )
                )))),
            }
        );
    }
}

#[cfg(feature = "typescript")]
mod typescript {
    use std::{collections::BTreeMap, sync::Arc};

    use super::{Generated, Helpers, generate_modules, kit, root, with_root_package};
    use crate::{
        generation::{
            bincode::{self, BincodePlugin},
            typescript::{TypeScriptCodeGenerator, render_type, requalify},
        },
        reflection::format::Format,
    };

    const HELPERS: Helpers = Helpers {
        requalify,
        render_type,
        write_serialize_value: bincode::typescript::write_serialize_value,
        declaration: |name, ty| format!("public {name}: {ty}"),
        value_expr: |name| format!("this.{name}"),
    };

    /// The modules the TypeScript installer generates for package `example`.
    fn generate() -> BTreeMap<String, Generated> {
        generate_modules(
            "example",
            with_root_package("example"),
            |config, registry, capture| {
                let generator = TypeScriptCodeGenerator::new(config)
                    .with_plugins(vec![Arc::new(BincodePlugin), capture]);
                let mut output = Vec::new();
                generator.output(&mut output, registry).unwrap();
                String::from_utf8(output).unwrap()
            },
        )
    }

    #[test]
    fn requalify_agrees_with_the_root_module() {
        let modules = generate();
        let module = &modules["example"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "Event".to_string(),
                "serializeEvent(this.event, serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "row", &kit("Row")),
            (
                "Kit.Row".to_string(),
                "this.row.serialize(serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")),
            (
                "Kit.Status".to_string(),
                "Kit.serializeStatus(this.status, serializer);".to_string()
            )
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
        HELPERS.assert_enum_lookups(module, &kit("Row"), false);
    }

    #[test]
    fn requalify_agrees_with_a_namespaced_module() {
        let modules = generate();
        let module = &modules["kit"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "Example.Event".to_string(),
                "Example.serializeEvent(this.event, serializer);".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")),
            (
                "Status".to_string(),
                "serializeStatus(this.status, serializer);".to_string()
            )
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);

        // The registry spelling misses both enums, and would call a method
        // a union does not have.
        assert_eq!(
            HELPERS.unrequalified(module, "event", &root("Event")),
            "this.event.serialize(serializer);"
        );
        assert_eq!(
            HELPERS.unrequalified(module, "status", &kit("Status")),
            "this.status.serialize(serializer);"
        );
    }

    #[test]
    fn requalify_is_not_idempotent() {
        let modules = generate();
        let config = &modules["kit"].config;

        let once = requalify(config, &kit("Status"));
        assert_eq!(
            render_type(&Format::TypeName(requalify(config, &once)), config),
            "Example.Status"
        );
    }
}

#[cfg(feature = "kotlin")]
mod kotlin {
    use std::{collections::BTreeMap, sync::Arc};

    use super::{Generated, Helpers, generate_modules, kit, root};
    use crate::{
        generation::{
            bincode::{self, BincodePlugin},
            kotlin::{KotlinCodeGenerator, render_type, requalify},
        },
        reflection::format::Format,
    };

    const HELPERS: Helpers = Helpers {
        requalify,
        render_type,
        write_serialize_value: bincode::kotlin::write_serialize_value,
        declaration: |name, ty| format!("val {name}: {ty},"),
        value_expr: ToString::to_string,
    };

    /// The modules the Kotlin installer generates for package `com.example`.
    fn generate() -> BTreeMap<String, Generated> {
        generate_modules(
            "com.example",
            |config| config.clone().with_parent("com.example"),
            |config, registry, capture| {
                let generator = KotlinCodeGenerator::new(config)
                    .with_plugins(vec![Arc::new(BincodePlugin), capture]);
                let mut output = Vec::new();
                generator.output(&mut output, registry).unwrap();
                generator.companion_files(registry).unwrap();
                String::from_utf8(output).unwrap()
            },
        )
    }

    #[test]
    fn requalify_agrees_with_the_root_module() {
        let modules = generate();
        let module = &modules["com.example"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "com.example.Event".to_string(),
                "event.serialize(serializer)".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "row", &kit("Row")),
            (
                "com.example.kit.Row".to_string(),
                "row.serialize(serializer)".to_string()
            )
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")).0,
            "com.example.kit.Status"
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
        HELPERS.assert_enum_lookups(module, &kit("Row"), false);
    }

    #[test]
    fn requalify_agrees_with_a_namespaced_module() {
        let modules = generate();
        let module = &modules["kit"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")).0,
            "com.example.Event"
        );
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")).0,
            "com.example.kit.Status"
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
    }

    #[test]
    fn requalify_is_not_idempotent() {
        let modules = generate();
        let config = &modules["kit"].config;

        let once = requalify(config, &root("Event"));
        assert_eq!(
            render_type(&Format::TypeName(requalify(config, &once)), config),
            "com.example.com.example.Event"
        );
    }
}

#[cfg(feature = "swift")]
mod swift {
    use std::{collections::BTreeMap, sync::Arc};

    use super::{Generated, Helpers, generate_modules, kit, root, with_root_package};
    use crate::generation::{
        bincode::{self, BincodePlugin},
        swift::{SwiftCodeGenerator, render_type, requalify},
    };

    const HELPERS: Helpers = Helpers {
        requalify,
        render_type,
        write_serialize_value: bincode::swift::write_serialize_value,
        declaration: |name, ty| format!("public var {name}: {ty}\n"),
        value_expr: |name| format!("self.{name}"),
    };

    /// The modules the Swift installer generates for package `SharedTypes`.
    fn generate() -> BTreeMap<String, Generated> {
        generate_modules(
            "SharedTypes",
            with_root_package("SharedTypes"),
            |config, registry, capture| {
                let generator = SwiftCodeGenerator::new(config)
                    .with_plugins(vec![Arc::new(BincodePlugin), capture]);
                let mut output = Vec::new();
                generator.output(&mut output, registry).unwrap();
                generator.companion_files(registry).unwrap();
                String::from_utf8(output).unwrap()
            },
        )
    }

    #[test]
    fn requalify_agrees_with_the_root_module() {
        let modules = generate();
        let module = &modules["SharedTypes"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")),
            (
                "Event".to_string(),
                "try self.event.serialize(serializer: serializer)".to_string()
            )
        );
        assert_eq!(HELPERS.agree(module, "row", &kit("Row")).0, "Kit.Row");
        assert_eq!(
            HELPERS.agree(module, "status", &kit("Status")).0,
            "Kit.Status"
        );

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
        HELPERS.assert_enum_lookups(module, &kit("Row"), false);
    }

    #[test]
    fn requalify_agrees_with_a_namespaced_module() {
        let modules = generate();
        let module = &modules["kit"];

        assert_eq!(
            HELPERS.agree(module, "event", &root("Event")).0,
            "SharedTypes.Event"
        );
        assert_eq!(HELPERS.agree(module, "status", &kit("Status")).0, "Status");

        HELPERS.assert_enum_lookups(module, &root("Event"), true);
        HELPERS.assert_enum_lookups(module, &kit("Status"), true);
    }

    #[test]
    fn requalify_is_idempotent() {
        let modules = generate();
        for module in modules.values() {
            let config = &module.config;
            for raw in [root("Event"), kit("Status")] {
                let once = requalify(config, &raw);
                assert_eq!(requalify(config, &once), once);
            }
        }
    }
}
