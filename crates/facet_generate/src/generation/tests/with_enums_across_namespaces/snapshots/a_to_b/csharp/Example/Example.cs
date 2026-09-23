using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example;

public partial class App : ObservableObject, IFacetSerializable, IFacetDeserializable<App> {
    [ObservableProperty]
    private Example.A.Row _row;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Row.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static App Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var row = Example.A.Row.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new App {
            Row = row,
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
