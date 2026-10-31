use super::*;
use crate as fg;
use crate::{generation::bincode::BincodePlugin, reflect};
use facet::Facet;

#[test]
fn invalid_external_namespace_mappings_fail_before_writing() {
    for (with_package, namespace, message) in [
        (false, "Vendor.Models", "requires an external package entry"),
        (true, "", "requires a nonempty dotted CLR namespace"),
        (
            true,
            "Vendor..Models",
            "requires a nonempty dotted CLR namespace",
        ),
        (true, "global::Vendor.Models", "without an alias qualifier"),
        (
            true,
            "Exa\u{200D}mple.App",
            "requires plain CLR identifiers",
        ),
        (true, r"Ex\u0061mple.App", "requires plain CLR identifiers"),
        (true, "Example .App", "requires plain CLR identifiers"),
        (
            true,
            "Example/*comment*/.App",
            "requires plain CLR identifiers",
        ),
        (true, "Example.\nApp", "requires plain CLR identifiers"),
        (true, "Example.2Models", "requires plain CLR identifiers"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let mut installer =
            Installer::new("Example.App", dir.path()).external_namespace("geometry", namespace);
        if with_package {
            installer = installer.external_packages(&[ExternalPackage {
                for_namespace: "geometry".to_string(),
                location: PackageLocation::Path("../Contracts/Contracts.csproj".to_string()),
                module_name: None,
                version: None,
            }]);
        }
        let error = installer.generate(&Registry::new()).unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        assert!(dir.path().read_dir().unwrap().next().is_none());
    }
}

#[test]
fn plain_unicode_clr_namespaces_are_supported() {
    for namespace in [
        "例.模型",
        "Δοκιμή.Μοντέλα",
        "_Vendor.Models2",
        "Ⅳ.Models",
        "Vendor.@event",
    ] {
        let dir = tempfile::tempdir().unwrap();
        external_geometry(dir.path(), namespace)
            .generate(&collision_registry(None, "Local", "Point"))
            .unwrap();
    }
}

#[test]
fn namespace_mapping_keeps_the_nuget_package_id_and_skips_external_types() {
    #[derive(Facet)]
    #[facet(fg::namespace = "geometry")]
    struct Point {
        x: u32,
    }

    #[derive(Facet)]
    struct Envelope {
        point: Point,
    }

    let dir = tempfile::tempdir().unwrap();
    Installer::new("Example.App", dir.path())
        .external_namespace("geometry", "Vendor.API.Models")
        .external_packages(&[ExternalPackage {
            for_namespace: "geometry".to_string(),
            location: PackageLocation::Url("https://www.nuget.org/packages/Other.Id".to_string()),
            module_name: Some("Vendor.PackageId".to_string()),
            version: Some("1.2.3".to_string()),
        }])
        .plugin(BincodePlugin)
        .generate(&reflect!(Envelope).unwrap())
        .unwrap();

    let manifest = std::fs::read_to_string(dir.path().join("Example.App.csproj")).unwrap();
    assert!(
        manifest.contains(r#"<PackageReference Include="Vendor.PackageId" Version="1.2.3" />"#)
    );
    let source = std::fs::read_to_string(dir.path().join("Example/App/App.cs")).unwrap();
    assert!(source.contains("global::Vendor.API.Models.Point"));
    assert!(!dir.path().join("Example/App/geometry/Geometry.cs").exists());
}

fn collision_registry(local_namespace: Option<&str>, local: &str, external: &str) -> Registry {
    use crate::reflection::format::{ContainerFormat, Doc, Format, Named};

    let mut registry = Registry::new();
    let external = QualifiedTypeName::namespaced("geometry".to_string(), external.to_string());
    registry.insert(external.clone(), ContainerFormat::UnitStruct(Doc::new()));
    let local = match local_namespace {
        Some(namespace) => QualifiedTypeName::namespaced(namespace.to_string(), local.to_string()),
        None => QualifiedTypeName::root(local.to_string()),
    };
    registry.insert(local, ContainerFormat::UnitStruct(Doc::new()));
    registry.insert(
        QualifiedTypeName::root("Envelope".to_string()),
        ContainerFormat::Struct(
            vec![Named::new(&Format::TypeName(external), "point".to_string())],
            Doc::new(),
        ),
    );
    registry
}

fn external_geometry(dir: &Path, mapped: &str) -> Installer {
    Installer::new("Example.App", dir)
        .external_packages(&[ExternalPackage {
            for_namespace: "geometry".to_string(),
            location: PackageLocation::Path("../Contracts/Contracts.csproj".to_string()),
            module_name: None,
            version: None,
        }])
        .external_namespace("geometry", mapped)
}

#[test]
fn mapped_clr_collisions_fail_before_writing() {
    for (namespace, local, external, mapped, diagnostic) in [
        (
            None,
            "Point",
            "Point",
            "Example.App",
            "conflicts with generated type `Point`",
        ),
        (
            Some("views"),
            "Point",
            "Point",
            "Example.App.Views",
            "Example.App.Views.Point",
        ),
        (None, "Point", "Point", "@Example.@App", "Example.App.Point"),
        (
            None,
            "Point",
            "Row",
            "Example.App.Point.Child",
            "conflicts with generated type `Point`",
        ),
        (
            Some("point"),
            "Row",
            "Point",
            "Example.App",
            "is a generated namespace",
        ),
        (
            Some("deep.child"),
            "Row",
            "Deep",
            "Example.App",
            "is a generated namespace",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let registry = collision_registry(namespace, local, external);
        let error = external_geometry(dir.path(), mapped)
            .generate(&registry)
            .unwrap_err();
        let Error::Io(error) = error else {
            panic!("expected an invalid-input error")
        };
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains(diagnostic), "{mapped}: {error}");
        assert!(
            dir.path().read_dir().unwrap().next().is_none(),
            "{mapped}: output was written"
        );
    }
}

#[test]
fn mapped_clr_names_keep_case_and_segment_boundaries() {
    for (mapped, external) in [
        ("example.App", "Point"),
        ("Example.App.Points", "Point"),
        ("Example.App", "Other"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        external_geometry(dir.path(), mapped)
            .generate(&collision_registry(None, "Point", external))
            .unwrap();
    }
}

#[test]
fn mapped_types_cannot_bind_to_builtin_plugin_helpers() {
    use crate::generation::json::JsonPlugin;
    use crate::reflection::format::{
        ContainerFormat, Doc, EnumTagging, Format, Named, VariantFormat,
    };

    for (external, field, json) in [
        ("LocalBincode", None, false),
        ("LocalJsonConverter", None, true),
        ("CharSerde", Some(Format::Char), false),
        ("UuidSerde", Some(Format::Uuid), false),
        ("JsonCharConverter", Some(Format::Char), true),
    ] {
        let mut registry = collision_registry(None, "Local", external);
        let local = match field {
            Some(field) => {
                ContainerFormat::Struct(vec![Named::new(&field, "value".to_string())], Doc::new())
            }
            None => ContainerFormat::Enum(
                BTreeMap::from([(0, Named::new(&VariantFormat::Unit, "Value".to_string()))]),
                EnumTagging::External,
                Doc::new(),
            ),
        };
        registry.insert(QualifiedTypeName::root("Local".to_string()), local);
        let dir = tempfile::tempdir().unwrap();
        let installer = external_geometry(dir.path(), "Example.App");
        let installer = if json {
            installer.plugin(JsonPlugin)
        } else {
            installer.plugin(BincodePlugin)
        };
        let error = installer.generate(&registry).unwrap_err();
        assert!(error.to_string().contains("plugin declaration"), "{error}");
        assert!(error.to_string().contains(external), "{error}");
        assert!(dir.path().read_dir().unwrap().next().is_none());
    }
}

#[derive(Debug)]
struct DeclaredHelper;

impl EmitterPlugin<CSharp> for DeclaredHelper {
    fn module_declarations(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec!["@CustomHelper".to_string()]
    }

    fn companion_files(
        &self,
        _config: &CodeGeneratorConfig,
    ) -> Vec<crate::generation::plugin::CompanionFile> {
        vec![crate::generation::plugin::CompanionFile {
            file_name: "Custom.cs".to_string(),
            imports: vec![],
            contents: "public class @CustomHelper {}".to_string(),
        }]
    }
}

#[test]
fn mapped_types_cannot_bind_to_declared_companion_types() {
    let dir = tempfile::tempdir().unwrap();
    let error = external_geometry(dir.path(), "Example.App")
        .plugin(DeclaredHelper)
        .generate(&collision_registry(None, "Local", "CustomHelper"))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("plugin declaration `@CustomHelper`"),
        "{error}"
    );
    assert!(dir.path().read_dir().unwrap().next().is_none());
}

#[derive(Debug)]
struct EnumConditionalHelper;

impl EmitterPlugin<CSharp> for EnumConditionalHelper {
    fn module_declarations(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let local = QualifiedTypeName::root("Local".to_string());
        let rendered = crate::generation::csharp::requalify(config, &local);
        if config.is_unit_enum(&rendered) {
            vec!["CustomHelper".to_string()]
        } else {
            vec![]
        }
    }

    fn module_helpers(
        &self,
        w: &mut dyn crate::generation::IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> std::io::Result<()> {
        if !self.module_declarations(config).is_empty() {
            w.write_all(b"public class CustomHelper {}\n")?;
        }
        Ok(())
    }
}

#[test]
fn declaration_metadata_uses_the_emission_enum_config() {
    use crate::reflection::format::{ContainerFormat, Doc, EnumTagging, Named, VariantFormat};

    let mut registry = collision_registry(None, "Local", "CustomHelper");
    registry.insert(
        QualifiedTypeName::root("Local".to_string()),
        ContainerFormat::Enum(
            BTreeMap::from([(0, Named::new(&VariantFormat::Unit, "Value".to_string()))]),
            EnumTagging::External,
            Doc::new(),
        ),
    );
    let dir = tempfile::tempdir().unwrap();
    let error = external_geometry(dir.path(), "Example.App")
        .plugin(EnumConditionalHelper)
        .generate(&registry)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("plugin declaration `CustomHelper`"),
        "{error}"
    );
    assert!(dir.path().read_dir().unwrap().next().is_none());
}

#[derive(Debug, Clone, Copy)]
enum RuntimeCodec {
    Core,
    Bincode,
    Json,
}

#[rstest::rstest]
#[case::core_unit("Facet.Runtime.Serde", "Unit", RuntimeCodec::Core)]
#[case::serde_interface("Facet.Runtime.Serde", "IDeserializer", RuntimeCodec::Bincode)]
#[case::bincode_serializer("Facet.Runtime.Bincode", "BincodeSerializer", RuntimeCodec::Bincode)]
#[case::json_serde("Facet.Runtime.Json", "JsonSerde", RuntimeCodec::Json)]
#[case::json_enum("Facet.Runtime.Json", "JsonEnum", RuntimeCodec::Json)]
#[case::runtime_namespace("Facet.Runtime", "Serde", RuntimeCodec::Core)]
fn installed_runtime_types_participate_in_collision_checks(
    #[case] namespace: &str,
    #[case] name: &str,
    #[case] codec: RuntimeCodec,
    #[values(false, true)] external_serde: bool,
) {
    use crate::generation::json::JsonPlugin;

    let registry = collision_registry(None, "Local", name);
    let dir = tempfile::tempdir().unwrap();
    let mut packages = vec![ExternalPackage {
        for_namespace: "geometry".to_string(),
        location: PackageLocation::Path("../Contracts/Contracts.csproj".to_string()),
        module_name: None,
        version: None,
    }];
    if external_serde {
        packages.push(ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Runtime/Runtime.csproj".to_string()),
            module_name: None,
            version: None,
        });
    }
    let installer = external_geometry(dir.path(), namespace).external_packages(&packages);
    let installer = match codec {
        RuntimeCodec::Bincode => installer.plugin(BincodePlugin),
        RuntimeCodec::Json => installer.plugin(JsonPlugin),
        RuntimeCodec::Core => installer,
    };
    let result = installer.generate(&registry);
    if external_serde {
        result.unwrap();
        assert!(!dir.path().join("Facet").exists());
    } else {
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("runtime declaration")
                || error.to_string().contains("generated namespace"),
            "{error}"
        );
        assert!(dir.path().read_dir().unwrap().next().is_none());
    }
}

