//! Snapshot tests for the C# emitter — **JSON encoding**.
//!
//! Mirrors [`super::tests`] but with [`JsonPlugin`]. Every generated type
//! names its `{Type}JsonConverter`, written after it, which writes the JSON
//! `serde_json` writes; a struct's fields get `[JsonPropertyName]` with their
//! wire names; and types other than unit enums get `JsonSerialize` /
//! `JsonDeserialize` convenience methods backed by the `JsonSerde` static
//! helper.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::*;
use crate::generation::json::JsonPlugin;
use crate::{self as fg, emit};

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    [JsonConverter(typeof(UnitStructJsonConverter))]
    public sealed record UnitStruct {
        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static UnitStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<UnitStruct>(input);
        }
    }

    public sealed class UnitStructJsonConverter : JsonConverter<UnitStruct> {
        public override bool HandleNull => true;

        public override UnitStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.ReadUnit(ref reader, "UnitStruct");
            return new UnitStruct();
        }

        public override void Write(Utf8JsonWriter writer, UnitStruct value, JsonSerializerOptions options)
        {
            writer.WriteNullValue();
        }
    }
    "#);
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    [JsonConverter(typeof(NewTypeJsonConverter))]
    public partial class NewType : ObservableObject {
        [ObservableProperty]
        private string _value;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static NewType JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<NewType>(input);
        }
    }

    public sealed class NewTypeJsonConverter : JsonConverter<NewType> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;

        public override bool HandleNull => true;

        public override NewType Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return new NewType { Value = FacetJson.Read(_0, ref reader, options) };
        }

        public override void Write(Utf8JsonWriter writer, NewType value, JsonSerializerOptions options)
        {
            _0.Write(writer, value.Value, options);
        }

        public override NewType ReadAsPropertyName(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return new NewType { Value = FacetJson.ReadKey(_0, ref reader, options) };
        }

        public override void WriteAsPropertyName(Utf8JsonWriter writer, NewType value, JsonSerializerOptions options)
        {
            _0.WriteAsPropertyName(writer, value.Value, options);
        }
    }
    ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(TupleStructJsonConverter))]
    public partial class TupleStruct : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static TupleStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<TupleStruct>(input);
        }
    }

    public sealed class TupleStructJsonConverter : JsonConverter<TupleStruct> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;

        public override bool HandleNull => true;

        public override TupleStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartArray(ref reader, "TupleStruct");
            var e0 = FacetJson.Element(_0, ref reader, options, "TupleStruct");
            var e1 = FacetJson.Element(_1, ref reader, options, "TupleStruct");
            FacetJson.EndArray(ref reader, "TupleStruct");
            return new TupleStruct { Field0 = e0, Field1 = e1 };
        }

        public override void Write(Utf8JsonWriter writer, TupleStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartArray();
            _0.Write(writer, value.Field0, options);
            _1.Write(writer, value.Field1, options);
            writer.WriteEndArray();
        }
    }
    "#);
}

