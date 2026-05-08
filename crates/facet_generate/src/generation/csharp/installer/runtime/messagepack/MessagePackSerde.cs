using System;
using Nerdbank.MessagePack;
using PolyType;
using PolyType.ReflectionProvider;

using Facet.Runtime.Serde;

namespace Facet.Runtime.MessagePack;

public static class MessagePackSerde
{
    /// <summary>
    /// Configured to use snake_case property names, matching
    /// the externally-tagged wire format produced by <c>rmp_serde::to_vec_named</c>.
    /// </summary>
    private static readonly MessagePackSerializer Serializer = new()
    {
        PropertyNamingPolicy = MessagePackNamingPolicy.SnakeLowerCase,
    };

    /// <summary>
    /// Reflection-based shape provider so that properties generated at
    /// compile-time by other source generators (e.g. CommunityToolkit.Mvvm's
    /// [ObservableProperty]) are visible at runtime.
    /// </summary>
    private static readonly ITypeShapeProvider ShapeProvider =
        ReflectionTypeShapeProvider.Default;

    public static byte[] Serialize<T, TWitness>(T value)
        where TWitness : IShapeable<T>
    {
        if (value is null)
            throw new ArgumentNullException(nameof(value));
        var shape = GetShape<T>();
        return Serializer.Serialize(value, shape);
    }

    public static T Deserialize<T, TWitness>(byte[] input)
        where TWitness : IShapeable<T>
    {
        var reader = new MessagePackReader(input);
        var shape = GetShape<T>();
        var result = Serializer.Deserialize(ref reader, shape);
        if (!reader.End)
            throw new DeserializationError(
                $"Unexpected trailing bytes after deserializing {typeof(T).Name}");
        if (result is null)
            throw new DeserializationError(
                $"Deserialization produced null for {typeof(T).Name}");
        return result;
    }

    private static ITypeShape<T> GetShape<T>()
    {
        return ShapeProvider.GetTypeShape<T>()
            ?? throw new InvalidOperationException(
                $"No shape available for {typeof(T).Name}");
    }
}
