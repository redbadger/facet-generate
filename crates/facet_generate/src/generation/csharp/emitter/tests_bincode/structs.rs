//! Bincode struct tests — unit/newtype/tuple/regular structs with
//! `ISerializer`/`IDeserializer` methods.

#![allow(clippy::too_many_lines)]

use facet::Facet;

use super::super::*;
use crate::{self as fg, emit, generation::bincode::BincodePlugin};

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    public sealed record UnitStruct : IFacetSerializable, IFacetDeserializable<UnitStruct> {
        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.DecreaseContainerDepth();
        }

        public static UnitStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            deserializer.DecreaseContainerDepth();
            return new UnitStruct();
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static UnitStruct BincodeDeserialize(byte[] input)
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
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class NewType : ObservableObject, IFacetSerializable, IFacetDeserializable<NewType> {
        [ObservableProperty]
        private string _value;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(Value);
            serializer.DecreaseContainerDepth();
        }

        public static NewType Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var value = deserializer.DeserializeStr();
            deserializer.DecreaseContainerDepth();
            return new NewType {
                Value = value,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static NewType BincodeDeserialize(byte[] input)
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
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class TupleStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<TupleStruct> {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(Field0);
            serializer.SerializeI32(Field1);
            serializer.DecreaseContainerDepth();
        }

        public static TupleStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var field0 = deserializer.DeserializeStr();
            var field1 = deserializer.DeserializeI32();
            deserializer.DecreaseContainerDepth();
            return new TupleStruct {
                Field0 = field0,
                Field1 = field1,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static TupleStruct BincodeDeserialize(byte[] input)
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

    let actual = emit!(StructWithFields as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class StructWithFields : ObservableObject, IFacetSerializable, IFacetDeserializable<StructWithFields> {
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

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeUnit(Unit);
            serializer.SerializeBool(Bool);
            serializer.SerializeI8(I8);
            serializer.SerializeI16(I16);
            serializer.SerializeI32(I32);
            serializer.SerializeI64(I64);
            serializer.SerializeI128(I128);
            serializer.SerializeU8(U8);
            serializer.SerializeU16(U16);
            serializer.SerializeU32(U32);
            serializer.SerializeU64(U64);
            serializer.SerializeU128(U128);
            serializer.SerializeF32(F32);
            serializer.SerializeF64(F64);
            serializer.SerializeChar(Char);
            serializer.SerializeStr(String);
            serializer.DecreaseContainerDepth();
        }

        public static StructWithFields Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var unit = deserializer.DeserializeUnit();
            var bool = deserializer.DeserializeBool();
            var i8 = deserializer.DeserializeI8();
            var i16 = deserializer.DeserializeI16();
            var i32 = deserializer.DeserializeI32();
            var i64 = deserializer.DeserializeI64();
            var i128 = deserializer.DeserializeI128();
            var u8 = deserializer.DeserializeU8();
            var u16 = deserializer.DeserializeU16();
            var u32 = deserializer.DeserializeU32();
            var u64 = deserializer.DeserializeU64();
            var u128 = deserializer.DeserializeU128();
            var f32 = deserializer.DeserializeF32();
            var f64 = deserializer.DeserializeF64();
            var char = deserializer.DeserializeChar();
            var string = deserializer.DeserializeStr();
            deserializer.DecreaseContainerDepth();
            return new StructWithFields {
                Unit = unit,
                Bool = bool,
                I8 = i8,
                I16 = i16,
                I32 = i32,
                I64 = i64,
                I128 = i128,
                U8 = u8,
                U16 = u16,
                U32 = u32,
                U64 = u64,
                U128 = u128,
                F32 = f32,
                F64 = f64,
                Char = char,
                String = string,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static StructWithFields BincodeDeserialize(byte[] input)
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

    let actual = emit!(Outer as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class Inner1 : ObservableObject, IFacetSerializable, IFacetDeserializable<Inner1> {
        [ObservableProperty]
        private string _field1;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(Field1);
            serializer.DecreaseContainerDepth();
        }

        public static Inner1 Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var field1 = deserializer.DeserializeStr();
            deserializer.DecreaseContainerDepth();
            return new Inner1 {
                Field1 = field1,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Inner1 BincodeDeserialize(byte[] input)
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

    public partial class Inner2 : ObservableObject, IFacetSerializable, IFacetDeserializable<Inner2> {
        [ObservableProperty]
        private string _value;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(Value);
            serializer.DecreaseContainerDepth();
        }

        public static Inner2 Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var value = deserializer.DeserializeStr();
            deserializer.DecreaseContainerDepth();
            return new Inner2 {
                Value = value,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Inner2 BincodeDeserialize(byte[] input)
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

    public partial class Inner3 : ObservableObject, IFacetSerializable, IFacetDeserializable<Inner3> {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(Field0);
            serializer.SerializeI32(Field1);
            serializer.DecreaseContainerDepth();
        }

        public static Inner3 Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var field0 = deserializer.DeserializeStr();
            var field1 = deserializer.DeserializeI32();
            deserializer.DecreaseContainerDepth();
            return new Inner3 {
                Field0 = field0,
                Field1 = field1,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Inner3 BincodeDeserialize(byte[] input)
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

    public partial class Outer : ObservableObject, IFacetSerializable, IFacetDeserializable<Outer> {
        [ObservableProperty]
        private Inner1 _one;
        [ObservableProperty]
        private Inner2 _two;
        [ObservableProperty]
        private Inner3 _three;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            One.Serialize(serializer);
            Two.Serialize(serializer);
            Three.Serialize(serializer);
            serializer.DecreaseContainerDepth();
        }

        public static Outer Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var one = Inner1.Deserialize(deserializer);
            var two = Inner2.Deserialize(deserializer);
            var three = Inner3.Deserialize(deserializer);
            deserializer.DecreaseContainerDepth();
            return new Outer {
                One = one,
                Two = two,
                Three = three,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Outer BincodeDeserialize(byte[] input)
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
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private (string, int) _one;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(One.Item1);
            serializer.SerializeI32(One.Item2);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var one_item1 = deserializer.DeserializeStr();
            var one_item2 = deserializer.DeserializeI32();
            var one = (one_item1, one_item2);
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                One = one,
            };
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
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private (string, int, ushort) _one;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(One.Item1);
            serializer.SerializeI32(One.Item2);
            serializer.SerializeU16(One.Item3);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var one_item1 = deserializer.DeserializeStr();
            var one_item2 = deserializer.DeserializeI32();
            var one_item3 = deserializer.DeserializeU16();
            var one = (one_item1, one_item2, one_item3);
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                One = one,
            };
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
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private (string, int, ushort, float) _one;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(One.Item1);
            serializer.SerializeI32(One.Item2);
            serializer.SerializeU16(One.Item3);
            serializer.SerializeF32(One.Item4);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var one_item1 = deserializer.DeserializeStr();
            var one_item2 = deserializer.DeserializeI32();
            var one_item3 = deserializer.DeserializeU16();
            var one_item4 = deserializer.DeserializeF32();
            var one = (one_item1, one_item2, one_item3, one_item4);
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                One = one,
            };
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
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(fg::bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeBytes(Data);
            serializer.SerializeStr(Name);
            serializer.SerializeBytes(Header);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var data = deserializer.DeserializeBytes();
            var name = deserializer.DeserializeStr();
            var header = deserializer.DeserializeBytes();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                Data = data,
                Name = name,
                Header = header,
            };
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

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;
        [ObservableProperty]
        private ObservableCollection<byte>? _optionalBytes;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeBytes(Data);
            serializer.SerializeStr(Name);
            serializer.SerializeBytes(Header);
            FacetHelpers.SerializeOptionRef(OptionalBytes, serializer, (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeU8(item)));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var data = deserializer.DeserializeBytes();
            var name = deserializer.DeserializeStr();
            var header = deserializer.DeserializeBytes();
            var optionalBytes = FacetHelpers.DeserializeOptionRef(deserializer, d => FacetHelpers.DeserializeList(d, d => d.DeserializeU8()));
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                Data = data,
                Name = name,
                Header = header,
                OptionalBytes = optionalBytes,
            };
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
