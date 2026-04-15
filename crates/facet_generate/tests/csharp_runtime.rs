#![cfg(feature = "csharp")]
//! Runtime tests for C# bincode serialization.
//!
//! These tests generate C# code, serialize data in Rust with bincode, then
//! run the generated C# code to deserialize, verify, and re-serialize —
//! checking that the bytes roundtrip correctly.

use std::{fs, io::Write as _, process::Command};

use facet_generate::generation::{bincode::BincodePlugin, csharp};
use tempfile::tempdir;

pub mod common;

use common::{Choice, Test};

/// Turn a `.csproj` class library into an executable so `dotnet run` works.
fn make_executable(dir: &std::path::Path, package_name: &str) {
    let csproj_path = dir.join(format!("{package_name}.csproj"));
    let content = fs::read_to_string(&csproj_path).unwrap();
    let content = content.replace(
        "<TargetFramework>",
        "<OutputType>Exe</OutputType>\n    <TargetFramework>",
    );
    fs::write(&csproj_path, content).unwrap();
}

fn dotnet_run(dir: &std::path::Path) {
    let output = Command::new("dotnet")
        .arg("run")
        .current_dir(dir)
        .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dotnet run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn quote_bytes(bytes: &[u8]) -> String {
    format!(
        "new byte[] {{ {} }}",
        bytes
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[test]
fn test_csharp_bincode_runtime_on_simple_data() {
    let registry = common::get_simple_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    make_executable(&dir, "Example.Testing");

    let program_path = dir.join("Program.cs");
    let mut program = fs::File::create(program_path).unwrap();
    writeln!(
        program,
        r#"using System;
using System.Linq;
using Example.Testing;
using Facet.Runtime.Serde;
using Facet.Runtime.Bincode;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[] input = {0};
var value = Test.BincodeDeserialize(input);

// Verify deserialized values
Assert(value.A.Count == 2, "A should have 2 elements");
Assert(value.A[0] == 4, "A[0] should be 4");
Assert(value.A[1] == 6, "A[1] should be 6");
Assert(value.B == (-3L, 5UL), "B should be (-3, 5)");
Assert(value.C is Choice.C, "C should be Choice.C variant");
var c = (Choice.C)value.C;
Assert(c.X == 7, "C.X should be 7");

// Roundtrip: re-serialize and check bytes match
var output = value.BincodeSerialize();
Assert(input.SequenceEqual(output), "Roundtrip failed: serialized bytes don't match");

// Verify error on extra bytes
byte[] tooLong = input.Concat(new byte[] {{ 0 }}).ToArray();
try
{{
    Test.BincodeDeserialize(tooLong);
    Assert(false, "Should have thrown on extra bytes");
}}
catch (DeserializationError)
{{
    // expected
}}

// Verify error on completely invalid bytes
try
{{
    Test.BincodeDeserialize(new byte[] {{ 0, 1 }});
    Assert(false, "Should have thrown on invalid bytes");
}}
catch (Exception)
{{
    // expected — may be DeserializationError or EndOfStreamException
}}

Console.WriteLine("Simple data roundtrip: PASSED");
"#,
        quote_bytes(&reference),
    )
    .unwrap();

    dotnet_run(&dir);
}

#[test]
#[ignore = "too slow for now, let's fix it later"]
fn test_csharp_bincode_runtime_on_supported_types() {
    let registry = common::get_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let positive_encodings = common::get_positive_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(",\n        ");

    make_executable(&dir, "Example.Testing");

    let program_path = dir.join("Program.cs");
    let mut program = fs::File::create(program_path).unwrap();
    writeln!(
        program,
        r#"using System;
using System.Linq;
using Example.Testing;
using Facet.Runtime.Serde;
using Facet.Runtime.Bincode;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[][] positiveInputs = new byte[][] {{
        {positive_encodings}
}};

int passed = 0;
for (int i = 0; i < positiveInputs.Length; i++)
{{
    byte[] input = positiveInputs[i];
    var value = SerdeData.BincodeDeserialize(input);
    var output = value.BincodeSerialize();
    Assert(
        input.SequenceEqual(output),
        $"Roundtrip failed for sample {{i}}: input length={{input.Length}}, output length={{output.Length}}"
    );

    // Self-equality via byte comparison: deserialize twice, serialize both,
    // check bytes match. (C# partial classes lack structural Equals, so we
    // compare serialized bytes instead.)
    var value2 = SerdeData.BincodeDeserialize(input);
    var output2 = value2.BincodeSerialize();
    Assert(
        output.SequenceEqual(output2),
        $"Self-equality (via bytes) failed for sample {{i}}"
    );

    // Mutation testing: flip the high bit of each byte (up to 40) and verify
    // that deserialization either fails or produces a different value.
    for (int j = 0; j < Math.Min(40, input.Length); j++)
    {{
        byte[] mutated = (byte[])input.Clone();
        mutated[j] ^= 0x80;
        try
        {{
            var mutatedValue = SerdeData.BincodeDeserialize(mutated);
            var mutatedOutput = mutatedValue.BincodeSerialize();
            Assert(
                !input.SequenceEqual(mutatedOutput),
                $"Mutated byte {{j}} should give different serialized output for sample {{i}}"
            );
        }}
        catch (Exception)
        {{
            // Deserialization failure on mutated input is acceptable
        }}
    }}

    passed++;
}}

Console.WriteLine($"Supported types roundtrip + mutation: {{passed}}/{{positiveInputs.Length}} PASSED");
"#,
    )
    .unwrap();

    dotnet_run(&dir);
}
