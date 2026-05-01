namespace Facet.Runtime.Serde;

public interface IDeserializer
{
    void IncreaseContainerDepth();

    void DecreaseContainerDepth();

    Unit DeserializeUnit();

    bool DeserializeBool();

    sbyte DeserializeI8();

    short DeserializeI16();

    int DeserializeI32();

    long DeserializeI64();

    Int128 DeserializeI128();

    byte DeserializeU8();

    ushort DeserializeU16();

    uint DeserializeU32();

    ulong DeserializeU64();

    UInt128 DeserializeU128();

    float DeserializeF32();

    double DeserializeF64();

    char DeserializeChar();

    string DeserializeStr();

    byte[] DeserializeBytes();

    ulong DeserializeLen();

    uint DeserializeVariantIndex();

    bool DeserializeOptionTag();

    int GetBufferOffset();
}
