using System;
using System.IO;

using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public sealed class BincodeSerializer : ISerializer
{
    private readonly MemoryStream stream = new();
    private readonly BinaryWriter writer;
    private long containerDepthBudget = long.MaxValue;

    public BincodeSerializer()
    {
        writer = new BinaryWriter(stream);
    }

    public void IncreaseContainerDepth()
    {
        if (containerDepthBudget == 0)
        {
            throw new SerializationError("Exceeded maximum container depth");
        }

        containerDepthBudget -= 1;
    }

    public void DecreaseContainerDepth()
    {
        containerDepthBudget += 1;
    }

    public void SerializeUnit(Unit value)
    {
    }

    public void SerializeBool(bool value)
    {
        writer.Write((byte)(value ? 1 : 0));
    }

    public void SerializeI8(sbyte value)
    {
        writer.Write(value);
    }

    public void SerializeI16(short value)
    {
        writer.Write(value);
    }

    public void SerializeI32(int value)
    {
        writer.Write(value);
    }

    public void SerializeI64(long value)
    {
        writer.Write(value);
    }

    public void SerializeI128(Int128 value)
    {
        SerializeU128(unchecked((UInt128)value));
    }

    public void SerializeU8(byte value)
    {
        writer.Write(value);
    }

    public void SerializeU16(ushort value)
    {
        writer.Write(value);
    }

    public void SerializeU32(uint value)
    {
        writer.Write(value);
    }

    public void SerializeU64(ulong value)
    {
        writer.Write(value);
    }

    public void SerializeU128(UInt128 value)
    {
        writer.Write((ulong)(value & ulong.MaxValue));
        writer.Write((ulong)(value >> 64));
    }

    public void SerializeF32(float value)
    {
        writer.Write(value);
    }

    public void SerializeF64(double value)
    {
        writer.Write(value);
    }

    public void SerializeChar(char value)
    {
        SerializeU32(value);
    }

    public void SerializeStr(string value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        SerializeBytes(System.Text.Encoding.UTF8.GetBytes(value));
    }

    public void SerializeBytes(byte[] value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        SerializeLen((ulong)value.Length);
        writer.Write(value);
    }

    public void SerializeLen(ulong value)
    {
        SerializeU64(value);
    }

    public void SerializeVariantIndex(uint value)
    {
        SerializeU32(value);
    }

    public void SerializeOptionTag(bool value)
    {
        SerializeBool(value);
    }

    public byte[] GetBytes()
    {
        return stream.ToArray();
    }

    public int GetBufferOffset()
    {
        return checked((int)stream.Position);
    }

    public static byte[] Serialize<T>(T value) where T : notnull
    {
        var serializer = new BincodeSerializer();
        switch (value)
        {
            case IFacetSerializable serializable:
                serializable.Serialize(serializer);
                break;
            default:
                throw new SerializationError($"Type {typeof(T).Name} does not implement IFacetSerializable");
        }

        return serializer.GetBytes();
    }
}
