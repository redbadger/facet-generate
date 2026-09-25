//! Project scaffolding — writes a ready-to-build TypeScript project to disk.
//!
//! The [`Installer`] is the final stage of the TypeScript generation pipeline.
//! While [`TypeScriptCodeGenerator`] produces the *contents* of a single source file,
//! the installer is responsible for the surrounding project structure:
//!
//! 1. **Runtime files** — copies the serde and/or bincode runtime `.ts`
//!    sources into the output directory, adapting file names and import paths
//!    using extensionless imports (`index.ts` entry points, `.ts` stripped
//!    from import paths).
//!
//! 2. **Per-module source files** — splits the registry by namespace (via
//!    [`module::split`]) and calls [`TypeScriptCodeGenerator`] once per namespace,
//!    writing each to its own `.ts` file. TypeScript has no `namespace`
//!    keyword here — the crate's namespace concept maps to **ES modules**
//!    (separate `.ts` files), and cross-module type references use
//!    `import * as Namespace` wildcard imports with `Namespace.Type` syntax.
//!
//! 3. **`package.json`** — generates an NPM manifest with dependencies
//!    (external packages as `file:` paths or versioned registry references,
//!    plus any pairs the plugins declare) and devDependencies (`typescript`).
//!
//! A module is a single file here, so there is nothing to put beside it:
//! [`companion_files`](crate::generation::plugin::EmitterPlugin::companion_files)
//! is ignored, and a TypeScript plugin emits extra declarations through
//! [`after_type`](crate::generation::plugin::EmitterPlugin::after_type) instead.

use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use heck::ToUpperCamelCase as _;
use serde_json::{Value, json};

use std::collections::BTreeSet;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SERDE_NAMESPACE, SourceInstaller,
        bincode::BincodePlugin,
        collision::{self, Origin, TypeName},
        json::JsonPlugin,
        module::{self, Module},
        plugin::EmitterPlugin,
        typescript::{TypeScript, TypeScriptCodeGenerator, naming},
    },
};

