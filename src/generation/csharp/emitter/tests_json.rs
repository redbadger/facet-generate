use facet::Facet;

use crate::{
    emit,
    generation::{Emitter, Encoding, csharp::CSharp},
};

#[test]
fn json_struct_shape_matches_encoding_none() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        public uint Id { get; set; }

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }
    "#);
}

#[test]
fn json_unit_enum_uses_string_enum_converter() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Status {
        Ready,
        Done,
    }

    let actual = emit!(Status as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum Status {
        Ready,
        Done
    }
    "#);
}
