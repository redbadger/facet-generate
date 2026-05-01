using System;

namespace Facet.Runtime.Serde;

public sealed class DeserializationError : Exception
{
    public DeserializationError(string message) : base(message)
    {
    }
}
