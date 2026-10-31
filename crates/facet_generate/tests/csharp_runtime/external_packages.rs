//! Cross-project namespace and serialization compatibility regression for #240.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, PackageLocation,
        bincode::BincodePlugin,
        csharp,
        indent::IndentWrite,
        json::JsonPlugin,
        plugin::{CompanionFile, EmitterPlugin},
    },
    reflect,
    reflection::format::{Format, QualifiedTypeName},
};
use serde::Serialize;

use super::{dotnet_run, quote_bytes};

#[derive(Facet, Serialize)]
#[facet(fg::namespace = "geometry")]
struct Point {
    x: u32,
}

#[derive(Facet, Serialize)]
#[facet(fg::namespace = "geometry")]
#[repr(C)]
enum Color {
    Red,
    Blue,
}

#[derive(Facet, Serialize)]
#[facet(fg::namespace = "geometry")]
#[repr(C)]
enum Shape {
    Dot(Point),
}

#[derive(Facet, Serialize)]
#[facet(fg::namespace = "views")]
struct Drawing {
    point: Point,
    colors: Vec<Option<Color>>,
    shape: Shape,
    nothing: (),
}

// This type hides the first segment of the external CLR namespace.
#[derive(Facet, Serialize)]
struct Vendor {
    id: u8,
}

#[derive(Facet, Serialize)]
struct Envelope {
    point: Point,
    drawing: Drawing,
    vendor: Vendor,
}

#[derive(Facet, Serialize)]
#[facet(rename = "Point")]
struct LocalPoint {
    x: u64,
}

#[derive(Facet, Serialize)]
struct Collision {
    local: LocalPoint,
    point: Point,
}

fn project(namespace: &str, path: &str) -> ExternalPackage {
    ExternalPackage {
        for_namespace: namespace.to_string(),
        location: PackageLocation::Path(path.to_string()),
        module_name: None,
        version: None,
    }
}

fn install_dependencies(root: &Path, clr_namespace: &str) -> ExternalPackage {
    csharp::Installer::new("Runtime", root.join("Runtime"))
        .plugin(BincodePlugin)
        .plugin(JsonPlugin)
        .generate(&Registry::new())
        .unwrap();
    let serde = project("serde", "../Runtime/Runtime.csproj");
    let contracts = root.join("Contracts");
    csharp::Installer::new("Example.Contracts", &contracts)
        .plugin(BincodePlugin)
        .plugin(JsonPlugin)
        .external_packages(std::slice::from_ref(&serde))
        .generate(&reflect!(Point, Color, Shape).unwrap())
        .unwrap();

    // Emulate an external assembly using acronym casing, which must be
    // preserved rather than converted to PascalCase.
    let source = contracts.join("Example/Contracts/geometry/Geometry.cs");
    let contents = fs::read_to_string(&source).unwrap();
    assert!(contents.contains("namespace Example.Contracts.Geometry;"));
    fs::write(
        source,
        contents.replace(
            "namespace Example.Contracts.Geometry;",
            &format!("namespace {clr_namespace};"),
        ),
    )
    .unwrap();
    serde
}

fn write_runner(root: &Path, name: &str, package_name: &str, program: &str) -> PathBuf {
    let runner = root.join(format!("Runner{name}"));
    fs::create_dir_all(&runner).unwrap();
    fs::write(
        runner.join("Runner.csproj"),
        format!(
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../{name}/{package_name}.csproj" />
  </ItemGroup>
</Project>
"#
        ),
    )
    .unwrap();
    fs::write(runner.join("Program.cs"), program).unwrap();
    runner
}

fn app_installer(
    root: &Path,
    name: &str,
    package_name: &str,
    serde: &ExternalPackage,
    clr_namespace: &str,
    plugins: (bool, bool),
) -> csharp::Installer {
    let mut installer = csharp::Installer::new(package_name, root.join(name))
        .external_packages(&[
            serde.clone(),
            project("geometry", "../Contracts/Example.Contracts.csproj"),
        ])
        .with_csharp_config(
            csharp::CSharpConfig::default().external_namespace("geometry", clr_namespace),
        );
    if plugins.0 {
        installer = installer.plugin(BincodePlugin);
    }
    if plugins.1 {
        installer = installer.plugin(JsonPlugin);
    }
    installer
}

