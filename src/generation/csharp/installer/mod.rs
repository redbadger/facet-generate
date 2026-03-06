use std::{
    fmt::Write as _,
    io::Write as _,
    path::{Path, PathBuf},
};

use heck::ToUpperCamelCase as _;
use indoc::writedoc;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SourceInstaller, csharp::CodeGenerator, module,
    },
};

/// Installer for generated source files in C#.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    encoding: Encoding,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`encoding`](Self::encoding) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>) -> Self {
        Installer {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages: ExternalPackages::new(),
            encoding: Encoding::default(),
        }
    }

    /// Set the encoding for serialization/deserialization.
    #[must_use]
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
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
        if !self.encoding.is_none() {
            self.install_serde_runtime()?;
            if let Encoding::Bincode = self.encoding {
                self.install_bincode_runtime()?;
            }
        }

        for (m, module_registry) in module::split(&self.package_name, registry) {
            let config = m
                .config()
                .clone()
                .with_parent(&self.package_name)
                .with_encoding(self.encoding);
            self.install_module(&config, &module_registry)?;
        }

        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        let mut references = vec![
            "    <PackageReference Include=\"CommunityToolkit.Mvvm\" Version=\"8.4.0\" />"
                .to_string(),
        ];

        for external_package in self.external_packages.values() {
            match &external_package.location {
                PackageLocation::Path(path) => {
                    references.push(format!("    <ProjectReference Include=\"{path}\" />"));
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

                    references.push(format!(
                        "    <PackageReference Include=\"{package_name}\" Version=\"{version}\" />"
                    ));
                }
            }
        }

        let references = references.join("\n");
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
            {references}
              </ItemGroup>
            </Project>
            "#
        )
        .expect("writing to String cannot fail");

        manifest
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
            .unwrap_or(config.module_name())
            .to_upper_camel_case();
        let source_path = module_dir.join(format!("{file_name}.cs"));
        let mut file = std::fs::File::create(source_path)?;

        let generator = CodeGenerator::new(&updated_config);
        generator.output(&mut file, registry)?;

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Error> {
        self.install_runtime_file(
            "Facet/Runtime/Serde/ISerializer.cs",
            r#"namespace Facet.Runtime.Serde;

public interface ISerializer
{
    void IncreaseContainerDepth();

    void DecreaseContainerDepth();

    void SerializeUnit(Unit value);

    void SerializeBool(bool value);

    void SerializeI8(sbyte value);

    void SerializeI16(short value);

    void SerializeI32(int value);

    void SerializeI64(long value);

    void SerializeI128(Int128 value);

    void SerializeU8(byte value);

    void SerializeU16(ushort value);

    void SerializeU32(uint value);

    void SerializeU64(ulong value);

    void SerializeU128(UInt128 value);

    void SerializeF32(float value);

    void SerializeF64(double value);

    void SerializeChar(char value);

    void SerializeStr(string value);

    void SerializeBytes(byte[] value);

    void SerializeLen(ulong value);

    void SerializeVariantIndex(uint value);

    void SerializeOptionTag(bool value);

    byte[] GetBytes();

    int GetBufferOffset();
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Serde/IDeserializer.cs",
            r#"namespace Facet.Runtime.Serde;

public interface IDeserializer
{
    void IncreaseContainerDepth();

    void DecreaseContainerDepth();

    Unit DeserializeUnit();

    bool DeserializeBool();

    sbyte DeserializeI8();

    short DeserializeI16();

    int DeserializeI32();

    long DeserializeI64();

    Int128 DeserializeI128();

    byte DeserializeU8();

    ushort DeserializeU16();

    uint DeserializeU32();

    ulong DeserializeU64();

    UInt128 DeserializeU128();

    float DeserializeF32();

    double DeserializeF64();

    char DeserializeChar();

    string DeserializeStr();

    byte[] DeserializeBytes();

    ulong DeserializeLen();

    uint DeserializeVariantIndex();

    bool DeserializeOptionTag();

    int GetBufferOffset();
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Serde/DeserializationError.cs",
            r#"using System;

namespace Facet.Runtime.Serde;

public sealed class DeserializationError : Exception
{
    public DeserializationError(string message) : base(message)
    {
    }
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Serde/SerializationError.cs",
            r#"using System;

namespace Facet.Runtime.Serde;

public sealed class SerializationError : Exception
{
    public SerializationError(string message) : base(message)
    {
    }
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Serde/Unit.cs",
            r#"namespace Facet.Runtime.Serde;

public readonly struct Unit
{
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Serde/Option.cs",
            r#"using System;

namespace Facet.Runtime.Serde;

public readonly struct Option<T>
{
    private readonly T? value;

    public bool HasValue { get; }

    public T Value => HasValue ? value! : throw new InvalidOperationException("Option has no value");

    private Option(T value)
    {
        this.value = value;
        HasValue = true;
    }

    public static Option<T> Some(T value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        return new Option<T>(value);
    }

    public static Option<T> None() => default;
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Json/JsonSerde.cs",
            r#"using System;
using System.Text.Json;
using System.Text.Json.Serialization;

using Facet.Runtime.Serde;

namespace Facet.Runtime.Json;

public static class JsonSerde
{
    internal static readonly JsonSerializerOptions Options = new()
    {
        Converters = { new JsonStringEnumConverter(), new OptionJsonConverterFactory() }
    };

    public static string Serialize<T>(T value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        return JsonSerializer.Serialize(value, Options);
    }

    public static T Deserialize<T>(string input)
    {
        if (string.IsNullOrWhiteSpace(input))
        {
            throw new DeserializationError("Cannot deserialize empty input");
        }

        var value = JsonSerializer.Deserialize<T>(input, Options);
        if (value is null)
        {
            throw new DeserializationError($"Deserialization produced null for {typeof(T).Name}");
        }

        return value;
    }
}

internal sealed class OptionJsonConverterFactory : JsonConverterFactory
{
    public override bool CanConvert(Type typeToConvert)
    {
        return typeToConvert.IsGenericType &&
               typeToConvert.GetGenericTypeDefinition() == typeof(Option<>);
    }

    public override JsonConverter CreateConverter(Type typeToConvert, JsonSerializerOptions options)
    {
        var innerType = typeToConvert.GetGenericArguments()[0];
        var converterType = typeof(OptionJsonConverter<>).MakeGenericType(innerType);
        return (JsonConverter)Activator.CreateInstance(converterType)!;
    }

    private sealed class OptionJsonConverter<T> : JsonConverter<Option<T>>
    {
        public override Option<T> Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.Null)
            {
                return Option<T>.None();
            }

            var value = JsonSerializer.Deserialize<T>(ref reader, options);
            if (value is null)
            {
                return Option<T>.None();
            }

            return Option<T>.Some(value);
        }

        public override void Write(Utf8JsonWriter writer, Option<T> value, JsonSerializerOptions options)
        {
            if (!value.HasValue)
            {
                writer.WriteNullValue();
                return;
            }

            JsonSerializer.Serialize(writer, value.Value, options);
        }
    }
}
"#,
        )?;
        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Error> {
        self.install_runtime_file(
            "Facet/Runtime/Bincode/BincodeSerializer.cs",
            r#"using System;
using System.IO;

using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public sealed class BincodeSerializer : ISerializer
{
    private readonly MemoryStream stream = new();
    private readonly BinaryWriter writer;
    private long containerDepthBudget = long.MaxValue;

    public BincodeSerializer()
    {
        writer = new BinaryWriter(stream);
    }

    public void IncreaseContainerDepth()
    {
        if (containerDepthBudget == 0)
        {
            throw new SerializationError("Exceeded maximum container depth");
        }

        containerDepthBudget -= 1;
    }

    public void DecreaseContainerDepth()
    {
        containerDepthBudget += 1;
    }

    public void SerializeUnit(Unit value)
    {
    }

    public void SerializeBool(bool value)
    {
        writer.Write((byte)(value ? 1 : 0));
    }

    public void SerializeI8(sbyte value)
    {
        writer.Write(value);
    }

    public void SerializeI16(short value)
    {
        writer.Write(value);
    }

    public void SerializeI32(int value)
    {
        writer.Write(value);
    }

    public void SerializeI64(long value)
    {
        writer.Write(value);
    }

    public void SerializeI128(Int128 value)
    {
        SerializeU128(unchecked((UInt128)value));
    }

    public void SerializeU8(byte value)
    {
        writer.Write(value);
    }

    public void SerializeU16(ushort value)
    {
        writer.Write(value);
    }

    public void SerializeU32(uint value)
    {
        writer.Write(value);
    }

    public void SerializeU64(ulong value)
    {
        writer.Write(value);
    }

    public void SerializeU128(UInt128 value)
    {
        writer.Write((ulong)(value & ulong.MaxValue));
        writer.Write((ulong)(value >> 64));
    }

    public void SerializeF32(float value)
    {
        writer.Write(value);
    }

    public void SerializeF64(double value)
    {
        writer.Write(value);
    }

    public void SerializeChar(char value)
    {
        SerializeU32(value);
    }

    public void SerializeStr(string value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        SerializeBytes(System.Text.Encoding.UTF8.GetBytes(value));
    }

    public void SerializeBytes(byte[] value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        SerializeLen((ulong)value.Length);
        writer.Write(value);
    }

    public void SerializeLen(ulong value)
    {
        SerializeU64(value);
    }

    public void SerializeVariantIndex(uint value)
    {
        SerializeU32(value);
    }

    public void SerializeOptionTag(bool value)
    {
        SerializeBool(value);
    }

    public byte[] GetBytes()
    {
        return stream.ToArray();
    }

    public int GetBufferOffset()
    {
        return checked((int)stream.Position);
    }

    public static byte[] Serialize<T>(T value) where T : notnull
    {
        var serializer = new BincodeSerializer();
        switch (value)
        {
            case IFacetSerializable serializable:
                serializable.Serialize(serializer);
                break;
            default:
                throw new SerializationError($"Type {typeof(T).Name} does not implement IFacetSerializable");
        }

        return serializer.GetBytes();
    }
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Bincode/BincodeDeserializer.cs",
            r#"using Facet.Runtime.Serde;
using System;
using System.IO;

namespace Facet.Runtime.Bincode;

public sealed class BincodeDeserializer : IDeserializer
{
    private readonly MemoryStream stream;
    private readonly BinaryReader reader;
    private long containerDepthBudget = long.MaxValue;

    public BincodeDeserializer(byte[] input)
    {
        if (input is null || input.Length == 0)
        {
            throw new DeserializationError("Cannot deserialize null or empty input");
        }

        stream = new MemoryStream(input);
        reader = new BinaryReader(stream);
    }

    public void IncreaseContainerDepth()
    {
        if (containerDepthBudget == 0)
        {
            throw new DeserializationError("Exceeded maximum container depth");
        }

        containerDepthBudget -= 1;
    }

    public void DecreaseContainerDepth()
    {
        containerDepthBudget += 1;
    }

    public Unit DeserializeUnit()
    {
        return new Unit();
    }

    public bool DeserializeBool()
    {
        var value = reader.ReadByte();
        return value switch
        {
            0 => false,
            1 => true,
            _ => throw new DeserializationError("Incorrect boolean value")
        };
    }

    public sbyte DeserializeI8()
    {
        return reader.ReadSByte();
    }

    public short DeserializeI16()
    {
        return reader.ReadInt16();
    }

    public int DeserializeI32()
    {
        return reader.ReadInt32();
    }

    public long DeserializeI64()
    {
        return reader.ReadInt64();
    }

    public Int128 DeserializeI128()
    {
        return unchecked((Int128)DeserializeU128());
    }

    public byte DeserializeU8()
    {
        return reader.ReadByte();
    }

    public ushort DeserializeU16()
    {
        return reader.ReadUInt16();
    }

    public uint DeserializeU32()
    {
        return reader.ReadUInt32();
    }

    public ulong DeserializeU64()
    {
        return reader.ReadUInt64();
    }

    public UInt128 DeserializeU128()
    {
        var lower = reader.ReadUInt64();
        var upper = reader.ReadUInt64();
        return ((UInt128)upper << 64) | lower;
    }

    public float DeserializeF32()
    {
        return reader.ReadSingle();
    }

    public double DeserializeF64()
    {
        return reader.ReadDouble();
    }

    public char DeserializeChar()
    {
        return checked((char)DeserializeU32());
    }

    public string DeserializeStr()
    {
        var bytes = DeserializeBytes();
        return System.Text.Encoding.UTF8.GetString(bytes);
    }

    public byte[] DeserializeBytes()
    {
        var length = DeserializeLen();
        if (length > int.MaxValue)
        {
            throw new DeserializationError("Incorrect length value for byte array");
        }

        return reader.ReadBytes((int)length);
    }

    public ulong DeserializeLen()
    {
        return DeserializeU64();
    }

    public uint DeserializeVariantIndex()
    {
        return DeserializeU32();
    }

    public bool DeserializeOptionTag()
    {
        return DeserializeBool();
    }

    public int GetBufferOffset()
    {
        return checked((int)stream.Position);
    }

    public static T Deserialize<T>(byte[] input)
        where T : IFacetDeserializable<T>
    {
        var deserializer = new BincodeDeserializer(input);
        var value = T.Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }

        return value;
    }
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Bincode/IFacetSerializable.cs",
            r#"using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public interface IFacetSerializable
{
    void Serialize(ISerializer serializer);
}
"#,
        )?;
        self.install_runtime_file(
            "Facet/Runtime/Bincode/IFacetDeserializable.cs",
            r#"using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public interface IFacetDeserializable<T>
{
    static abstract T Deserialize(IDeserializer deserializer);
}
"#,
        )?;
        Ok(())
    }

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
