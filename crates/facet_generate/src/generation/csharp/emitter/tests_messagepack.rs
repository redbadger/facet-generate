//! Snapshot tests for the C# emitter — **`MessagePack` encoding**.
//!
//! Mirrors [`tests_json`](super::tests_json) but uses [`MessagePackPlugin`].
//!
//! Key differences from JSON:
//! - Non-unit enums get `[MessagePackConverter(typeof(TypeNameConverter))]` annotation
//!   plus a private nested `TypeNameConverter` class in the type body
//! - Unit-only enums get `[MessagePackConverter(typeof(EnumAsStringConverter<T>))]`
//! - All non-unit types get `MessagePackSerialize`/`MessagePackDeserialize` helpers
//! - Field annotations (`[JsonPropertyName]`) are absent (`MessagePack` uses property names directly)

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::*;
use crate::generation::messagepack::MessagePackPlugin;
use crate::{self as fg, emit};

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public sealed record UnitStruct {
        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<UnitStruct, FacetMessagePackWitness>(this);

        public static UnitStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<UnitStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class NewType : ObservableObject {
        [ObservableProperty]
        private string _value;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<NewType, FacetMessagePackWitness>(this);

        public static NewType MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<NewType, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class TupleStruct : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<TupleStruct, FacetMessagePackWitness>(this);

        public static TupleStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<TupleStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(StructWithFields as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class StructWithFields : ObservableObject {
        [ObservableProperty]
        private Unit _unit;
        [ObservableProperty]
        private bool _bool;
        [ObservableProperty]
        private sbyte _i8;
        [ObservableProperty]
        private short _i16;
        [ObservableProperty]
        private int _i32;
        [ObservableProperty]
        private long _i64;
        [ObservableProperty]
        private Int128 _i128;
        [ObservableProperty]
        private byte _u8;
        [ObservableProperty]
        private ushort _u16;
        [ObservableProperty]
        private uint _u32;
        [ObservableProperty]
        private ulong _u64;
        [ObservableProperty]
        private UInt128 _u128;
        [ObservableProperty]
        private float _f32;
        [ObservableProperty]
        private double _f64;
        [ObservableProperty]
        private char _char;
        [ObservableProperty]
        private string _string;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<StructWithFields, FacetMessagePackWitness>(this);

        public static StructWithFields MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<StructWithFields, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(Outer as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class Inner1 : ObservableObject {
        [ObservableProperty]
        private string _field1;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<Inner1, FacetMessagePackWitness>(this);

        public static Inner1 MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<Inner1, FacetMessagePackWitness>(input);
    }

    public partial class Inner2 : ObservableObject {
        [ObservableProperty]
        private string _value;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<Inner2, FacetMessagePackWitness>(this);

        public static Inner2 MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<Inner2, FacetMessagePackWitness>(input);
    }

    public partial class Inner3 : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<Inner3, FacetMessagePackWitness>(this);

        public static Inner3 MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<Inner3, FacetMessagePackWitness>(input);
    }

    public partial class Outer : ObservableObject {
        [ObservableProperty]
        private Inner1 _one;
        [ObservableProperty]
        private Inner2 _two;
        [ObservableProperty]
        private Inner3 _three;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<Outer, FacetMessagePackWitness>(this);

        public static Outer MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<Outer, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int) _one;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int, ushort) _one;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int, ushort, float) _one;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(EnumWithUnitVariants as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    [MessagePackConverter(typeof(EnumAsStringConverter<EnumWithUnitVariants>))]
    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
    }
    ");
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    [MessagePackConverter(typeof(EnumAsStringConverter<MyEnum>))]
    public enum MyEnum {
        Variant1
    }
    ");
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [MessagePackConverter(typeof(MyEnumConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        private sealed class MyEnumConverter : MessagePackConverter<MyEnum>
        {
            public override MyEnum? Read(ref MessagePackReader reader, SerializationContext context)
            {
                if (reader.NextMessagePackType == MessagePackType.Map)
                {
                    var count = reader.ReadMapHeader();
                    if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for MyEnum, got {count} entries");
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "Variant1" => new Variant1(context.GetConverter<string>().Read(ref reader, context)!),
                        _ => throw new MessagePackSerializationException($"Unknown variant for MyEnum: {tag}"),
                    };
                }
                throw new MessagePackSerializationException($"Unexpected MessagePack type for MyEnum");
            }

            public override void Write(ref MessagePackWriter writer, in MyEnum? value, SerializationContext context)
            {
                switch (value)
                {
                    case null: writer.WriteNil(); break;
                    case Variant1 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant1");
                        context.GetConverter<string>().Write(ref writer, v.Value, context);
                        break;
                }
            }
        }

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyEnum, FacetMessagePackWitness>(this);

        public static MyEnum MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyEnum, FacetMessagePackWitness>(input);
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

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [MessagePackConverter(typeof(MyEnumConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public sealed record Variant2(int Value) : MyEnum;

        private sealed class MyEnumConverter : MessagePackConverter<MyEnum>
        {
            public override MyEnum? Read(ref MessagePackReader reader, SerializationContext context)
            {
                if (reader.NextMessagePackType == MessagePackType.Map)
                {
                    var count = reader.ReadMapHeader();
                    if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for MyEnum, got {count} entries");
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "Variant1" => new Variant1(context.GetConverter<string>().Read(ref reader, context)!),
                        "Variant2" => new Variant2(context.GetConverter<int>().Read(ref reader, context).GetValueOrDefault()),
                        _ => throw new MessagePackSerializationException($"Unknown variant for MyEnum: {tag}"),
                    };
                }
                throw new MessagePackSerializationException($"Unexpected MessagePack type for MyEnum");
            }

            public override void Write(ref MessagePackWriter writer, in MyEnum? value, SerializationContext context)
            {
                switch (value)
                {
                    case null: writer.WriteNil(); break;
                    case Variant1 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant1");
                        context.GetConverter<string>().Write(ref writer, v.Value, context);
                        break;
                    case Variant2 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant2");
                        context.GetConverter<int>().Write(ref writer, v.Value, context);
                        break;
                }
            }
        }

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyEnum, FacetMessagePackWitness>(this);

        public static MyEnum MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyEnum, FacetMessagePackWitness>(input);
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

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [MessagePackConverter(typeof(MyEnumConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Field0, int Field1) : MyEnum;

        public sealed record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

        private sealed class MyEnumConverter : MessagePackConverter<MyEnum>
        {
            public override MyEnum? Read(ref MessagePackReader reader, SerializationContext context)
            {
                if (reader.NextMessagePackType == MessagePackType.Map)
                {
                    var count = reader.ReadMapHeader();
                    if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for MyEnum, got {count} entries");
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "Variant1" => ReadVariant1(ref reader, context),
                        "Variant2" => ReadVariant2(ref reader, context),
                        _ => throw new MessagePackSerializationException($"Unknown variant for MyEnum: {tag}"),
                    };
                }
                throw new MessagePackSerializationException($"Unexpected MessagePack type for MyEnum");
            }

            private static Variant1 ReadVariant1(ref MessagePackReader reader, SerializationContext context)
            {
                var len = reader.ReadArrayHeader();
                _ = len;
                var field0 = context.GetConverter<string>().Read(ref reader, context)!;
                var field1 = context.GetConverter<int>().Read(ref reader, context).GetValueOrDefault();
                return new Variant1(field0, field1);
            }

            private static Variant2 ReadVariant2(ref MessagePackReader reader, SerializationContext context)
            {
                var len = reader.ReadArrayHeader();
                _ = len;
                var field0 = context.GetConverter<bool>().Read(ref reader, context).GetValueOrDefault();
                var field1 = context.GetConverter<double>().Read(ref reader, context).GetValueOrDefault();
                var field2 = context.GetConverter<byte>().Read(ref reader, context).GetValueOrDefault();
                return new Variant2(field0, field1, field2);
            }

            public override void Write(ref MessagePackWriter writer, in MyEnum? value, SerializationContext context)
            {
                switch (value)
                {
                    case null: writer.WriteNil(); break;
                    case Variant1 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant1");
                        writer.WriteArrayHeader(2);
                        context.GetConverter<string>().Write(ref writer, v.Field0, context);
                        context.GetConverter<int>().Write(ref writer, v.Field1, context);
                        break;
                    case Variant2 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant2");
                        writer.WriteArrayHeader(3);
                        context.GetConverter<bool>().Write(ref writer, v.Field0, context);
                        context.GetConverter<double>().Write(ref writer, v.Field1, context);
                        context.GetConverter<byte>().Write(ref writer, v.Field2, context);
                        break;
                }
            }
        }

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyEnum, FacetMessagePackWitness>(this);

        public static MyEnum MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyEnum, FacetMessagePackWitness>(input);
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

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [MessagePackConverter(typeof(MyEnumConverter))]
    public abstract record MyEnum {
        public sealed record Variant1(string Field1, int Field2) : MyEnum;

        private sealed class MyEnumConverter : MessagePackConverter<MyEnum>
        {
            public override MyEnum? Read(ref MessagePackReader reader, SerializationContext context)
            {
                if (reader.NextMessagePackType == MessagePackType.Map)
                {
                    var count = reader.ReadMapHeader();
                    if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for MyEnum, got {count} entries");
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "Variant1" => ReadVariant1(ref reader, context),
                        _ => throw new MessagePackSerializationException($"Unknown variant for MyEnum: {tag}"),
                    };
                }
                throw new MessagePackSerializationException($"Unexpected MessagePack type for MyEnum");
            }

            private static Variant1 ReadVariant1(ref MessagePackReader reader, SerializationContext context)
            {
                string? field1 = null;
                int field2 = default;
                var mapLen = reader.ReadMapHeader();
                for (var i = 0; i < mapLen; i++)
                {
                    var key = reader.ReadString()!;
                    switch (key)
                    {
                        case "field1": field1 = context.GetConverter<string>().Read(ref reader, context)!; break;
                        case "field2": field2 = context.GetConverter<int>().Read(ref reader, context).GetValueOrDefault(); break;
                        default: reader.Skip(); break;
                    }
                }
                return new Variant1(field1!, field2);
            }

            public override void Write(ref MessagePackWriter writer, in MyEnum? value, SerializationContext context)
            {
                switch (value)
                {
                    case null: writer.WriteNil(); break;
                    case Variant1 v:
                        writer.WriteMapHeader(1);
                        writer.Write("Variant1");
                        writer.WriteMapHeader(2);
                        writer.Write("field1");
                        context.GetConverter<string>().Write(ref writer, v.Field1, context);
                        writer.Write("field2");
                        context.GetConverter<int>().Write(ref writer, v.Field2, context);
                        break;
                }
            }
        }

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyEnum, FacetMessagePackWitness>(this);

        public static MyEnum MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyEnum, FacetMessagePackWitness>(input);
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

    let actual = emit!(MyEnum as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [MessagePackConverter(typeof(MyEnumConverter))]
    public abstract record MyEnum {
        public sealed record Unit() : MyEnum;

        public sealed record NewType(string Value) : MyEnum;

        public sealed record Tuple(string Field0, int Field1) : MyEnum;

        public sealed record Struct(bool Field) : MyEnum;

        private sealed class MyEnumConverter : MessagePackConverter<MyEnum>
        {
            public override MyEnum? Read(ref MessagePackReader reader, SerializationContext context)
            {
                if (reader.NextMessagePackType == MessagePackType.String)
                {
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "Unit" => new Unit(),
                        _ => throw new MessagePackSerializationException($"Unknown unit variant for MyEnum: {tag}"),
                    };
                }
                if (reader.NextMessagePackType == MessagePackType.Map)
                {
                    var count = reader.ReadMapHeader();
                    if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for MyEnum, got {count} entries");
                    var tag = reader.ReadString()!;
                    return tag switch
                    {
                        "NewType" => new NewType(context.GetConverter<string>().Read(ref reader, context)!),
                        "Tuple" => ReadTuple(ref reader, context),
                        "Struct" => ReadStruct(ref reader, context),
                        _ => throw new MessagePackSerializationException($"Unknown variant for MyEnum: {tag}"),
                    };
                }
                throw new MessagePackSerializationException($"Unexpected MessagePack type for MyEnum");
            }

            private static Tuple ReadTuple(ref MessagePackReader reader, SerializationContext context)
            {
                var len = reader.ReadArrayHeader();
                _ = len;
                var field0 = context.GetConverter<string>().Read(ref reader, context)!;
                var field1 = context.GetConverter<int>().Read(ref reader, context).GetValueOrDefault();
                return new Tuple(field0, field1);
            }

            private static Struct ReadStruct(ref MessagePackReader reader, SerializationContext context)
            {
                bool field = default;
                var mapLen = reader.ReadMapHeader();
                for (var i = 0; i < mapLen; i++)
                {
                    var key = reader.ReadString()!;
                    switch (key)
                    {
                        case "field": field = context.GetConverter<bool>().Read(ref reader, context).GetValueOrDefault(); break;
                        default: reader.Skip(); break;
                    }
                }
                return new Struct(field);
            }

            public override void Write(ref MessagePackWriter writer, in MyEnum? value, SerializationContext context)
            {
                switch (value)
                {
                    case null: writer.WriteNil(); break;
                    case Unit: writer.Write("Unit"); break;
                    case NewType v:
                        writer.WriteMapHeader(1);
                        writer.Write("NewType");
                        context.GetConverter<string>().Write(ref writer, v.Value, context);
                        break;
                    case Tuple v:
                        writer.WriteMapHeader(1);
                        writer.Write("Tuple");
                        writer.WriteArrayHeader(2);
                        context.GetConverter<string>().Write(ref writer, v.Field0, context);
                        context.GetConverter<int>().Write(ref writer, v.Field1, context);
                        break;
                    case Struct v:
                        writer.WriteMapHeader(1);
                        writer.Write("Struct");
                        writer.WriteMapHeader(1);
                        writer.Write("field");
                        context.GetConverter<bool>().Write(ref writer, v.Field, context);
                        break;
                }
            }
        }

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyEnum, FacetMessagePackWitness>(this);

        public static MyEnum MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyEnum, FacetMessagePackWitness>(input);
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<string> _items;
        [ObservableProperty]
        private ObservableCollection<int> _numbers;
        [ObservableProperty]
        private ObservableCollection<ObservableCollection<string>> _nestedItems;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string? _optionalString;
        [ObservableProperty]
        private int? _optionalNumber;
        [ObservableProperty]
        private bool? _optionalBool;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<string>? _optionalList;
        [ObservableProperty]
        private ObservableCollection<int?> _listOfOptionals;
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [ObservableProperty]
        private Dictionary<string, int>? _optionalMap;
        [ObservableProperty]
        private ObservableCollection<Dictionary<string, ObservableCollection<bool>>?> _complex;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private int[] _fixedArray;
        [ObservableProperty]
        private byte[] _byteArray;
        [ObservableProperty]
        private string[] _stringArray;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _boxedString;
        [ObservableProperty]
        private int _boxedInt;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _rcString;
        [ObservableProperty]
        private int _rcInt;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _arcString;
        [ObservableProperty]
        private int _arcInt;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<HashSet<string>> _vecOfSets;
        [ObservableProperty]
        private Dictionary<string, int>? _optionalBtree;
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [ObservableProperty]
        private string? _arcOption;
        [ObservableProperty]
        private int[] _arrayOfBoxes;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
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

    let actual = emit!(MyStruct as CSharp with MessagePackPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;
        [ObservableProperty]
        private ObservableCollection<byte>? _optionalBytes;

        public byte[] MessagePackSerialize()
            => MessagePackSerde.Serialize<MyStruct, FacetMessagePackWitness>(this);

        public static MyStruct MessagePackDeserialize(byte[] input)
            => MessagePackSerde.Deserialize<MyStruct, FacetMessagePackWitness>(input);
    }
    ");
}