fn runner_program(sample: &Envelope, bincode: bool, json: bool) -> String {
    let mut program = String::from(
        r#"var point = new global::Vendor.API.Geometry.Point { X = 42 };
var envelope = new Example.App.Envelope { Point = point };
if (envelope.Point.X != 42)
    throw new Exception("External type property failed");
"#,
    );
    if bincode {
        write!(
            program,
            r#"var input = {};
var value = Example.App.Envelope.BincodeDeserialize(input);
if (value.Point.X != 42 || value.Drawing.Point.X != 7 ||
    value.Drawing.Colors[0] != global::Vendor.API.Geometry.Color.Blue ||
    value.Drawing.Colors[1] != null || value.Vendor.Id != 3 ||
    value.Drawing.Shape is not global::Vendor.API.Geometry.Shape.Dot dot || dot.Value.X != 9)
    throw new Exception("Bincode external types did not deserialize correctly");
if (!input.SequenceEqual(value.BincodeSerialize()))
    throw new Exception("Bincode bytes differ from Rust");
"#,
            quote_bytes(&bincode::serialize(sample).unwrap())
        )
        .unwrap();
    }
    if json {
        let input = serde_json::to_string(sample).unwrap();
        write!(
            program,
            r#"var jsonInput = {};
var jsonValue = Example.App.Envelope.JsonDeserialize(jsonInput);
if (jsonValue.Point.X != 42 || jsonValue.Drawing.Point.X != 7)
    throw new Exception("JSON external types did not deserialize correctly");
if (!System.Text.Json.Nodes.JsonNode.DeepEquals(
    System.Text.Json.Nodes.JsonNode.Parse(jsonInput),
    System.Text.Json.Nodes.JsonNode.Parse(jsonValue.JsonSerialize())))
    throw new Exception("JSON differs from Rust");
"#,
            serde_json::to_string(&input).unwrap()
        )
        .unwrap();
    }
    program.push_str("Console.WriteLine(\"External packages passed\");\n");
    program
}

/// A custom plugin qualifies independently reflected types using the same
/// C# settings as the generator, in both module helpers and companion files.
#[derive(Debug)]
struct ExternalEnumHelper(csharp::CSharpConfig);

