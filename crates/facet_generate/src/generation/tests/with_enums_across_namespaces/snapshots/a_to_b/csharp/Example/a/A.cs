using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Facet.Runtime.Bincode;

namespace Example.A;

public partial class Row : ObservableObject, IFacetSerializable, IFacetDeserializable<Row> {
    [ObservableProperty]
    private Example.B.Status _status;
    [ObservableProperty]
    private Example.B.Signal _signal;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        Example.B.StatusBincode.Serialize(Status, serializer);
        Signal.Serialize(serializer);
        serializer.DecreaseContainerDepth();
    }

    public static Row Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var status = Example.B.StatusBincode.Deserialize(deserializer);
        var signal = Example.B.Signal.Deserialize(deserializer);
        deserializer.DecreaseContainerDepth();
        return new Row {
            Status = status,
            Signal = signal,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Row BincodeDeserialize(byte[] input)
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
