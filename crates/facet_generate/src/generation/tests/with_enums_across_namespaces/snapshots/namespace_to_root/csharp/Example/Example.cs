using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example;

public partial class App : ObservableObject, IFacetSerializable, IFacetDeserializable<App> {
    [ObservableProperty]
    private Example.Kv.Entry _entry;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Entry.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static App Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var entry = Example.Kv.Entry.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new App {
            Entry = entry,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static App BincodeDeserialize(byte[] input)
    {
        if (input is null)
        {
            throw new DeserializationError("Cannot deserialize null array");
        }
        var deserializer = new BincodeDeserializer(input);
        var value = Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }
        return value;
    }
}

public enum Level {
    Low,
    High
}

/// <summary>
/// Bincode serialization helpers for <see cref="Level"/>.
/// </summary>
public static class LevelBincode {
    public static void Serialize(Level value, ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeVariantIndex((uint)value);
        serializer.DecreaseContainerDepth();
    }

    public static Level Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var index = deserializer.DeserializeVariantIndex();
        deserializer.DecreaseContainerDepth();
        return index switch
        {
            0 => Level.Low,
            1 => Level.High,
            _ => throw new DeserializationError("Unknown variant index for Level: " + index),
        }
        ;
    }

    public static byte[] BincodeSerialize(Level value)
    {
        var serializer = new BincodeSerializer();
        Serialize(value, serializer);
        return serializer.GetBytes();
    }

    public static Level BincodeDeserialize(byte[] input)
    {
        if (input is null)
        {
            throw new DeserializationError("Cannot deserialize null array");
        }
        var deserializer = new BincodeDeserializer(input);
        var value = Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }
        return value;
    }
}

public abstract record Outcome : IFacetSerializable, IFacetDeserializable<Outcome> {
    public sealed partial record Score(uint Value) : Outcome;

    public sealed partial record Missing() : Outcome;

    public abstract void Serialize(ISerializer serializer);

    private static Outcome DeserializeScore(IDeserializer deserializer)
    {
        var value = deserializer.DeserializeU32();
        return new Score(value);
    }

    public sealed partial record Score
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(0);
            serializer.SerializeU32(Value);
            serializer.DecreaseContainerDepth();
        }

    }
    private static Outcome DeserializeMissing(IDeserializer deserializer)
    {
        return new Missing();
    }

    public sealed partial record Missing
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(1);
            serializer.DecreaseContainerDepth();
        }

    }
    public static Outcome Deserialize(IDeserializer deserializer)
    {
        var index = deserializer.DeserializeVariantIndex();
        return index switch
        {
            0 => DeserializeScore(deserializer),
            1 => DeserializeMissing(deserializer),
            _ => throw new DeserializationError("Unknown variant index for Outcome: " + index),
        }
        ;
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Outcome BincodeDeserialize(byte[] input)
    {
        if (input is null)
        {
            throw new DeserializationError("Cannot deserialize null array");
        }
        var deserializer = new BincodeDeserializer(input);
        var value = Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }
        return value;
    }
}
