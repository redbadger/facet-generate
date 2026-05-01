using System;

namespace Facet.Runtime.Serde;

public sealed class SerializationError : Exception
{
    public SerializationError(string message) : base(message)
    {
    }
}