#[test]
fn generic_runtime_declarations_do_not_conflict_with_nongeneric_types() {
    let dir = tempfile::tempdir().unwrap();
    external_geometry(dir.path(), "Facet.Runtime.Bincode")
        .plugin(BincodePlugin)
        .generate(&collision_registry(None, "Local", "IFacetDeserializable"))
        .unwrap();
}

#[derive(Debug)]
struct CustomRuntime;

impl EmitterPlugin<CSharp> for CustomRuntime {
    fn runtime_declarations(&self) -> Vec<String> {
        vec!["Vendor.Runtime.Helper".to_string()]
    }

    fn runtime_files(&self) -> Vec<crate::generation::plugin::RuntimeFile> {
        vec![crate::generation::plugin::RuntimeFile {
            relative_path: "Vendor/Helper.cs".to_string(),
            contents: b"namespace Vendor.Runtime; public class Helper {}".to_vec(),
        }]
    }
}

#[test]
fn custom_runtime_metadata_is_included() {
    let dir = tempfile::tempdir().unwrap();
    let error = external_geometry(dir.path(), "Vendor.Runtime")
        .plugin(CustomRuntime)
        .generate(&collision_registry(None, "Local", "Helper"))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("runtime declaration `Vendor.Runtime.Helper`"),
        "{error}"
    );
    assert!(dir.path().read_dir().unwrap().next().is_none());
}

