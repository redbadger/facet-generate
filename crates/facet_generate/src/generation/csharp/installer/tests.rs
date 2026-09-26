//! Snapshot tests for the C# [`Installer`] — **project scaffolding**.
//!
//! # Coverage
//!
//! - Basic `.csproj` manifest generation
//! - External `NuGet` URL dependencies (`PackageReference`)
//! - External path dependencies (`ProjectReference`)
//! - Bincode runtime file installation (serde interfaces, serializer,
//!   deserializer, error types)
//! - JSON runtime installation (`JsonSerde.cs`)
//! - No plugins skips serde/bincode runtimes
//! - Core `Unit.cs` always present regardless of plugins
//! - Plugin companion files written into the module's namespace directory

use facet::Facet;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, PackageLocation,
        bincode::BincodePlugin,
        csharp::{CSharp, Installer},
        json::JsonPlugin,
        plugin::{CompanionFile, EmitterPlugin},
    },
    reflect,
};

#[test]
fn test_new_installer() {
    let _installer = Installer::new("Example.Types", "/tmp");
}

#[test]
fn test_make_manifest_basic() {
    let installer = Installer::new("Example.Types", "/tmp");
    let manifest = installer.make_manifest("Example.Types");

    insta::assert_snapshot!(manifest, @r#"
    <Project Sdk="Microsoft.NET.Sdk">
      <PropertyGroup>
        <TargetFramework>net10.0</TargetFramework>
        <ImplicitUsings>enable</ImplicitUsings>
        <Nullable>enable</Nullable>
        <RootNamespace>Example.Types</RootNamespace>
      </PropertyGroup>

      <ItemGroup>
        <PackageReference Include="CommunityToolkit.Mvvm" Version="8.4.0" />
      </ItemGroup>
    </Project>
    "#);
}

#[test]
fn test_make_manifest_with_external_packages() {
    let external_packages = vec![
        ExternalPackage {
            for_namespace: "internal.shared".to_string(),
            module_name: None,
            location: PackageLocation::Path(
                "../internal.shared/internal.shared.csproj".to_string(),
            ),
            version: None,
        },
        ExternalPackage {
            for_namespace: "acme.types".to_string(),
            module_name: None,
            location: PackageLocation::Url("https://nuget.org/packages/Acme.Contracts".to_string()),
            version: Some("2.4.1".to_string()),
        },
    ];

    let installer = Installer::new("Example.Types", "/tmp").external_packages(&external_packages);
    let manifest = installer.make_manifest("Example.Types");

    insta::assert_snapshot!(manifest, @r#"
    <Project Sdk="Microsoft.NET.Sdk">
      <PropertyGroup>
        <TargetFramework>net10.0</TargetFramework>
        <ImplicitUsings>enable</ImplicitUsings>
        <Nullable>enable</Nullable>
        <RootNamespace>Example.Types</RootNamespace>
      </PropertyGroup>

      <ItemGroup>
        <PackageReference Include="CommunityToolkit.Mvvm" Version="8.4.0" />
        <PackageReference Include="Acme.Contracts" Version="2.4.1" />
      </ItemGroup>

      <ItemGroup>
        <ProjectReference Include="../internal.shared/internal.shared.csproj" />
      </ItemGroup>
    </Project>
    "#);
}

#[test]
fn test_generate_bincode_installs_runtime_files() {
    let install_dir = tempfile::tempdir().unwrap();
    let installer = Installer::new("Example.Types", install_dir.path()).plugin(BincodePlugin);
    let registry = Registry::new();

    installer.generate(&registry).unwrap();

    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/ISerializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/IDeserializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/DeserializationError.cs")
            .exists()
    );
    assert!(
        !install_dir
            .path()
            .join("Facet/Runtime/Json/JsonSerde.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/Unit.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/BincodeSerializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/BincodeDeserializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/IFacetSerializable.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/IFacetDeserializable.cs")
            .exists()
    );

    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/SerializationError.cs")
            .exists()
    );

    let serializer = std::fs::read_to_string(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/BincodeSerializer.cs"),
    )
    .unwrap();
    assert!(serializer.contains("new BincodeSerializer()"));
    assert!(serializer.contains("IFacetSerializable"));
}

#[test]
fn test_generate_no_encoding_skips_runtime_files() {
    let install_dir = tempfile::tempdir().unwrap();
    let installer = Installer::new("Example.Types", install_dir.path());
    let registry = Registry::new();

    installer.generate(&registry).unwrap();

    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/Unit.cs")
            .exists()
    );
    assert!(
        !install_dir
            .path()
            .join("Facet/Runtime/Serde/ISerializer.cs")
            .exists()
    );
    assert!(!install_dir.path().join("Facet/Runtime/Bincode").exists());
    assert!(!install_dir.path().join("Facet/Runtime/Json").exists());
    assert!(install_dir.path().join("Example.Types.csproj").exists());
}

