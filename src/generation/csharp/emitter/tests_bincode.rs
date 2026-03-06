use facet::Facet;

use crate::{
    emit,
    generation::{csharp::CSharp, Emitter, Encoding},
};

#[test]
fn bincode_struct_emits_serialize_helpers() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        public uint Id { get; set; }

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeU32(Id);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var Id = deserializer.DeserializeU32();
            deserializer.DecreaseContainerDepth();
            return new MyStruct
            {
                Id = Id,
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyStruct BincodeDeserialize(byte[] input)
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
    "#);
}

#[test]
fn bincode_unit_struct_emits_serialize_helpers() {
    #[derive(Facet)]
    struct Marker;

    let actual = emit!(Marker as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class Marker : ObservableObject, IFacetSerializable, IFacetDeserializable<Marker> {
        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.DecreaseContainerDepth();
        }

        public static Marker Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            deserializer.DecreaseContainerDepth();
            return new Marker
            {
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Marker BincodeDeserialize(byte[] input)
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
    "#);
}
