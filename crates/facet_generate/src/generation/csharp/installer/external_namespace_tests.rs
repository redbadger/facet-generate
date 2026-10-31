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
