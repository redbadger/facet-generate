using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example.B;

public abstract record Signal : IFacetSerializable, IFacetDeserializable<Signal> {
    public sealed partial record Level(byte Value) : Signal;

    public sealed partial record Silent() : Signal;

    public abstract void Serialize(ISerializer serializer);

    private static Signal DeserializeLevel(IDeserializer deserializer)
    {
        var value = deserializer.DeserializeU8();
        return new Level(value);
    }

    public sealed partial record Level
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(0);
            serializer.SerializeU8(Value);
            serializer.DecreaseContainerDepth();
        }

    }
    private static Signal DeserializeSilent(IDeserializer deserializer)
    {
        return new Silent();
    }

    public sealed partial record Silent
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(1);
            serializer.DecreaseContainerDepth();
        }

    }
    public static Signal Deserialize(IDeserializer deserializer)
    {
        var index = deserializer.DeserializeVariantIndex();
        return index switch
        {
            0 => DeserializeLevel(deserializer),
            1 => DeserializeSilent(deserializer),
            _ => throw new DeserializationError("Unknown variant index for Signal: " + index),
        }
        ;
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Signal BincodeDeserialize(byte[] input)
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

public enum Status {
    Up,
    Down
}

/// <summary>
/// Bincode serialization helpers for <see cref="Status"/>.
/// </summary>
public static class StatusBincode {
    public static void Serialize(Status value, ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeVariantIndex((uint)value);
        serializer.DecreaseContainerDepth();
    }

    public static Status Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var index = deserializer.DeserializeVariantIndex();
        deserializer.DecreaseContainerDepth();
        return index switch
        {
            0 => Status.Up,
            1 => Status.Down,
            _ => throw new DeserializationError("Unknown variant index for Status: " + index),
        }
        ;
    }

    public static byte[] BincodeSerialize(Status value)
    {
        var serializer = new BincodeSerializer();
        Serialize(value, serializer);
        return serializer.GetBytes();
    }

    public static Status BincodeDeserialize(byte[] input)
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
