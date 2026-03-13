//! Bincode collection tests — `ObservableCollection<T>`, `HashSet<T>`,
//! `Dictionary<K,V>`, fixed-size arrays, nested generics.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use facet::Facet;

use super::super::*;
use crate::emit;

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
            FacetHelpers.SerializeCollection(Items, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeCollection(Numbers, serializer, (item, s) => s.SerializeI32(item));
            FacetHelpers.SerializeCollection(NestedItems, serializer, (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeStr(item)));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var items = FacetHelpers.DeserializeList(deserializer, d => d.DeserializeStr());
            var numbers = FacetHelpers.DeserializeList(deserializer, d => d.DeserializeI32());
            var nestedItems = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeList(d, d => d.DeserializeStr()));
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
        private string? _optionalString;
        [ObservableProperty]
        private int? _optionalNumber;
        [ObservableProperty]
        private bool? _optionalBool;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            FacetHelpers.SerializeOptionRef(OptionalString, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeOption(OptionalNumber, serializer, (item, s) => s.SerializeI32(item));
            FacetHelpers.SerializeOption(OptionalBool, serializer, (item, s) => s.SerializeBool(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var optionalString = FacetHelpers.DeserializeOptionRef(deserializer, d => d.DeserializeStr());
            var optionalNumber = FacetHelpers.DeserializeOption(deserializer, d => d.DeserializeI32());
            var optionalBool = FacetHelpers.DeserializeOption(deserializer, d => d.DeserializeBool());
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
            FacetHelpers.SerializeMap(StringToInt, serializer, (item, s) => s.SerializeStr(item), (item, s) => s.SerializeI32(item));
            FacetHelpers.SerializeMap(IntToBool, serializer, (item, s) => s.SerializeI32(item), (item, s) => s.SerializeBool(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringToInt = FacetHelpers.DeserializeMap(deserializer, d => d.DeserializeStr(), d => d.DeserializeI32());
            var intToBool = FacetHelpers.DeserializeMap(deserializer, d => d.DeserializeI32(), d => d.DeserializeBool());
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
        private ObservableCollection<string>? _optionalList;
        [ObservableProperty]
        private ObservableCollection<int?> _listOfOptionals;
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [ObservableProperty]
        private Dictionary<string, int>? _optionalMap;
        [ObservableProperty]
        private ObservableCollection<Dictionary<string, ObservableCollection<bool>>?> _complex;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            FacetHelpers.SerializeOptionRef(OptionalList, serializer, (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeStr(item)));
            FacetHelpers.SerializeCollection(ListOfOptionals, serializer, (item, s) => FacetHelpers.SerializeOption(item, s, (item, s) => s.SerializeI32(item)));
            FacetHelpers.SerializeMap(MapToList, serializer, (item, s) => s.SerializeStr(item), (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeBool(item)));
            FacetHelpers.SerializeOptionRef(OptionalMap, serializer, (item, s) => FacetHelpers.SerializeMap(item, s, (item, s) => s.SerializeStr(item), (item, s) => s.SerializeI32(item)));
            FacetHelpers.SerializeCollection(Complex, serializer, (item, s) => FacetHelpers.SerializeOptionRef(item, s, (item, s) => FacetHelpers.SerializeMap(item, s, (item, s) => s.SerializeStr(item), (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeBool(item)))));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var optionalList = FacetHelpers.DeserializeOptionRef(deserializer, d => FacetHelpers.DeserializeList(d, d => d.DeserializeStr()));
            var listOfOptionals = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeOption(d, d => d.DeserializeI32()));
            var mapToList = FacetHelpers.DeserializeMap(deserializer, d => d.DeserializeStr(), d => FacetHelpers.DeserializeList(d, d => d.DeserializeBool()));
            var optionalMap = FacetHelpers.DeserializeOptionRef(deserializer, d => FacetHelpers.DeserializeMap(d, d => d.DeserializeStr(), d => d.DeserializeI32()));
            var complex = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeOptionRef(d, d => FacetHelpers.DeserializeMap(d, d => d.DeserializeStr(), d => FacetHelpers.DeserializeList(d, d => d.DeserializeBool()))));
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
            FacetHelpers.SerializeArray(FixedArray, serializer, (item, s) => s.SerializeI32(item));
            FacetHelpers.SerializeArray(ByteArray, serializer, (item, s) => s.SerializeU8(item));
            FacetHelpers.SerializeArray(StringArray, serializer, (item, s) => s.SerializeStr(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var fixedArray = FacetHelpers.DeserializeArray(deserializer, 5, d => d.DeserializeI32());
            var byteArray = FacetHelpers.DeserializeArray(deserializer, 32, d => d.DeserializeU8());
            var stringArray = FacetHelpers.DeserializeArray(deserializer, 3, d => d.DeserializeStr());
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
            FacetHelpers.SerializeMap(StringToInt, serializer, (item, s) => s.SerializeStr(item), (item, s) => s.SerializeI32(item));
            FacetHelpers.SerializeMap(IntToBool, serializer, (item, s) => s.SerializeI32(item), (item, s) => s.SerializeBool(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringToInt = FacetHelpers.DeserializeMap(deserializer, d => d.DeserializeStr(), d => d.DeserializeI32());
            var intToBool = FacetHelpers.DeserializeMap(deserializer, d => d.DeserializeI32(), d => d.DeserializeBool());
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
            FacetHelpers.SerializeCollection(StringSet, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeCollection(IntSet, serializer, (item, s) => s.SerializeI32(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringSet = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeStr());
            var intSet = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeI32());
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
            FacetHelpers.SerializeCollection(StringSet, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeCollection(IntSet, serializer, (item, s) => s.SerializeI32(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var stringSet = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeStr());
            var intSet = FacetHelpers.DeserializeSet(deserializer, d => d.DeserializeI32());
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

/// Nullable value types (e.g. `float?` from `Option<f32>`) use `FacetHelpers.SerializeOption`
/// which handles `.Value` unwrapping internally.
#[test]
fn option_value_type_needs_dot_value_for_serialize() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct HasOptionals {
        maybe_float: Option<f32>,
        maybe_double: Option<f64>,
        maybe_char: Option<char>,
        maybe_int: Option<i32>,
    }

    let actual = emit!(HasOptionals as CSharp with Encoding::Bincode).unwrap();
    assert!(
        actual.contains("FacetHelpers.SerializeOption(MaybeFloat, serializer,"),
        "float? should use FacetHelpers.SerializeOption\n{actual}"
    );
    assert!(
        actual.contains("FacetHelpers.SerializeOption(MaybeDouble, serializer,"),
        "double? should use FacetHelpers.SerializeOption\n{actual}"
    );
    assert!(
        actual.contains("FacetHelpers.SerializeOption(MaybeChar, serializer,"),
        "char? should use FacetHelpers.SerializeOption\n{actual}"
    );
    assert!(
        actual.contains("FacetHelpers.SerializeOption(MaybeInt, serializer,"),
        "int? should use FacetHelpers.SerializeOption\n{actual}"
    );
}

/// `Vec<Vec<T>>` deserialization generates nested loops that reuse `item` and `i`,
/// but C# does not allow shadowing locals in nested scopes.
#[test]
fn nested_seq_deserialization_uses_unique_variable_names() {
    #[derive(Facet)]
    struct Inner {
        x: u32,
    }

    #[derive(Facet)]
    struct HasNestedVec {
        nested: Vec<Vec<Inner>>,
    }

    let actual = emit!(HasNestedVec as CSharp with Encoding::Bincode).unwrap();
    // The inner loop must not reuse `i` or `item` from the outer loop.
    let i_decl_count = actual.matches("ulong i ").count();
    assert!(
        i_decl_count <= 1,
        "Found {i_decl_count} `ulong i` declarations — nested loops shadow `i`\n{actual}"
    );
}

/// Map deserialization declares `key` and `value` locals inside the loop body.
/// `value` collides with `var value = Deserialize(...)` in `BincodeDeserialize`.
#[test]
fn map_deserialization_uses_unique_variable_names() {
    #[derive(Facet)]
    struct HasMap {
        lookup: std::collections::BTreeMap<String, u32>,
    }

    let actual = emit!(HasMap as CSharp with Encoding::Bincode).unwrap();
    let value_decl_count = actual.matches("var value ").count();
    assert!(
        value_decl_count <= 1,
        "Found {value_decl_count} `var value` declarations — map loop `value` collides \
         with BincodeDeserialize outer `value`\n{actual}"
    );
}
