using System;
using System.Collections.ObjectModel;
using System.Text.Json;
using System.Text.Json.Serialization;

using Facet.Runtime.Serde;

namespace Facet.Runtime.Json;

public static class JsonSerde
{
    internal static readonly JsonSerializerOptions Options = new()
    {
        Converters =
        {
            new JsonStringEnumConverter(),
            new ObservableCollectionJsonConverterFactory()
        }
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

internal sealed class ObservableCollectionJsonConverterFactory : JsonConverterFactory
{
    public override bool CanConvert(Type typeToConvert)
    {
        return typeToConvert.IsGenericType &&
               typeToConvert.GetGenericTypeDefinition() == typeof(ObservableCollection<>);
    }

    public override JsonConverter CreateConverter(Type typeToConvert, JsonSerializerOptions options)
    {
        var elementType = typeToConvert.GetGenericArguments()[0];
        var converterType = typeof(ObservableCollectionJsonConverter<>).MakeGenericType(elementType);
        return (JsonConverter)Activator.CreateInstance(converterType)!;
    }

    private sealed class ObservableCollectionJsonConverter<T> : JsonConverter<ObservableCollection<T>>
    {
        public override ObservableCollection<T> Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options)
        {
            var list = JsonSerializer.Deserialize<List<T>>(ref reader, options)
                ?? throw new DeserializationError("Failed to deserialize collection");
            return new ObservableCollection<T>(list);
        }

        public override void Write(
            Utf8JsonWriter writer,
            ObservableCollection<T> value,
            JsonSerializerOptions options)
        {
            JsonSerializer.Serialize(writer, (IEnumerable<T>)value, options);
        }
    }
}
