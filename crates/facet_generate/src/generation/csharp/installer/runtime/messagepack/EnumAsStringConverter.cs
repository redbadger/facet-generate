using System;
using Nerdbank.MessagePack;
using PolyType;

namespace Facet.Runtime.MessagePack;

public sealed class EnumAsStringConverter<T> : MessagePackConverter<T>
    where T : struct, Enum
{
    public override T Read(ref MessagePackReader reader, SerializationContext context)
    {
        var s = reader.ReadString()
            ?? throw new MessagePackSerializationException("Expected string for enum value");
        return Enum.Parse<T>(s);
    }

    public override void Write(ref MessagePackWriter writer, in T value, SerializationContext context)
    {
        writer.Write(value.ToString());
    }
}
