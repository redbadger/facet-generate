#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use std::sync::Arc;

use facet::Facet;
use facet_generate::generation::{
    CodeGeneratorConfig, SourceInstaller, bincode::BincodePlugin, json::JsonPlugin,
    plugin::EmitterPlugin, typescript,
};
use facet_generate::reflection::RegistryBuilder;
use serde_json::Value;
use std::{collections::BTreeMap, fs::File, path::Path, process::Command};
use tempfile::tempdir;

fn test_typescript_code_generates_with_config(
    dir_path: &Path,
    config: &CodeGeneratorConfig,
    plugins: Vec<Arc<dyn EmitterPlugin<typescript::TypeScript>>>,
) -> std::path::PathBuf {
    let registry = common::get_registry();
    std::fs::create_dir_all(dir_path.join("testing")).unwrap_or(());

    let mut installer = typescript::Installer::new("testing", dir_path);
    installer.install_serde_runtime().unwrap();

    let source_path = dir_path.join("testing").join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    let generator = typescript::TypeScriptCodeGenerator::new(config).with_plugins(plugins);
    generator.output(&mut source, &registry).unwrap();

    dir_path.join("testing")
}

#[test]
fn test_typescript_code_generates_with_bincode() {
    let dir = tempdir().unwrap();
    let config = CodeGeneratorConfig::new("testing".to_string());
    test_typescript_code_generates_with_config(dir.path(), &config, vec![Arc::new(BincodePlugin)]);
}

#[test]
fn test_typescript_code_generates_with_comments() {
    /// Some
    /// comments
    #[derive(Facet)]
    struct CommentedType {
        value: String,
    }

    let dir = tempdir().unwrap();
    let registry = RegistryBuilder::new()
        .add_type::<CommentedType>()
        .unwrap()
        .build()
        .unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let source_path = dir.path().join("testing");
    std::fs::create_dir_all(&source_path).unwrap();
    let source_file = source_path.join("test.ts");
    let mut file = File::create(&source_file).unwrap();

    let generator = typescript::TypeScriptCodeGenerator::new(&config);
    generator.output(&mut file, &registry).unwrap();

    let content = std::fs::read_to_string(&source_file).unwrap();
    assert!(
        content.contains("/// Some\n/// comments\n"),
        "Doc comments should be present in output:\n{content}"
    );
}

#[test]
fn test_typescript_code_generates_with_external_definitions() {
    let dir = tempdir().unwrap();

    // create external definition
    std::fs::create_dir_all(dir.path().join("external")).unwrap_or(());
    std::fs::write(
        dir.path().join("external/index.ts"),
        "export const CustomType = 5;",
    )
    .unwrap();

    let mut external_definitions = BTreeMap::new();
    external_definitions.insert(String::from("external"), vec![String::from("CustomType")]);
    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_external_definitions(external_definitions);

    test_typescript_code_generates_with_config(dir.path(), &config, vec![]);
}

#[test]
fn test_typescript_manifest_generation() {
    let dir = tempdir().unwrap();

    let installer = typescript::Installer::new("my-typescript-package", dir.path());
    installer.install_manifest("my-typescript-package").unwrap();

    // Check that package.json was created
    let manifest_path = dir.path().join("package.json");
    assert!(manifest_path.exists());

    // Check that package.json has correct content
    let manifest_content = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: Value = serde_json::from_str(&manifest_content).unwrap();

    assert_eq!(manifest["name"], "my-typescript-package");
    assert_eq!(manifest["version"], "0.1.0");
    assert_eq!(manifest["devDependencies"]["typescript"], "^5.8.3");
}

