using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example.Kit;

public partial class Set : ObservableObject, IFacetSerializable, IFacetDeserializable<Set> {
    [ObservableProperty]
    private uint _value;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeU32(Value);
        serializer.DecreaseContainerDepth();
    }

    public static Set Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var value = deserializer.DeserializeU32();
        deserializer.DecreaseContainerDepth();
        return new Set {
            Value = value,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Set BincodeDeserialize(byte[] input)
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

public partial class Tray : ObservableObject, IFacetSerializable, IFacetDeserializable<Tray> {
    [ObservableProperty]
    private global::Facet.Runtime.Serde.Unit _nothing;
    [ObservableProperty]
    private HashSet<uint> _ids;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeUnit(Nothing);
        FacetHelpers.SerializeCollection(Ids, serializer, (item, s) => s.SerializeU32(item));
        serializer.DecreaseContainerDepth();
    }

    public static Tray Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var nothing = deserializer.DeserializeUnit();
        var ids = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeU32());
        deserializer.DecreaseContainerDepth();
        return new Tray {
            Nothing = nothing,
            Ids = ids,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Tray BincodeDeserialize(byte[] input)
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
