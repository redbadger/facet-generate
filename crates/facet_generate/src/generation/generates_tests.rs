//! Tests that [`CodeGeneratorConfig::generates`] picks out the module that
//! generates a type, in the configs each language's installer hands its
//! plugins.
//!
//! Every test installs one small registry — a ROOT struct and a `kit` struct
//! — through the language's real installer, with a plugin that records every
//! config it is handed, and checks each recorded config: the root module's
//! generates the ROOT type and not the `kit` one, and the `kit` module's the
//! other way round.
//!
//! # Coverage
//!
//! | Language | Package | Modules |
//! |---|---|---|
//! | Swift | `Example` | `Example`, `kit` |
//! | TypeScript | `shared` | `shared`, `kit` (whose `parent` is set) |
//! | Kotlin | `com.example` | `com.example`, `com.example.kit` |
//! | C# | `Example.Shared` (dotted) and `Example` | `<package>`, `<package>.kit` |

use std::{
    collections::BTreeMap,
    io,
    sync::{Arc, Mutex},
};

use super::{
    CodeGeneratorConfig,
    indent::IndentWrite,
    plugin::{CompanionFile, EmitContext, EmitterPlugin},
};
use crate::{
    Registry,
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName},
};

/// Records every config a plugin is handed, keyed by module name.
#[derive(Debug, Clone, Default)]
struct Recorder(Arc<Mutex<BTreeMap<String, Vec<CodeGeneratorConfig>>>>);

impl Recorder {
    fn record(&self, config: &CodeGeneratorConfig) {
        self.0
            .lock()
            .unwrap()
            .entry(config.module_name().to_string())
            .or_default()
            .push(config.clone());
    }

    fn configs(&self) -> BTreeMap<String, Vec<CodeGeneratorConfig>> {
        self.0.lock().unwrap().clone()
    }
}

impl<L> EmitterPlugin<L> for Recorder {
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        self.record(config);
        vec![]
    }

    fn after_type(&self, _w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        self.record(ctx.config);
        Ok(())
    }

    fn companion_files(&self, config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        self.record(config);
        vec![]
    }

    fn target_dependencies(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        self.record(config);
        vec![]
    }

    fn referenced_types(&self, config: &CodeGeneratorConfig) -> Vec<QualifiedTypeName> {
        self.record(config);
        vec![]
    }
}

fn app() -> QualifiedTypeName {
    QualifiedTypeName::root("App".to_string())
}

fn presence() -> QualifiedTypeName {
    QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string())
}

/// A ROOT struct `App` with a field of the `kit` struct `Presence`.
fn registry() -> Registry {
    Registry::from([
        (
            app(),
            ContainerFormat::Struct(
                vec![Named {
                    name: "presence".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(presence()),
                }],
                Doc::new(),
            ),
        ),
        (
            presence(),
            ContainerFormat::Struct(
                vec![Named {
                    name: "online".to_string(),
                    doc: Doc::new(),
                    value: Format::Bool,
                }],
                Doc::new(),
            ),
        ),
    ])
}

/// Checks that the plugin saw configs for exactly `root_module` and
/// `kit_module`, and that each one generates its own module's type only.
fn assert_generates(recorder: &Recorder, root_module: &str, kit_module: &str) {
    let configs = recorder.configs();
    assert_eq!(
        configs.keys().map(String::as_str).collect::<Vec<_>>(),
        {
            let mut expected = vec![root_module, kit_module];
            expected.sort_unstable();
            expected
        },
        "modules the plugin was handed a config for"
    );

    for config in &configs[root_module] {
        assert!(config.generates(&app()), "{root_module} generates App");
        assert!(
            !config.generates(&presence()),
            "{root_module} does not generate kit.Presence"
        );
    }
    for config in &configs[kit_module] {
        assert!(
            config.generates(&presence()),
            "{kit_module} generates kit.Presence"
        );
        assert!(
            !config.generates(&app()),
            "{kit_module} does not generate App"
        );
    }
}

#[test]
fn swift() {
    let recorder = Recorder::default();
    let install_dir = tempfile::tempdir().unwrap();
    super::swift::Installer::new("Example", install_dir.path())
        .plugin(recorder.clone())
        .generate(&registry())
        .unwrap();

    assert_generates(&recorder, "Example", "kit");
}

#[test]
fn typescript() {
    let recorder = Recorder::default();
    let install_dir = tempfile::tempdir().unwrap();
    super::typescript::Installer::new("shared", install_dir.path())
        .plugin(recorder.clone())
        .generate(&registry())
        .unwrap();

    assert_generates(&recorder, "shared", "kit");
    for config in &recorder.configs()["kit"] {
        assert_eq!(config.parent.as_deref(), Some("shared"));
    }
}

#[test]
fn kotlin() {
    let recorder = Recorder::default();
    let install_dir = tempfile::tempdir().unwrap();
    super::kotlin::Installer::new("com.example", install_dir.path())
        .plugin(recorder.clone())
        .generate(&registry())
        .unwrap();

    assert_generates(&recorder, "com.example", "com.example.kit");
}

#[test]
fn csharp_dotted_package() {
    let recorder = Recorder::default();
    let install_dir = tempfile::tempdir().unwrap();
    super::csharp::Installer::new("Example.Shared", install_dir.path())
        .plugin(recorder.clone())
        .generate(&registry())
        .unwrap();

    assert_generates(&recorder, "Example.Shared", "Example.Shared.kit");
}

#[test]
fn csharp_undotted_package() {
    let recorder = Recorder::default();
    let install_dir = tempfile::tempdir().unwrap();
    super::csharp::Installer::new("Example", install_dir.path())
        .plugin(recorder.clone())
        .generate(&registry())
        .unwrap();

    assert_generates(&recorder, "Example", "Example.kit");
}

/// A config made with [`CodeGeneratorConfig::new`] is a root module's.
#[test]
fn a_new_config_generates_root_types() {
    let config = CodeGeneratorConfig::new("shared".to_string());
    assert!(config.generates(&app()));
    assert!(!config.generates(&presence()));
}

/// The answer does not depend on the registry having the type.
#[test]
fn a_type_missing_from_the_registry_is_placed_by_its_namespace() {
    let modules = super::module::split("shared", &registry());
    let absent_root = QualifiedTypeName::root("Missing".to_string());
    let absent_kit = QualifiedTypeName::namespaced("kit".to_string(), "Missing".to_string());
    for module in modules.keys() {
        let config = module.config();
        let is_root = config.module_name() == "shared";
        assert_eq!(config.generates(&absent_root), is_root);
        assert_eq!(config.generates(&absent_kit), !is_root);
    }
}
