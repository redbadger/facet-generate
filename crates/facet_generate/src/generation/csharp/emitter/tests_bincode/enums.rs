//! Bincode enum tests — unit-only enums (`{EnumName}Bincode` static helper)
//! and mixed-variant enums (`abstract record` hierarchy with `partial record`
//! overrides).

#![allow(clippy::too_many_lines)]

use facet::Facet;

use super::super::*;
use crate::{emit, generation::bincode::BincodePlugin};

#[test]
fn struct_variant_with_csharp_keyword_fields_escapes_deserialize_locals() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum KeywordVariant {
        Values { event: bool, lock: bool },
    }

    let actual = emit!(KeywordVariant as CSharp with BincodePlugin).unwrap();

    assert!(
        actual.contains("var @event = deserializer.DeserializeBool();"),
        "{actual}"
    );
    assert!(
        actual.contains("var @lock = deserializer.DeserializeBool();"),
        "{actual}"
    );
    assert!(
        actual.contains("return new Values(@event, @lock);"),
        "{actual}"
    );
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

    let actual = emit!(EnumWithUnitVariants as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyEnum as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(Painted as CSharp with BincodePlugin).unwrap();
    assert!(
        actual.contains("ColorBincode.Serialize(Color, serializer)"),
        "c-style enum serialize should dispatch to static helper\n{actual}"
    );
    assert!(
        actual.contains("ColorBincode.Deserialize(deserializer)"),
        "c-style enum deserialize should dispatch to static helper\n{actual}"
    );
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

    let actual = emit!(KeywordEnum as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record KeywordEnum : IFacetSerializable, IFacetDeserializable<KeywordEnum> {
        public sealed partial record Default() : KeywordEnum;

        public sealed partial record Switch(string Value) : KeywordEnum;

        public sealed partial record Where(int In, string Default) : KeywordEnum {
            public new string Default { get; init; } = Default;
        }

        public abstract void Serialize(ISerializer serializer);

        private static KeywordEnum DeserializeDefault(IDeserializer deserializer)
        {
            return new Default();
        }

        public sealed partial record Default
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.DecreaseContainerDepth();
            }

        }
        private static KeywordEnum DeserializeSwitch(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeStr();
            return new Switch(value);
        }

        public sealed partial record Switch
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeStr(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        private static KeywordEnum DeserializeWhere(IDeserializer deserializer)
        {
            var @in = deserializer.DeserializeI32();
            var @default = deserializer.DeserializeStr();
            return new Where(@in, @default);
        }

        public sealed partial record Where
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(2);
                serializer.SerializeI32(In);
                serializer.SerializeStr(Default);
                serializer.DecreaseContainerDepth();
            }

        }
        public static KeywordEnum Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeDefault(deserializer),
                1 => DeserializeSwitch(deserializer),
                2 => DeserializeWhere(deserializer),
                _ => throw new DeserializationError("Unknown variant index for KeywordEnum: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static KeywordEnum BincodeDeserialize(byte[] input)
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

/// A property of a variant's nested record is in scope in the variant's
/// `Serialize` override, where one named like a helper class hides it, but
/// not in the base record's static `Deserialize{Variant}`, where a static call
/// on a type stays bare (#159).
#[test]
fn variant_property_named_like_a_helper_class_qualifies_calls_on_it() {
    #[derive(Facet)]
    struct Presence {
        x: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Mood {
        Happy,
        Sad,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Event {
        Seen { presence: u32, other: Presence },
        Moody { mood_bincode: u32, mood: Mood },
    }

    let actual = emit!(Event as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Event : IFacetSerializable, IFacetDeserializable<Event> {
        public sealed partial record Seen(uint Presence, Presence Other) : Event;

        public sealed partial record Moody(uint MoodBincode, Mood Mood) : Event;

        public abstract void Serialize(ISerializer serializer);

        private static Event DeserializeSeen(IDeserializer deserializer)
        {
            var presence = deserializer.DeserializeU32();
            var other = Presence.Deserialize(deserializer);
            return new Seen(presence, other);
        }

        public sealed partial record Seen
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeU32(Presence);
                Other.Serialize(serializer);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeMoody(IDeserializer deserializer)
        {
            var moodBincode = deserializer.DeserializeU32();
            var mood = MoodBincode.Deserialize(deserializer);
            return new Moody(moodBincode, mood);
        }

        public sealed partial record Moody
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeU32(MoodBincode);
                global::Test.MoodBincode.Serialize(Mood, serializer);
                serializer.DecreaseContainerDepth();
            }

        }
        public static Event Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeSeen(deserializer),
                1 => DeserializeMoody(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Event: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Event BincodeDeserialize(byte[] input)
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

    public enum Mood {
        Happy,
        Sad
    }

    /// <summary>
    /// Bincode serialization helpers for <see cref="Mood"/>.
    /// </summary>
    public static class MoodBincode {
        public static void Serialize(Mood value, ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex((uint)value);
            serializer.DecreaseContainerDepth();
        }

        public static Mood Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var index = deserializer.DeserializeVariantIndex();
            deserializer.DecreaseContainerDepth();
            return index switch
            {
                0 => Mood.Happy,
                1 => Mood.Sad,
                _ => throw new DeserializationError("Unknown variant index for Mood: " + index),
            }
            ;
        }

        public static byte[] BincodeSerialize(Mood value)
        {
            var serializer = new BincodeSerializer();
            Serialize(value, serializer);
            return serializer.GetBytes();
        }

        public static Mood BincodeDeserialize(byte[] input)
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

    public partial class Presence : ObservableObject, IFacetSerializable, IFacetDeserializable<Presence> {
        [ObservableProperty]
        private uint _x;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeU32(X);
            serializer.DecreaseContainerDepth();
        }

        public static Presence Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var x = deserializer.DeserializeU32();
            deserializer.DecreaseContainerDepth();
            return new Presence {
                X = x,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Presence BincodeDeserialize(byte[] input)
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

/// Inside a variant hierarchy a variant's name hides a type of the same name
/// (#174). The payload of `Presence(Presence)` is written through its
/// `global::` name, since bare it would mean the variant itself (CS8910).
#[test]
fn newtype_variant_named_like_its_payload_type_qualifies_the_type() {
    #[derive(Facet)]
    struct Presence {
        x: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Event {
        Presence(Presence),
    }

    let actual = emit!(Event as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Event : IFacetSerializable, IFacetDeserializable<Event> {
        public sealed partial record Presence(global::Test.Presence Value) : Event;

        public abstract void Serialize(ISerializer serializer);

        private static Event DeserializePresence(IDeserializer deserializer)
        {
            var value = global::Test.Presence.Deserialize(deserializer);
            return new Presence(value);
        }

        public sealed partial record Presence
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                Value.Serialize(serializer);
                serializer.DecreaseContainerDepth();
            }

        }
        public static Event Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializePresence(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Event: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Event BincodeDeserialize(byte[] input)
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

    public partial class Presence : ObservableObject, IFacetSerializable, IFacetDeserializable<Presence> {
        [ObservableProperty]
        private uint _x;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeU32(X);
            serializer.DecreaseContainerDepth();
        }

        public static Presence Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var x = deserializer.DeserializeU32();
            deserializer.DecreaseContainerDepth();
            return new Presence {
                X = x,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Presence BincodeDeserialize(byte[] input)
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

/// A sibling variant's name hides a type too, however deep in the field's
/// type it appears, so `Presence` there would mean the variant `Event.Presence`
/// (#174).
#[test]
fn struct_variant_field_of_a_type_named_like_a_sibling_variant_qualifies_the_type() {
    #[derive(Facet)]
    struct Presence {
        x: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Event {
        Presence {
            x: u32,
        },
        Seen {
            other: Presence,
            others: Vec<Option<Presence>>,
        },
    }

    let actual = emit!(Event as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Event : IFacetSerializable, IFacetDeserializable<Event> {
        public sealed partial record Presence(uint X) : Event;

        public sealed partial record Seen(global::Test.Presence Other, ObservableCollection<global::Test.Presence?> Others) : Event;

        public abstract void Serialize(ISerializer serializer);

        private static Event DeserializePresence(IDeserializer deserializer)
        {
            var x = deserializer.DeserializeU32();
            return new Presence(x);
        }

        public sealed partial record Presence
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeU32(X);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeSeen(IDeserializer deserializer)
        {
            var other = global::Test.Presence.Deserialize(deserializer);
            var others = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeOptionRef(d, d => global::Test.Presence.Deserialize(d)));
            return new Seen(other, others);
        }

        public sealed partial record Seen
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                Other.Serialize(serializer);
                FacetHelpers.SerializeCollection(Others, serializer, (item, s) => FacetHelpers.SerializeOptionRef(item, s, (item, s) => item.Serialize(s)));
                serializer.DecreaseContainerDepth();
            }

        }
        public static Event Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializePresence(deserializer),
                1 => DeserializeSeen(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Event: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Event BincodeDeserialize(byte[] input)
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

    public partial class Presence : ObservableObject, IFacetSerializable, IFacetDeserializable<Presence> {
        [ObservableProperty]
        private uint _x;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeU32(X);
            serializer.DecreaseContainerDepth();
        }

        public static Presence Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var x = deserializer.DeserializeU32();
            deserializer.DecreaseContainerDepth();
            return new Presence {
                X = x,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Presence BincodeDeserialize(byte[] input)
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

/// A positional property named like a sibling variant is declared again with
/// `new`, so it hides the inherited nested record rather than colliding with
/// it (CS8866, #174).
#[test]
fn struct_variant_field_named_like_a_sibling_variant_redeclares_the_property() {
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
    }

    let actual = emit!(Event as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Event : IFacetSerializable, IFacetDeserializable<Event> {
        public sealed partial record Presence(global::Test.Presence Value) : Event;

        public sealed partial record Seen(uint Presence, global::Test.Presence Other) : Event {
            public new uint Presence { get; init; } = Presence;
        }

        public abstract void Serialize(ISerializer serializer);

        private static Event DeserializePresence(IDeserializer deserializer)
        {
            var value = global::Test.Presence.Deserialize(deserializer);
            return new Presence(value);
        }

        public sealed partial record Presence
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                Value.Serialize(serializer);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeSeen(IDeserializer deserializer)
        {
            var presence = deserializer.DeserializeU32();
            var other = global::Test.Presence.Deserialize(deserializer);
            return new Seen(presence, other);
        }

        public sealed partial record Seen
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeU32(Presence);
                Other.Serialize(serializer);
                serializer.DecreaseContainerDepth();
            }

        }
        public static Event Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializePresence(deserializer),
                1 => DeserializeSeen(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Event: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Event BincodeDeserialize(byte[] input)
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

    public partial class Presence : ObservableObject, IFacetSerializable, IFacetDeserializable<Presence> {
        [ObservableProperty]
        private uint _x;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeU32(X);
            serializer.DecreaseContainerDepth();
        }

        public static Presence Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var x = deserializer.DeserializeU32();
            deserializer.DecreaseContainerDepth();
            return new Presence {
                X = x,
            };
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Presence BincodeDeserialize(byte[] input)
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

/// The same for the properties a newtype and a tuple variant name
/// themselves: `Value` beside a variant `Value`, and `Field0` beside a variant
/// `Field0` (CS8866, #174).
#[test]
fn newtype_and_tuple_variant_properties_named_like_a_sibling_variant_are_redeclared() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Shape {
        Value { x: u32 },
        Wrap(u32),
        Pair(u32, u32),
        Field0,
    }

    let actual = emit!(Shape as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Shape : IFacetSerializable, IFacetDeserializable<Shape> {
        public sealed partial record Value(uint X) : Shape;

        public sealed partial record Wrap(uint Value) : Shape {
            public new uint Value { get; init; } = Value;
        }

        public sealed partial record Pair(uint Field0, uint Field1) : Shape {
            public new uint Field0 { get; init; } = Field0;
        }

        public sealed partial record Field0() : Shape;

        public abstract void Serialize(ISerializer serializer);

        private static Shape DeserializeValue(IDeserializer deserializer)
        {
            var x = deserializer.DeserializeU32();
            return new Value(x);
        }

        public sealed partial record Value
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeU32(X);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Shape DeserializeWrap(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeU32();
            return new Wrap(value);
        }

        public sealed partial record Wrap
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeU32(Value);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Shape DeserializePair(IDeserializer deserializer)
        {
            var field0 = deserializer.DeserializeU32();
            var field1 = deserializer.DeserializeU32();
            return new Pair(field0, field1);
        }

        public sealed partial record Pair
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(2);
                serializer.SerializeU32(Field0);
                serializer.SerializeU32(Field1);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Shape DeserializeField0(IDeserializer deserializer)
        {
            return new Field0();
        }

        public sealed partial record Field0
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(3);
                serializer.DecreaseContainerDepth();
            }

        }
        public static Shape Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializeValue(deserializer),
                1 => DeserializeWrap(deserializer),
                2 => DeserializePair(deserializer),
                3 => DeserializeField0(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Shape: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Shape BincodeDeserialize(byte[] input)
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

/// A property named like its own variant would be a member named like its
/// enclosing type (CS0542), so it takes a trailing underscore (#193). Here
/// `Presence`'s `presence`, the newtype `Value`'s `Value` and the tuple
/// `Field0`'s `Field0`; `Presence`'s `Status` is only named like a sibling,
/// so it is declared again with `new`, as before.
#[test]
fn variants_with_a_property_named_like_the_variant_rename_it() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Event {
        Presence { presence: u32, status: u8 },
        Status { status: u8 },
        Value(u32),
        Field0(u32, u32),
    }

    let actual = emit!(Event as CSharp with BincodePlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public abstract record Event : IFacetSerializable, IFacetDeserializable<Event> {
        public sealed partial record Presence(uint Presence_, byte Status) : Event {
            public new byte Status { get; init; } = Status;
        }

        public sealed partial record Status(byte Status_) : Event;

        public sealed partial record Value(uint Value_) : Event;

        public sealed partial record Field0(uint Field0_, uint Field1) : Event;

        public abstract void Serialize(ISerializer serializer);

        private static Event DeserializePresence(IDeserializer deserializer)
        {
            var presence = deserializer.DeserializeU32();
            var status = deserializer.DeserializeU8();
            return new Presence(presence, status);
        }

        public sealed partial record Presence
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(0);
                serializer.SerializeU32(Presence_);
                serializer.SerializeU8(Status);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeStatus(IDeserializer deserializer)
        {
            var status = deserializer.DeserializeU8();
            return new Status(status);
        }

        public sealed partial record Status
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(1);
                serializer.SerializeU8(Status_);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeValue(IDeserializer deserializer)
        {
            var value = deserializer.DeserializeU32();
            return new Value(value);
        }

        public sealed partial record Value
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(2);
                serializer.SerializeU32(Value_);
                serializer.DecreaseContainerDepth();
            }

        }
        private static Event DeserializeField0(IDeserializer deserializer)
        {
            var field0 = deserializer.DeserializeU32();
            var field1 = deserializer.DeserializeU32();
            return new Field0(field0, field1);
        }

        public sealed partial record Field0
        {
            public override void Serialize(ISerializer serializer)
            {
                serializer.IncreaseContainerDepth();
                serializer.SerializeVariantIndex(3);
                serializer.SerializeU32(Field0_);
                serializer.SerializeU32(Field1);
                serializer.DecreaseContainerDepth();
            }

        }
        public static Event Deserialize(IDeserializer deserializer)
        {
            var index = deserializer.DeserializeVariantIndex();
            return index switch
            {
                0 => DeserializePresence(deserializer),
                1 => DeserializeStatus(deserializer),
                2 => DeserializeValue(deserializer),
                3 => DeserializeField0(deserializer),
                _ => throw new DeserializationError("Unknown variant index for Event: " + index),
            }
            ;
        }

        public byte[] BincodeSerialize()
        {
            var serializer = new BincodeSerializer();
            Serialize(serializer);
            return serializer.GetBytes();
        }

        public static Event BincodeDeserialize(byte[] input)
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
