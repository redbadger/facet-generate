using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example.Kv;

public partial class Entry : ObservableObject, IFacetSerializable, IFacetDeserializable<Entry> {
    [ObservableProperty]
    private Example.Kv.Level _level;
    [ObservableProperty]
    private Example.Kv.Outcome _outcome;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Example.Kv.LevelBincode.Serialize(Level, serializer);
        Outcome.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Entry Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var level = Example.Kv.LevelBincode.Deserialize(deserializer);
        var outcome = Example.Kv.Outcome.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Entry {
            Level = level,
            Outcome = outcome,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Entry BincodeDeserialize(byte[] input)
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
