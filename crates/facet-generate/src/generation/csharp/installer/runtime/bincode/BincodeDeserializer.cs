using Facet.Runtime.Serde;
using System;
using System.IO;

namespace Facet.Runtime.Bincode;

public sealed class BincodeDeserializer : IDeserializer
{
    private readonly MemoryStream stream;
    private readonly BinaryReader reader;
    private long containerDepthBudget = long.MaxValue;

    public BincodeDeserializer(byte[] input)
    {
        if (input is null || input.Length == 0)
        {
            throw new DeserializationError("Cannot deserialize null or empty input");
        }

        stream = new MemoryStream(input);
        reader = new BinaryReader(stream);
    }

    public void IncreaseContainerDepth()
    {
        if (containerDepthBudget == 0)
        {
            throw new DeserializationError("Exceeded maximum container depth");
        }

        containerDepthBudget -= 1;
    }

    public void DecreaseContainerDepth()
    {
        containerDepthBudget += 1;
    }

    public Unit DeserializeUnit()
    {
        return new Unit();
    }

    public bool DeserializeBool()
    {
        var value = reader.ReadByte();
        return value switch
        {
            0 => false,
            1 => true,
            _ => throw new DeserializationError("Incorrect boolean value")
        };
    }

    public sbyte DeserializeI8()
    {
        return reader.ReadSByte();
    }

    public short DeserializeI16()
    {
        return reader.ReadInt16();
    }

    public int DeserializeI32()
    {
        return reader.ReadInt32();
    }

    public long DeserializeI64()
    {
        return reader.ReadInt64();
    }

    public Int128 DeserializeI128()
    {
        return unchecked((Int128)DeserializeU128());
    }

    public byte DeserializeU8()
    {
        return reader.ReadByte();
    }

    public ushort DeserializeU16()
    {
        return reader.ReadUInt16();
    }

    public uint DeserializeU32()
    {
        return reader.ReadUInt32();
    }

    public ulong DeserializeU64()
    {
        return reader.ReadUInt64();
    }

    public UInt128 DeserializeU128()
    {
        var lower = reader.ReadUInt64();
        var upper = reader.ReadUInt64();
        return ((UInt128)upper << 64) | lower;
    }

    public float DeserializeF32()
    {
        return reader.ReadSingle();
    }

    public double DeserializeF64()
    {
        return reader.ReadDouble();
    }

    public char DeserializeChar()
    {
        return checked((char)DeserializeU32());
    }

    public string DeserializeStr()
    {
        var bytes = DeserializeBytes();
        return System.Text.Encoding.UTF8.GetString(bytes);
    }

    public byte[] DeserializeBytes()
    {
        var length = DeserializeLen();
        if (length > int.MaxValue)
        {
            throw new DeserializationError("Incorrect length value for byte array");
        }

        return reader.ReadBytes((int)length);
    }

    public ulong DeserializeLen()
    {
        return DeserializeU64();
    }

    public uint DeserializeVariantIndex()
    {
        return DeserializeU32();
    }

    public bool DeserializeOptionTag()
    {
        return DeserializeBool();
    }

    public int GetBufferOffset()
    {
        return checked((int)stream.Position);
    }

    public static T Deserialize<T>(byte[] input)
        where T : IFacetDeserializable<T>
    {
        var deserializer = new BincodeDeserializer(input);
        var value = T.Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }

        return value;
    }
}