#[test]
fn struct_with_fields_of_primitive_types() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct StructWithFields {
        unit: (),
        bool: bool,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128,
        f32: f32,
        f64: f64,
        char: char,
        string: String,
    }

    let actual = emit!(StructWithFields as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(StructWithFieldsJsonConverter))]
    public partial class StructWithFields : ObservableObject {
        [property: JsonPropertyName("unit")]
        [ObservableProperty]
        private Unit _unit;
        [property: JsonPropertyName("bool")]
        [ObservableProperty]
        private bool _bool;
        [property: JsonPropertyName("i8")]
        [ObservableProperty]
        private sbyte _i8;
        [property: JsonPropertyName("i16")]
        [ObservableProperty]
        private short _i16;
        [property: JsonPropertyName("i32")]
        [ObservableProperty]
        private int _i32;
        [property: JsonPropertyName("i64")]
        [ObservableProperty]
        private long _i64;
        [property: JsonPropertyName("i128")]
        [ObservableProperty]
        private Int128 _i128;
        [property: JsonPropertyName("u8")]
        [ObservableProperty]
        private byte _u8;
        [property: JsonPropertyName("u16")]
        [ObservableProperty]
        private ushort _u16;
        [property: JsonPropertyName("u32")]
        [ObservableProperty]
        private uint _u32;
        [property: JsonPropertyName("u64")]
        [ObservableProperty]
        private ulong _u64;
        [property: JsonPropertyName("u128")]
        [ObservableProperty]
        private UInt128 _u128;
        [property: JsonPropertyName("f32")]
        [ObservableProperty]
        private float _f32;
        [property: JsonPropertyName("f64")]
        [ObservableProperty]
        private double _f64;
        [property: JsonPropertyName("char")]
        [ObservableProperty]
        private char _char;
        [property: JsonPropertyName("string")]
        [ObservableProperty]
        private string _string;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static StructWithFields JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<StructWithFields>(input);
        }
    }

    public sealed class StructWithFieldsJsonConverter : JsonConverter<StructWithFields> {
        private static readonly JsonConverter<Unit> _0 = FacetJson.Unit;
        private static readonly JsonConverter<bool> _1 = FacetJson.Bool;
        private static readonly JsonConverter<sbyte> _2 = FacetJson.I8;
        private static readonly JsonConverter<short> _3 = FacetJson.I16;
        private static readonly JsonConverter<int> _4 = FacetJson.I32;
        private static readonly JsonConverter<long> _5 = FacetJson.I64;
        private static readonly JsonConverter<Int128> _6 = FacetJson.I128;
        private static readonly JsonConverter<byte> _7 = FacetJson.U8;
        private static readonly JsonConverter<ushort> _8 = FacetJson.U16;
        private static readonly JsonConverter<uint> _9 = FacetJson.U32;
        private static readonly JsonConverter<ulong> _10 = FacetJson.U64;
        private static readonly JsonConverter<UInt128> _11 = FacetJson.U128;
        private static readonly JsonConverter<float> _12 = FacetJson.F32;
        private static readonly JsonConverter<double> _13 = FacetJson.F64;
        private static readonly JsonConverter<char> _14 = FacetJson.Char;
        private static readonly JsonConverter<string> _15 = FacetJson.Str;

        public override bool HandleNull => true;

        public override StructWithFields Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "StructWithFields");
            Unit f0 = default!;
            var has0 = false;
            bool f1 = default!;
            var has1 = false;
            sbyte f2 = default!;
            var has2 = false;
            short f3 = default!;
            var has3 = false;
            int f4 = default!;
            var has4 = false;
            long f5 = default!;
            var has5 = false;
            Int128 f6 = default!;
            var has6 = false;
            byte f7 = default!;
            var has7 = false;
            ushort f8 = default!;
            var has8 = false;
            uint f9 = default!;
            var has9 = false;
            ulong f10 = default!;
            var has10 = false;
            UInt128 f11 = default!;
            var has11 = false;
            float f12 = default!;
            var has12 = false;
            double f13 = default!;
            var has13 = false;
            char f14 = default!;
            var has14 = false;
            string f15 = default!;
            var has15 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "unit":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "bool":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "i8":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "i16":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        has3 = true;
                        break;
                    case "i32":
                        f4 = FacetJson.Read(_4, ref reader, options);
                        has4 = true;
                        break;
                    case "i64":
                        f5 = FacetJson.Read(_5, ref reader, options);
                        has5 = true;
                        break;
                    case "i128":
                        f6 = FacetJson.Read(_6, ref reader, options);
                        has6 = true;
                        break;
                    case "u8":
                        f7 = FacetJson.Read(_7, ref reader, options);
                        has7 = true;
                        break;
                    case "u16":
                        f8 = FacetJson.Read(_8, ref reader, options);
                        has8 = true;
                        break;
                    case "u32":
                        f9 = FacetJson.Read(_9, ref reader, options);
                        has9 = true;
                        break;
                    case "u64":
                        f10 = FacetJson.Read(_10, ref reader, options);
                        has10 = true;
                        break;
                    case "u128":
                        f11 = FacetJson.Read(_11, ref reader, options);
                        has11 = true;
                        break;
                    case "f32":
                        f12 = FacetJson.Read(_12, ref reader, options);
                        has12 = true;
                        break;
                    case "f64":
                        f13 = FacetJson.Read(_13, ref reader, options);
                        has13 = true;
                        break;
                    case "char":
                        f14 = FacetJson.Read(_14, ref reader, options);
                        has14 = true;
                        break;
                    case "string":
                        f15 = FacetJson.Read(_15, ref reader, options);
                        has15 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new StructWithFields
            {
                Unit = FacetJson.Required(has0, f0, "unit", "StructWithFields"),
                Bool = FacetJson.Required(has1, f1, "bool", "StructWithFields"),
                I8 = FacetJson.Required(has2, f2, "i8", "StructWithFields"),
                I16 = FacetJson.Required(has3, f3, "i16", "StructWithFields"),
                I32 = FacetJson.Required(has4, f4, "i32", "StructWithFields"),
                I64 = FacetJson.Required(has5, f5, "i64", "StructWithFields"),
                I128 = FacetJson.Required(has6, f6, "i128", "StructWithFields"),
                U8 = FacetJson.Required(has7, f7, "u8", "StructWithFields"),
                U16 = FacetJson.Required(has8, f8, "u16", "StructWithFields"),
                U32 = FacetJson.Required(has9, f9, "u32", "StructWithFields"),
                U64 = FacetJson.Required(has10, f10, "u64", "StructWithFields"),
                U128 = FacetJson.Required(has11, f11, "u128", "StructWithFields"),
                F32 = FacetJson.Required(has12, f12, "f32", "StructWithFields"),
                F64 = FacetJson.Required(has13, f13, "f64", "StructWithFields"),
                Char = FacetJson.Required(has14, f14, "char", "StructWithFields"),
                String = FacetJson.Required(has15, f15, "string", "StructWithFields"),
            };
        }

        public override void Write(Utf8JsonWriter writer, StructWithFields value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "unit", _0, value.Unit, options);
            FacetJson.WriteField(writer, "bool", _1, value.Bool, options);
            FacetJson.WriteField(writer, "i8", _2, value.I8, options);
            FacetJson.WriteField(writer, "i16", _3, value.I16, options);
            FacetJson.WriteField(writer, "i32", _4, value.I32, options);
            FacetJson.WriteField(writer, "i64", _5, value.I64, options);
            FacetJson.WriteField(writer, "i128", _6, value.I128, options);
            FacetJson.WriteField(writer, "u8", _7, value.U8, options);
            FacetJson.WriteField(writer, "u16", _8, value.U16, options);
            FacetJson.WriteField(writer, "u32", _9, value.U32, options);
            FacetJson.WriteField(writer, "u64", _10, value.U64, options);
            FacetJson.WriteField(writer, "u128", _11, value.U128, options);
            FacetJson.WriteField(writer, "f32", _12, value.F32, options);
            FacetJson.WriteField(writer, "f64", _13, value.F64, options);
            FacetJson.WriteField(writer, "char", _14, value.Char, options);
            FacetJson.WriteField(writer, "string", _15, value.String, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_fields_of_user_types() {
    #[derive(Facet)]
    struct Inner1 {
        field1: String,
    }

    #[derive(Facet)]
    struct Inner2(String);

    #[derive(Facet)]
    struct Inner3(String, i32);

    #[derive(Facet)]
    struct Outer {
        one: Inner1,
        two: Inner2,
        three: Inner3,
    }

    let actual = emit!(Outer as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(Inner1JsonConverter))]
    public partial class Inner1 : ObservableObject {
        [property: JsonPropertyName("field1")]
        [ObservableProperty]
        private string _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner1 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner1>(input);
        }
    }

    public sealed class Inner1JsonConverter : JsonConverter<Inner1> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;

        public override bool HandleNull => true;

        public override Inner1 Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Inner1");
            string f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "field1":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Inner1
            {
                Field1 = FacetJson.Required(has0, f0, "field1", "Inner1"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Inner1 value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "field1", _0, value.Field1, options);
            writer.WriteEndObject();
        }
    }

    [JsonConverter(typeof(Inner2JsonConverter))]
    public partial class Inner2 : ObservableObject {
        [ObservableProperty]
        private string _value;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner2 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner2>(input);
        }
    }

    public sealed class Inner2JsonConverter : JsonConverter<Inner2> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;

        public override bool HandleNull => true;

        public override Inner2 Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return new Inner2 { Value = FacetJson.Read(_0, ref reader, options) };
        }

        public override void Write(Utf8JsonWriter writer, Inner2 value, JsonSerializerOptions options)
        {
            _0.Write(writer, value.Value, options);
        }

        public override Inner2 ReadAsPropertyName(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return new Inner2 { Value = FacetJson.ReadKey(_0, ref reader, options) };
        }

        public override void WriteAsPropertyName(Utf8JsonWriter writer, Inner2 value, JsonSerializerOptions options)
        {
            _0.WriteAsPropertyName(writer, value.Value, options);
        }
    }

    [JsonConverter(typeof(Inner3JsonConverter))]
    public partial class Inner3 : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner3 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner3>(input);
        }
    }

    public sealed class Inner3JsonConverter : JsonConverter<Inner3> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;

        public override bool HandleNull => true;

        public override Inner3 Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartArray(ref reader, "Inner3");
            var e0 = FacetJson.Element(_0, ref reader, options, "Inner3");
            var e1 = FacetJson.Element(_1, ref reader, options, "Inner3");
            FacetJson.EndArray(ref reader, "Inner3");
            return new Inner3 { Field0 = e0, Field1 = e1 };
        }

        public override void Write(Utf8JsonWriter writer, Inner3 value, JsonSerializerOptions options)
        {
            writer.WriteStartArray();
            _0.Write(writer, value.Field0, options);
            _1.Write(writer, value.Field1, options);
            writer.WriteEndArray();
        }
    }

    [JsonConverter(typeof(OuterJsonConverter))]
    public partial class Outer : ObservableObject {
        [property: JsonPropertyName("one")]
        [ObservableProperty]
        private Inner1 _one;
        [property: JsonPropertyName("two")]
        [ObservableProperty]
        private Inner2 _two;
        [property: JsonPropertyName("three")]
        [ObservableProperty]
        private Inner3 _three;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Outer JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Outer>(input);
        }
    }

    public sealed class OuterJsonConverter : JsonConverter<Outer> {
        private static readonly JsonConverter<Inner1> _0 = FacetJson.Of<Inner1>();
        private static readonly JsonConverter<Inner2> _1 = FacetJson.Of<Inner2>();
        private static readonly JsonConverter<Inner3> _2 = FacetJson.Of<Inner3>();

        public override bool HandleNull => true;

        public override Outer Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Outer");
            Inner1 f0 = default!;
            var has0 = false;
            Inner2 f1 = default!;
            var has1 = false;
            Inner3 f2 = default!;
            var has2 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "one":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "two":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "three":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Outer
            {
                One = FacetJson.Required(has0, f0, "one", "Outer"),
                Two = FacetJson.Required(has1, f1, "two", "Outer"),
                Three = FacetJson.Required(has2, f2, "three", "Outer"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Outer value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "one", _0, value.One, options);
            FacetJson.WriteField(writer, "two", _1, value.Two, options);
            FacetJson.WriteField(writer, "three", _2, value.Three, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<(string, int)> _0 = FacetJson.Tuple(FacetJson.Str, FacetJson.I32);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            (string, int) f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "one":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                One = FacetJson.Required(has0, f0, "one", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "one", _0, value.One, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int, ushort) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<(string, int, ushort)> _0 = FacetJson.Tuple(FacetJson.Str, FacetJson.I32, FacetJson.U16);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            (string, int, ushort) f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "one":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                One = FacetJson.Required(has0, f0, "one", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "one", _0, value.One, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int, ushort, float) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<(string, int, ushort, float)> _0 = FacetJson.Tuple(FacetJson.Str, FacetJson.I32, FacetJson.U16, FacetJson.F32);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            (string, int, ushort, float) f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "one":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                One = FacetJson.Required(has0, f0, "one", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "one", _0, value.One, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn enum_with_unit_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3,
    }

    let actual = emit!(EnumWithUnitVariants as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(EnumWithUnitVariantsJsonConverter))]
    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
    }

    public sealed class EnumWithUnitVariantsJsonConverter : JsonConverter<EnumWithUnitVariants> {
        private static readonly JsonEnum _enum = new("EnumWithUnitVariants");

        public override bool HandleNull => true;

        public override EnumWithUnitVariants Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, EnumWithUnitVariants value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case EnumWithUnitVariants.Variant1:
                    _enum.WriteVariant(writer, "Variant1");
                    break;
                case EnumWithUnitVariants.Variant2:
                    _enum.WriteVariant(writer, "Variant2");
                    break;
                case EnumWithUnitVariants.Variant3:
                    _enum.WriteVariant(writer, "Variant3");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        public override EnumWithUnitVariants ReadAsPropertyName(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            var variant = reader.GetString()!;
            switch (variant)
            {
                case "Variant1":
                    return EnumWithUnitVariants.Variant1;
                case "Variant2":
                    return EnumWithUnitVariants.Variant2;
                case "Variant3":
                    return EnumWithUnitVariants.Variant3;
                default:
                    throw _enum.Unknown(variant);
            }
        }

        public override void WriteAsPropertyName(Utf8JsonWriter writer, EnumWithUnitVariants value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case EnumWithUnitVariants.Variant1:
                    writer.WritePropertyName("Variant1");
                    break;
                case EnumWithUnitVariants.Variant2:
                    writer.WritePropertyName("Variant2");
                    break;
                case EnumWithUnitVariants.Variant3:
                    writer.WritePropertyName("Variant3");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static EnumWithUnitVariants _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                    _enum.Unit(hasPayload, ref reader, "Variant1");
                    return EnumWithUnitVariants.Variant1;
                case "Variant2":
                    _enum.Unit(hasPayload, ref reader, "Variant2");
                    return EnumWithUnitVariants.Variant2;
                case "Variant3":
                    _enum.Unit(hasPayload, ref reader, "Variant3");
                    return EnumWithUnitVariants.Variant3;
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public enum MyEnum {
        Variant1
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1:
                    _enum.WriteVariant(writer, "Variant1");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        public override MyEnum ReadAsPropertyName(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            var variant = reader.GetString()!;
            switch (variant)
            {
                case "Variant1":
                    return MyEnum.Variant1;
                default:
                    throw _enum.Unknown(variant);
            }
        }

        public override void WriteAsPropertyName(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1:
                    writer.WritePropertyName("Variant1");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                    _enum.Unit(hasPayload, ref reader, "Variant1");
                    return MyEnum.Variant1;
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
        }
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1 v:
                    _enum.WriteVariant(writer, "Variant1", w => _0.Write(w, v.Value, options));
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                    _enum.Payload(hasPayload, "Variant1");
                    return new MyEnum.Variant1(FacetJson.Read(_0, ref reader, options));
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_newtype_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
        Variant2(i32),
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public sealed record Variant2(int Value) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
        }
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1 v:
                    _enum.WriteVariant(writer, "Variant1", w => _0.Write(w, v.Value, options));
                    break;
                case MyEnum.Variant2 v:
                    _enum.WriteVariant(writer, "Variant2", w => _1.Write(w, v.Value, options));
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                    _enum.Payload(hasPayload, "Variant1");
                    return new MyEnum.Variant1(FacetJson.Read(_0, ref reader, options));
                case "Variant2":
                    _enum.Payload(hasPayload, "Variant2");
                    return new MyEnum.Variant2(FacetJson.Read(_1, ref reader, options));
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String, i32),
        Variant2(bool, f64, u8),
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Field0, int Field1) : MyEnum;

        public sealed record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
        }
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonConverter<bool> _2 = FacetJson.Bool;
        private static readonly JsonConverter<double> _3 = FacetJson.F64;
        private static readonly JsonConverter<byte> _4 = FacetJson.U8;
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1 v:
                    _enum.WriteVariant(writer, "Variant1", w =>
                    {
                        w.WriteStartArray();
                        _0.Write(w, v.Field0, options);
                        _1.Write(w, v.Field1, options);
                        w.WriteEndArray();
                    });
                    break;
                case MyEnum.Variant2 v:
                    _enum.WriteVariant(writer, "Variant2", w =>
                    {
                        w.WriteStartArray();
                        _2.Write(w, v.Field0, options);
                        _3.Write(w, v.Field1, options);
                        _4.Write(w, v.Field2, options);
                        w.WriteEndArray();
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                {
                    _enum.Payload(hasPayload, "Variant1");
                    FacetJson.StartArray(ref reader, "MyEnum.Variant1");
                    var e0 = FacetJson.Element(_0, ref reader, options, "MyEnum.Variant1");
                    var e1 = FacetJson.Element(_1, ref reader, options, "MyEnum.Variant1");
                    FacetJson.EndArray(ref reader, "MyEnum.Variant1");
                    return new MyEnum.Variant1(e0, e1);
                }
                case "Variant2":
                {
                    _enum.Payload(hasPayload, "Variant2");
                    FacetJson.StartArray(ref reader, "MyEnum.Variant2");
                    var e0 = FacetJson.Element(_2, ref reader, options, "MyEnum.Variant2");
                    var e1 = FacetJson.Element(_3, ref reader, options, "MyEnum.Variant2");
                    var e2 = FacetJson.Element(_4, ref reader, options, "MyEnum.Variant2");
                    FacetJson.EndArray(ref reader, "MyEnum.Variant2");
                    return new MyEnum.Variant2(e0, e1, e2);
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Field1, int Field2) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
        }
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Variant1 v:
                    _enum.WriteStructVariant(writer, "Variant1", w =>
                    {
                        FacetJson.WriteField(w, "field1", _0, v.Field1, options);
                        FacetJson.WriteField(w, "field2", _1, v.Field2, options);
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Variant1":
                {
                    _enum.Payload(hasPayload, "Variant1");
                    FacetJson.StartObject(ref reader, "MyEnum.Variant1");
                    string f0 = default!;
                    var has0 = false;
                    int f1 = default!;
                    var has1 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "field1":
                                f0 = FacetJson.Read(_0, ref reader, options);
                                has0 = true;
                                break;
                            case "field2":
                                f1 = FacetJson.Read(_1, ref reader, options);
                                has1 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new MyEnum.Variant1(
                        FacetJson.Required(has0, f0, "field1", "MyEnum.Variant1"),
                        FacetJson.Required(has1, f1, "field2", "MyEnum.Variant1"));
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn enum_with_mixed_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Unit,
        NewType(String),
        Tuple(String, i32),
        Struct { field: bool },
    }

    let actual = emit!(MyEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyEnumJsonConverter))]
    public abstract record MyEnum {
        public sealed record Unit() : MyEnum;

        public sealed record NewType(string Value) : MyEnum;

        public sealed record Tuple(string Field0, int Field1) : MyEnum;

        public sealed record Struct(bool Field) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
        }
    }

    public sealed class MyEnumJsonConverter : JsonConverter<MyEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<string> _1 = FacetJson.Str;
        private static readonly JsonConverter<int> _2 = FacetJson.I32;
        private static readonly JsonConverter<bool> _3 = FacetJson.Bool;
        private static readonly JsonEnum _enum = new("MyEnum");

        public override bool HandleNull => true;

        public override MyEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, MyEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case MyEnum.Unit:
                    _enum.WriteVariant(writer, "Unit");
                    break;
                case MyEnum.NewType v:
                    _enum.WriteVariant(writer, "NewType", w => _0.Write(w, v.Value, options));
                    break;
                case MyEnum.Tuple v:
                    _enum.WriteVariant(writer, "Tuple", w =>
                    {
                        w.WriteStartArray();
                        _1.Write(w, v.Field0, options);
                        _2.Write(w, v.Field1, options);
                        w.WriteEndArray();
                    });
                    break;
                case MyEnum.Struct v:
                    _enum.WriteStructVariant(writer, "Struct", w =>
                    {
                        FacetJson.WriteField(w, "field", _3, v.Field, options);
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static MyEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Unit":
                    _enum.Unit(hasPayload, ref reader, "Unit");
                    return new MyEnum.Unit();
                case "NewType":
                    _enum.Payload(hasPayload, "NewType");
                    return new MyEnum.NewType(FacetJson.Read(_0, ref reader, options));
                case "Tuple":
                {
                    _enum.Payload(hasPayload, "Tuple");
                    FacetJson.StartArray(ref reader, "MyEnum.Tuple");
                    var e0 = FacetJson.Element(_1, ref reader, options, "MyEnum.Tuple");
                    var e1 = FacetJson.Element(_2, ref reader, options, "MyEnum.Tuple");
                    FacetJson.EndArray(ref reader, "MyEnum.Tuple");
                    return new MyEnum.Tuple(e0, e1);
                }
                case "Struct":
                {
                    _enum.Payload(hasPayload, "Struct");
                    FacetJson.StartObject(ref reader, "MyEnum.Struct");
                    bool f0 = default!;
                    var has0 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "field":
                                f0 = FacetJson.Read(_3, ref reader, options);
                                has0 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new MyEnum.Struct(
                        FacetJson.Required(has0, f0, "field", "MyEnum.Struct"));
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

#[test]
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("items")]
        [ObservableProperty]
        private ObservableCollection<string> _items;
        [property: JsonPropertyName("numbers")]
        [ObservableProperty]
        private ObservableCollection<int> _numbers;
        [property: JsonPropertyName("nested_items")]
        [ObservableProperty]
        private ObservableCollection<ObservableCollection<string>> _nestedItems;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<ObservableCollection<string>> _0 = FacetJson.List(FacetJson.Str);
        private static readonly JsonConverter<ObservableCollection<int>> _1 = FacetJson.List(FacetJson.I32);
        private static readonly JsonConverter<ObservableCollection<ObservableCollection<string>>> _2 = FacetJson.List(FacetJson.List(FacetJson.Str));

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            ObservableCollection<string> f0 = default!;
            var has0 = false;
            ObservableCollection<int> f1 = default!;
            var has1 = false;
            ObservableCollection<ObservableCollection<string>> f2 = default!;
            var has2 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "items":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "numbers":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "nested_items":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                Items = FacetJson.Required(has0, f0, "items", "MyStruct"),
                Numbers = FacetJson.Required(has1, f1, "numbers", "MyStruct"),
                NestedItems = FacetJson.Required(has2, f2, "nested_items", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "items", _0, value.Items, options);
            FacetJson.WriteField(writer, "numbers", _1, value.Numbers, options);
            FacetJson.WriteField(writer, "nested_items", _2, value.NestedItems, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_option_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        optional_string: Option<String>,
        optional_number: Option<i32>,
        optional_bool: Option<bool>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("optional_string")]
        [ObservableProperty]
        private string? _optionalString;
        [property: JsonPropertyName("optional_number")]
        [ObservableProperty]
        private int? _optionalNumber;
        [property: JsonPropertyName("optional_bool")]
        [ObservableProperty]
        private bool? _optionalBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<string?> _0 = FacetJson.OptionRef(FacetJson.Str);
        private static readonly JsonConverter<int?> _1 = FacetJson.Option(FacetJson.I32);
        private static readonly JsonConverter<bool?> _2 = FacetJson.Option(FacetJson.Bool);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            string? f0 = default;
            int? f1 = default;
            bool? f2 = default;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "optional_string":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        break;
                    case "optional_number":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        break;
                    case "optional_bool":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                OptionalString = f0,
                OptionalNumber = f1,
                OptionalBool = f2,
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "optional_string", _0, value.OptionalString, options);
            FacetJson.WriteField(writer, "optional_number", _1, value.OptionalNumber, options);
            FacetJson.WriteField(writer, "optional_bool", _2, value.OptionalBool, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("string_to_int")]
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [property: JsonPropertyName("int_to_bool")]
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<Dictionary<string, int>> _0 = FacetJson.Map(FacetJson.Str, FacetJson.I32);
        private static readonly JsonConverter<Dictionary<int, bool>> _1 = FacetJson.Map(FacetJson.I32, FacetJson.Bool);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            Dictionary<string, int> f0 = default!;
            var has0 = false;
            Dictionary<int, bool> f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "string_to_int":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "int_to_bool":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                StringToInt = FacetJson.Required(has0, f0, "string_to_int", "MyStruct"),
                IntToBool = FacetJson.Required(has1, f1, "int_to_bool", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "string_to_int", _0, value.StringToInt, options);
            FacetJson.WriteField(writer, "int_to_bool", _1, value.IntToBool, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("optional_list")]
        [ObservableProperty]
        private ObservableCollection<string>? _optionalList;
        [property: JsonPropertyName("list_of_optionals")]
        [ObservableProperty]
        private ObservableCollection<int?> _listOfOptionals;
        [property: JsonPropertyName("map_to_list")]
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [property: JsonPropertyName("optional_map")]
        [ObservableProperty]
        private Dictionary<string, int>? _optionalMap;
        [property: JsonPropertyName("complex")]
        [ObservableProperty]
        private ObservableCollection<Dictionary<string, ObservableCollection<bool>>?> _complex;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<ObservableCollection<string>?> _0 = FacetJson.OptionRef(FacetJson.List(FacetJson.Str));
        private static readonly JsonConverter<ObservableCollection<int?>> _1 = FacetJson.List(FacetJson.Option(FacetJson.I32));
        private static readonly JsonConverter<Dictionary<string, ObservableCollection<bool>>> _2 = FacetJson.Map(FacetJson.Str, FacetJson.List(FacetJson.Bool));
        private static readonly JsonConverter<Dictionary<string, int>?> _3 = FacetJson.OptionRef(FacetJson.Map(FacetJson.Str, FacetJson.I32));
        private static readonly JsonConverter<ObservableCollection<Dictionary<string, ObservableCollection<bool>>?>> _4 = FacetJson.List(FacetJson.OptionRef(FacetJson.Map(FacetJson.Str, FacetJson.List(FacetJson.Bool))));

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            ObservableCollection<string>? f0 = default;
            ObservableCollection<int?> f1 = default!;
            var has1 = false;
            Dictionary<string, ObservableCollection<bool>> f2 = default!;
            var has2 = false;
            Dictionary<string, int>? f3 = default;
            ObservableCollection<Dictionary<string, ObservableCollection<bool>>?> f4 = default!;
            var has4 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "optional_list":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        break;
                    case "list_of_optionals":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "map_to_list":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "optional_map":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        break;
                    case "complex":
                        f4 = FacetJson.Read(_4, ref reader, options);
                        has4 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                OptionalList = f0,
                ListOfOptionals = FacetJson.Required(has1, f1, "list_of_optionals", "MyStruct"),
                MapToList = FacetJson.Required(has2, f2, "map_to_list", "MyStruct"),
                OptionalMap = f3,
                Complex = FacetJson.Required(has4, f4, "complex", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "optional_list", _0, value.OptionalList, options);
            FacetJson.WriteField(writer, "list_of_optionals", _1, value.ListOfOptionals, options);
            FacetJson.WriteField(writer, "map_to_list", _2, value.MapToList, options);
            FacetJson.WriteField(writer, "optional_map", _3, value.OptionalMap, options);
            FacetJson.WriteField(writer, "complex", _4, value.Complex, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_array_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        fixed_array: [i32; 5],
        byte_array: [u8; 32],
        string_array: [String; 3],
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("fixed_array")]
        [ObservableProperty]
        private int[] _fixedArray;
        [property: JsonPropertyName("byte_array")]
        [ObservableProperty]
        private byte[] _byteArray;
        [property: JsonPropertyName("string_array")]
        [ObservableProperty]
        private string[] _stringArray;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<int[]> _0 = FacetJson.Array(FacetJson.I32, 5);
        private static readonly JsonConverter<byte[]> _1 = FacetJson.Array(FacetJson.U8, 32);
        private static readonly JsonConverter<string[]> _2 = FacetJson.Array(FacetJson.Str, 3);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            int[] f0 = default!;
            var has0 = false;
            byte[] f1 = default!;
            var has1 = false;
            string[] f2 = default!;
            var has2 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "fixed_array":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "byte_array":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "string_array":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                FixedArray = FacetJson.Required(has0, f0, "fixed_array", "MyStruct"),
                ByteArray = FacetJson.Required(has1, f1, "byte_array", "MyStruct"),
                StringArray = FacetJson.Required(has2, f2, "string_array", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "fixed_array", _0, value.FixedArray, options);
            FacetJson.WriteField(writer, "byte_array", _1, value.ByteArray, options);
            FacetJson.WriteField(writer, "string_array", _2, value.StringArray, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("string_to_int")]
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [property: JsonPropertyName("int_to_bool")]
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<Dictionary<string, int>> _0 = FacetJson.Map(FacetJson.Str, FacetJson.I32);
        private static readonly JsonConverter<Dictionary<int, bool>> _1 = FacetJson.Map(FacetJson.I32, FacetJson.Bool);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            Dictionary<string, int> f0 = default!;
            var has0 = false;
            Dictionary<int, bool> f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "string_to_int":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "int_to_bool":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                StringToInt = FacetJson.Required(has0, f0, "string_to_int", "MyStruct"),
                IntToBool = FacetJson.Required(has1, f1, "int_to_bool", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "string_to_int", _0, value.StringToInt, options);
            FacetJson.WriteField(writer, "int_to_bool", _1, value.IntToBool, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("string_set")]
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [property: JsonPropertyName("int_set")]
        [ObservableProperty]
        private HashSet<int> _intSet;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<HashSet<string>> _0 = FacetJson.Set(FacetJson.Str);
        private static readonly JsonConverter<HashSet<int>> _1 = FacetJson.Set(FacetJson.I32);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            HashSet<string> f0 = default!;
            var has0 = false;
            HashSet<int> f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "string_set":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "int_set":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                StringSet = FacetJson.Required(has0, f0, "string_set", "MyStruct"),
                IntSet = FacetJson.Required(has1, f1, "int_set", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "string_set", _0, value.StringSet, options);
            FacetJson.WriteField(writer, "int_set", _1, value.IntSet, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("string_set")]
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [property: JsonPropertyName("int_set")]
        [ObservableProperty]
        private HashSet<int> _intSet;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<HashSet<string>> _0 = FacetJson.Set(FacetJson.Str);
        private static readonly JsonConverter<HashSet<int>> _1 = FacetJson.Set(FacetJson.I32);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            HashSet<string> f0 = default!;
            var has0 = false;
            HashSet<int> f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "string_set":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "int_set":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                StringSet = FacetJson.Required(has0, f0, "string_set", "MyStruct"),
                IntSet = FacetJson.Required(has1, f1, "int_set", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "string_set", _0, value.StringSet, options);
            FacetJson.WriteField(writer, "int_set", _1, value.IntSet, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("boxed_string")]
        [ObservableProperty]
        private string _boxedString;
        [property: JsonPropertyName("boxed_int")]
        [ObservableProperty]
        private int _boxedInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            string f0 = default!;
            var has0 = false;
            int f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "boxed_string":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "boxed_int":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                BoxedString = FacetJson.Required(has0, f0, "boxed_string", "MyStruct"),
                BoxedInt = FacetJson.Required(has1, f1, "boxed_int", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "boxed_string", _0, value.BoxedString, options);
            FacetJson.WriteField(writer, "boxed_int", _1, value.BoxedInt, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("rc_string")]
        [ObservableProperty]
        private string _rcString;
        [property: JsonPropertyName("rc_int")]
        [ObservableProperty]
        private int _rcInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            string f0 = default!;
            var has0 = false;
            int f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "rc_string":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "rc_int":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                RcString = FacetJson.Required(has0, f0, "rc_string", "MyStruct"),
                RcInt = FacetJson.Required(has1, f1, "rc_int", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "rc_string", _0, value.RcString, options);
            FacetJson.WriteField(writer, "rc_int", _1, value.RcInt, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("arc_string")]
        [ObservableProperty]
        private string _arcString;
        [property: JsonPropertyName("arc_int")]
        [ObservableProperty]
        private int _arcInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            string f0 = default!;
            var has0 = false;
            int f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "arc_string":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "arc_int":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                ArcString = FacetJson.Required(has0, f0, "arc_string", "MyStruct"),
                ArcInt = FacetJson.Required(has1, f1, "arc_int", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "arc_string", _0, value.ArcString, options);
            FacetJson.WriteField(writer, "arc_int", _1, value.ArcInt, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_mixed_collections_and_pointers() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        vec_of_sets: Vec<HashSet<String>>,
        optional_btree: Option<BTreeMap<String, i32>>,
        boxed_vec: Box<Vec<String>>,
        arc_option: Arc<Option<String>>,
        array_of_boxes: [Box<i32>; 3],
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("vec_of_sets")]
        [ObservableProperty]
        private ObservableCollection<HashSet<string>> _vecOfSets;
        [property: JsonPropertyName("optional_btree")]
        [ObservableProperty]
        private Dictionary<string, int>? _optionalBtree;
        [property: JsonPropertyName("boxed_vec")]
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [property: JsonPropertyName("arc_option")]
        [ObservableProperty]
        private string? _arcOption;
        [property: JsonPropertyName("array_of_boxes")]
        [ObservableProperty]
        private int[] _arrayOfBoxes;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<ObservableCollection<HashSet<string>>> _0 = FacetJson.List(FacetJson.Set(FacetJson.Str));
        private static readonly JsonConverter<Dictionary<string, int>?> _1 = FacetJson.OptionRef(FacetJson.Map(FacetJson.Str, FacetJson.I32));
        private static readonly JsonConverter<ObservableCollection<string>> _2 = FacetJson.List(FacetJson.Str);
        private static readonly JsonConverter<string?> _3 = FacetJson.OptionRef(FacetJson.Str);
        private static readonly JsonConverter<int[]> _4 = FacetJson.Array(FacetJson.I32, 3);

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            ObservableCollection<HashSet<string>> f0 = default!;
            var has0 = false;
            Dictionary<string, int>? f1 = default;
            ObservableCollection<string> f2 = default!;
            var has2 = false;
            string? f3 = default;
            int[] f4 = default!;
            var has4 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "vec_of_sets":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "optional_btree":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        break;
                    case "boxed_vec":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "arc_option":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        break;
                    case "array_of_boxes":
                        f4 = FacetJson.Read(_4, ref reader, options);
                        has4 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                VecOfSets = FacetJson.Required(has0, f0, "vec_of_sets", "MyStruct"),
                OptionalBtree = f1,
                BoxedVec = FacetJson.Required(has2, f2, "boxed_vec", "MyStruct"),
                ArcOption = f3,
                ArrayOfBoxes = FacetJson.Required(has4, f4, "array_of_boxes", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "vec_of_sets", _0, value.VecOfSets, options);
            FacetJson.WriteField(writer, "optional_btree", _1, value.OptionalBtree, options);
            FacetJson.WriteField(writer, "boxed_vec", _2, value.BoxedVec, options);
            FacetJson.WriteField(writer, "arc_option", _3, value.ArcOption, options);
            FacetJson.WriteField(writer, "array_of_boxes", _4, value.ArrayOfBoxes, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(fg::bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("data")]
        [ObservableProperty]
        private byte[] _data;
        [property: JsonPropertyName("name")]
        [ObservableProperty]
        private string _name;
        [property: JsonPropertyName("header")]
        [ObservableProperty]
        private byte[] _header;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<byte[]> _0 = FacetJson.Bytes;
        private static readonly JsonConverter<string> _1 = FacetJson.Str;
        private static readonly JsonConverter<byte[]> _2 = FacetJson.Bytes;

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            byte[] f0 = default!;
            var has0 = false;
            string f1 = default!;
            var has1 = false;
            byte[] f2 = default!;
            var has2 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "data":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "name":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "header":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                Data = FacetJson.Required(has0, f0, "data", "MyStruct"),
                Name = FacetJson.Required(has1, f1, "name", "MyStruct"),
                Header = FacetJson.Required(has2, f2, "header", "MyStruct"),
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "data", _0, value.Data, options);
            FacetJson.WriteField(writer, "name", _1, value.Name, options);
            FacetJson.WriteField(writer, "header", _2, value.Header, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn struct_with_bytes_field_and_slice() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        #[facet(fg::bytes)]
        data: &'a [u8],
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit!(MyStruct as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MyStructJsonConverter))]
    public partial class MyStruct : ObservableObject {
        [property: JsonPropertyName("data")]
        [ObservableProperty]
        private byte[] _data;
        [property: JsonPropertyName("name")]
        [ObservableProperty]
        private string _name;
        [property: JsonPropertyName("header")]
        [ObservableProperty]
        private byte[] _header;
        [property: JsonPropertyName("optional_bytes")]
        [ObservableProperty]
        private ObservableCollection<byte>? _optionalBytes;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }

    public sealed class MyStructJsonConverter : JsonConverter<MyStruct> {
        private static readonly JsonConverter<byte[]> _0 = FacetJson.Bytes;
        private static readonly JsonConverter<string> _1 = FacetJson.Str;
        private static readonly JsonConverter<byte[]> _2 = FacetJson.Bytes;
        private static readonly JsonConverter<ObservableCollection<byte>?> _3 = FacetJson.OptionRef(FacetJson.List(FacetJson.U8));

        public override bool HandleNull => true;

        public override MyStruct Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "MyStruct");
            byte[] f0 = default!;
            var has0 = false;
            string f1 = default!;
            var has1 = false;
            byte[] f2 = default!;
            var has2 = false;
            ObservableCollection<byte>? f3 = default;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "data":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "name":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "header":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "optional_bytes":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new MyStruct
            {
                Data = FacetJson.Required(has0, f0, "data", "MyStruct"),
                Name = FacetJson.Required(has1, f1, "name", "MyStruct"),
                Header = FacetJson.Required(has2, f2, "header", "MyStruct"),
                OptionalBytes = f3,
            };
        }

        public override void Write(Utf8JsonWriter writer, MyStruct value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "data", _0, value.Data, options);
            FacetJson.WriteField(writer, "name", _1, value.Name, options);
            FacetJson.WriteField(writer, "header", _2, value.Header, options);
            FacetJson.WriteField(writer, "optional_bytes", _3, value.OptionalBytes, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn keyword_fields_struct() {
    #[derive(Facet)]
    #[allow(clippy::struct_excessive_bools)]
    struct KeywordFields {
        r#default: String,
        r#in: i32,
        object: bool,
        import: bool,
    }

    let actual = emit!(KeywordFields as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(KeywordFieldsJsonConverter))]
    public partial class KeywordFields : ObservableObject {
        [property: JsonPropertyName("default")]
        [ObservableProperty]
        private string _default;
        [property: JsonPropertyName("in")]
        [ObservableProperty]
        private int _in;
        [property: JsonPropertyName("object")]
        [ObservableProperty]
        private bool _object;
        [property: JsonPropertyName("import")]
        [ObservableProperty]
        private bool _import;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static KeywordFields JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<KeywordFields>(input);
        }
    }

    public sealed class KeywordFieldsJsonConverter : JsonConverter<KeywordFields> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonConverter<bool> _2 = FacetJson.Bool;
        private static readonly JsonConverter<bool> _3 = FacetJson.Bool;

        public override bool HandleNull => true;

        public override KeywordFields Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "KeywordFields");
            string f0 = default!;
            var has0 = false;
            int f1 = default!;
            var has1 = false;
            bool f2 = default!;
            var has2 = false;
            bool f3 = default!;
            var has3 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "default":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "in":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "object":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "import":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        has3 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new KeywordFields
            {
                Default = FacetJson.Required(has0, f0, "default", "KeywordFields"),
                In = FacetJson.Required(has1, f1, "in", "KeywordFields"),
                Object = FacetJson.Required(has2, f2, "object", "KeywordFields"),
                Import = FacetJson.Required(has3, f3, "import", "KeywordFields"),
            };
        }

        public override void Write(Utf8JsonWriter writer, KeywordFields value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "default", _0, value.Default, options);
            FacetJson.WriteField(writer, "in", _1, value.In, options);
            FacetJson.WriteField(writer, "object", _2, value.Object, options);
            FacetJson.WriteField(writer, "import", _3, value.Import, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

#[test]
fn keyword_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum KeywordEnum {
        Default,
        Switch(String),
        Where { r#in: i32, r#default: String },
    }

    let actual = emit!(KeywordEnum as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(KeywordEnumJsonConverter))]
    public abstract record KeywordEnum {
        public sealed record Default() : KeywordEnum;

        public sealed record Switch(string Value) : KeywordEnum;

        public sealed record Where(int In, string Default) : KeywordEnum {
            public new string Default { get; init; } = Default;
        }

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static KeywordEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<KeywordEnum>(input);
        }
    }

    public sealed class KeywordEnumJsonConverter : JsonConverter<KeywordEnum> {
        private static readonly JsonConverter<string> _0 = FacetJson.Str;
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonConverter<string> _2 = FacetJson.Str;
        private static readonly JsonEnum _enum = new("KeywordEnum");

        public override bool HandleNull => true;

        public override KeywordEnum Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, KeywordEnum value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case KeywordEnum.Default:
                    _enum.WriteVariant(writer, "Default");
                    break;
                case KeywordEnum.Switch v:
                    _enum.WriteVariant(writer, "Switch", w => _0.Write(w, v.Value, options));
                    break;
                case KeywordEnum.Where v:
                    _enum.WriteStructVariant(writer, "Where", w =>
                    {
                        FacetJson.WriteField(w, "in", _1, v.In, options);
                        FacetJson.WriteField(w, "default", _2, v.Default, options);
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static KeywordEnum _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Default":
                    _enum.Unit(hasPayload, ref reader, "Default");
                    return new KeywordEnum.Default();
                case "Switch":
                    _enum.Payload(hasPayload, "Switch");
                    return new KeywordEnum.Switch(FacetJson.Read(_0, ref reader, options));
                case "Where":
                {
                    _enum.Payload(hasPayload, "Where");
                    FacetJson.StartObject(ref reader, "KeywordEnum.Where");
                    int f0 = default!;
                    var has0 = false;
                    string f1 = default!;
                    var has1 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "in":
                                f0 = FacetJson.Read(_1, ref reader, options);
                                has0 = true;
                                break;
                            case "default":
                                f1 = FacetJson.Read(_2, ref reader, options);
                                has1 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new KeywordEnum.Where(
                        FacetJson.Required(has0, f0, "in", "KeywordEnum.Where"),
                        FacetJson.Required(has1, f1, "default", "KeywordEnum.Where"));
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

/// A property named `JsonSerde` hides the runtime class the JSON helpers call,
/// so they reach it through its qualified name (#159).
#[test]
fn property_named_like_the_json_runtime_class_qualifies_calls_on_it() {
    #[derive(Facet)]
    struct Wire {
        json_serde: u32,
    }

    let actual = emit!(Wire as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(WireJsonConverter))]
    public partial class Wire : ObservableObject {
        [property: JsonPropertyName("json_serde")]
        [ObservableProperty]
        private uint _jsonSerde;

        public string JsonSerialize()
        {
            return global::Facet.Runtime.Json.JsonSerde.Serialize(this);
        }

        public static Wire JsonDeserialize(string input)
        {
            return global::Facet.Runtime.Json.JsonSerde.Deserialize<Wire>(input);
        }
    }

    public sealed class WireJsonConverter : JsonConverter<Wire> {
        private static readonly JsonConverter<uint> _0 = FacetJson.U32;

        public override bool HandleNull => true;

        public override Wire Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Wire");
            uint f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "json_serde":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Wire
            {
                JsonSerde = FacetJson.Required(has0, f0, "json_serde", "Wire"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Wire value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "json_serde", _0, value.JsonSerde, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

/// A variant named like a type it holds, or like a sibling's property
/// (#174): the type is written through its `global::` name, and the property
/// is declared again with `new`.
#[test]
fn variants_named_like_a_payload_type_or_a_sibling_property() {
    #[derive(Facet)]
    struct Presence {
        x: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Event {
        Presence(Presence),
        Seen { presence: u32, other: Presence },
        Value,
        Wrap(u32),
    }

    let actual = emit!(Event as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(EventJsonConverter))]
    public abstract record Event {
        public sealed record Presence(global::Test.Presence Value) : Event {
            public new global::Test.Presence Value { get; init; } = Value;
        }

        public sealed record Seen(uint Presence, global::Test.Presence Other) : Event {
            public new uint Presence { get; init; } = Presence;
        }

        public sealed record Value() : Event;

        public sealed record Wrap(uint Value) : Event {
            public new uint Value { get; init; } = Value;
        }

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Event JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Event>(input);
        }
    }

    public sealed class EventJsonConverter : JsonConverter<Event> {
        private static readonly JsonConverter<Presence> _0 = FacetJson.Of<Presence>();
        private static readonly JsonConverter<uint> _1 = FacetJson.U32;
        private static readonly JsonConverter<Presence> _2 = FacetJson.Of<Presence>();
        private static readonly JsonConverter<uint> _3 = FacetJson.U32;
        private static readonly JsonEnum _enum = new("Event");

        public override bool HandleNull => true;

        public override Event Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, Event value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Event.Presence v:
                    _enum.WriteVariant(writer, "Presence", w => _0.Write(w, v.Value, options));
                    break;
                case Event.Seen v:
                    _enum.WriteStructVariant(writer, "Seen", w =>
                    {
                        FacetJson.WriteField(w, "presence", _1, v.Presence, options);
                        FacetJson.WriteField(w, "other", _2, v.Other, options);
                    });
                    break;
                case Event.Value:
                    _enum.WriteVariant(writer, "Value");
                    break;
                case Event.Wrap v:
                    _enum.WriteVariant(writer, "Wrap", w => _3.Write(w, v.Value, options));
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static Event _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Presence":
                    _enum.Payload(hasPayload, "Presence");
                    return new Event.Presence(FacetJson.Read(_0, ref reader, options));
                case "Seen":
                {
                    _enum.Payload(hasPayload, "Seen");
                    FacetJson.StartObject(ref reader, "Event.Seen");
                    uint f0 = default!;
                    var has0 = false;
                    Presence f1 = default!;
                    var has1 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "presence":
                                f0 = FacetJson.Read(_1, ref reader, options);
                                has0 = true;
                                break;
                            case "other":
                                f1 = FacetJson.Read(_2, ref reader, options);
                                has1 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new Event.Seen(
                        FacetJson.Required(has0, f0, "presence", "Event.Seen"),
                        FacetJson.Required(has1, f1, "other", "Event.Seen"));
                }
                case "Value":
                    _enum.Unit(hasPayload, ref reader, "Value");
                    return new Event.Value();
                case "Wrap":
                    _enum.Payload(hasPayload, "Wrap");
                    return new Event.Wrap(FacetJson.Read(_3, ref reader, options));
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }

    [JsonConverter(typeof(PresenceJsonConverter))]
    public partial class Presence : ObservableObject {
        [property: JsonPropertyName("x")]
        [ObservableProperty]
        private uint _x;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Presence JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Presence>(input);
        }
    }

    public sealed class PresenceJsonConverter : JsonConverter<Presence> {
        private static readonly JsonConverter<uint> _0 = FacetJson.U32;

        public override bool HandleNull => true;

        public override Presence Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Presence");
            uint f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "x":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Presence
            {
                X = FacetJson.Required(has0, f0, "x", "Presence"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Presence value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "x", _0, value.X, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

/// The wire names are the registry's: renamed fields and variants keep their
/// spelling, and a unit variant is a map key.
#[test]
fn renamed_fields_and_variants() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    struct Renamed {
        snake_case_field: u8,
        #[facet(rename = "$ref")]
        reference: String,
        #[facet(rename = "with-dash")]
        dashed: Option<u8>,
    }

    #[derive(Facet, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(C)]
    #[allow(unused)]
    enum Level {
        Low,
        #[facet(rename = "HIGH")]
        High,
    }

    #[derive(Facet)]
    struct Holder {
        renamed: Renamed,
        by_level: BTreeMap<Level, u8>,
    }

    let actual = emit!(Holder as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(HolderJsonConverter))]
    public partial class Holder : ObservableObject {
        [property: JsonPropertyName("renamed")]
        [ObservableProperty]
        private Renamed _renamed;
        [property: JsonPropertyName("by_level")]
        [ObservableProperty]
        private Dictionary<Level, byte> _byLevel;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Holder JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Holder>(input);
        }
    }

    public sealed class HolderJsonConverter : JsonConverter<Holder> {
        private static readonly JsonConverter<Renamed> _0 = FacetJson.Of<Renamed>();
        private static readonly JsonConverter<Dictionary<Level, byte>> _1 = FacetJson.Map(FacetJson.Of<Level>(), FacetJson.U8);

        public override bool HandleNull => true;

        public override Holder Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Holder");
            Renamed f0 = default!;
            var has0 = false;
            Dictionary<Level, byte> f1 = default!;
            var has1 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "renamed":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "by_level":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Holder
            {
                Renamed = FacetJson.Required(has0, f0, "renamed", "Holder"),
                ByLevel = FacetJson.Required(has1, f1, "by_level", "Holder"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Holder value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "renamed", _0, value.Renamed, options);
            FacetJson.WriteField(writer, "by_level", _1, value.ByLevel, options);
            writer.WriteEndObject();
        }
    }

    [JsonConverter(typeof(LevelJsonConverter))]
    public enum Level {
        Low,
        High
    }

    public sealed class LevelJsonConverter : JsonConverter<Level> {
        private static readonly JsonEnum _enum = new("Level");

        public override bool HandleNull => true;

        public override Level Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, Level value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Level.Low:
                    _enum.WriteVariant(writer, "Low");
                    break;
                case Level.High:
                    _enum.WriteVariant(writer, "HIGH");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        public override Level ReadAsPropertyName(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            var variant = reader.GetString()!;
            switch (variant)
            {
                case "Low":
                    return Level.Low;
                case "HIGH":
                    return Level.High;
                default:
                    throw _enum.Unknown(variant);
            }
        }

        public override void WriteAsPropertyName(Utf8JsonWriter writer, Level value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Level.Low:
                    writer.WritePropertyName("Low");
                    break;
                case Level.High:
                    writer.WritePropertyName("HIGH");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static Level _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Low":
                    _enum.Unit(hasPayload, ref reader, "Low");
                    return Level.Low;
                case "HIGH":
                    _enum.Unit(hasPayload, ref reader, "HIGH");
                    return Level.High;
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }

    [JsonConverter(typeof(RenamedJsonConverter))]
    public partial class Renamed : ObservableObject {
        [property: JsonPropertyName("snakeCaseField")]
        [ObservableProperty]
        private byte _snakeCaseField;
        [property: JsonPropertyName("$ref")]
        [ObservableProperty]
        private string _ref;
        [property: JsonPropertyName("with-dash")]
        [ObservableProperty]
        private byte? _withDash;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Renamed JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Renamed>(input);
        }
    }

    public sealed class RenamedJsonConverter : JsonConverter<Renamed> {
        private static readonly JsonConverter<byte> _0 = FacetJson.U8;
        private static readonly JsonConverter<string> _1 = FacetJson.Str;
        private static readonly JsonConverter<byte?> _2 = FacetJson.Option(FacetJson.U8);

        public override bool HandleNull => true;

        public override Renamed Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Renamed");
            byte f0 = default!;
            var has0 = false;
            string f1 = default!;
            var has1 = false;
            byte? f2 = default;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "snakeCaseField":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "$ref":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "with-dash":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Renamed
            {
                SnakeCaseField = FacetJson.Required(has0, f0, "snakeCaseField", "Renamed"),
                Ref = FacetJson.Required(has1, f1, "$ref", "Renamed"),
                WithDash = f2,
            };
        }

        public override void Write(Utf8JsonWriter writer, Renamed value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "snakeCaseField", _0, value.SnakeCaseField, options);
            FacetJson.WriteField(writer, "$ref", _1, value.Ref, options);
            FacetJson.WriteField(writer, "with-dash", _2, value.WithDash, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

/// `#[facet(tag)]`: the tag joins a struct variant's fields, or those of the
/// struct a newtype variant holds.
#[test]
fn internally_tagged_enum() {
    #[derive(Facet)]
    struct Point {
        x: i32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "type")]
    #[allow(unused)]
    enum Internal {
        Unit,
        Wrapped(Point),
        Struct { x: i32 },
    }

    let actual = emit!(Internal as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(InternalJsonConverter))]
    public abstract record Internal {
        public sealed record Unit() : Internal;

        public sealed record Wrapped(Point Value) : Internal;

        public sealed record Struct(int X) : Internal;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Internal JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Internal>(input);
        }
    }

    public sealed class InternalJsonConverter : JsonConverter<Internal> {
        private static readonly JsonConverter<Point> _0 = FacetJson.Of<Point>();
        private static readonly JsonConverter<int> _1 = FacetJson.I32;
        private static readonly JsonEnum _enum = new("Internal", tag: "type");

        public override bool HandleNull => true;

        public override Internal Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, Internal value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Internal.Unit:
                    _enum.WriteVariant(writer, "Unit");
                    break;
                case Internal.Wrapped v:
                    _enum.WriteVariant(writer, "Wrapped", w => _0.Write(w, v.Value, options));
                    break;
                case Internal.Struct v:
                    _enum.WriteStructVariant(writer, "Struct", w =>
                    {
                        FacetJson.WriteField(w, "x", _1, v.X, options);
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static Internal _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Unit":
                    _enum.Unit(hasPayload, ref reader, "Unit");
                    return new Internal.Unit();
                case "Wrapped":
                    _enum.Payload(hasPayload, "Wrapped");
                    return new Internal.Wrapped(FacetJson.Read(_0, ref reader, options));
                case "Struct":
                {
                    _enum.Payload(hasPayload, "Struct");
                    FacetJson.StartObject(ref reader, "Internal.Struct");
                    int f0 = default!;
                    var has0 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "x":
                                f0 = FacetJson.Read(_1, ref reader, options);
                                has0 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new Internal.Struct(
                        FacetJson.Required(has0, f0, "x", "Internal.Struct"));
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }

    [JsonConverter(typeof(PointJsonConverter))]
    public partial class Point : ObservableObject {
        [property: JsonPropertyName("x")]
        [ObservableProperty]
        private int _x;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Point JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Point>(input);
        }
    }

    public sealed class PointJsonConverter : JsonConverter<Point> {
        private static readonly JsonConverter<int> _0 = FacetJson.I32;

        public override bool HandleNull => true;

        public override Point Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Point");
            int f0 = default!;
            var has0 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "x":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Point
            {
                X = FacetJson.Required(has0, f0, "x", "Point"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Point value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "x", _0, value.X, options);
            writer.WriteEndObject();
        }
    }
    "#);
}

/// An internally tagged enum of unit variants is an object holding the tag,
/// not a bare string.
#[test]
fn internally_tagged_unit_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "kind")]
    #[allow(unused)]
    enum Mode {
        Fast,
        Slow,
    }

    let actual = emit!(Mode as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(ModeJsonConverter))]
    public enum Mode {
        Fast,
        Slow
    }

    public sealed class ModeJsonConverter : JsonConverter<Mode> {
        private static readonly JsonEnum _enum = new("Mode", tag: "kind");

        public override bool HandleNull => true;

        public override Mode Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, Mode value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Mode.Fast:
                    _enum.WriteVariant(writer, "Fast");
                    break;
                case Mode.Slow:
                    _enum.WriteVariant(writer, "Slow");
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static Mode _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Fast":
                    _enum.Unit(hasPayload, ref reader, "Fast");
                    return Mode.Fast;
                case "Slow":
                    _enum.Unit(hasPayload, ref reader, "Slow");
                    return Mode.Slow;
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

/// `#[facet(tag, content)]`: the tag and the payload are two fields.
#[test]
fn adjacently_tagged_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "t", content = "c")]
    #[allow(unused)]
    enum Adjacent {
        Unit,
        NewType(Option<u8>),
        Tuple(u8, String),
        Struct { name: String },
    }

    let actual = emit!(Adjacent as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(AdjacentJsonConverter))]
    public abstract record Adjacent {
        public sealed record Unit() : Adjacent;

        public sealed record NewType(byte? Value) : Adjacent;

        public sealed record Tuple(byte Field0, string Field1) : Adjacent;

        public sealed record Struct(string Name) : Adjacent;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Adjacent JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Adjacent>(input);
        }
    }

    public sealed class AdjacentJsonConverter : JsonConverter<Adjacent> {
        private static readonly JsonConverter<byte?> _0 = FacetJson.Option(FacetJson.U8);
        private static readonly JsonConverter<byte> _1 = FacetJson.U8;
        private static readonly JsonConverter<string> _2 = FacetJson.Str;
        private static readonly JsonConverter<string> _3 = FacetJson.Str;
        private static readonly JsonEnum _enum = new("Adjacent", tag: "t", content: "c");

        public override bool HandleNull => true;

        public override Adjacent Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            return _enum.Read(ref reader, options, _readVariant);
        }

        public override void Write(Utf8JsonWriter writer, Adjacent value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case Adjacent.Unit:
                    _enum.WriteVariant(writer, "Unit");
                    break;
                case Adjacent.NewType v:
                    _enum.WriteVariant(writer, "NewType", w => _0.Write(w, v.Value, options));
                    break;
                case Adjacent.Tuple v:
                    _enum.WriteVariant(writer, "Tuple", w =>
                    {
                        w.WriteStartArray();
                        _1.Write(w, v.Field0, options);
                        _2.Write(w, v.Field1, options);
                        w.WriteEndArray();
                    });
                    break;
                case Adjacent.Struct v:
                    _enum.WriteStructVariant(writer, "Struct", w =>
                    {
                        FacetJson.WriteField(w, "name", _3, v.Name, options);
                    });
                    break;
                default:
                    throw new global::System.ArgumentOutOfRangeException(nameof(value));
            }
        }

        private static Adjacent _readVariant(string variant, bool hasPayload, ref Utf8JsonReader reader, JsonSerializerOptions options)
        {
            switch (variant)
            {
                case "Unit":
                    _enum.Unit(hasPayload, ref reader, "Unit");
                    return new Adjacent.Unit();
                case "NewType":
                    _enum.Payload(hasPayload, "NewType");
                    return new Adjacent.NewType(FacetJson.Read(_0, ref reader, options));
                case "Tuple":
                {
                    _enum.Payload(hasPayload, "Tuple");
                    FacetJson.StartArray(ref reader, "Adjacent.Tuple");
                    var e0 = FacetJson.Element(_1, ref reader, options, "Adjacent.Tuple");
                    var e1 = FacetJson.Element(_2, ref reader, options, "Adjacent.Tuple");
                    FacetJson.EndArray(ref reader, "Adjacent.Tuple");
                    return new Adjacent.Tuple(e0, e1);
                }
                case "Struct":
                {
                    _enum.Payload(hasPayload, "Struct");
                    FacetJson.StartObject(ref reader, "Adjacent.Struct");
                    string f0 = default!;
                    var has0 = false;
                    while (FacetJson.NextField(ref reader, out var key))
                    {
                        switch (key)
                        {
                            case "name":
                                f0 = FacetJson.Read(_3, ref reader, options);
                                has0 = true;
                                break;
                            default:
                                reader.Skip();
                                break;
                        }
                    }
                    return new Adjacent.Struct(
                        FacetJson.Required(has0, f0, "name", "Adjacent.Struct"));
                }
                default:
                    throw _enum.Unknown(variant);
            }
        }
    }
    "#);
}

