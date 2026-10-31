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