#[test]
fn test_generate_json_encoding_installs_serde_but_not_bincode() {
    let install_dir = tempfile::tempdir().unwrap();
    let installer = Installer::new("Example.Types", install_dir.path()).plugin(JsonPlugin);
    let registry = Registry::new();

    installer.generate(&registry).unwrap();

    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/ISerializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/IDeserializer.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/SerializationError.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/DeserializationError.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Serde/Unit.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Json/JsonSerde.cs")
            .exists()
    );
    assert!(
        install_dir
            .path()
            .join("Facet/Runtime/Json/FacetJson.cs")
            .exists()
    );
    assert!(!install_dir.path().join("Facet/Runtime/Bincode").exists());
}

/// A plugin standing in for one that bridges to an FFI package: it contributes
/// a companion source file that needs a `using` of its own.
#[derive(Debug)]
struct FfiPlugin;

impl EmitterPlugin<CSharp> for FfiPlugin {
    fn companion_files(&self, _config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        vec![CompanionFile {
            file_name: "FfiBridge.cs".to_string(),
            imports: vec!["using Example.Shared;".to_string()],
            contents: "public sealed class FfiBridge;".to_string(),
        }]
    }
}

#[test]
fn companion_file_is_written_in_the_namespace_directory() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("Example.Types", install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    let companion =
        std::fs::read_to_string(install_dir.path().join("Example/Types/FfiBridge.cs")).unwrap();

    // The module's usings and namespace, merged with the companion's, and none
    // of the module helpers.
    insta::assert_snapshot!(companion, @r"
    using CommunityToolkit.Mvvm.ComponentModel;
    using Facet.Runtime.Serde;
    using System.Collections.Generic;
    using System.Collections.ObjectModel;
    using Facet.Runtime.Bincode;
    using Example.Shared;

    namespace Example.Types;

    public sealed class FfiBridge;
    ");
}

/// Generates `App`, which holds `Ext` of namespace `shared`, as package
/// `package` with `shared` provided by an external project, and returns the
/// install directory.
fn generate_with_external_shared(package: &str) -> tempfile::TempDir {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Ext {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        e: Ext,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new(package, install_dir.path())
        .plugin(BincodePlugin)
        .external_packages(&[ExternalPackage {
            for_namespace: "shared".to_string(),
            module_name: None,
            location: PackageLocation::Path("../Shared/Shared.csproj".to_string()),
            version: None,
        }])
        .generate(&registry)
        .unwrap();
    install_dir
}

fn files_in(dir: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(
                    path.strip_prefix(dir)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    files.retain(|f| !f.starts_with("Facet/"));
    files.sort();
    files
}

/// The root module is named after the package, whose last segment `shared`
/// is spelled like the external namespace: it is still written (#186).
#[test]
fn root_module_is_written_when_the_package_ends_in_an_external_namespace() {
    let install_dir = generate_with_external_shared("Acme.shared");

    assert_eq!(
        files_in(install_dir.path()),
        ["Acme.shared.csproj", "Acme/shared/Shared.cs"]
    );
    let root = std::fs::read_to_string(install_dir.path().join("Acme/shared/Shared.cs")).unwrap();
    assert!(root.contains("namespace Acme.Shared;"), "{root}");
    assert!(root.contains("partial class App"), "{root}");
    assert!(!root.contains("class Ext"), "{root}");

    let manifest = std::fs::read_to_string(install_dir.path().join("Acme.shared.csproj")).unwrap();
    assert!(
        manifest.contains(r#"<ProjectReference Include="../Shared/Shared.csproj" />"#),
        "{manifest}"
    );
}

/// A namespace provided by an external package is not generated, and the
/// manifest references the package instead.
#[test]
fn external_namespace_module_is_skipped() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Ext {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "internal")]
    struct Inner {
        y: u32,
    }

    #[derive(Facet)]
    struct App {
        e: Ext,
        i: Inner,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Acme", install_dir.path())
        .plugin(BincodePlugin)
        .external_packages(&[
            ExternalPackage {
                for_namespace: "shared".to_string(),
                module_name: None,
                location: PackageLocation::Path("../Shared/Shared.csproj".to_string()),
                version: None,
            },
            ExternalPackage {
                for_namespace: "internal".to_string(),
                module_name: Some("Internal.Shared".to_string()),
                location: PackageLocation::Url(
                    "https://www.nuget.org/packages/Internal.Shared/1.2.3".to_string(),
                ),
                version: None,
            },
        ])
        .generate(&registry)
        .unwrap();

    assert_eq!(
        files_in(install_dir.path()),
        ["Acme.csproj", "Acme/Acme.cs"]
    );
    let manifest = std::fs::read_to_string(install_dir.path().join("Acme.csproj")).unwrap();
    assert!(
        manifest.contains(r#"<ProjectReference Include="../Shared/Shared.csproj" />"#),
        "{manifest}"
    );
    assert!(
        manifest.contains(r#"<PackageReference Include="Internal.Shared" Version="1.2.3" />"#),
        "{manifest}"
    );
}

/// The package ID and version read from each form of `NuGet` URL.
#[test]
fn nuget_url_names_the_package() {
    use super::NuGetUrl;

    for (url, id, version) in [
        (
            "https://www.nuget.org/packages/Acme.Contracts",
            Some("Acme.Contracts"),
            None,
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/",
            Some("Acme.Contracts"),
            None,
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/2.4.1",
            Some("Acme.Contracts"),
            Some("2.4.1"),
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/2.4.1/",
            Some("Acme.Contracts"),
            Some("2.4.1"),
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/2.4.1-beta.2?tab=readme#top",
            Some("Acme.Contracts"),
            Some("2.4.1-beta.2"),
        ),
        (
            "https://www.nuget.org/api/v2/package/Acme.Contracts/2.4.1",
            Some("Acme.Contracts"),
            Some("2.4.1"),
        ),
        (
            "https://www.nuget.org/api/v2/package/Acme.Contracts",
            Some("Acme.Contracts"),
            None,
        ),
        (
            "https://api.nuget.org/v3-flatcontainer/acme.contracts/2.4.1/acme.contracts.2.4.1.nupkg",
            Some("acme.contracts"),
            Some("2.4.1"),
        ),
        (
            "https://example.com/feed/Acme.Contracts.2.4.1.1-rc.1.nupkg",
            Some("Acme.Contracts"),
            Some("2.4.1.1-rc.1"),
        ),
        (
            "https://example.com/feed/Acme.Contracts.nupkg",
            Some("Acme.Contracts"),
            None,
        ),
        (
            "https://example.com/feed/Acme.Contracts/3.0.0",
            Some("Acme.Contracts"),
            Some("3.0.0"),
        ),
        (
            "https://example.com/feed/Acme.Contracts",
            Some("Acme.Contracts"),
            None,
        ),
        ("https://example.com/2.4.1", None, Some("2.4.1")),
        ("https://example.com/", None, None),
        ("https://example.com", None, None),
        ("Acme.Contracts", Some("Acme.Contracts"), None),
    ] {
        assert_eq!(NuGetUrl::parse(url), NuGetUrl { id, version }, "{url}");
    }
}

/// The `PackageReference` a URL package becomes: named by its `module_name`
/// first, then by its URL, then by its namespace; versioned by its `version`
/// first, then by its URL, then `1.0.0`.
#[test]
fn url_package_reference_name_and_version() {
    let reference = |url: &str, module_name: Option<&str>, version: Option<&str>| {
        let installer = Installer::new("Example", "/tmp").external_packages(&[ExternalPackage {
            for_namespace: "acme".to_string(),
            module_name: module_name.map(ToString::to_string),
            location: PackageLocation::Url(url.to_string()),
            version: version.map(ToString::to_string),
        }]);
        installer
            .make_manifest("Example")
            .lines()
            .find(|line| line.contains("PackageReference") && !line.contains("CommunityToolkit"))
            .unwrap()
            .trim()
            .to_string()
    };

    for (url, module_name, version, expected) in [
        (
            "https://www.nuget.org/packages/Acme.Contracts",
            None,
            Some("2.4.1"),
            r#"<PackageReference Include="Acme.Contracts" Version="2.4.1" />"#,
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/2.4.1",
            None,
            None,
            r#"<PackageReference Include="Acme.Contracts" Version="2.4.1" />"#,
        ),
        (
            "https://www.nuget.org/api/v2/package/Acme.Contracts/2.4.1",
            None,
            Some("3.0.0"),
            r#"<PackageReference Include="Acme.Contracts" Version="3.0.0" />"#,
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/",
            None,
            None,
            r#"<PackageReference Include="Acme.Contracts" Version="1.0.0" />"#,
        ),
        (
            "https://www.nuget.org/packages/Acme.Contracts/2.4.1",
            Some("Acme.Other"),
            None,
            r#"<PackageReference Include="Acme.Other" Version="2.4.1" />"#,
        ),
        (
            "https://example.com/feed/",
            Some("Acme.Other"),
            Some("1.1.0"),
            r#"<PackageReference Include="Acme.Other" Version="1.1.0" />"#,
        ),
        (
            "https://example.com/",
            None,
            None,
            r#"<PackageReference Include="acme" Version="1.0.0" />"#,
        ),
    ] {
        assert_eq!(reference(url, module_name, version), expected, "{url}");
    }
}
