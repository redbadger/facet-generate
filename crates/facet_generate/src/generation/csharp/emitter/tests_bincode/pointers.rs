//! Bincode pointer tests — `Box`, `Rc`, `Arc` (transparent), mixed
//! collections with smart pointers.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::super::*;
use crate::emit;
use crate::generation::bincode::BincodePlugin;

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
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

    let actual = emit!(MyStruct as CSharp with BincodePlugin).unwrap();
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
            FacetHelpers.SerializeCollection(VecOfSets, serializer, (item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeStr(item)));
            FacetHelpers.SerializeOptionRef(OptionalBtree, serializer, (item, s) => FacetHelpers.SerializeMap(item, s, (item, s) => s.SerializeStr(item), (item, s) => s.SerializeI32(item)));
            FacetHelpers.SerializeCollection(BoxedVec, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeOptionRef(ArcOption, serializer, (item, s) => s.SerializeStr(item));
            FacetHelpers.SerializeArray(ArrayOfBoxes, serializer, (item, s) => s.SerializeI32(item));
            serializer.DecreaseContainerDepth();
        }

        public static MyStruct Deserialize(IDeserializer deserializer)
        {
            deserializer.IncreaseContainerDepth();
            var vecOfSets = FacetHelpers.DeserializeList(deserializer, d => FacetHelpers.DeserializeSet(d, d => d.DeserializeStr()));
            var optionalBtree = FacetHelpers.DeserializeOptionRef(deserializer, d => FacetHelpers.DeserializeMap(d, d => d.DeserializeStr(), d => d.DeserializeI32()));
            var boxedVec = FacetHelpers.DeserializeList(deserializer, d => d.DeserializeStr());
            var arcOption = FacetHelpers.DeserializeOptionRef(deserializer, d => d.DeserializeStr());
            var arrayOfBoxes = FacetHelpers.DeserializeArray(deserializer, 3, d => d.DeserializeI32());
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
