using System;
using System.Text.Json;

using Facet.Runtime.Serde;

namespace Facet.Runtime.Json;

public static class JsonSerde
{
    // Every generated type names its own converter, so the defaults serve.
    internal static readonly JsonSerializerOptions Options = new();

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