/// Tuples are arrays, bytes an array of numbers, and 128-bit integers exact
/// numbers, each through the runtime's converter for it.
#[test]
fn struct_with_tuples_bytes_and_big_integers() {
    #[derive(Facet)]
    struct Mixed {
        pair: (u8, String),
        units: Vec<()>,
        #[facet(fg::bytes)]
        bytes: Vec<u8>,
        big: u128,
        by_big: BTreeMap<i128, Option<char>>,
        array: [u16; 3],
    }

    let actual = emit!(Mixed as CSharp with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(MixedJsonConverter))]
    public partial class Mixed : ObservableObject {
        [property: JsonPropertyName("pair")]
        [ObservableProperty]
        private (byte, string) _pair;
        [property: JsonPropertyName("units")]
        [ObservableProperty]
        private ObservableCollection<Unit> _units;
        [property: JsonPropertyName("bytes")]
        [ObservableProperty]
        private byte[] _bytes;
        [property: JsonPropertyName("big")]
        [ObservableProperty]
        private UInt128 _big;
        [property: JsonPropertyName("by_big")]
        [ObservableProperty]
        private Dictionary<Int128, char?> _byBig;
        [property: JsonPropertyName("array")]
        [ObservableProperty]
        private ushort[] _array;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Mixed JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Mixed>(input);
        }
    }

    public sealed class MixedJsonConverter : JsonConverter<Mixed> {
        private static readonly JsonConverter<(byte, string)> _0 = FacetJson.Tuple(FacetJson.U8, FacetJson.Str);
        private static readonly JsonConverter<ObservableCollection<Unit>> _1 = FacetJson.List(FacetJson.Unit);
        private static readonly JsonConverter<byte[]> _2 = FacetJson.Bytes;
        private static readonly JsonConverter<UInt128> _3 = FacetJson.U128;
        private static readonly JsonConverter<Dictionary<Int128, char?>> _4 = FacetJson.Map(FacetJson.I128, FacetJson.Option(FacetJson.Char));
        private static readonly JsonConverter<ushort[]> _5 = FacetJson.Array(FacetJson.U16, 3);

        public override bool HandleNull => true;

        public override Mixed Read(ref Utf8JsonReader reader, global::System.Type typeToConvert, JsonSerializerOptions options)
        {
            FacetJson.StartObject(ref reader, "Mixed");
            (byte, string) f0 = default!;
            var has0 = false;
            ObservableCollection<Unit> f1 = default!;
            var has1 = false;
            byte[] f2 = default!;
            var has2 = false;
            UInt128 f3 = default!;
            var has3 = false;
            Dictionary<Int128, char?> f4 = default!;
            var has4 = false;
            ushort[] f5 = default!;
            var has5 = false;
            while (FacetJson.NextField(ref reader, out var key))
            {
                switch (key)
                {
                    case "pair":
                        f0 = FacetJson.Read(_0, ref reader, options);
                        has0 = true;
                        break;
                    case "units":
                        f1 = FacetJson.Read(_1, ref reader, options);
                        has1 = true;
                        break;
                    case "bytes":
                        f2 = FacetJson.Read(_2, ref reader, options);
                        has2 = true;
                        break;
                    case "big":
                        f3 = FacetJson.Read(_3, ref reader, options);
                        has3 = true;
                        break;
                    case "by_big":
                        f4 = FacetJson.Read(_4, ref reader, options);
                        has4 = true;
                        break;
                    case "array":
                        f5 = FacetJson.Read(_5, ref reader, options);
                        has5 = true;
                        break;
                    default:
                        reader.Skip();
                        break;
                }
            }
            return new Mixed
            {
                Pair = FacetJson.Required(has0, f0, "pair", "Mixed"),
                Units = FacetJson.Required(has1, f1, "units", "Mixed"),
                Bytes = FacetJson.Required(has2, f2, "bytes", "Mixed"),
                Big = FacetJson.Required(has3, f3, "big", "Mixed"),
                ByBig = FacetJson.Required(has4, f4, "by_big", "Mixed"),
                Array = FacetJson.Required(has5, f5, "array", "Mixed"),
            };
        }

        public override void Write(Utf8JsonWriter writer, Mixed value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
            FacetJson.WriteField(writer, "pair", _0, value.Pair, options);
            FacetJson.WriteField(writer, "units", _1, value.Units, options);
            FacetJson.WriteField(writer, "bytes", _2, value.Bytes, options);
            FacetJson.WriteField(writer, "big", _3, value.Big, options);
            FacetJson.WriteField(writer, "by_big", _4, value.ByBig, options);
            FacetJson.WriteField(writer, "array", _5, value.Array, options);
            writer.WriteEndObject();
        }
    }
    "#);
}
