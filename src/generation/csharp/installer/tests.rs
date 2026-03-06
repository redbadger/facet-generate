use crate::{
    generation::{csharp::Installer, Encoding, ExternalPackage, PackageLocation},
    Registry,
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
        <ProjectReference Include="../internal.shared/internal.shared.csproj" />
      </ItemGroup>
    </Project>
    "#);
}

#[test]
fn test_generate_bincode_installs_runtime_files() {
    let install_dir = tempfile::tempdir().unwrap();
    let installer = Installer::new("Example.Types", install_dir.path()).encoding(Encoding::Bincode);
    let registry = Registry::new();

    installer.generate(&registry).unwrap();

    assert!(install_dir
        .path()
        .join("Facet/Runtime/Serde/ISerializer.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Serde/IDeserializer.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Serde/DeserializationError.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Json/JsonSerde.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Serde/Option.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Serde/Unit.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Bincode/BincodeSerializer.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Bincode/BincodeDeserializer.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Bincode/IFacetSerializable.cs")
        .exists());
    assert!(install_dir
        .path()
        .join("Facet/Runtime/Bincode/IFacetDeserializable.cs")
        .exists());

    let serializer = std::fs::read_to_string(
        install_dir
            .path()
            .join("Facet/Runtime/Bincode/BincodeSerializer.cs"),
    )
    .unwrap();
    assert!(serializer.contains("new BincodeSerializer()"));
    assert!(serializer.contains("IFacetSerializable"));
}