#[test]
fn test_typescript_code_generation_file_layout() {
    let dir = tempdir().unwrap();
    let registry = common::get_registry();

    let config = CodeGeneratorConfig::new("testing".to_string());

    let mut installer = typescript::Installer::new("testing", dir.path()).plugin(BincodePlugin);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    // Module is written as a flat .ts file (Node convention)
    let module_path = dir.path().join("testing.ts");
    assert!(module_path.exists());

    // Generated content uses the expected type-only, extensionless serde import
    let content = std::fs::read_to_string(&module_path).unwrap();
    assert!(content.contains(r#"import type { Serializer, Deserializer } from "./serde";"#));
    assert!(!content.contains(r#"from "./serde/mod.ts""#));

    // Runtime entry point is index.ts (Node convention)
    let serde_index = dir.path().join("serde").join("index.ts");
    assert!(serde_index.exists());

    // Runtime imports have .ts extensions stripped
    let serde_content = std::fs::read_to_string(&serde_index).unwrap();
    assert!(serde_content.contains("from \"./types\""));
    assert!(!serde_content.contains("from \"./types.ts\""));

    // Other serde files also have .ts stripped
    let binary_deserializer = dir.path().join("serde").join("binaryDeserializer.ts");
    assert!(binary_deserializer.exists());

    let binary_deserializer_content = std::fs::read_to_string(&binary_deserializer).unwrap();
    assert!(binary_deserializer_content.contains("from \"./deserializer\""));
    assert!(!binary_deserializer_content.contains("from \"./deserializer.ts\""));
}

/// Field and variant names that collide with TypeScript reserved words must
/// produce code that type-checks: reserved words stay as property names but
/// binding identifiers (constructor parameters, `const` locals) are renamed
/// with a trailing underscore.
#[test]
fn test_that_typescript_code_with_keyword_names_type_checks() {
    for plugin in [
        Arc::new(BincodePlugin) as Arc<dyn EmitterPlugin<typescript::TypeScript>>,
        Arc::new(JsonPlugin),
    ] {
        let dir = tempdir().unwrap();
        let registry = common::get_keyword_registry();

        let mut installer = typescript::Installer::new("testing", dir.path());
        installer.install_serde_runtime().unwrap();
        installer.install_bincode_runtime().unwrap();

        let source_path = dir.path().join("testing.ts");
        let mut source = File::create(&source_path).unwrap();
        let config = CodeGeneratorConfig::new("testing".to_string());
        let generator =
            typescript::TypeScriptCodeGenerator::new(&config).with_plugins(vec![plugin]);
        generator.output(&mut source, &registry).unwrap();
        drop(source);

        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("check")
            .arg("--sloppy-imports")
            .arg(&source_path)
            .status()
            .unwrap();
        assert!(status.success(), "deno check failed");
    }
}

/// A type named `Set` must not break the generated module, and a type named
/// `Map` must leave the global `Map` reachable through `globalThis`.
#[test]
fn test_that_typescript_code_shadowing_builtin_names_type_checks() {
    for plugin in [
        Arc::new(BincodePlugin) as Arc<dyn EmitterPlugin<typescript::TypeScript>>,
        Arc::new(JsonPlugin),
    ] {
        let dir = tempdir().unwrap();
        let registry = common::get_shadowing_registry();

        let mut installer = typescript::Installer::new("testing", dir.path());
        installer.install_serde_runtime().unwrap();
        installer.install_bincode_runtime().unwrap();

        let source_path = dir.path().join("testing.ts");
        let mut source = File::create(&source_path).unwrap();
        let config = CodeGeneratorConfig::new("testing".to_string());
        let generator =
            typescript::TypeScriptCodeGenerator::new(&config).with_plugins(vec![plugin]);
        generator.output(&mut source, &registry).unwrap();
        drop(source);

        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("check")
            .arg("--sloppy-imports")
            .arg(&source_path)
            .status()
            .unwrap();
        assert!(status.success(), "deno check failed");
    }
}

/// A type named `String` shadows the global one, which the Bincode plugin's
/// `char` helper constructs a string with (#213).
#[test]
fn test_that_typescript_code_with_chars_beside_a_type_named_string_type_checks() {
    #[derive(Facet)]
    pub struct String {
        pub letter: char,
        pub letters: Vec<char>,
        pub maybe: Option<char>,
    }

    let registry = facet_generate::reflect!(String).unwrap();
    for plugin in [
        Arc::new(BincodePlugin) as Arc<dyn EmitterPlugin<typescript::TypeScript>>,
        Arc::new(JsonPlugin),
    ] {
        let dir = tempdir().unwrap();
        let mut installer = typescript::Installer::new("testing", dir.path());
        installer.install_serde_runtime().unwrap();
        installer.install_bincode_runtime().unwrap();

        let source_path = dir.path().join("testing.ts");
        let mut source = File::create(&source_path).unwrap();
        let config = CodeGeneratorConfig::new("testing".to_string());
        let generator =
            typescript::TypeScriptCodeGenerator::new(&config).with_plugins(vec![plugin]);
        generator.output(&mut source, &registry).unwrap();
        drop(source);

        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("check")
            .arg("--sloppy-imports")
            .arg(&source_path)
            .status()
            .unwrap();
        assert!(status.success(), "deno check failed");
    }
}

/// Generate `registry` with the installer and `plugin`, then type-check every
/// module it wrote.
fn assert_installed_modules_type_check(
    registry: &facet_generate::Registry,
    plugin: impl EmitterPlugin<typescript::TypeScript> + 'static,
) {
    let dir = tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(plugin)
        .generate(registry)
        .unwrap();

    let mut modules: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
        .collect();
    modules.sort();
    assert!(!modules.is_empty(), "the installer wrote no modules");

    let status = Command::new("deno")
        .current_dir(dir.path())
        .arg("check")
        .arg("--sloppy-imports")
        .args(&modules)
        .status()
        .unwrap();
    assert!(status.success(), "deno check failed");
}

/// An enum from another namespace is serialized through the standalone
/// functions its own module exports (`Kit.serializePresence`), not as if it
/// were a class, and a type in a module keeps its own enum's bare name. A
/// namespaced module reaches ROOT types through the root module
/// (`Example.Shared`), and its own `Presence` still means the local struct.
#[test]
fn test_that_typescript_code_with_enums_from_other_namespaces_type_checks() {
    for registry in [
        common::across_namespaces::get_registry(),
        common::across_namespaces::get_sibling_registry(),
        common::across_namespaces::to_root::get_registry(),
        common::across_namespaces::inherited::get_registry(),
    ] {
        assert_installed_modules_type_check(&registry, BincodePlugin);
        assert_installed_modules_type_check(&registry, JsonPlugin);
    }
}

/// A plugin whose output in the root module names `Kit.Presence`, a type
/// nothing else in that module references.
#[derive(Debug)]
struct NamesPresencePlugin;

impl EmitterPlugin<typescript::TypeScript> for NamesPresencePlugin {
    fn referenced_types(
        &self,
        config: &CodeGeneratorConfig,
    ) -> Vec<facet_generate::reflection::format::QualifiedTypeName> {
        if config.module_name() == "example" {
            vec![
                facet_generate::reflection::format::QualifiedTypeName::namespaced(
                    "kit".to_string(),
                    "Presence".to_string(),
                ),
            ]
        } else {
            vec![]
        }
    }

    fn module_helpers(
        &self,
        w: &mut dyn facet_generate::generation::indent::IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> std::io::Result<()> {
        if config.module_name() == "example" {
            writeln!(w, "export type CurrentPresence = Kit.Presence;")?;
        }
        Ok(())
    }
}

/// A type that only a plugin's output names is imported, so the module
/// type-checks (redbadger/crux#614).
#[test]
fn test_that_typescript_code_naming_a_plugin_s_referenced_type_type_checks() {
    use facet_generate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Presence {
        online: bool,
    }

    #[derive(Facet)]
    struct App {
        id: u32,
    }

    let registry = facet_generate::reflect!(App, Presence).unwrap();
    assert_installed_modules_type_check(&registry, NamesPresencePlugin);
}

/// Generate `registry` with the installer and the JSON plugin, then
/// type-check every module it wrote, and the JSON runtime.
fn assert_json_modules_type_check(registry: &facet_generate::Registry) {
    let dir = tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(JsonPlugin)
        .generate(registry)
        .unwrap();
    let runtime = dir.path().join("serde/json.ts");
    assert!(runtime.exists(), "no JSON runtime");

    let mut modules: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
        .collect();
    modules.sort();
    modules.push(runtime);

    let status = Command::new("deno")
        .current_dir(dir.path())
        .arg("check")
        .arg("--sloppy-imports")
        .args(&modules)
        .status()
        .unwrap();
    assert!(status.success(), "deno check failed");
}

/// The JSON plugin's code, and its runtime, type-check for every shape the
/// fixtures hold, for types shadowing builtins or named like keywords, and
/// across namespaces.
#[test]
fn test_that_typescript_json_code_type_checks() {
    for registry in [
        common::get_registry(),
        common::get_keyword_registry(),
        common::get_shadowing_registry(),
        common::get_uuid_registry(),
        common::across_namespaces::get_registry(),
        common::across_namespaces::get_sibling_registry(),
        common::across_namespaces::to_root::get_registry(),
        common::across_namespaces::inherited::get_registry(),
        serde_namespace_registry(),
    ] {
        assert_json_modules_type_check(&registry);
    }
}

/// A ROOT type holding one in the namespace `serde`, whose module is
/// `serde.ts`, beside the JSON runtime in `serde/`.
fn serde_namespace_registry() -> facet_generate::Registry {
    use facet_generate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "serde")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    facet_generate::reflect!(App).unwrap()
}

/// A field whose wire name isn't an identifier type-checks under the Bincode
/// plugin, on a struct and on a struct variant (#197). Written bare,
/// `this.with-dash` reads as `this.with - dash` (`TS2339 [ERROR]: Property
/// 'with' does not exist on type 'Renamed'.`, `TS2304 [ERROR]: Cannot find
/// name 'dash'.`), and the variant's `return { kind: "Other", with-dash }`
/// doesn't parse (`SyntaxError: Expression expected`).
#[test]
fn test_that_typescript_bincode_code_with_non_identifier_field_names_type_checks() {
    #[derive(Facet)]
    struct Renamed {
        #[facet(rename = "with-dash")]
        dashed: u8,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Choice {
        Other {
            #[facet(rename = "with-dash")]
            dashed: u8,
        },
    }

    #[derive(Facet)]
    #[facet(tag = "type", content = "content")]
    #[repr(C)]
    #[allow(unused)]
    enum Adjacent {
        Other {
            #[facet(rename = "with-dash")]
            dashed: u8,
        },
    }

    let registry = facet_generate::reflect!(Renamed, Choice, Adjacent).unwrap();
    assert_installed_modules_type_check(&registry, BincodePlugin);
}

/// `[T; N]`, in every position a format nests, type-checks with no plugin,
/// with Bincode and with JSON (#190). Its `ListTuple` alias was written
/// through `Tuple`, which a module without a tuple does not declare
/// (`TS2304 [ERROR]: Cannot find name 'Tuple'.`), and Bincode read an element
/// as `[item]`, a `number[]` rather than a `[number]` (`TS2322 [ERROR]: Type
/// 'number[][]' is not assignable to type 'ListTuple<[number]>'.`).
#[test]
fn test_that_typescript_code_with_fixed_size_arrays_type_checks() {
    let registry = common::fixed_arrays::get_registry();
    for install in [
        (|i| i) as fn(typescript::Installer) -> typescript::Installer,
        |i| i.plugin(BincodePlugin),
        |i| i.plugin(JsonPlugin),
    ] {
        let dir = tempdir().unwrap();
        install(typescript::Installer::new("example", dir.path()))
            .generate(&registry)
            .unwrap();
        let module = dir.path().join("example.ts");

        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("check")
            .arg("--sloppy-imports")
            .arg(&module)
            .status()
            .unwrap();
        assert!(status.success(), "deno check failed");
    }
}

/// A tuple nested in a tuple, two tuples in one type, and a tuple inside a
/// list, an option, a map, a `[T; N]` and an enum variant, type-check with
/// Bincode and with JSON (#211). Bincode read each tuple's elements into
/// `const field0`, `const field1`, … in the enclosing scope, so a second tuple
/// declared them again (`TS2451 [ERROR]: Cannot redeclare block-scoped
/// variable 'field0'.`), and an outer tuple was built from its inner tuple's
/// elements (`TS2352 [ERROR]: Conversion of type '[number, boolean]' to type
/// '[number, [string, boolean]]' may be a mistake`).
#[test]
fn test_that_typescript_code_with_nested_tuples_type_checks() {
    let registry = common::tuples::get_registry();
    assert_installed_modules_type_check(&registry, BincodePlugin);
    assert_installed_modules_type_check(&registry, JsonPlugin);
}

/// A `Uuid` field type-checks with no plugin, with each plugin, and with both
/// on one module (#191). Only the plugins declared the `Uuid` alias, so with
/// none it was missing (`TS2304 [ERROR]: Cannot find name 'Uuid'.`), and with
/// both it was declared twice (`TS2300 [ERROR]: Duplicate identifier
/// 'Uuid'.`).
#[test]
fn test_that_typescript_code_with_a_uuid_type_checks() {
    let registry = common::get_uuid_registry();
    for install in [
        (|i| i) as fn(typescript::Installer) -> typescript::Installer,
        |i| i.plugin(BincodePlugin),
        |i| i.plugin(JsonPlugin),
        |i| i.plugin(BincodePlugin).plugin(JsonPlugin),
    ] {
        let dir = tempdir().unwrap();
        install(typescript::Installer::new("example", dir.path()))
            .generate(&registry)
            .unwrap();
        let module = dir.path().join("example.ts");

        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("check")
            .arg("--sloppy-imports")
            .arg(&module)
            .status()
            .unwrap();
        assert!(status.success(), "deno check failed");
    }
}
