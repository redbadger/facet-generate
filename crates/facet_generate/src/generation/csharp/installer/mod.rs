//! Project scaffolding for generated C# code.
//!
//! The [`Installer`] writes a complete, ready-to-build C# project:
//!
//! 1. **Runtime files** — always installs `Unit.cs` (core), then conditionally
//!    `ISerializer.cs`/`IDeserializer.cs`/error types (serde),
//!    `JsonSerde.cs` + `ObservableCollectionJsonConverterFactory` (JSON), or
//!    `BincodeSerializer.cs`/`BincodeDeserializer.cs`/`IFacetSerializable.cs`/
//!    `IFacetDeserializable.cs` (Bincode). All placed under `Facet/Runtime/`
//!    subdirectories.
//!
//! 2. **Per-module source files** — splits the registry by namespace and writes
//!    each to `<dotted-path>/<LeafName>.cs`. C# uses file-scoped `namespace`
//!    declarations — each namespace becomes a directory matching the dotted
//!    module path, and cross-namespace references use fully qualified dotted
//!    names (e.g. `Company.Models.Shared.Child`).
//!
//! 3. **`.csproj` manifest** — generates an `MSBuild` project file targeting
//!    `net10.0` with `CommunityToolkit.Mvvm` as a base package reference,
//!    plus `NuGet` `PackageReference` (URL) or `ProjectReference` (path) for
//!    external packages.

use std::{
    collections::BTreeSet,
    fmt::Write as _,
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use heck::ToUpperCamelCase as _;
use indoc::writedoc;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SourceInstaller,
        bincode::BincodePlugin,
        csharp::{CSharp, CSharpCodeGenerator},
        json::JsonPlugin,
        module,
        plugin::EmitterPlugin,
    },
};

