#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::super::*;
use crate::emit;

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
        private Dictionary<string, int>? _optionalBtree;
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [ObservableProperty]
        private string? _arcOption;
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
            if (OptionalBtree is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeLen((ulong)OptionalBtree.Count);
                foreach (var entry in OptionalBtree)
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
            if (ArcOption is not null)
            {
                serializer.SerializeOptionTag(true);
                serializer.SerializeStr(ArcOption);
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
            for (ulong vecOfSets_idx = 0; vecOfSets_idx < vecOfSets_len; vecOfSets_idx++)
            {
                var vecOfSets_item_len = deserializer.DeserializeLen();
                var vecOfSets_item = new HashSet<string>();
                for (ulong vecOfSets_item_idx = 0; vecOfSets_item_idx < vecOfSets_item_len; vecOfSets_item_idx++)
                {
                    var vecOfSets_item_item = deserializer.DeserializeStr();
                    vecOfSets_item.Add(vecOfSets_item_item);
                }
                vecOfSets.Add(vecOfSets_item);
            }
            Dictionary<string, int>? optionalBtree;
            if (deserializer.DeserializeOptionTag())
            {
                var optionalBtree_value_len = deserializer.DeserializeLen();
                var optionalBtree_value = new Dictionary<string, int>();
                for (ulong optionalBtree_value_idx = 0; optionalBtree_value_idx < optionalBtree_value_len; optionalBtree_value_idx++)
                {
                    var optionalBtree_value_key = deserializer.DeserializeStr();
                    var optionalBtree_value_val = deserializer.DeserializeI32();
                    optionalBtree_value.Add(optionalBtree_value_key, optionalBtree_value_val);
                }
                optionalBtree = optionalBtree_value;
            }
            else
            {
                optionalBtree = null;
            }
            var boxedVec_len = deserializer.DeserializeLen();
            var boxedVec = new ObservableCollection<string>();
            for (ulong boxedVec_idx = 0; boxedVec_idx < boxedVec_len; boxedVec_idx++)
            {
                var boxedVec_item = deserializer.DeserializeStr();
                boxedVec.Add(boxedVec_item);
            }
            string? arcOption;
            if (deserializer.DeserializeOptionTag())
            {
                var arcOption_value = deserializer.DeserializeStr();
                arcOption = arcOption_value;
            }
            else
            {
                arcOption = null;
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
