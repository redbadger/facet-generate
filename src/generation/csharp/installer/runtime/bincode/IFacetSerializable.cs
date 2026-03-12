using Facet.Runtime.Serde;

namespace Facet.Runtime.Bincode;

public interface IFacetSerializable
{
    void Serialize(ISerializer serializer);
}