/// Installer for generated source files in TypeScript.
///
/// # Examples
///
/// ```rust
/// use facet_generate::generation::typescript;
///
/// let output_dir = std::path::PathBuf::from("output");
/// let installer = typescript::Installer::new("my-package", &output_dir);
/// ```
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    plugins: Vec<Arc<dyn EmitterPlugin<TypeScript>>>,
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
    ///
    /// When multiple plugins are added, they are invoked in the order they were registered.
    #[must_use]
    pub fn plugin<P: EmitterPlugin<TypeScript> + 'static>(mut self, plugin: P) -> Self {
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
    /// 1. Installs the appropriate runtimes based on the configured encoding
    /// 2. Splits the registry by namespace and installs each module
    /// 3. Writes the package manifest
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation or code generation step fails,
    /// and fails before writing anything when two modules would share a file,
    /// or when a module would import another under a name that it also
    /// declares or imports. Such output would not type-check, or would lose a
    /// module; the error names the namespace and what it collides with.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        let modules = module::split(&self.package_name, registry);
        self.check_namespaces(&modules)?;

        // Build a lang tag to get the active plugins, then use them to install
        // runtime files (replacing the old encoding-based install_serde/bincode calls).
        let mut config = CodeGeneratorConfig::new(self.package_name.clone());
        config.update_from(registry);
        let lang = {
            let mut base = TypeScript::new(&config, registry);
            for p in &self.plugins {
                base = base.with_plugin(p.clone());
            }
            base
        };

        if !self.external_packages.contains_key(SERDE_NAMESPACE) {
            let mut written: BTreeSet<String> = BTreeSet::new();
            for plugin in lang.plugins() {
                for file in plugin.runtime_files() {
                    if written.insert(file.relative_path.clone()) {
                        let dest = self.install_dir.join(&file.relative_path);
                        if let Some(parent) = dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&dest, &file.contents)?;
                    }
                }
            }
        }

        // Install each namespace's module
        for (m, module_registry) in &modules {
            let config = m.config().clone();
            self.install_module(&config, module_registry)?;
        }

        // Write the package manifest
        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    /// Fails when a namespace's module cannot be generated as it is named.
    ///
    /// A module is the file `<namespace>.ts` (`<package>.ts` for the root
    /// module), and another module imports it as
    /// `import * as <Namespace> from "./<namespace>"`, with the namespace in
    /// `UpperCamelCase`. So this fails when:
    ///
    /// - two modules' files differ only in case (`kv.ts` and `Kv.ts`, or
    ///   `app.ts` beside the root package's `App.ts`), which are the same file
    ///   on a case-insensitive file system, so one module would be lost.
    ///   These are rejected whatever the file system, so that the output does
    ///   not depend on where it is generated.
    /// - a module imports two namespaces under the same name (`my_ns` and
    ///   `MyNs`), or imports one under the name of a type it declares
    ///   (TS2440), as a ROOT type named like a namespace would be.
    /// - a module imports a namespace under the name of a global that the
    ///   plugins' code for it constructs (`map` as `Map`, beside a map
    ///   field), which the namespace would shadow (TS2351).
    /// - a module is written to `serde.ts`, which the generated code's
    ///   `./serde` imports would find instead of the runtime installed in
    ///   `serde/`.
    ///
    /// Namespaces provided by external packages are not generated, so only
    /// the names they are imported under are checked.
    ///
    /// It also fails when the root package is named exactly like a namespace
    /// that an external package provides, as that namespace's types would be
    /// merged into the root module (see [`module::split`]).
    fn check_namespaces(&self, modules: &BTreeMap<Module, Registry>) -> Result<(), Error> {
        const LANGUAGE: &str = "TypeScript";
        collision::check_root_package(LANGUAGE, &self.package_name, &self.external_packages)?;

        collision::check_files(
            LANGUAGE,
            modules
                .keys()
                .map(|m| m.config().module_name())
                .filter(|name| !self.external_packages.contains_key(*name))
                .map(|name| {
                    (
                        Origin::of_module(name, &self.package_name),
                        format!("{name}.ts"),
                    )
                }),
        )?;

        let installs_serde = !self.external_packages.contains_key(SERDE_NAMESPACE)
            && self.plugins.iter().any(|plugin| {
                plugin
                    .runtime_files()
                    .iter()
                    .any(|file| file.relative_path.starts_with("serde/"))
            });
        if installs_serde
            && let Some(name) = modules
                .keys()
                .map(|m| m.config().module_name())
                .filter(|name| !self.external_packages.contains_key(*name))
                .find(|name| name.eq_ignore_ascii_case(SERDE_NAMESPACE))
        {
            let origin = Origin::of_module(name, &self.package_name);
            return Err(collision::error(
                LANGUAGE,
                format_args!("{origin} is written to `{name}.ts`"),
                "the runtime module `./serde`, which the generated code imports `Serializer` \
                 and `Deserializer` from",
                origin.choose_namespace(),
            )
            .into());
        }

        for (m, registry) in modules {
            let config = m.config();
            if self.external_packages.contains_key(config.module_name()) {
                continue;
            }
            let module_config = TypeScriptCodeGenerator::new(&self.module_config(config))
                .with_plugins(self.plugins.clone())
                .module_config(registry)?;

            let mut bindings = BTreeMap::<String, &str>::new();
            for name in &module_config.referenced_namespaces {
                let binding = name.to_upper_camel_case();
                let origin = Origin::of_module(name, &self.package_name);
                let subject = format!(
                    "{origin} is imported as `{binding}` in `{}.ts`",
                    config.module_name()
                );
                if let Some(existing) = bindings.insert(binding.clone(), name) {
                    let existing = Origin::of_module(existing, &self.package_name);
                    let fix = if matches!(origin, Origin::RootPackage(_)) {
                        origin.choose_namespace()
                    } else {
                        existing.choose_namespace()
                    };
                    return Err(collision::error(LANGUAGE, subject, existing, fix).into());
                }
                if let Some(declared) = registry
                    .keys()
                    .find(|t| t.name.to_upper_camel_case() == binding)
                {
                    return Err(collision::error(
                        LANGUAGE,
                        subject,
                        TypeName(declared),
                        origin.rename_type(),
                    )
                    .into());
                }
                if let Some((global, _)) =
                    naming::CONSTRUCTED_GLOBALS
                        .iter()
                        .find(|(global, constructs)| {
                            *global == binding
                                && !self.plugins.is_empty()
                                && registry.values().any(constructs)
                        })
                {
                    return Err(collision::error(
                        LANGUAGE,
                        subject,
                        format_args!(
                            "the global `{global}`, which `{}.ts` constructs, so \
                             `new {global}(...)` would find the namespace instead",
                            config.module_name()
                        ),
                        origin.choose_namespace(),
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    /// The config a module is generated with: `config`, as split from the
    /// registry, with the installer's external packages, and with the root
    /// package as its parent when it is a namespaced module, which imports the
    /// ROOT types it references from the root package's module. Its name stays
    /// the namespace's, which is also its file name, so this sets the parent
    /// without `with_parent`.
    fn module_config(&self, config: &CodeGeneratorConfig) -> CodeGeneratorConfig {
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();
        if config.module_name() != self.package_name {
            updated_config.parent = Some(self.package_name.clone());
        }
        updated_config
    }

    /// Installs the serde TypeScript runtime sources into the output directory.
    ///
    /// Delegates to the JSON plugin's [`runtime_files`](crate::generation::plugin::EmitterPlugin::runtime_files)
    /// which embeds the serde sources via `include_dir!`.  Most callers should
    /// prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_serde_runtime(&mut self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(self.package_name.clone());
        let lang = TypeScript::new(&config, &BTreeMap::default())
            .with_plugin(std::sync::Arc::new(JsonPlugin));
        for plugin in lang.plugins() {
            for file in plugin.runtime_files() {
                let dest = self.install_dir.join(&file.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, &file.contents)?;
            }
        }
        Ok(())
    }

    /// Installs the bincode TypeScript runtime sources into the output directory.
    ///
    /// Delegates to the bincode plugin's `runtime_files`, writing only the
    /// `bincode/` files.  Most callers should prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_bincode_runtime(&self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(self.package_name.clone());
        let lang = TypeScript::new(&config, &BTreeMap::default())
            .with_plugin(std::sync::Arc::new(BincodePlugin));
        for plugin in lang.plugins() {
            for file in plugin
                .runtime_files()
                .into_iter()
                .filter(|f| f.relative_path.starts_with("bincode/"))
            {
                let dest = self.install_dir.join(&file.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, &file.contents)?;
            }
        }
        Ok(())
    }

    /// Produce the contents of a `package.json` manifest.
    ///
    /// Dependencies are derived from external packages: `Path` locations
    /// become `file:` references, `Url` locations use the extracted package
    /// name with an optional version string. Plugins contribute further
    /// dependencies through
    /// [`manifest_dependencies`](crate::generation::plugin::EmitterPlugin::manifest_dependencies),
    /// each entry a `package.json` pair such as `"shared": "file:../pkg"`.
    /// `typescript` is always added as a devDependency.
    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> Value {
        let mut manifest = json!({
            "name": package_name,
            "version": "0.1.0"
        });

        let mut dependencies = BTreeMap::new();

        for external_package in self.external_packages.values() {
            let (name, version) = match &external_package.location {
                PackageLocation::Path(path) => (
                    external_package.for_namespace.clone(),
                    format!("file:{path}"),
                ),
                PackageLocation::Url(url) => (
                    {
                        // Extract package name from URL
                        let parts: Vec<&str> = url.split('/').collect();
                        if parts.len() >= 2 && parts[parts.len() - 2].starts_with('@') {
                            // Scoped package: @scope/package-name
                            format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
                        } else if let Some(last_segment) = parts.last() {
                            // Regular package: package-name
                            (*last_segment).to_string()
                        } else {
                            url.clone()
                        }
                    },
                    external_package
                        .version
                        .clone()
                        .unwrap_or_else(|| "*".to_string()),
                ),
            };
            dependencies.insert(name, version);
        }

        // Plugin dependencies are `package.json` pairs — parse each as a
        // one-entry object and merge it in. An entry that is not a
        // `"name": "version"` pair is ignored.
        for entry in self.plugins.iter().flat_map(|p| p.manifest_dependencies()) {
            let Ok(Value::Object(pairs)) = serde_json::from_str::<Value>(&format!("{{{entry}}}"))
            else {
                continue;
            };
            for (name, version) in pairs {
                if let Value::String(version) = version {
                    dependencies.insert(name, version);
                }
            }
        }

        if !dependencies.is_empty() {
            manifest["dependencies"] = json!(dependencies);
        }

        // Always add devDependencies
        manifest["devDependencies"] = json!({
            "typescript": "^5.8.3"
        });

        manifest
    }
}

impl SourceInstaller for Installer {
    /// Generate a single `.ts` source file for one namespace.
    ///
    /// The file is written as `<namespace>.ts` directly in the install
    /// directory. Namespaces that correspond to external packages are skipped
    /// — their types are imported rather than generated.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<(), Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());
        if skip_module {
            return Ok(());
        }
        create_dir_all(&self.install_dir)?;
        let module_name = config.module_name();
        let file_name = self.install_dir.join(format!("{module_name}.ts"));
        let mut file = File::create(file_name)?;

        let updated_config = self.module_config(config);
        let generator =
            TypeScriptCodeGenerator::new(&updated_config).with_plugins(self.plugins.clone());
        generator.output(&mut file, registry)?;

        Ok(())
    }

    /// Write `package.json` to the output directory.
    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
        let manifest = self.make_manifest(package_name);
        let manifest = serde_json::to_string_pretty(&manifest)?;

        let manifest_path = self.install_dir.join("package.json");
        let mut file = File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "collision_tests.rs"]
mod collision_tests;
