namespace Facet.Runtime.Serde;

public interface ISerializer
{
    void IncreaseContainerDepth();

    void DecreaseContainerDepth();

    void SerializeUnit(Unit value);

    void SerializeBool(bool value);

    void SerializeI8(sbyte value);

    void SerializeI16(short value);

    void SerializeI32(int value);

    void SerializeI64(long value);

    void SerializeI128(Int128 value);

    void SerializeU8(byte value);

    void SerializeU16(ushort value);

    void SerializeU32(uint value);

    void SerializeU64(ulong value);

    void SerializeU128(UInt128 value);

    void SerializeF32(float value);

    void SerializeF64(double value);

    void SerializeChar(char value);

    void SerializeStr(string value);

    void SerializeBytes(byte[] value);

    void SerializeLen(ulong value);

    void SerializeVariantIndex(uint value);

    void SerializeOptionTag(bool value);

    byte[] GetBytes();

    int GetBufferOffset();
}
