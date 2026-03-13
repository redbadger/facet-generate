using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public interface IFacetDeserializable<T>
{
    static abstract T Deserialize(IDeserializer deserializer);
}
