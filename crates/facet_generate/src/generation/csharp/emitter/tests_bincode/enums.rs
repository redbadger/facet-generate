//! Bincode enum tests — unit-only enums (`{EnumName}Bincode` static helper)
//! and mixed-variant enums (`abstract record` hierarchy with `partial record`
//! overrides).

#![allow(clippy::too_many_lines)]

use facet::Facet;

use super::super::*;
use crate::{emit, generation::Encoding};

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

    /// <summary>
    /// Bincode serialization helpers for <see cref="EnumWithUnitVariants"/>.
    /// </summary>
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

    /// <summary>
    /// Bincode serialization helpers for <see cref="MyEnum"/>.
    /// </summary>
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

    public abstract record MyEnum : IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed partial record Variant1(string Value) : MyEnum;

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

    public abstract record MyEnum : IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed partial record Variant1(string Value) : MyEnum;

        public sealed partial record Variant2(int Value) : MyEnum;

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

    public abstract record MyEnum : IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed partial record Variant1(string Field0, int Field1) : MyEnum;

        public sealed partial record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

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

    public abstract record MyEnum : IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed partial record Variant1(string Field1, int Field2) : MyEnum;

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

    public abstract record MyEnum : IFacetSerializable, IFacetDeserializable<MyEnum> {
        public sealed partial record Unit() : MyEnum;

        public sealed partial record NewType(string Value) : MyEnum;

        public sealed partial record Tuple(string Field0, int Field1) : MyEnum;

        public sealed partial record Struct(bool Field) : MyEnum;

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

/// When a field references a C-style enum (all unit variants), bincode
/// serialize must call `{Enum}Bincode.Serialize(val, serializer)` rather than
/// `val.Serialize(serializer)`, because C# enums have no instance methods.
/// Deserialization must similarly use `{Enum}Bincode.Deserialize(deserializer)`.
#[test]
fn c_style_enum_field_uses_static_bincode_helpers() {
    #[derive(Facet)]
    #[allow(dead_code)]
    #[repr(C)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    #[derive(Facet)]
    struct Painted {
        color: Color,
    }

    let actual = emit!(Painted as CSharp with Encoding::Bincode).unwrap();
    assert!(
        actual.contains("ColorBincode.Serialize(Color, serializer)"),
        "c-style enum serialize should dispatch to static helper\n{actual}"
    );
    assert!(
        actual.contains("ColorBincode.Deserialize(deserializer)"),
        "c-style enum deserialize should dispatch to static helper\n{actual}"
    );
}