/// Installer for generated source files in C#.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    encoding: Encoding,
    plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`encoding`](Self::encoding) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>) -> Self {
        Self {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages: ExternalPackages::new(),
            encoding: Encoding::default(),
            plugins: vec![],
        }
    }

    /// Set the encoding for serialization/deserialization.
    #[must_use]
    pub const fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Add a plugin to be used during code generation.
    ///
    /// When plugins are added explicitly, they take priority over the
    /// [`encoding`](Self::encoding) setting.
    #[must_use]
    pub fn plugin(mut self, plugin: impl Into<Arc<dyn EmitterPlugin<CSharp>>>) -> Self {
        self.plugins.push(plugin.into());
        self
    }

    /// Set external packages to reference.
    #[must_use]
    pub fn external_packages(mut self, packages: &[ExternalPackage]) -> Self {
        self.external_packages = packages
            .iter()
            .map(|d| (d.for_namespace.clone(), d.clone()))
            .collect();
        self
    }

    /// Generate all code for the given registry.
    ///
    /// This method:
    /// 1. Installs the appropriate runtimes based on the configured encoding
    /// 2. Splits the registry by namespace and installs each module
    /// 3. Writes the package manifest
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation or code generation step fails.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        // Unit.cs is always required (even without encoding) because Format::Unit
        // maps to the C# Unit struct in generated type declarations.
        self.install_core_runtime()?;

        let mut config = CodeGeneratorConfig::new(self.package_name.clone());
        config.update_from(registry);

        // Resolve plugins: explicit plugins take priority over encoding
        if self.plugins.is_empty() {
            self.plugins = match self.encoding {
                Encoding::Bincode => {
                    vec![Arc::new(BincodePlugin) as Arc<dyn EmitterPlugin<CSharp>>]
                }
                Encoding::Json => vec![Arc::new(JsonPlugin) as Arc<dyn EmitterPlugin<CSharp>>],
                Encoding::None => vec![],
            };
        }

        // Install encoding-specific runtime files from plugins.
        if !self.plugins.is_empty() {
            let lang = {
                let mut base = CSharp::new(&config, registry);
                for p in &self.plugins {
                    base = base.with_plugin(p.clone());
                }
                base
            };

            // Unit.cs was already written above; skip it when iterating plugin files.
            let mut written: BTreeSet<String> =
                BTreeSet::from(["Facet/Runtime/Serde/Unit.cs".to_string()]);
            for plugin in lang.plugins() {
                for file in plugin.runtime_files() {
                    if written.insert(file.relative_path.clone()) {
                        let dest = self.install_dir.join(&file.relative_path);
                        if let Some(parent) = dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        let mut f = std::fs::File::create(&dest)?;
                        f.write_all(&file.contents)?;
                    }
                }
            }
        }

        for (m, module_registry) in module::split(&self.package_name, registry) {
            let config = m.config().clone().with_parent(&self.package_name);
            self.install_module(&config, &module_registry)?;
        }

        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    /// Produce the contents of a `.csproj` project file.
    ///
    /// The manifest includes a base `CommunityToolkit.Mvvm` `NuGet` reference,
    /// plus any external `NuGet` `PackageReference` (URL) or `ProjectReference`
    /// (path) entries configured via [`external_packages`](Self::external_packages).
    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        let mut package_references = vec![
            "    <PackageReference Include=\"CommunityToolkit.Mvvm\" Version=\"8.4.0\" />"
                .to_string(),
        ];
        let mut project_references = Vec::new();

        for external_package in self.external_packages.values() {
            match &external_package.location {
                PackageLocation::Path(path) => {
                    project_references.push(format!("    <ProjectReference Include=\"{path}\" />"));
                }
                PackageLocation::Url(url) => {
                    let package_name = url
                        .split('/')
                        .next_back()
                        .filter(|segment| !segment.is_empty())
                        .map_or_else(
                            || external_package.for_namespace.clone(),
                            ToString::to_string,
                        );

                    let version = external_package
                        .version
                        .clone()
                        .unwrap_or_else(|| "1.0.0".to_string());

                    package_references.push(format!(
                        "    <PackageReference Include=\"{package_name}\" Version=\"{version}\" />"
                    ));
                }
            }
        }

        let package_refs = package_references.join("\n");
        let mut manifest = String::new();
        writedoc!(
            &mut manifest,
            r#"
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFramework>net10.0</TargetFramework>
                <ImplicitUsings>enable</ImplicitUsings>
                <Nullable>enable</Nullable>
                <RootNamespace>{package_name}</RootNamespace>
              </PropertyGroup>

              <ItemGroup>
            {package_refs}
              </ItemGroup>
            "#
        )
        .expect("writing to String cannot fail");

        if !project_references.is_empty() {
            let project_refs = project_references.join("\n");
            writedoc!(
                &mut manifest,
                r"

                  <ItemGroup>
                {project_refs}
                  </ItemGroup>
                "
            )
            .expect("writing to String cannot fail");
        }

        writedoc!(
            &mut manifest,
            r"
            </Project>
            "
        )
        .expect("writing to String cannot fail");

        manifest
    }

    fn install_core_runtime(&self) -> std::result::Result<(), Error> {
        self.install_runtime_file(
            "Facet/Runtime/Serde/Unit.cs",
            include_str!("runtime/core/Unit.cs"),
        )?;
        Ok(())
    }

    fn install_runtime_file(
        &self,
        relative_path: &str,
        content: &str,
    ) -> std::result::Result<(), Error> {
        let full_path = self.install_dir.join(relative_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(full_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl SourceInstaller for Installer {
    /// Generate a single `.cs` source file for one namespace.
    ///
    /// The directory path is derived from the dotted module name (dots become
    /// path separators). External packages are skipped — they are expected to
    /// be provided by `NuGet` or project references.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error> {
        let namespace = config.module_name().rsplit('.').next().unwrap_or_default();
        let skip_module = self.external_packages.contains_key(namespace);
        if skip_module {
            return Ok(());
        }

        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let module_path = config.module_name().replace('.', "/");
        let module_dir = self.install_dir.join(module_path);
        std::fs::create_dir_all(&module_dir)?;

        let file_name = config
            .module_name()
            .rsplit('.')
            .next()
            .unwrap_or_else(|| config.module_name())
            .to_upper_camel_case();
        let source_path = module_dir.join(format!("{file_name}.cs"));
        let mut file = std::fs::File::create(source_path)?;

        let generator = CSharpCodeGenerator::new(&updated_config)
            .with_encoding(self.encoding)
            .with_plugins(self.plugins.clone());
        generator.output(&mut file, registry)?;

        Ok(())
    }

    /// Write the `.csproj` manifest to the output directory.
    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
        let manifest = self.make_manifest(package_name);

        let manifest_path = self.install_dir.join(format!("{package_name}.csproj"));
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