impl EmitterPlugin<csharp::CSharp> for ExternalEnumHelper {
    fn module_declarations(&self, _: &CodeGeneratorConfig) -> Vec<String> {
        vec![
            "ExternalEnumHelper".to_string(),
            "ExternalEnumCompanion".to_string(),
        ]
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> std::io::Result<()> {
        let color = QualifiedTypeName::namespaced("geometry".to_string(), "Color".to_string());
        let color = self.0.requalify(config, &color);
        assert!(
            config.is_unit_enum(&color),
            "external enum metadata must use the mapped name"
        );
        let rendered = csharp::render_type(&Format::TypeName(color), config);
        writeln!(
            w,
            "public static class ExternalEnumHelper {{ public static System.Type EnumType => typeof({rendered}); }}"
        )
    }

    fn companion_files(&self, config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        let mut color = Format::TypeName(QualifiedTypeName::namespaced(
            "geometry".to_string(),
            "Color".to_string(),
        ));
        self.0.requalify_format(config, &mut color);
        let rendered = csharp::render_type(&color, config);
        vec![CompanionFile {
            file_name: "ExternalEnumCompanion.cs".to_string(),
            imports: vec![],
            contents: format!(
                "public static class ExternalEnumCompanion {{ public static System.Type EnumType => typeof({rendered}); }}\n"
            ),
        }]
    }
}

/// Distinct project roots work from root and child modules, with no serialization
/// plugins, each codec separately, and both together. A consuming executable checks
/// `NuGet`'s full dependency graph and C# round trips against Rust data.
#[test]
fn external_packages_with_distinct_namespaces() {
    let sample = Envelope {
        point: Point { x: 42 },
        drawing: Drawing {
            point: Point { x: 7 },
            colors: vec![Some(Color::Blue), None, Some(Color::Red)],
            shape: Shape::Dot(Point { x: 9 }),
            nothing: (),
        },
        vendor: Vendor { id: 3 },
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let serde = install_dependencies(root, "Vendor.API.Geometry");
    for (name, bincode, json) in [
        ("Plain", false, false),
        ("Bincode", true, false),
        ("Json", false, true),
        ("Both", true, true),
    ] {
        let csharp_config =
            csharp::CSharpConfig::default().external_namespace("geometry", "Vendor.API.Geometry");
        app_installer(
            root,
            name,
            "Example.App",
            &serde,
            "Vendor.API.Geometry",
            (bincode, json),
        )
        .with_csharp_config(csharp_config.clone())
        .plugin(ExternalEnumHelper(csharp_config))
        .generate(&reflect!(Envelope).unwrap())
        .unwrap();
        let mut program = runner_program(&sample, bincode, json);
        program.push_str(
            r#"
foreach (var enumType in new[] {
    Example.App.ExternalEnumHelper.EnumType,
    Example.App.Views.ExternalEnumHelper.EnumType,
    Example.App.ExternalEnumCompanion.EnumType,
    Example.App.Views.ExternalEnumCompanion.EnumType
}) {
    if (enumType != typeof(global::Vendor.API.Geometry.Color) ||
        enumType.Assembly.GetName().Name != "Example.Contracts")
        throw new Exception("Custom plugin used the wrong external enum");
}
"#,
        );
        let runner = write_runner(root, name, "Example.App", &program);
        let output = dotnet_run(&runner);
        assert!(
            output.contains("External packages passed"),
            "{name}: {output}"
        );
    }
}

fn collision_runner_program(sample: &Collision, bincode: bool, json: bool) -> String {
    let mut program = String::from(
        r#"var type = typeof(Example.Safe.Collision);
if (type.GetProperty("Point").PropertyType.Assembly.GetName().Name != "Example.Contracts")
    throw new Exception("External Point bound to the local assembly");
if (type.GetProperty("Local").PropertyType.Assembly.GetName().Name != "Example.Safe")
    throw new Exception("Local Point bound to the external assembly");
var sample = new Example.Safe.Collision {
    Local = new Example.Safe.Point { X = 1099511627783UL },
    Point = new global::Example.App.Point { X = 42 }
};
"#,
    );
    if bincode {
        write!(
            program,
            r#"var bytes = {};
var value = Example.Safe.Collision.BincodeDeserialize(bytes);
if (value.Local.X != sample.Local.X || value.Point.X != 42 ||
    !bytes.SequenceEqual(value.BincodeSerialize()) ||
    !bytes.SequenceEqual(sample.BincodeSerialize()))
    throw new Exception("Bincode used the wrong Point schema");
"#,
            quote_bytes(&bincode::serialize(sample).unwrap())
        )
        .unwrap();
    }
    if json {
        let input = serde_json::to_string(sample).unwrap();
        write!(
            program,
            r#"var input = {};
var valueFromJson = Example.Safe.Collision.JsonDeserialize(input);
if (valueFromJson.Local.X != sample.Local.X || valueFromJson.Point.X != 42 ||
    !System.Text.Json.Nodes.JsonNode.DeepEquals(
        System.Text.Json.Nodes.JsonNode.Parse(input),
        System.Text.Json.Nodes.JsonNode.Parse(valueFromJson.JsonSerialize())))
    throw new Exception("JSON used the wrong Point schema");
"#,
            serde_json::to_string(&input).unwrap()
        )
        .unwrap();
    }
    program.push_str("Console.WriteLine(\"Distinct Point assemblies passed\");\n");
    program
}

/// Reproduce identical CLR names with different Rust schemas (u64/u32), reject
/// that mapping before output, then compile and run a corrected mapping and
/// verify that C# binds each property to the intended assembly in every mode.
#[test]
fn external_type_collisions_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let serde = install_dependencies(root, "Example.App");
    let registry = reflect!(Collision).unwrap();
    let sample = Collision {
        local: LocalPoint {
            x: (1_u64 << 40) + 7,
        },
        point: Point { x: 42 },
    };
    for (name, bincode, json) in [
        ("Plain", false, false),
        ("Bincode", true, false),
        ("Json", false, true),
        ("Both", true, true),
    ] {
        let unsafe_name = format!("Unsafe{name}");
        let error = app_installer(
            root,
            &unsafe_name,
            "Example.App",
            &serde,
            "Example.App",
            (bincode, json),
        )
        .generate(&registry)
        .expect_err("the external and local Point have the same CLR name");
        assert!(error.to_string().contains("Example.App.Point"), "{error}");
        assert!(
            !root.join(&unsafe_name).exists(),
            "rejection must precede file writes"
        );

        let safe_name = format!("Safe{name}");
        app_installer(
            root,
            &safe_name,
            "Example.Safe",
            &serde,
            "Example.App",
            (bincode, json),
        )
        .generate(&registry)
        .unwrap();
        let runner = write_runner(
            root,
            &safe_name,
            "Example.Safe",
            &collision_runner_program(&sample, bincode, json),
        );
        let output = dotnet_run(&runner);
        assert!(
            output.contains("Distinct Point assemblies passed"),
            "{name}: {output}"
        );
    }
}

/// A provider's Unit must not silently bind to the installer's own Unit.cs.
/// Reject that configuration, then prove a distinct external Unit namespace
/// builds and preserves both assembly identities in a real consumer.
#[test]
fn external_types_cannot_bind_to_the_local_runtime() {
    #[derive(Facet)]
    #[facet(fg::namespace = "geometry", rename = "Unit")]
    struct ExternalUnit {
        x: u32,
    }

    #[derive(Facet)]
    struct RuntimeCollision {
        point: ExternalUnit,
    }

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("Contracts")).unwrap();
    fs::write(root.join("Contracts/Contracts.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>").unwrap();
    let provider = root.join("Contracts/Unit.cs");
    fs::write(
        &provider,
        "namespace Facet.Runtime.Serde; public struct Unit { public uint X { get; set; } }",
    )
    .unwrap();
    let registry = reflect!(RuntimeCollision).unwrap();
    let packages = [project("geometry", "../Contracts/Contracts.csproj")];
    let error = csharp::Installer::new("Example.App", root.join("Unsafe"))
        .external_packages(&packages)
        .external_namespace("geometry", "Facet.Runtime.Serde")
        .generate(&registry)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("runtime declaration `Facet.Runtime.Serde.Unit`"),
        "{error}"
    );
    assert!(!root.join("Unsafe").exists());

