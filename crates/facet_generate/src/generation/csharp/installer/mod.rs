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
    collections::{BTreeMap, BTreeSet},
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
        CodeGeneratorConfig, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SourceInstaller,
        collision::{self, Fix, Origin, TypeName},
        csharp::{CSharp, CSharpCodeGenerator, naming},
        module::{self, Module},
        naming::mentions,
        plugin::EmitterPlugin,
        registry_references,
    },
    reflection::format::{Namespace, QualifiedTypeName},
};
/// The language name the collision errors begin with.
const LANGUAGE: &str = "C#";

/// Installer for generated source files in C#.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`plugin`](Self::plugin) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>) -> Self {
        Self {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages: ExternalPackages::new(),
            plugins: vec![],
        }
    }

    /// Add a plugin to be used during code generation.
    #[must_use]
    pub fn plugin<P: crate::generation::plugin::EmitterPlugin<CSharp> + 'static>(
        mut self,
        plugin: P,
    ) -> Self {
        self.plugins.push(std::sync::Arc::new(plugin));
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
    /// 1. Installs the appropriate runtimes based on the active plugins
    /// 2. Splits the registry by namespace and installs each module
    /// 3. Writes the package manifest
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation or code generation step fails,
    /// and fails before writing anything when a namespace collides with a
    /// type, a builtin or another namespace's source file, or when a name in
    /// scope hides the first segment of a qualified type reference. Such
    /// output would not build, or would lose a module; the error names the
    /// namespace or package and what it collides with.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        let modules = module::split(&self.package_name, registry);
        self.check_namespaces(&modules)?;

        // Unit.cs is always required (even with no plugins) because Format::Unit
        // maps to the C# Unit struct in generated type declarations.
        self.install_core_runtime()?;

        let mut config = CodeGeneratorConfig::new(self.package_name.clone());
        config.update_from(registry);

        // Install plugin runtime files.
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

        for (m, module_registry) in &modules {
            let config = m.config().clone().with_parent(&self.package_name);
            self.install_module(&config, module_registry)?;
        }

        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    /// Fails when a namespace's module cannot be generated as it is named.
    ///
    /// A namespace becomes the C# namespace `<RootPackage>.<Namespace>`, in
    /// `UpperCamelCase`, written to `<root/package>/<namespace>/<Namespace>.cs`,
    /// and it is a member of the root package's namespace, which every
    /// generated module is nested in. So this fails when:
    ///
    /// - a namespace becomes the name of a ROOT type (`kv` beside `Kv`), which
    ///   the root package's namespace cannot hold both of (CS0101).
    /// - a namespace becomes the name of a builtin that the generated code
    ///   writes unqualified (`unit` for `Unit`), when the registry has a
    ///   format that writes it: every module would find the namespace instead.
    /// - two modules' source files differ only in case (namespaces `kv` and
    ///   `Kv`), which are the same file on a case-insensitive file system, so
    ///   one module would be lost. These are rejected whatever the file
    ///   system, so that the output does not depend on where it is generated.
    /// - a module writes a qualified type reference whose first segment is
    ///   the name of a type or namespace in scope there — one the module
    ///   declares, a ROOT type, or a namespace (a ROOT type `Example` in
    ///   package `Example`, or namespace `app` in package `App`) — which C#
    ///   would look the rest of the reference up in.
    ///
    /// Namespaces provided by external packages are not generated, so they
    /// are not checked.
    fn check_namespaces(&self, modules: &BTreeMap<Module, Registry>) -> Result<(), Error> {
        let generated: Vec<(CodeGeneratorConfig, &Registry)> = modules
            .iter()
            .filter(|(m, _)| match &m.config().namespace {
                Namespace::Root => true,
                Namespace::Named(namespace) => !self.external_packages.contains_key(namespace),
            })
            .map(|(m, registry)| {
                let mut config = m.config().clone().with_parent(&self.package_name);
                config.external_packages = self.external_packages.clone();
                (config, registry)
            })
            .collect();
        let root_types: Vec<&QualifiedTypeName> = generated
            .iter()
            .filter(|(config, _)| config.namespace == Namespace::Root)
            .flat_map(|(_, registry)| registry.keys())
            .collect();
        // Each namespace, with the C# namespace it becomes and its last segment.
        let namespaces: Vec<(&str, String, String)> = generated
            .iter()
            .filter_map(|(config, _)| match &config.namespace {
                Namespace::Root => None,
                Namespace::Named(namespace) => Some((
                    namespace.as_str(),
                    namespace_name(config.module_name()),
                    namespace.to_upper_camel_case(),
                )),
            })
            .collect();

        for (namespace, csharp_namespace, name) in &namespaces {
            let subject = format!("namespace \"{namespace}\" becomes `{csharp_namespace}`");
            if let Some(declared) = root_types
                .iter()
                .find(|t| &t.name.to_upper_camel_case() == name)
            {
                return Err(collision::error(
                    LANGUAGE,
                    subject,
                    TypeName(declared),
                    Origin::Namespace(namespace).rename_type(),
                )
                .into());
            }
            if let Some((builtin, _)) = naming::QUALIFIED_FORMATS.iter().find(|(bare, uses)| {
                bare == name
                    && modules
                        .values()
                        .flat_map(Registry::values)
                        .any(|container| mentions(container, *uses))
            }) {
                return Err(collision::error(
                    LANGUAGE,
                    subject,
                    format_args!(
                        "the builtin `{builtin}`, which the generated code writes unqualified, \
                         so every module in `{}` would find the namespace instead",
                        namespace_name(&self.package_name)
                    ),
                    Fix::ChooseNamespace,
                )
                .into());
            }
        }

        collision::check_files(
            LANGUAGE,
            generated.iter().map(|(config, _)| {
                (
                    Origin::of_namespace(&config.namespace, &self.package_name),
                    Self::source_path(config),
                )
            }),
        )?;

        for (config, registry) in &generated {
            Self::check_qualified_references(config, registry, &root_types, &namespaces)?;
        }

        Ok(())
    }

    /// Fails when `registry`'s module writes a qualified type reference whose
    /// first segment is the name of a type or namespace in scope there: a
    /// type the module declares, a ROOT type (`root_types`), or a namespace
    /// (`namespaces`, each with the C# namespace it becomes and its last
    /// segment). C# resolves the first segment through ordinary lookup, so it
    /// would look the rest of the reference up in that type or namespace.
    fn check_qualified_references(
        config: &CodeGeneratorConfig,
        registry: &Registry,
        root_types: &[&QualifiedTypeName],
        namespaces: &[(&str, String, String)],
    ) -> Result<(), Error> {
        for reference in registry_references(registry) {
            let qualified = CSharpCodeGenerator::requalify(config, &reference);
            let Namespace::Named(path) = &qualified.namespace else {
                continue;
            };
            let path = namespace_name(path);
            let first = path.split('.').next().unwrap_or(&path);
            let subject = format!(
                "`{}` refers to {} as `{path}.{}`, whose first segment is `{first}`",
                namespace_name(config.module_name()),
                TypeName(&reference),
                qualified.name.to_upper_camel_case(),
            );
            if let Some(hiding) = registry
                .keys()
                .chain(root_types.iter().copied())
                .find(|t| t.name.to_upper_camel_case() == first)
            {
                return Err(collision::error(
                    LANGUAGE,
                    subject,
                    TypeName(hiding),
                    Fix::RenameTypeOrPackage,
                )
                .into());
            }
            if let Some((namespace, csharp_namespace, _)) =
                namespaces.iter().find(|(_, _, name)| name == first)
            {
                return Err(collision::error(
                    LANGUAGE,
                    subject,
                    format_args!("namespace \"{namespace}\", which becomes `{csharp_namespace}`"),
                    Fix::ChooseNamespaceOrPackage,
                )
                .into());
            }
        }

        Ok(())
    }

    /// Where [`install_module`](SourceInstaller::install_module) writes the
    /// module for `config`, relative to the install directory, with `/` as
    /// the separator: the module name's directory, and a file named after its
    /// last segment.
    fn source_path(config: &CodeGeneratorConfig) -> String {
        let file_name = config
            .module_name()
            .rsplit('.')
            .next()
            .unwrap_or_else(|| config.module_name())
            .to_upper_camel_case();
        format!("{}/{file_name}.cs", config.module_name().replace('.', "/"))
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

        let generator =
            CSharpCodeGenerator::new(&updated_config).with_plugins(self.plugins.clone());
        generator.output(&mut file, registry)?;

        // Companion files live in the module's namespace directory, beside its
        // own source file.
        for companion in generator.companion_files(registry)? {
            std::fs::write(module_dir.join(&companion.file_name), companion.contents)?;
        }

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

/// The C# spelling of the dotted module name `name`, as the emitter declares
/// and qualifies it: each segment in `UpperCamelCase`.
fn namespace_name(name: &str) -> String {
    name.split('.')
        .map(str::to_upper_camel_case)
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "collision_tests.rs"]
mod collision_tests;
