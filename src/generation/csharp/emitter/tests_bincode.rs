#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    public partial class UnitStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<UnitStruct> {
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

    let actual = emit!(NewType as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(TupleStruct as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(StructWithFields as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(Outer as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
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

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
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
fn enum_with_unit_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3,
    }

    let actual = emit!(EnumWithUnitVariants as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
    }

    public static class EnumWithUnitVariantsBincode {
        public static void Serialize(EnumWithUnitVariants value, ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex((uint)value);
            serializer.DecreaseContainerDepth();
        }

        public static EnumWithUnitVariants Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var index = deserializer.DeserializeVariantIndex();
            deserializer.DecreaseContainerDepth();
            return index switch
            {
                0 => EnumWithUnitVariants.Variant1,
                1 => EnumWithUnitVariants.Variant2,
                2 => EnumWithUnitVariants.Variant3,
                _ => throw new DeserializationError("Unknown variant index for EnumWithUnitVariants: " + index),
            }
            ;
        }

        public static byte[] BincodeSerialize(EnumWithUnitVariants value)
        {
            var serializer = new BincodeSerializer();
            Serialize(value, serializer);
            return serializer.GetBytes();
        }

        public static EnumWithUnitVariants BincodeDeserialize(byte[] input)
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
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public enum MyEnum {
        Variant1
    }

    public static class MyEnumBincode {
        public static void Serialize(MyEnum value, ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex((uint)value);
            serializer.DecreaseContainerDepth();
        }

        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var index = deserializer.DeserializeVariantIndex();
            deserializer.DecreaseContainerDepth();
            return index switch
            {
                0 => MyEnum.Variant1,
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public static byte[] BincodeSerialize(MyEnum value)
        {
            var serializer = new BincodeSerializer();
            Serialize(value, serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record MyEnum, IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed record Variant1(string Value) : MyEnum;

        public abstract void Serialize(ISerializer serializer);

        private static MyEnum DeserializeVariant1(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeStr();
            return new Variant1(value);
        }

        public sealed partial record Variant1
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeStr(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeVariant1(deserializer),
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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
fn enum_with_newtype_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
        Variant2(i32),
    }

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record MyEnum, IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed record Variant1(string Value) : MyEnum;

        public sealed record Variant2(int Value) : MyEnum;

        public abstract void Serialize(ISerializer serializer);

        private static MyEnum DeserializeVariant1(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeStr();
            return new Variant1(value);
        }

        public sealed partial record Variant1
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeStr(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        private static MyEnum DeserializeVariant2(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeI32();
            return new Variant2(value);
        }

        public sealed partial record Variant2
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeI32(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeVariant1(deserializer),
                1 => DeserializeVariant2(deserializer),
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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
fn enum_with_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String, i32),
        Variant2(bool, f64, u8),
    }

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record MyEnum, IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed record Variant1(string Field0, int Field1) : MyEnum;

        public sealed record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

        public abstract void Serialize(ISerializer serializer);

        private static MyEnum DeserializeVariant1(IDeserializer deserializer)
        {
            var field0 = deserializer.DeserializeStr();
            var field1 = deserializer.DeserializeI32();
            return new Variant1(field0, field1);
        }

        public sealed partial record Variant1
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeStr(Field0);
                serializer.SerializeI32(Field1);
                serializer.DecreaseContainerDepth();
            }

        }
        private static MyEnum DeserializeVariant2(IDeserializer deserializer)
        {
            var field0 = deserializer.DeserializeBool();
            var field1 = deserializer.DeserializeF64();
            var field2 = deserializer.DeserializeU8();
            return new Variant2(field0, field1, field2);
        }

        public sealed partial record Variant2
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeBool(Field0);
                serializer.SerializeF64(Field1);
                serializer.SerializeU8(Field2);
                serializer.DecreaseContainerDepth();
            }

        }
        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeVariant1(deserializer),
                1 => DeserializeVariant2(deserializer),
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record MyEnum, IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed record Variant1(string Field1, int Field2) : MyEnum;

        public abstract void Serialize(ISerializer serializer);

        private static MyEnum DeserializeVariant1(IDeserializer deserializer)
        {
            var field1 = deserializer.DeserializeStr();
            var field2 = deserializer.DeserializeI32();
            return new Variant1(field1, field2);
        }

        public sealed partial record Variant1
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeStr(Field1);
                serializer.SerializeI32(Field2);
                serializer.DecreaseContainerDepth();
            }

        }
        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeVariant1(deserializer),
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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

    let actual = emit!(MyEnum as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record MyEnum, IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed record Unit() : MyEnum;

        public sealed record NewType(string Value) : MyEnum;

        public sealed record Tuple(string Field0, int Field1) : MyEnum;

        public sealed record Struct(bool Field) : MyEnum;

        public abstract void Serialize(ISerializer serializer);

        private static MyEnum DeserializeUnit(IDeserializer deserializer)
        {
            return new Unit();
        }

        public sealed partial record Unit
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.DecreaseContainerDepth();
            }

        }
        private static MyEnum DeserializeNewType(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeStr();
            return new NewType(value);
        }

        public sealed partial record NewType
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeStr(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        private static MyEnum DeserializeTuple(IDeserializer deserializer)
        {
            var field0 = deserializer.DeserializeStr();
            var field1 = deserializer.DeserializeI32();
            return new Tuple(field0, field1);
        }

        public sealed partial record Tuple
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(2);
                serializer.SerializeStr(Field0);
                serializer.SerializeI32(Field1);
                serializer.DecreaseContainerDepth();
            }

        }
        private static MyEnum DeserializeStruct(IDeserializer deserializer)
        {
            var field = deserializer.DeserializeBool();
            return new Struct(field);
        }

        public sealed partial record Struct
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(3);
                serializer.SerializeBool(Field);
                serializer.DecreaseContainerDepth();
            }

        }
        public static MyEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeUnit(deserializer),
                1 => DeserializeNewType(deserializer),
                2 => DeserializeTuple(deserializer),
                3 => DeserializeStruct(deserializer),
                _ => throw new DeserializationError("Unknown variant index for MyEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static MyEnum BincodeDeserialize(byte[] input)
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
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private ObservableCollection<string> _items;
        [ObservableProperty]
        private ObservableCollection<int> _numbers;
        [ObservableProperty]
        private ObservableCollection<ObservableCollection<string>> _nestedItems;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)Items.Count);
            foreach (var item in Items)
            {
                serializer.SerializeStr(item);
            }
            serializer.SerializeLen((ulong)Numbers.Count);
            foreach (var item in Numbers)
            {
                serializer.SerializeI32(item);
            }
            serializer.SerializeLen((ulong)NestedItems.Count);
            foreach (var item in NestedItems)
            {
                serializer.SerializeLen((ulong)item.Count);
                foreach (var item in item)
                {
                    serializer.SerializeStr(item);
                }
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var items_len = deserializer.DeserializeLen();
            var items = new ObservableCollection<string>();
            for (ulong i = 0; i < items_len; i++)
            {
                var item = deserializer.DeserializeStr();
                items.Add(item);
            }
            var numbers_len = deserializer.DeserializeLen();
            var numbers = new ObservableCollection<int>();
            for (ulong i = 0; i < numbers_len; i++)
            {
                var item = deserializer.DeserializeI32();
                numbers.Add(item);
            }
            var nestedItems_len = deserializer.DeserializeLen();
            var nestedItems = new ObservableCollection<ObservableCollection<string>>();
            for (ulong i = 0; i < nestedItems_len; i++)
            {
                var item_len = deserializer.DeserializeLen();
                var item = new ObservableCollection<string>();
                for (ulong i = 0; i < item_len; i++)
                {
                    var item = deserializer.DeserializeStr();
                    item.Add(item);
                }
                nestedItems.Add(item);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                Items = items,
                Numbers = numbers,
                NestedItems = nestedItems,
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
fn struct_with_option_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        optional_string: Option<String>,
        optional_number: Option<i32>,
        optional_bool: Option<bool>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private Option<string> _optionalString;
        [ObservableProperty]
        private Option<int> _optionalNumber;
        [ObservableProperty]
        private Option<bool> _optionalBool;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            if (OptionalString.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeStr(OptionalString.Value);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            if (OptionalNumber.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeI32(OptionalNumber.Value);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            if (OptionalBool.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeBool(OptionalBool.Value);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            Option<string> optionalString;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalString_value = deserializer.DeserializeStr();
                optionalString = Option<string>.Some(optionalString_value);
            }
            else
            {
                optionalString = Option<string>.None();
            }
            Option<int> optionalNumber;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalNumber_value = deserializer.DeserializeI32();
                optionalNumber = Option<int>.Some(optionalNumber_value);
            }
            else
            {
                optionalNumber = Option<int>.None();
            }
            Option<bool> optionalBool;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalBool_value = deserializer.DeserializeBool();
                optionalBool = Option<bool>.Some(optionalBool_value);
            }
            else
            {
                optionalBool = Option<bool>.None();
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                OptionalString = optionalString,
                OptionalNumber = optionalNumber,
                OptionalBool = optionalBool,
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
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)StringToInt.Count);
            foreach (var entry in StringToInt)
            {
                serializer.SerializeStr(entry.Key);
                serializer.SerializeI32(entry.Value);
            }
            serializer.SerializeLen((ulong)IntToBool.Count);
            foreach (var entry in IntToBool)
            {
                serializer.SerializeI32(entry.Key);
                serializer.SerializeBool(entry.Value);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringToInt_len = deserializer.DeserializeLen();
            var stringToInt = new Dictionary<string, int>();
            for (ulong i = 0; i < stringToInt_len; i++)
            {
                var key = deserializer.DeserializeStr();
                var value = deserializer.DeserializeI32();
                stringToInt.Add(key, value);
            }
            var intToBool_len = deserializer.DeserializeLen();
            var intToBool = new Dictionary<int, bool>();
            for (ulong i = 0; i < intToBool_len; i++)
            {
                var key = deserializer.DeserializeI32();
                var value = deserializer.DeserializeBool();
                intToBool.Add(key, value);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                StringToInt = stringToInt,
                IntToBool = intToBool,
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
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private Option<ObservableCollection<string>> _optionalList;
        [ObservableProperty]
        private ObservableCollection<Option<int>> _listOfOptionals;
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [ObservableProperty]
        private Option<Dictionary<string, int>> _optionalMap;
        [ObservableProperty]
        private ObservableCollection<Option<Dictionary<string, ObservableCollection<bool>>>> _complex;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            if (OptionalList.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalList.Value.Count);
                foreach (var item in OptionalList.Value)
                {
                    serializer.SerializeStr(item);
                }
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.SerializeLen((ulong)ListOfOptionals.Count);
            foreach (var item in ListOfOptionals)
            {
                if (item.HasValue)
                {
                    serializer.SerializeOptionTag(true);
                    serializer.SerializeI32(item.Value);
                }
                else
                {
                    serializer.SerializeOptionTag(false);
                }
            }
            serializer.SerializeLen((ulong)MapToList.Count);
            foreach (var entry in MapToList)
            {
                serializer.SerializeStr(entry.Key);
                serializer.SerializeLen((ulong)entry.Value.Count);
                foreach (var item in entry.Value)
                {
                    serializer.SerializeBool(item);
                }
            }
            if (OptionalMap.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalMap.Value.Count);
                foreach (var entry in OptionalMap.Value)
                {
                    serializer.SerializeStr(entry.Key);
                    serializer.SerializeI32(entry.Value);
                }
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.SerializeLen((ulong)Complex.Count);
            foreach (var item in Complex)
            {
                if (item.HasValue)
                {
                    serializer.SerializeOptionTag(true);
                    serializer.SerializeLen((ulong)item.Value.Count);
                    foreach (var entry in item.Value)
                    {
                        serializer.SerializeStr(entry.Key);
                        serializer.SerializeLen((ulong)entry.Value.Count);
                        foreach (var item in entry.Value)
                        {
                            serializer.SerializeBool(item);
                        }
                    }
                }
                else
                {
                    serializer.SerializeOptionTag(false);
                }
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            Option<ObservableCollection<string>> optionalList;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalList_value_len = deserializer.DeserializeLen();
                var optionalList_value = new ObservableCollection<string>();
                for (ulong i = 0; i < optionalList_value_len; i++)
                {
                    var item = deserializer.DeserializeStr();
                    optionalList_value.Add(item);
                }
                optionalList = Option<ObservableCollection<string>>.Some(optionalList_value);
            }
            else
            {
                optionalList = Option<ObservableCollection<string>>.None();
            }
            var listOfOptionals_len = deserializer.DeserializeLen();
            var listOfOptionals = new ObservableCollection<Option<int>>();
            for (ulong i = 0; i < listOfOptionals_len; i++)
            {
                Option<int> item;
                if (deserializer.DeserializeOptionTag())
                {
                    var item_value = deserializer.DeserializeI32();
                    item = Option<int>.Some(item_value);
                }
                else
                {
                    item = Option<int>.None();
                }
                listOfOptionals.Add(item);
            }
            var mapToList_len = deserializer.DeserializeLen();
            var mapToList = new Dictionary<string, ObservableCollection<bool>>();
            for (ulong i = 0; i < mapToList_len; i++)
            {
                var key = deserializer.DeserializeStr();
                var value_len = deserializer.DeserializeLen();
                var value = new ObservableCollection<bool>();
                for (ulong i = 0; i < value_len; i++)
                {
                    var item = deserializer.DeserializeBool();
                    value.Add(item);
                }
                mapToList.Add(key, value);
            }
            Option<Dictionary<string, int>> optionalMap;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalMap_value_len = deserializer.DeserializeLen();
                var optionalMap_value = new Dictionary<string, int>();
                for (ulong i = 0; i < optionalMap_value_len; i++)
                {
                    var key = deserializer.DeserializeStr();
                    var value = deserializer.DeserializeI32();
                    optionalMap_value.Add(key, value);
                }
                optionalMap = Option<Dictionary<string, int>>.Some(optionalMap_value);
            }
            else
            {
                optionalMap = Option<Dictionary<string, int>>.None();
            }
            var complex_len = deserializer.DeserializeLen();
            var complex = new ObservableCollection<Option<Dictionary<string, ObservableCollection<bool>>>>();
            for (ulong i = 0; i < complex_len; i++)
            {
                Option<Dictionary<string, ObservableCollection<bool>>> item;
                if (deserializer.DeserializeOptionTag())
                {
                    var item_value_len = deserializer.DeserializeLen();
                    var item_value = new Dictionary<string, ObservableCollection<bool>>();
                    for (ulong i = 0; i < item_value_len; i++)
                    {
                        var key = deserializer.DeserializeStr();
                        var value_len = deserializer.DeserializeLen();
                        var value = new ObservableCollection<bool>();
                        for (ulong i = 0; i < value_len; i++)
                        {
                            var item = deserializer.DeserializeBool();
                            value.Add(item);
                        }
                        item_value.Add(key, value);
                    }
                    item = Option<Dictionary<string, ObservableCollection<bool>>>.Some(item_value);
                }
                else
                {
                    item = Option<Dictionary<string, ObservableCollection<bool>>>.None();
                }
                complex.Add(item);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                OptionalList = optionalList,
                ListOfOptionals = listOfOptionals,
                MapToList = mapToList,
                OptionalMap = optionalMap,
                Complex = complex,
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
fn struct_with_array_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        fixed_array: [i32; 5],
        byte_array: [u8; 32],
        string_array: [String; 3],
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private int[] _fixedArray;
        [ObservableProperty]
        private byte[] _byteArray;
        [ObservableProperty]
        private string[] _stringArray;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)FixedArray.Length);
            foreach (var item in FixedArray)
            {
                serializer.SerializeI32(item);
            }
            serializer.SerializeLen((ulong)ByteArray.Length);
            foreach (var item in ByteArray)
            {
                serializer.SerializeU8(item);
            }
            serializer.SerializeLen((ulong)StringArray.Length);
            foreach (var item in StringArray)
            {
                serializer.SerializeStr(item);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var fixedArray_len = deserializer.DeserializeLen();
            var fixedArray_list = new List<int>();
            for (ulong i = 0; i < fixedArray_len; i++)
            {
                var item = deserializer.DeserializeI32();
                fixedArray_list.Add(item);
            }
            var fixedArray = fixedArray_list.ToArray();
            var byteArray_len = deserializer.DeserializeLen();
            var byteArray_list = new List<byte>();
            for (ulong i = 0; i < byteArray_len; i++)
            {
                var item = deserializer.DeserializeU8();
                byteArray_list.Add(item);
            }
            var byteArray = byteArray_list.ToArray();
            var stringArray_len = deserializer.DeserializeLen();
            var stringArray_list = new List<string>();
            for (ulong i = 0; i < stringArray_len; i++)
            {
                var item = deserializer.DeserializeStr();
                stringArray_list.Add(item);
            }
            var stringArray = stringArray_list.ToArray();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                FixedArray = fixedArray,
                ByteArray = byteArray,
                StringArray = stringArray,
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
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)StringToInt.Count);
            foreach (var entry in StringToInt)
            {
                serializer.SerializeStr(entry.Key);
                serializer.SerializeI32(entry.Value);
            }
            serializer.SerializeLen((ulong)IntToBool.Count);
            foreach (var entry in IntToBool)
            {
                serializer.SerializeI32(entry.Key);
                serializer.SerializeBool(entry.Value);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringToInt_len = deserializer.DeserializeLen();
            var stringToInt = new Dictionary<string, int>();
            for (ulong i = 0; i < stringToInt_len; i++)
            {
                var key = deserializer.DeserializeStr();
                var value = deserializer.DeserializeI32();
                stringToInt.Add(key, value);
            }
            var intToBool_len = deserializer.DeserializeLen();
            var intToBool = new Dictionary<int, bool>();
            for (ulong i = 0; i < intToBool_len; i++)
            {
                var key = deserializer.DeserializeI32();
                var value = deserializer.DeserializeBool();
                intToBool.Add(key, value);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                StringToInt = stringToInt,
                IntToBool = intToBool,
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
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)StringSet.Count);
            foreach (var item in StringSet)
            {
                serializer.SerializeStr(item);
            }
            serializer.SerializeLen((ulong)IntSet.Count);
            foreach (var item in IntSet)
            {
                serializer.SerializeI32(item);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringSet_len = deserializer.DeserializeLen();
            var stringSet = new HashSet<string>();
            for (ulong i = 0; i < stringSet_len; i++)
            {
                var item = deserializer.DeserializeStr();
                stringSet.Add(item);
            }
            var intSet_len = deserializer.DeserializeLen();
            var intSet = new HashSet<int>();
            for (ulong i = 0; i < intSet_len; i++)
            {
                var item = deserializer.DeserializeI32();
                intSet.Add(item);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                StringSet = stringSet,
                IntSet = intSet,
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
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)StringSet.Count);
            foreach (var item in StringSet)
            {
                serializer.SerializeStr(item);
            }
            serializer.SerializeLen((ulong)IntSet.Count);
            foreach (var item in IntSet)
            {
                serializer.SerializeI32(item);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringSet_len = deserializer.DeserializeLen();
            var stringSet = new HashSet<string>();
            for (ulong i = 0; i < stringSet_len; i++)
            {
                var item = deserializer.DeserializeStr();
                stringSet.Add(item);
            }
            var intSet_len = deserializer.DeserializeLen();
            var intSet = new HashSet<int>();
            for (ulong i = 0; i < intSet_len; i++)
            {
                var item = deserializer.DeserializeI32();
                intSet.Add(item);
            }
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                StringSet = stringSet,
                IntSet = intSet,
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
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private string _boxedString;
        [ObservableProperty]
        private int _boxedInt;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(BoxedString);
            serializer.SerializeI32(BoxedInt);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var boxedString = deserializer.DeserializeStr();
            var boxedInt = deserializer.DeserializeI32();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                BoxedString = boxedString,
                BoxedInt = boxedInt,
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
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private string _rcString;
        [ObservableProperty]
        private int _rcInt;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(RcString);
            serializer.SerializeI32(RcInt);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var rcString = deserializer.DeserializeStr();
            var rcInt = deserializer.DeserializeI32();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                RcString = rcString,
                RcInt = rcInt,
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
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private string _arcString;
        [ObservableProperty]
        private int _arcInt;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeStr(ArcString);
            serializer.SerializeI32(ArcInt);
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var arcString = deserializer.DeserializeStr();
            var arcInt = deserializer.DeserializeI32();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                ArcString = arcString,
                ArcInt = arcInt,
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

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private ObservableCollection<HashSet<string>> _vecOfSets;
        [ObservableProperty]
        private Option<Dictionary<string, int>> _optionalBtree;
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [ObservableProperty]
        private Option<string> _arcOption;
        [ObservableProperty]
        private int[] _arrayOfBoxes;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeLen((ulong)VecOfSets.Count);
            foreach (var item in VecOfSets)
            {
                serializer.SerializeLen((ulong)item.Count);
                foreach (var item in item)
                {
                    serializer.SerializeStr(item);
                }
            }
            if (OptionalBtree.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalBtree.Value.Count);
                foreach (var entry in OptionalBtree.Value)
                {
                    serializer.SerializeStr(entry.Key);
                    serializer.SerializeI32(entry.Value);
                }
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.SerializeLen((ulong)BoxedVec.Count);
            foreach (var item in BoxedVec)
            {
                serializer.SerializeStr(item);
            }
            if (ArcOption.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeStr(ArcOption.Value);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.SerializeLen((ulong)ArrayOfBoxes.Length);
            foreach (var item in ArrayOfBoxes)
            {
                serializer.SerializeI32(item);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var vecOfSets_len = deserializer.DeserializeLen();
            var vecOfSets = new ObservableCollection<HashSet<string>>();
            for (ulong i = 0; i < vecOfSets_len; i++)
            {
                var item_len = deserializer.DeserializeLen();
                var item = new HashSet<string>();
                for (ulong i = 0; i < item_len; i++)
                {
                    var item = deserializer.DeserializeStr();
                    item.Add(item);
                }
                vecOfSets.Add(item);
            }
            Option<Dictionary<string, int>> optionalBtree;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalBtree_value_len = deserializer.DeserializeLen();
                var optionalBtree_value = new Dictionary<string, int>();
                for (ulong i = 0; i < optionalBtree_value_len; i++)
                {
                    var key = deserializer.DeserializeStr();
                    var value = deserializer.DeserializeI32();
                    optionalBtree_value.Add(key, value);
                }
                optionalBtree = Option<Dictionary<string, int>>.Some(optionalBtree_value);
            }
            else
            {
                optionalBtree = Option<Dictionary<string, int>>.None();
            }
            var boxedVec_len = deserializer.DeserializeLen();
            var boxedVec = new ObservableCollection<string>();
            for (ulong i = 0; i < boxedVec_len; i++)
            {
                var item = deserializer.DeserializeStr();
                boxedVec.Add(item);
            }
            Option<string> arcOption;
            if (deserializer.DeserializeOptionTag())
            {
                var arcOption_value = deserializer.DeserializeStr();
                arcOption = Option<string>.Some(arcOption_value);
            }
            else
            {
                arcOption = Option<string>.None();
            }
            var arrayOfBoxes_len = deserializer.DeserializeLen();
            var arrayOfBoxes_list = new List<int>();
            for (ulong i = 0; i < arrayOfBoxes_len; i++)
            {
                var item = deserializer.DeserializeI32();
                arrayOfBoxes_list.Add(item);
            }
            var arrayOfBoxes = arrayOfBoxes_list.ToArray();
            deserializer.DecreaseContainerDepth();
            return new MyStruct {
                VecOfSets = vecOfSets,
                OptionalBtree = optionalBtree,
                BoxedVec = boxedVec,
                ArcOption = arcOption,
                ArrayOfBoxes = arrayOfBoxes,
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
        #[facet(bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
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
        #[facet(bytes)]
        data: &'a [u8],
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject, IFacetSerializable, IFacetDeserializable<MyStruct> {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;
        [ObservableProperty]
        private Option<ObservableCollection<byte>> _optionalBytes;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeBytes(Data);
            serializer.SerializeStr(Name);
            serializer.SerializeBytes(Header);
            if (OptionalBytes.HasValue)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalBytes.Value.Count);
                foreach (var item in OptionalBytes.Value)
                {
                    serializer.SerializeU8(item);
                }
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var data = deserializer.DeserializeBytes();
            var name = deserializer.DeserializeStr();
            var header = deserializer.DeserializeBytes();
            Option<ObservableCollection<byte>> optionalBytes;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalBytes_value_len = deserializer.DeserializeLen();
                var optionalBytes_value = new ObservableCollection<byte>();
                for (ulong i = 0; i < optionalBytes_value_len; i++)
                {
                    var item = deserializer.DeserializeU8();
                    optionalBytes_value.Add(item);
                }
                optionalBytes = Option<ObservableCollection<byte>>.Some(optionalBytes_value);
            }
            else
            {
                optionalBytes = Option<ObservableCollection<byte>>.None();
            }
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
