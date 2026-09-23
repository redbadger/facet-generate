using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example;

public partial class Card : ObservableObject, IFacetSerializable, IFacetDeserializable<Card> {
    [ObservableProperty]
    private Example.Kit.Presence _presence;
    [ObservableProperty]
    private Example.Kit.Shape _shape;
    [ObservableProperty]
    private ObservableCollection<Example.Kit.Shape?> _shapes;
    [ObservableProperty]
    private Example.Kit.Badge _badge;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Example.Kit.PresenceBincode.Serialize(Presence, serializer);
        Shape.Serialize(serializer);
        FacetHelpers.SerializeCollection(Shapes, serializer, (item, s) => FacetHelpers.SerializeOptionRef(item, s, (item, s) => item.Serialize(s)));
        Badge.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Card Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var presence = Example.Kit.PresenceBincode.Deserialize(deserializer);
        var shape = Example.Kit.Shape.Deserialize(deserializer);
        var shapes = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeOptionRef(d, d => Example.Kit.Shape.Deserialize(d)));
        var badge = Example.Kit.Badge.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Card {
            Presence = presence,
            Shape = shape,
            Shapes = shapes,
            Badge = badge,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Card BincodeDeserialize(byte[] input)
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

public partial class Presence : ObservableObject, IFacetSerializable, IFacetDeserializable<Presence> {
    [ObservableProperty]
    private ulong _since;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeU64(Since);
        serializer.DecreaseContainerDepth();
    }

    public static Presence Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var since = deserializer.DeserializeU64();
        deserializer.DecreaseContainerDepth();
        return new Presence {
            Since = since,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
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

public partial class Sighting : ObservableObject, IFacetSerializable, IFacetDeserializable<Sighting> {
    [ObservableProperty]
    private Presence _lastSeen;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        LastSeen.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Sighting Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var lastSeen = Presence.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Sighting {
            LastSeen = lastSeen,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Sighting BincodeDeserialize(byte[] input)
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
