using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example;

public partial class Shelf : ObservableObject, IFacetSerializable, IFacetDeserializable<Shelf> {
    [ObservableProperty]
    private Example.Kit.Set _set;
    [ObservableProperty]
    private HashSet<uint> _ids;
    [ObservableProperty]
    private Unit _unit;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Set.Serialize(serializer);
        FacetHelpers.SerializeCollection(Ids, serializer, (item, s) => s.SerializeU32(item));
        Unit.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Shelf Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var set = Example.Kit.Set.Deserialize(deserializer);
        var ids = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeU32());
        var unit = Unit.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Shelf {
            Set = set,
            Ids = ids,
            Unit = unit,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Shelf BincodeDeserialize(byte[] input)
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

public partial class Unit : ObservableObject, IFacetSerializable, IFacetDeserializable<Unit> {
    [ObservableProperty]
    private uint _value;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeU32(Value);
        serializer.DecreaseContainerDepth();
    }

    public static Unit Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var value = deserializer.DeserializeU32();
        deserializer.DecreaseContainerDepth();
        return new Unit {
            Value = value,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Unit BincodeDeserialize(byte[] input)
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
