using System;
using Nerdbank.MessagePack;
using PolyType;

using Facet.Runtime.Serde;

namespace Facet.Runtime.MessagePack;

public static class MessagePackSerde
{
    private static readonly MessagePackSerializer Serializer = new();

    public static byte[] Serialize<T, TWitness>(T value)
        where TWitness : IShapeable<T>
    {
        if (value is null)
            throw new ArgumentNullException(nameof(value));
        return Serializer.Serialize<T, TWitness>(value);
    }

    public static T Deserialize<T, TWitness>(byte[] input)
        where TWitness : IShapeable<T>
    {
        var result = Serializer.Deserialize<T, TWitness>(input);
        if (result is null)
            throw new DeserializationError($"Deserialization produced null for {typeof(T).Name}");
        return result;
    }
}
