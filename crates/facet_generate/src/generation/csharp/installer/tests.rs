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