#[derive(Debug)]
struct SourceSpellingHelper {
    name: &'static str,
    runtime: bool,
}

impl EmitterPlugin<CSharp> for SourceSpellingHelper {
    fn module_declarations(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        if self.runtime {
            vec![]
        } else {
            vec![self.name.to_string()]
        }
    }

    fn module_helpers(
        &self,
        w: &mut dyn crate::generation::IndentWrite,
        _config: &CodeGeneratorConfig,
    ) -> std::io::Result<()> {
        if !self.runtime {
            writeln!(w, "public class {} {{}}", self.name)?;
        }
        Ok(())
    }

    fn runtime_declarations(&self) -> Vec<String> {
        if self.runtime {
            vec![format!("Vendor.Runtime.{}", self.name)]
        } else {
            vec![]
        }
    }

    fn runtime_files(&self) -> Vec<crate::generation::plugin::RuntimeFile> {
        if !self.runtime {
            return vec![];
        }
        vec![crate::generation::plugin::RuntimeFile {
            relative_path: "Vendor/Helper.cs".to_string(),
            contents: format!("namespace Vendor.Runtime; public class {} {{}}", self.name)
                .into_bytes(),
        }]
    }
}

#[test]
fn declaration_metadata_rejects_source_spelling_aliases() {
    for name in ["Cust\u{200D}omHelper", r"Cust\u006FmHelper"] {
        for runtime in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let namespace = if runtime {
                "Vendor.Runtime"
            } else {
                "Example.App"
            };
            let error = external_geometry(dir.path(), namespace)
                .plugin(SourceSpellingHelper { name, runtime })
                .generate(&collision_registry(None, "Local", "CustomHelper"))
                .unwrap_err();
            assert!(
                error.to_string().contains("requires a plain CLR type"),
                "{error}"
            );
            assert!(dir.path().read_dir().unwrap().next().is_none());
        }
    }
}
