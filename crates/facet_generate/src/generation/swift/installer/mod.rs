//! Project scaffolding — writes a ready-to-build Swift package to disk.
//!
//! The [`Installer`] is the final stage of the Swift generation pipeline.
//! While [`SwiftCodeGenerator`] produces the *contents* of a single source file,
//! the installer is responsible for the surrounding project structure:
//!
//! 1. **Runtime files** — copies the Serde runtime `.swift` sources from
//!    `runtime/swift/` into the output directory. These provide the
//!    `Serializer`, `Deserializer`, `BincodeSerializer`, etc. that the
//!    generated `serialize`/`deserialize` methods call into.
//!
//! 2. **Per-module source files** — splits the registry by namespace (via
//!    [`module::split`]) and calls [`SwiftCodeGenerator`] once per namespace,
//!    writing each to its own `.swift` file under
//!    `Sources/<Module>/<Module>.swift`. Swift has no `namespace` keyword —
//!    the crate's namespace concept maps directly to **SPM targets**, which
//!    serve as Swift's module-level namespacing. Cross-module type references
//!    use `Module.Type` syntax (e.g. `Foo.Tree`).
//!
//! 3. **Companion files** — writes any file a plugin contributes through
//!    [`companion_files`](crate::generation::plugin::EmitterPlugin::companion_files)
//!    into the module's own directory, next to the generated source file.
//!
//! 4. **`Package.swift`** — generates an SPM manifest with library products,
//!    targets (one per namespace plus `Serde` runtime), deployment
//!    [`platforms`](Installer::platforms), and dependencies (external URL or
//!    path packages, plus any the plugins declare).

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use heck::ToUpperCamelCase as _;

use indent::indent_all_with;
use indoc::formatdoc;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Error, ExternalPackage, ExternalPackages, SERDE_NAMESPACE,
        SourceInstaller,
        collision::{self, Origin, TypeName},
        module::{self, Module},
        plugin::EmitterPlugin,
        swift::{
            Swift, conformance::Conformance, emitter::base_imports,
            foundation_types::FOUNDATION_TYPES, generator::SwiftCodeGenerator, naming,
        },
    },
    reflection::format::QualifiedTypeName,
};

