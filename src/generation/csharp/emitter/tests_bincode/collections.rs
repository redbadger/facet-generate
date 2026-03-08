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
            for (ulong items_idx = 0; items_idx < items_len; items_idx++)
            {
                var items_item = deserializer.DeserializeStr();
                items.Add(items_item);
            }
            var numbers_len = deserializer.DeserializeLen();
            var numbers = new ObservableCollection<int>();
            for (ulong numbers_idx = 0; numbers_idx < numbers_len; numbers_idx++)
            {
                var numbers_item = deserializer.DeserializeI32();
                numbers.Add(numbers_item);
            }
            var nestedItems_len = deserializer.DeserializeLen();
            var nestedItems = new ObservableCollection<ObservableCollection<string>>();
            for (ulong nestedItems_idx = 0; nestedItems_idx < nestedItems_len; nestedItems_idx++)
            {
                var nestedItems_item_len = deserializer.DeserializeLen();
                var nestedItems_item = new ObservableCollection<string>();
                for (ulong nestedItems_item_idx = 0; nestedItems_item_idx < nestedItems_item_len; nestedItems_item_idx++)
                {
                    var nestedItems_item_item = deserializer.DeserializeStr();
                    nestedItems_item.Add(nestedItems_item_item);
                }
                nestedItems.Add(nestedItems_item);
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
        private string? _optionalString;
        [ObservableProperty]
        private int? _optionalNumber;
        [ObservableProperty]
        private bool? _optionalBool;

        public void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            if (OptionalString is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeStr(OptionalString);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            if (OptionalNumber is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeI32(OptionalNumber.Value);
            }
            else
            {
                serializer.SerializeOptionTag(false);
            }
            if (OptionalBool is not null)
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
            string? optionalString;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalString_value = deserializer.DeserializeStr();
                optionalString = optionalString_value;
            }
            else
            {
                optionalString = null;
            }
            int? optionalNumber;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalNumber_value = deserializer.DeserializeI32();
                optionalNumber = optionalNumber_value;
            }
            else
            {
                optionalNumber = null;
            }
            bool? optionalBool;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalBool_value = deserializer.DeserializeBool();
                optionalBool = optionalBool_value;
            }
            else
            {
                optionalBool = null;
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
            for (ulong stringToInt_idx = 0; stringToInt_idx < stringToInt_len; stringToInt_idx++)
            {
                var stringToInt_key = deserializer.DeserializeStr();
                var stringToInt_val = deserializer.DeserializeI32();
                stringToInt.Add(stringToInt_key, stringToInt_val);
            }
            var intToBool_len = deserializer.DeserializeLen();
            var intToBool = new Dictionary<int, bool>();
            for (ulong intToBool_idx = 0; intToBool_idx < intToBool_len; intToBool_idx++)
            {
                var intToBool_key = deserializer.DeserializeI32();
                var intToBool_val = deserializer.DeserializeBool();
                intToBool.Add(intToBool_key, intToBool_val);
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
            if (OptionalList is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalList.Count);
                foreach (var item in OptionalList)
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
                if (item is not null)
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
            if (OptionalMap is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalMap.Count);
                foreach (var entry in OptionalMap)
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
                if (item is not null)
                {
                    serializer.SerializeOptionTag(true);
                    serializer.SerializeLen((ulong)item.Count);
                    foreach (var entry in item)
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
            ObservableCollection<string>? optionalList;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalList_value_len = deserializer.DeserializeLen();
                var optionalList_value = new ObservableCollection<string>();
                for (ulong optionalList_value_idx = 0; optionalList_value_idx < optionalList_value_len; optionalList_value_idx++)
                {
                    var optionalList_value_item = deserializer.DeserializeStr();
                    optionalList_value.Add(optionalList_value_item);
                }
                optionalList = optionalList_value;
            }
            else
            {
                optionalList = null;
            }
            var listOfOptionals_len = deserializer.DeserializeLen();
            var listOfOptionals = new ObservableCollection<int?>();
            for (ulong listOfOptionals_idx = 0; listOfOptionals_idx < listOfOptionals_len; listOfOptionals_idx++)
            {
                int? listOfOptionals_item;
                if (deserializer.DeserializeOptionTag())
                {
                    var listOfOptionals_item_value = deserializer.DeserializeI32();
                    listOfOptionals_item = listOfOptionals_item_value;
                }
                else
                {
                    listOfOptionals_item = null;
                }
                listOfOptionals.Add(listOfOptionals_item);
            }
            var mapToList_len = deserializer.DeserializeLen();
            var mapToList = new Dictionary<string, ObservableCollection<bool>>();
            for (ulong mapToList_idx = 0; mapToList_idx < mapToList_len; mapToList_idx++)
            {
                var mapToList_key = deserializer.DeserializeStr();
                var mapToList_val_len = deserializer.DeserializeLen();
                var mapToList_val = new ObservableCollection<bool>();
                for (ulong mapToList_val_idx = 0; mapToList_val_idx < mapToList_val_len; mapToList_val_idx++)
                {
                    var mapToList_val_item = deserializer.DeserializeBool();
                    mapToList_val.Add(mapToList_val_item);
                }
                mapToList.Add(mapToList_key, mapToList_val);
            }
            Dictionary<string, int>? optionalMap;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalMap_value_len = deserializer.DeserializeLen();
                var optionalMap_value = new Dictionary<string, int>();
                for (ulong optionalMap_value_idx = 0; optionalMap_value_idx < optionalMap_value_len; optionalMap_value_idx++)
                {
                    var optionalMap_value_key = deserializer.DeserializeStr();
                    var optionalMap_value_val = deserializer.DeserializeI32();
                    optionalMap_value.Add(optionalMap_value_key, optionalMap_value_val);
                }
                optionalMap = optionalMap_value;
            }
            else
            {
                optionalMap = null;
            }
            var complex_len = deserializer.DeserializeLen();
            var complex = new ObservableCollection<Dictionary<string, ObservableCollection<bool>>?>();
            for (ulong complex_idx = 0; complex_idx < complex_len; complex_idx++)
            {
                Dictionary<string, ObservableCollection<bool>>? complex_item;
                if (deserializer.DeserializeOptionTag())
                {
                    var complex_item_value_len = deserializer.DeserializeLen();
                    var complex_item_value = new Dictionary<string, ObservableCollection<bool>>();
                    for (ulong complex_item_value_idx = 0; complex_item_value_idx < complex_item_value_len; complex_item_value_idx++)
                    {
                        var complex_item_value_key = deserializer.DeserializeStr();
                        var complex_item_value_val_len = deserializer.DeserializeLen();
                        var complex_item_value_val = new ObservableCollection<bool>();
                        for (ulong complex_item_value_val_idx = 0; complex_item_value_val_idx < complex_item_value_val_len; complex_item_value_val_idx++)
                        {
                            var complex_item_value_val_item = deserializer.DeserializeBool();
                            complex_item_value_val.Add(complex_item_value_val_item);
                        }
                        complex_item_value.Add(complex_item_value_key, complex_item_value_val);
                    }
                    complex_item = complex_item_value;
                }
                else
                {
                    complex_item = null;
                }
                complex.Add(complex_item);
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
            for (ulong stringToInt_idx = 0; stringToInt_idx < stringToInt_len; stringToInt_idx++)
            {
                var stringToInt_key = deserializer.DeserializeStr();
                var stringToInt_val = deserializer.DeserializeI32();
                stringToInt.Add(stringToInt_key, stringToInt_val);
            }
            var intToBool_len = deserializer.DeserializeLen();
            var intToBool = new Dictionary<int, bool>();
            for (ulong intToBool_idx = 0; intToBool_idx < intToBool_len; intToBool_idx++)
            {
                var intToBool_key = deserializer.DeserializeI32();
                var intToBool_val = deserializer.DeserializeBool();
                intToBool.Add(intToBool_key, intToBool_val);
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
            for (ulong stringSet_idx = 0; stringSet_idx < stringSet_len; stringSet_idx++)
            {
                var stringSet_item = deserializer.DeserializeStr();
                stringSet.Add(stringSet_item);
            }
            var intSet_len = deserializer.DeserializeLen();
            var intSet = new HashSet<int>();
            for (ulong intSet_idx = 0; intSet_idx < intSet_len; intSet_idx++)
            {
                var intSet_item = deserializer.DeserializeI32();
                intSet.Add(intSet_item);
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
            for (ulong stringSet_idx = 0; stringSet_idx < stringSet_len; stringSet_idx++)
            {
                var stringSet_item = deserializer.DeserializeStr();
                stringSet.Add(stringSet_item);
            }
            var intSet_len = deserializer.DeserializeLen();
            var intSet = new HashSet<int>();
            for (ulong intSet_idx = 0; intSet_idx < intSet_len; intSet_idx++)
            {
                var intSet_item = deserializer.DeserializeI32();
                intSet.Add(intSet_item);
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

/// Nullable value types (e.g. `float?` from `Option<f32>`) must be unwrapped
/// with `.Value` before passing to serializer methods like `SerializeF32`.
#[test]
fn option_value_type_needs_dot_value_for_serialize() {
    #[derive(Facet)]
    struct HasOptionals {
        maybe_float: Option<f32>,
        maybe_double: Option<f64>,
        maybe_char: Option<char>,
        maybe_int: Option<i32>,
    }

    let actual = emit!(HasOptionals as CSharp with Encoding::Bincode).unwrap();
    assert!(
        actual.contains("SerializeF32(MaybeFloat.Value)"),
        "float? must be unwrapped with .Value\n{actual}"
    );
    assert!(
        actual.contains("SerializeF64(MaybeDouble.Value)"),
        "double? must be unwrapped with .Value\n{actual}"
    );
    assert!(
        actual.contains("SerializeChar(MaybeChar.Value)"),
        "char? must be unwrapped with .Value\n{actual}"
    );
    assert!(
        actual.contains("SerializeI32(MaybeInt.Value)"),
        "int? must be unwrapped with .Value\n{actual}"
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

    let actual = emit!(HasNestedVec, Inner as CSharp with Encoding::Bincode).unwrap();
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
