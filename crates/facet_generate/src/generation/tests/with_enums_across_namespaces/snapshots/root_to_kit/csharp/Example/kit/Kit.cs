using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example.Kit;

public partial class Badge : ObservableObject, IFacetSerializable, IFacetDeserializable<Badge> {
    [ObservableProperty]
    private Presence _presence;
    [ObservableProperty]
    private Shape _shape;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        PresenceBincode.Serialize(Presence, serializer);
        Shape.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Badge Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var presence = PresenceBincode.Deserialize(deserializer);
        var shape = Shape.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Badge {
            Presence = presence,
            Shape = shape,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Badge BincodeDeserialize(byte[] input)
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

public enum Presence {
    Online,
    Offline
}

/// <summary>
/// Bincode serialization helpers for <see cref="Presence"/>.
/// </summary>
public static class PresenceBincode {
    public static void Serialize(Presence value, ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeVariantIndex((uint)value);
        serializer.DecreaseContainerDepth();
    }

    public static Presence Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var index = deserializer.DeserializeVariantIndex();
        deserializer.DecreaseContainerDepth();
        return index switch
        {
            0 => Presence.Online,
            1 => Presence.Offline,
            _ => throw new DeserializationError("Unknown variant index for Presence: " + index),
        }
        ;
    }

    public static byte[] BincodeSerialize(Presence value)
    {
        var serializer = new BincodeSerializer();
        Serialize(value, serializer);
        return serializer.GetBytes();
    }

    public static Presence BincodeDeserialize(byte[] input)
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

public abstract record Shape : IFacetSerializable, IFacetDeserializable<Shape> {
    public sealed partial record Circle(double Value) : Shape;

    public sealed partial record Empty() : Shape;

    public abstract void Serialize(ISerializer serializer);

    private static Shape DeserializeCircle(IDeserializer deserializer)
    {
        var value = deserializer.DeserializeF64();
        return new Circle(value);
    }

    public sealed partial record Circle
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(0);
            serializer.SerializeF64(Value);
            serializer.DecreaseContainerDepth();
        }

    }
    private static Shape DeserializeEmpty(IDeserializer deserializer)
    {
        return new Empty();
    }

    public sealed partial record Empty
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(1);
            serializer.DecreaseContainerDepth();
        }

    }
    public static Shape Deserialize(IDeserializer deserializer)
    {
        var index = deserializer.DeserializeVariantIndex();
        return index switch
        {
            0 => DeserializeCircle(deserializer),
            1 => DeserializeEmpty(deserializer),
            _ => throw new DeserializationError("Unknown variant index for Shape: " + index),
        }
        ;
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Shape BincodeDeserialize(byte[] input)
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