/// Writes a complete Swift package — runtime sources, per-module generated
/// code, and a `Package.swift` manifest — to the configured output directory.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    targets: BTreeMap<String, BTreeSet<String>>,
    /// The targets of the modules the installer generated, so the manifest
    /// declares the package's own target only when the root module was one of
    /// them.
    modules: BTreeSet<String>,
    /// Plugin-provided dependency edges, keyed by target name.
    ///
    /// Kept apart from [`targets`](Self::targets) because these are raw
    /// `Target.Dependency` expressions (e.g.
    /// `.product(name: "Shared", package: "Shared")`): they must be written
    /// verbatim rather than quoted, and they must not be mistaken for local
    /// targets when deciding which targets are top-level.
    plugin_target_dependencies: BTreeMap<String, BTreeSet<String>>,
    /// The type names behind each edge of [`targets`](Self::targets), keyed by
    /// target and then by the target it depends on, for naming the types when
    /// the edges form a cycle.
    target_references: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    external_packages: ExternalPackages,
    platforms: Vec<String>,
    plugins: Vec<Arc<dyn EmitterPlugin<Swift>>>,
    /// Which types conform to `Hashable` and `Equatable`, decided once over
    /// the whole registry by [`generate`](Self::generate) and handed to every
    /// module's generator, so that a module holding a type from another one
    /// agrees with it on its conformance.
    conformance: Option<Arc<Conformance>>,
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
            targets: BTreeMap::new(),
            modules: BTreeSet::new(),
            plugin_target_dependencies: BTreeMap::new(),
            target_references: BTreeMap::new(),
            external_packages: ExternalPackages::new(),
            platforms: vec![],
            plugins: vec![],
            conformance: None,
        }
    }

    /// Add a plugin to be used during code generation.
    #[must_use]
    pub fn plugin<P: crate::generation::plugin::EmitterPlugin<Swift> + 'static>(
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

    /// Set the deployment targets declared by the generated `Package.swift`.
    ///
    /// Each entry is a raw SPM platform expression, e.g. `".iOS(.v16)"`; they
    /// are rendered, in order, as `platforms: [.iOS(.v16), .macOS(.v13)],`.
    /// With no entries the manifest has no `platforms:` line, which leaves SPM
    /// on its own defaults.
    #[must_use]
    pub fn platforms(mut self, platforms: &[String]) -> Self {
        self.platforms = platforms.to_vec();
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
    /// and fails before writing anything when two modules would become the
    /// same target, or when a module would refer to another by a name that
    /// one of the types in scope there also has. Such output would not build,
    /// or would lose a module; the error names the namespace and what it
    /// collides with.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        let modules = module::split(&self.package_name, registry);
        self.check_namespaces(&modules)?;

        let mut config = CodeGeneratorConfig::new(self.package_name.clone());
        config.update_from(registry);

        let lang = {
            let mut base = Swift::new(&config, registry);
            for p in &self.plugins {
                base = base.with_plugin(p.clone());
            }
            base
        };

        if !self.external_packages.contains_key(SERDE_NAMESPACE) {
            let mut written: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            for plugin in lang.plugins() {
                for file in plugin.runtime_files() {
                    if written.insert(file.relative_path.clone()) {
                        let dest = self.install_dir.join(&file.relative_path);
                        if let Some(parent) = dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&dest, &file.contents)?;
                        // Register the "Serde" SPM target when its sources are written.
                        if file.relative_path.starts_with("Sources/Serde/") {
                            self.targets.entry("Serde".to_string()).or_default();
                        }
                    }
                }
            }
        }

        // Decide conformance over the whole registry, since a module's types
        // can hold types from other modules.
        self.conformance = Some(Arc::new(Conformance::of(registry, &self.external_packages)));

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
    /// A module is an SPM target named after its namespace in
    /// `UpperCamelCase`, and its types are referred to from other modules as
    /// `Module.Type`. So this fails when:
    ///
    /// - two modules become the same target — two namespaces that differ only
    ///   in case or in word separators (`kv` and `Kv`), or a namespace and the
    ///   root package (`app` in package `App`) — as both would be written to
    ///   `Sources/<Target>/`, and one would be lost. Targets that differ only
    ///   in case are rejected too, whatever the file system, as they would
    ///   share a directory on a case-insensitive one.
    /// - a module refers to another by a name that a type in scope there also
    ///   has: one it declares, or one declared by any module it imports. Swift
    ///   finds the type before the module, so `Kv.Get` would look for `Get`
    ///   inside the type `Kv`. This covers a ROOT type named like a namespace,
    ///   a type named like its own namespace's module, and a type named like
    ///   the root package's module, which namespaced modules qualify ROOT
    ///   types with. A type of the `Swift` or `_Concurrency` module
    ///   (`string` for `String`, `error` for `Error`) is always in scope, so
    ///   it hides the module the same way. So does a type of the `Serde`
    ///   runtime (`serializer`) in a module that imports it, and a type that
    ///   `Foundation` brings into scope (`data` for `Data`) in a module that
    ///   imports `Foundation` itself.
    /// - a module is named like one the package imports: `Swift`, the
    ///   `Serde` runtime, `Foundation` when any module imports it, or one of
    ///   the SDK modules `Foundation` loads (`system` for `System`) when that
    ///   module imports `Foundation`, directly, through the runtime or through
    ///   another module, as it would then depend on itself.
    ///
    /// Namespaces provided by external packages are not generated, so they
    /// are not checked for targets.
    ///
    /// It also fails when the root package is named exactly like a namespace
    /// that an external package provides, as that namespace's types would be
    /// merged into the root module (see [`module::split`]).
    fn check_namespaces(&self, modules: &BTreeMap<Module, Registry>) -> Result<(), Error> {
        const LANGUAGE: &str = "Swift";
        collision::check_root_package(LANGUAGE, &self.package_name, &self.external_packages)?;

        collision::check_files(
            LANGUAGE,
            modules
                .keys()
                .map(|m| m.config().module_name())
                .filter(|name| !self.external_packages.contains_key(*name))
                .map(|name| {
                    let target = name.to_upper_camel_case();
                    (
                        Origin::of_module(name, &self.package_name),
                        format!("Sources/{target}/{target}.swift"),
                    )
                }),
        )?;

        let mut generated = Vec::new();
        for (m, registry) in modules {
            if self
                .external_packages
                .contains_key(m.config().module_name())
            {
                continue;
            }
            let module_config = SwiftCodeGenerator::new(&self.module_config(m.config()))
                .with_plugins(self.plugins.clone())
                .module_config(registry)?;
            generated.push((m.config(), registry, module_config));
        }

        self.check_module_names(&generated)?;

        let types: BTreeMap<&str, Vec<&QualifiedTypeName>> = modules
            .iter()
            .map(|(m, registry)| (m.config().module_name(), registry.keys().collect()))
            .collect();
        let runtime_types = naming::runtime_types(
            &self
                .plugins
                .iter()
                .flat_map(|p| p.runtime_files())
                .collect::<Vec<_>>(),
        );
        for (config, registry, module_config) in &generated {
            // What the module imports itself: a module's names are not in
            // scope in the modules that import it.
            let imports: Vec<String> = base_imports(module_config)
                .into_iter()
                .chain(self.plugins.iter().flat_map(|p| p.imports(module_config)))
                .collect();
            // Every module this one imports, by the name it qualifies their
            // types with.
            let imported: Vec<&str> = module_config
                .external_definitions
                .keys()
                .map(String::as_str)
                .collect();
            let in_scope = || {
                registry.keys().chain(
                    imported
                        .iter()
                        .flat_map(|name| types.get(name).into_iter().flatten().copied()),
                )
            };
            for &name in &imported {
                let qualifier = name.to_upper_camel_case();
                let origin = Origin::of_module(name, &self.package_name);
                let subject = format!(
                    "{origin} becomes the module `{qualifier}`, which qualifies its types in \
                     module `{}`",
                    config.module_name().to_upper_camel_case()
                );
                if let Some(hiding) = in_scope().find(|t| t.name == qualifier) {
                    return Err(collision::error(
                        LANGUAGE,
                        subject,
                        TypeName(hiding),
                        origin.rename_type(),
                    )
                    .into());
                }
                if let Some(collider) = builtin_type(&qualifier, &imports, &runtime_types) {
                    return Err(collision::error(
                        LANGUAGE,
                        subject,
                        collider,
                        origin.choose_namespace(),
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Fails when a module is named like one the package imports (see
    /// [`check_namespaces`](Self::check_namespaces)). `generated` holds each
    /// generated module's config, registry and generator config.
    fn check_module_names(
        &self,
        generated: &[(&CodeGeneratorConfig, &Registry, CodeGeneratorConfig)],
    ) -> Result<(), Error> {
        const LANGUAGE: &str = "Swift";

        let imports_serde = generated.iter().any(|(_, _, config)| {
            self.plugins
                .iter()
                .any(|p| p.imports(config).iter().any(|i| i == "Serde"))
        });
        let imports_foundation = self.modules_importing_foundation(generated);
        for (config, _, _) in generated {
            let name = config.module_name();
            let target = name.to_upper_camel_case();
            // Every module finds a target named `Foundation` in its place, but
            // another SDK module's name only breaks a module that imports
            // `Foundation`, which then imports it back.
            let sdk_module_breaks = if target == "Foundation" {
                !imports_foundation.is_empty()
            } else {
                imports_foundation.contains(name)
            };
            let collider = if target == "Swift" {
                Some("the standard library's module `Swift`".to_string())
            } else if target == "Serde" && imports_serde {
                Some("the runtime module `Serde`, which the generated code imports".to_string())
            } else if sdk_module_breaks {
                naming::FOUNDATION_MODULES
                    .binary_search_by_key(&target.as_str(), |(module, _)| module)
                    .ok()
                    .map(|i| {
                        format!(
                            "the SDK module `{target}`, {}, so the target would depend on itself",
                            naming::FOUNDATION_MODULES[i].1
                        )
                    })
            } else {
                None
            };
            if let Some(collider) = collider {
                let origin = Origin::of_module(name, &self.package_name);
                return Err(collision::error(
                    LANGUAGE,
                    format_args!("{origin} becomes the module `{target}`"),
                    collider,
                    origin.choose_namespace(),
                )
                .into());
            }
        }

        Ok(())
    }

    /// The generated modules that import `Foundation`, directly, through the
    /// Serde runtime, or through a module they import.
    fn modules_importing_foundation<'a>(
        &self,
        generated: &[(&'a CodeGeneratorConfig, &Registry, CodeGeneratorConfig)],
    ) -> BTreeSet<&'a str> {
        let mut importing: BTreeSet<&str> = generated
            .iter()
            .filter(|(_, _, config)| {
                base_imports(config)
                    .into_iter()
                    .chain(self.plugins.iter().flat_map(|p| p.imports(config)))
                    .any(|i| i == "Foundation" || i == "Serde")
            })
            .map(|(config, _, _)| config.module_name())
            .collect();
        loop {
            let before = importing.len();
            for (config, _, module_config) in generated {
                if module_config
                    .external_definitions
                    .keys()
                    .any(|name| importing.contains(name.as_str()))
                {
                    importing.insert(config.module_name());
                }
            }
            if importing.len() == before {
                return importing;
            }
        }
    }

    /// The config a module is generated with: `config`, as split from the
    /// registry, with the installer's external packages, and with the root
    /// package as its parent when it is a namespaced module, which qualifies
    /// the ROOT types it references with the root package.
    fn module_config(&self, config: &CodeGeneratorConfig) -> CodeGeneratorConfig {
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();
        if config.module_name() != self.package_name {
            updated_config.parent = Some(self.package_name.clone());
        }
        updated_config
    }

    /// Installs the Serde Swift runtime sources into the output directory and
    /// registers `Serde` as a local SPM target.
    ///
    /// Delegates to `BincodePlugin::runtime_files` which embeds the
    /// `Sources/Serde/` sources via `include_dir!`.  Most callers should
    /// prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_serde_runtime(&mut self) -> Result<(), Error> {
        let default_config = CodeGeneratorConfig::new(self.package_name.clone());
        let lang = Swift::new(&default_config, &BTreeMap::default()).with_plugin(
            std::sync::Arc::new(crate::generation::bincode::BincodePlugin),
        );
        let mut written = BTreeSet::new();
        for plugin in lang.plugins() {
            for file in plugin.runtime_files() {
                if written.insert(file.relative_path.clone()) {
                    let dest = self.install_dir.join(&file.relative_path);
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&dest, &file.contents)?;
                    if file.relative_path.starts_with("Sources/Serde/") {
                        self.targets.entry("Serde".to_string()).or_default();
                    }
                }
            }
        }
        Ok(())
    }

    /// Produce the contents of a `Package.swift` file.
    ///
    /// Builds the SPM manifest with targets (one per namespace plus any
    /// runtime targets), inter-target dependency edges, external package
    /// dependencies, and a library product exposing the top-level targets.
    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        let all_targets = self.all_targets_with_package(package_name);
        let external_package_names = self.external_package_names();
        let library_targets_str =
            Self::library_targets_str(package_name, &all_targets, &external_package_names);
        let targets = self.render_targets(&all_targets, &external_package_names);
        let dependencies_section = self.dependencies_section();
        let platforms_section = self.platforms_section();

        formatdoc! {r#"
            // swift-tools-version: 5.8
            import PackageDescription

            let package = Package(
                name: "{package}",{platforms}
                products: [
                    .library(
                        name: "{package}",
                        targets: [{library_targets}]
                    )
                ],{dependencies}
                targets: [{targets}]
            )
            "#,
            package = self.package_name,
            platforms = platforms_section,
            library_targets = library_targets_str,
            dependencies = dependencies_section,
            targets = format!("\n{}\n    ", targets.join("\n"))
        }
    }

    /// All targets keyed by name, plus a synthetic package-level target
    /// aggregating every namespace target (used to compute the library
    /// product's target list).
    ///
    /// The aggregate leaves out the package target itself and every target
    /// that depends on it (a namespaced module that references a ROOT type),
    /// which would otherwise make a cycle.
    ///
    /// When the installer generated modules but not the root one (every type
    /// is in a named namespace), there is no package target: it would have no
    /// sources, which `SwiftPM` rejects, and the library product lists the
    /// top-level namespace targets instead.
    fn all_targets_with_package(&self, package_name: &str) -> BTreeMap<String, BTreeSet<String>> {
        let mut all_targets = self.targets.clone();
        let package_target = package_name.to_upper_camel_case();

        if !self.modules.is_empty() && !self.modules.contains(&package_target) {
            return all_targets;
        }

        let mut package_targets = BTreeSet::new();
        for targets in all_targets.values() {
            for target in targets {
                let target = target.to_upper_camel_case();
                if target != package_target
                    && !self.depends_on(&target, &package_target, &mut BTreeSet::new())
                {
                    package_targets.insert(target);
                }
            }
        }
        all_targets.insert(package_target, package_targets);

        all_targets
    }

    /// Whether `target` depends on `dependency`, directly or through other
    /// targets.
    fn depends_on(&self, target: &str, dependency: &str, visited: &mut BTreeSet<String>) -> bool {
        if !visited.insert(target.to_string()) {
            return false;
        }
        self.targets.get(target).is_some_and(|dependencies| {
            dependencies
                .iter()
                .any(|d| d == dependency || self.depends_on(d, dependency, visited))
        })
    }

    /// Fails when the targets' dependencies form a cycle, which `SwiftPM`
    /// rejects: typically a namespaced module referencing a ROOT type while
    /// the root module references that namespace.
    fn check_acyclic(&self) -> Result<(), Error> {
        let Some(cycle) = self.find_cycle() else {
            return Ok(());
        };

        let edges = cycle
            .windows(2)
            .map(|edge| {
                let types = self
                    .target_references
                    .get(&edge[0])
                    .and_then(|references| references.get(&edge[1]))
                    .map(|types| {
                        types
                            .iter()
                            .map(|t| format!("`{t}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                format!("`{}` references {types} in `{}`", edge[0], edge[1])
            })
            .collect::<Vec<_>>()
            .join("; ");

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Swift targets cannot depend on each other in a cycle, and these would: \
                 {edges}. Move the types that one of these targets references into a \
                 namespace of their own (`#[facet(fg::namespace = \"…\")]`), which the \
                 targets can both depend on"
            ),
        )
        .into())
    }

    /// The first cycle in the targets' dependencies, as the targets along it,
    /// starting and ending with the same one.
    fn find_cycle(&self) -> Option<Vec<String>> {
        fn visit(
            targets: &BTreeMap<String, BTreeSet<String>>,
            target: &str,
            path: &mut Vec<String>,
            done: &mut BTreeSet<String>,
        ) -> Option<Vec<String>> {
            if let Some(start) = path.iter().position(|t| t == target) {
                let mut cycle = path[start..].to_vec();
                cycle.push(target.to_string());
                return Some(cycle);
            }
            if done.contains(target) {
                return None;
            }
            path.push(target.to_string());
            for dependency in targets.get(target).into_iter().flatten() {
                if let Some(cycle) = visit(targets, dependency, path, done) {
                    return Some(cycle);
                }
            }
            path.pop();
            done.insert(target.to_string());
            None
        }

        let mut done = BTreeSet::new();
        self.targets
            .keys()
            .find_map(|target| visit(&self.targets, target, &mut Vec::new(), &mut done))
    }

    /// Names of external dependencies to exclude from target creation.
    fn external_package_names(&self) -> BTreeSet<String> {
        self.external_packages
            .values()
            .map(|d| d.for_namespace.to_upper_camel_case())
            .collect()
    }

    /// The quoted, comma-joined list of targets exposed by the library
    /// product: those that are not external packages and not a dependency
    /// of any other target, falling back to the main package if every
    /// target turns out to be a dependency.
    fn library_targets_str(
        package_name: &str,
        all_targets: &BTreeMap<String, BTreeSet<String>>,
        external_package_names: &BTreeSet<String>,
    ) -> String {
        let mut all_dependencies = BTreeSet::new();
        for dependencies in all_targets.values() {
            for dep in dependencies {
                all_dependencies.insert(dep.clone());
            }
        }

        let top_level_targets: Vec<String> = all_targets
            .keys()
            .filter(|name| {
                !external_package_names.contains(*name) && !all_dependencies.contains(*name)
            })
            .cloned()
            .collect();

        let library_targets = if top_level_targets.is_empty() {
            vec![package_name.to_string()]
        } else {
            top_level_targets
        };

        library_targets
            .iter()
            .map(|t| format!(r#""{t}""#))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Render each local target's `.target(name:dependencies:)` declaration.
    fn render_targets(
        &self,
        all_targets: &BTreeMap<String, BTreeSet<String>>,
        external_package_names: &BTreeSet<String>,
    ) -> Vec<String> {
        all_targets
            .iter()
            .filter(|(name, _)| !external_package_names.contains(*name))
            .map(|(name, dependencies)| {
                // Local targets are quoted; plugin-provided edges are already
                // `Target.Dependency` expressions and go in verbatim.
                let dependencies = dependencies
                    .iter()
                    .map(|dep| format!(r#""{dep}""#))
                    .chain(
                        self.plugin_target_dependencies
                            .get(name)
                            .into_iter()
                            .flatten()
                            .cloned(),
                    )
                    .collect::<Vec<String>>()
                    .join(", ");

                let base_target = formatdoc! {r#"
                    .target(
                        name: "{name}",
                        dependencies: [{dependencies}]
                    ),"#};

                indent_all_with("        ", &base_target)
            })
            .collect()
    }

    /// Package-level `dependencies:` section: the external packages, plus
    /// every `.package(...)` entry the plugins ask for (they arrive
    /// unindented).
    fn dependencies_section(&self) -> String {
        let dependencies: Vec<String> = self
            .external_packages
            .values()
            .cloned()
            .map(|d| ExternalPackage::to_swift(d, 2))
            .chain(
                self.plugins
                    .iter()
                    .flat_map(|p| p.manifest_dependencies())
                    .map(|d| indent_all_with("        ", &d)),
            )
            .collect();

        if dependencies.is_empty() {
            String::new()
        } else {
            format!(
                "\n    dependencies: [\n{}\n    ],",
                dependencies.join(",\n")
            )
        }
    }

    /// Package-level `platforms:` section, omitted entirely when empty so
    /// SPM falls back to its own defaults.
    fn platforms_section(&self) -> String {
        if self.platforms.is_empty() {
            String::new()
        } else {
            format!("\n    platforms: [{}],", self.platforms.join(", "))
        }
    }
}

/// The clause naming the type that hides a module qualified as `qualifier`
/// in a module that imports `imports`, if a builtin one does: a type of the
/// standard library, of the `Serde` runtime (whose types are `runtime_types`),
/// or one that `Foundation` brings into scope.
fn builtin_type(
    qualifier: &str,
    imports: &[String],
    runtime_types: &BTreeSet<String>,
) -> Option<String> {
    if naming::STDLIB_TYPES.binary_search(&qualifier).is_ok() {
        Some(format!(
            "the standard-library type `{qualifier}`, which Swift finds instead of the module"
        ))
    } else if imports.iter().any(|i| i == "Serde") && runtime_types.contains(qualifier) {
        Some(format!(
            "the runtime type `Serde.{qualifier}`, which Swift finds instead of the module"
        ))
    } else if imports.iter().any(|i| i == "Foundation") {
        FOUNDATION_TYPES
            .binary_search_by_key(&qualifier, |(name, _)| name)
            .ok()
            .map(|i| match FOUNDATION_TYPES[i].1 {
                "Foundation" => format!(
                    "the Foundation type `{qualifier}`, which Swift finds instead of the module"
                ),
                module => format!(
                    "the {module} type `{qualifier}`, which `Foundation` imports and Swift finds \
                     instead of the module"
                ),
            })
    } else {
        None
    }
}

impl SourceInstaller for Installer {
    /// Generate a single `.swift` source file for one namespace.
    ///
    /// Writes to `Sources/<Module>/<Module>.swift`. Skips namespaces that
    /// correspond to external packages. Tracks inter-target dependencies
    /// (external definitions, Serde runtime) for the manifest.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());

        if skip_module {
            return Ok(());
        }

        let module_name = config.module_name().to_upper_camel_case();
        self.modules.insert(module_name.clone());

        // A namespaced module qualifies the ROOT types it references with the
        // root package, whose target it then depends on.
        let updated_config = self.module_config(config);

        // The references the registry and the plugins make decide both the
        // module's imports and its target's dependencies, so the generator
        // works out its config once and the installer reads the edges from it.
        let mut generator =
            SwiftCodeGenerator::new(&updated_config).with_plugins(self.plugins.clone());
        if let Some(conformance) = &self.conformance {
            generator = generator.with_conformance(conformance.clone());
        }
        let module_config = generator.module_config(registry)?;

        let targets = self.targets.entry(module_name.clone()).or_default();
        let references = self
            .target_references
            .entry(module_name.clone())
            .or_default();
        for (target, types) in &module_config.external_definitions {
            targets.insert(target.to_upper_camel_case());
            references
                .entry(target.to_upper_camel_case())
                .or_default()
                .extend(types.iter().cloned());
        }

        // Depend on the Serde target when the installer has plugins
        // (i.e. serialization code will be generated).
        if !self.plugins.is_empty() {
            targets.insert("Serde".to_string());
        }

        // Plugin-provided target edges (e.g. `.product(name: "Shared", …)`)
        // belong to this module's target, not to the runtime targets.
        let plugin_target_dependencies: Vec<String> = self
            .plugins
            .iter()
            .flat_map(|p| p.target_dependencies(config))
            .collect();
        if !plugin_target_dependencies.is_empty() {
            self.plugin_target_dependencies
                .entry(module_name.clone())
                .or_default()
                .extend(plugin_target_dependencies);
        }

        let dir_path = self.install_dir.join("Sources").join(&module_name);
        std::fs::create_dir_all(&dir_path)?;
        let source_path = dir_path.join(format!("{module_name}.swift"));

        let mut file = std::fs::File::create(source_path)?;

        generator.write_module(&mut file, &module_config, registry)?;

        // Companion files live beside the module's own source file, whether or
        // not the serde runtime is external.
        for companion in generator.render_companion_files(&module_config, registry)? {
            std::fs::write(dir_path.join(&companion.file_name), companion.contents)?;
        }

        Ok(())
    }

    /// Write `Package.swift` to the output directory root.
    ///
    /// # Errors
    ///
    /// Fails, without writing the manifest, when the targets' dependencies
    /// form a cycle.
    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
        self.check_acyclic()?;

        let manifest = self.make_manifest(package_name);

        let manifest_path = self.install_dir.join("Package.swift");
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "collision_tests.rs"]
mod collision_tests;
