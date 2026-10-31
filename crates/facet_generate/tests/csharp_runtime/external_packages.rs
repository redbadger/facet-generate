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
        ExternalPackage, PackageLocation, bincode::BincodePlugin, csharp, json::JsonPlugin,
    },
    reflect,
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

fn project(namespace: &str, path: &str) -> ExternalPackage {
    ExternalPackage {
        for_namespace: namespace.to_string(),
        location: PackageLocation::Path(path.to_string()),
        module_name: None,
        version: None,
    }
}

fn install_dependencies(root: &Path) -> ExternalPackage {
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
            "namespace Vendor.API.Geometry;",
        ),
    )
    .unwrap();
    serde
}

fn write_runner(root: &Path, name: &str, program: &str) -> PathBuf {
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
    <ProjectReference Include="../{name}/Example.App.csproj" />
  </ItemGroup>
</Project>
"#
        ),
    )
    .unwrap();
    fs::write(runner.join("Program.cs"), program).unwrap();
    runner
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

/// Distinct project roots work from root and child modules, with no plugins,
/// each plugin separately, and both together. A consuming executable checks
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
    let serde = install_dependencies(root);
    for (name, bincode, json) in [
        ("Plain", false, false),
        ("Bincode", true, false),
        ("Json", false, true),
        ("Both", true, true),
    ] {
        let mut installer = csharp::Installer::new("Example.App", root.join(name))
            .external_packages(&[
                serde.clone(),
                project("geometry", "../Contracts/Example.Contracts.csproj"),
            ])
            .external_namespace("geometry", "Vendor.API.Geometry");
        if bincode {
            installer = installer.plugin(BincodePlugin);
        }
        if json {
            installer = installer.plugin(JsonPlugin);
        }
        installer.generate(&reflect!(Envelope).unwrap()).unwrap();
        let runner = write_runner(root, name, &runner_program(&sample, bincode, json));
        let output = dotnet_run(&runner);
        assert!(
            output.contains("External packages passed"),
            "{name}: {output}"
        );
    }
}
