// Reusable serialization helpers for generic container types (collections,
// maps, options, arrays).
//
// C# uses a single shared runtime helper file rather than per-module feature
// snippets (as Kotlin/Swift/TypeScript do). This works because C# file-scoped
// namespaces and `using` directives make a helper class in
// `Facet.Runtime.Bincode` accessible from any generated namespace without
// duplication. The semantic role is the same — reusable serialization helpers
// for generic container types — but the delivery mechanism differs to match
// C#'s scoping model.

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public static class FacetHelpers
{
    public static void SerializeCollection<T>(IReadOnlyCollection<T> collection, ISerializer serializer, Action<T, ISerializer> serializeElement)
    {
        serializer.SerializeLen((ulong)collection.Count);
        foreach (var item in collection)
        {
            serializeElement(item, serializer);
        }
    }

    public static ObservableCollection<T> DeserializeList<T>(IDeserializer deserializer, Func<IDeserializer, T> deserializeElement)
    {
        var len = deserializer.DeserializeLen();
        var list = new ObservableCollection<T>();
        for (ulong i = 0; i < len; i++)
        {
            list.Add(deserializeElement(deserializer));
        }
        return list;
    }

    public static HashSet<T> DeserializeSet<T>(IDeserializer deserializer, Func<IDeserializer, T> deserializeElement)
    {
        var len = deserializer.DeserializeLen();
        var set = new HashSet<T>();
        for (ulong i = 0; i < len; i++)
        {
            set.Add(deserializeElement(deserializer));
        }
        return set;
    }

    public static void SerializeMap<K, V>(IReadOnlyDictionary<K, V> map, ISerializer serializer, Action<K, ISerializer> serializeKey, Action<V, ISerializer> serializeValue)
    {
        serializer.SerializeLen((ulong)map.Count);
        foreach (var entry in map)
        {
            serializeKey(entry.Key, serializer);
            serializeValue(entry.Value, serializer);
        }
    }

    public static Dictionary<K, V> DeserializeMap<K, V>(IDeserializer deserializer, Func<IDeserializer, K> deserializeKey, Func<IDeserializer, V> deserializeValue)
    {
        var len = deserializer.DeserializeLen();
        var map = new Dictionary<K, V>();
        for (ulong i = 0; i < len; i++)
        {
            map.Add(deserializeKey(deserializer), deserializeValue(deserializer));
        }
        return map;
    }

    public static void SerializeArray<T>(T[] array, ISerializer serializer, Action<T, ISerializer> serializeElement)
    {
        foreach (var item in array)
        {
            serializeElement(item, serializer);
        }
    }

    public static T[] DeserializeArray<T>(IDeserializer deserializer, int size, Func<IDeserializer, T> deserializeElement)
    {
        var array = new T[size];
        for (int i = 0; i < size; i++)
        {
            array[i] = deserializeElement(deserializer);
        }
        return array;
    }

    public static void SerializeOption<T>(T? value, ISerializer serializer, Action<T, ISerializer> serializeValue) where T : struct
    {
        if (value is not null)
        {
            serializer.SerializeOptionTag(true);
            serializeValue(value.Value, serializer);
        }
        else
        {
            serializer.SerializeOptionTag(false);
        }
    }

    public static T? DeserializeOption<T>(IDeserializer deserializer, Func<IDeserializer, T> deserializeValue) where T : struct
    {
        if (deserializer.DeserializeOptionTag())
        {
            return deserializeValue(deserializer);
        }
        return null;
    }

    public static void SerializeOptionRef<T>(T? value, ISerializer serializer, Action<T, ISerializer> serializeValue) where T : class
    {
        if (value is not null)
        {
            serializer.SerializeOptionTag(true);
            serializeValue(value, serializer);
        }
        else
        {
            serializer.SerializeOptionTag(false);
        }
    }

    public static T? DeserializeOptionRef<T>(IDeserializer deserializer, Func<IDeserializer, T> deserializeValue) where T : class
    {
        if (deserializer.DeserializeOptionTag())
        {
            return deserializeValue(deserializer);
        }
        return null;
    }
}