    fs::write(
        &provider,
        "namespace Vendor.Runtime; public struct Unit { public uint X { get; set; } }",
    )
    .unwrap();
    csharp::Installer::new("Example.App", root.join("RuntimeSafe"))
        .external_packages(&packages)
        .external_namespace("geometry", "Vendor.Runtime")
        .generate(&registry)
        .unwrap();
    let runner = write_runner(
        root,
        "RuntimeSafe",
        "Example.App",
        r#"
var property = typeof(Example.App.RuntimeCollision).GetProperty("Point");
if (property.PropertyType.Assembly.GetName().Name != "Contracts")
    throw new Exception("External Unit bound to the local runtime");
if (typeof(global::Facet.Runtime.Serde.Unit).Assembly.GetName().Name != "Example.App")
    throw new Exception("Core Unit bound to the external provider");
var value = new Example.App.RuntimeCollision { Point = new Vendor.Runtime.Unit { X = 42 } };
if (value.Point.X != 42) throw new Exception("External Unit value lost");
Console.WriteLine("Runtime type identities passed");
"#,
    );
    assert!(dotnet_run(&runner).contains("Runtime type identities passed"));
}

/// Source-only identifier spellings must not bypass full CLR name checks.
/// After rejecting them, a Unicode namespace is compiled and used by a real
/// consumer to verify that validation still permits ordinary Unicode names.
#[test]
fn source_identifier_aliases_cannot_bypass_collision_checks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let serde = install_dependencies(root, "Example.App");
    let registry = reflect!(Collision).unwrap();
    for (index, namespace) in [
        "Exa\u{200D}mple.App",
        r"Ex\u0061mple.App",
        "Example .App",
        "Example/*comment*/.App",
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("Alias{index}");
        let error = app_installer(
            root,
            &name,
            "Example.App",
            &serde,
            namespace,
            (false, false),
        )
        .generate(&registry)
        .unwrap_err();
        assert!(
            error.to_string().contains("plain CLR identifiers"),
            "{error}"
        );
        assert!(!root.join(name).exists());
    }
    let serde = install_dependencies(root, "例.模型");
    app_installer(
        root,
        "Unicode",
        "Example.App",
        &serde,
        "例.模型",
        (false, false),
    )
    .generate(&registry)
    .unwrap();
    let runner = write_runner(
        root,
        "Unicode",
        "Example.App",
        r#"
var property = typeof(Example.App.Collision).GetProperty("Point");
if (property.PropertyType.Assembly.GetName().Name != "Example.Contracts")
    throw new Exception("Unicode external type bound to the local assembly");
var value = new Example.App.Collision { Point = new global::例.模型.Point { X = 42 } };
if (value.Point.X != 42) throw new Exception("Unicode external type value lost");
Console.WriteLine("Unicode namespaces passed");
"#,
    );
    assert!(dotnet_run(&runner).contains("Unicode namespaces passed"));
}
