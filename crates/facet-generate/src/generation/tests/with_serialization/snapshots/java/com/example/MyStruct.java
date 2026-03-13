package com.example;


public final class MyStruct {
    public final java.util.Map<String, Integer> string_to_int;
    public final java.util.Map<String, java.util.List<Integer>> map_to_list;
    public final java.util.Optional<java.util.List<java.util.List<String>>> option_of_vec_of_set;
    public final Parent parent;

    public MyStruct(java.util.Map<String, Integer> string_to_int, java.util.Map<String, java.util.List<Integer>> map_to_list, java.util.Optional<java.util.List<java.util.List<String>>> option_of_vec_of_set, Parent parent) {
        java.util.Objects.requireNonNull(string_to_int, "string_to_int must not be null");
        java.util.Objects.requireNonNull(map_to_list, "map_to_list must not be null");
        java.util.Objects.requireNonNull(option_of_vec_of_set, "option_of_vec_of_set must not be null");
        java.util.Objects.requireNonNull(parent, "parent must not be null");
        this.string_to_int = string_to_int;
        this.map_to_list = map_to_list;
        this.option_of_vec_of_set = option_of_vec_of_set;
        this.parent = parent;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        TraitHelpers.serialize_map_str_to_i32(string_to_int, serializer);
        TraitHelpers.serialize_map_str_to_vector_i32(map_to_list, serializer);
        TraitHelpers.serialize_option_vector_set_str(option_of_vec_of_set, serializer);
        parent.serialize(serializer);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.string_to_int = TraitHelpers.deserialize_map_str_to_i32(deserializer);
        builder.map_to_list = TraitHelpers.deserialize_map_str_to_vector_i32(deserializer);
        builder.option_of_vec_of_set = TraitHelpers.deserialize_option_vector_set_str(deserializer);
        builder.parent = Parent.deserialize(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static MyStruct bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        MyStruct value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        MyStruct other = (MyStruct) obj;
        if (!java.util.Objects.equals(this.string_to_int, other.string_to_int)) { return false; }
        if (!java.util.Objects.equals(this.map_to_list, other.map_to_list)) { return false; }
        if (!java.util.Objects.equals(this.option_of_vec_of_set, other.option_of_vec_of_set)) { return false; }
        if (!java.util.Objects.equals(this.parent, other.parent)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.string_to_int != null ? this.string_to_int.hashCode() : 0);
        value = 31 * value + (this.map_to_list != null ? this.map_to_list.hashCode() : 0);
        value = 31 * value + (this.option_of_vec_of_set != null ? this.option_of_vec_of_set.hashCode() : 0);
        value = 31 * value + (this.parent != null ? this.parent.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public java.util.Map<String, Integer> string_to_int;
        public java.util.Map<String, java.util.List<Integer>> map_to_list;
        public java.util.Optional<java.util.List<java.util.List<String>>> option_of_vec_of_set;
        public Parent parent;

        public MyStruct build() {
            return new MyStruct(
                string_to_int,
                map_to_list,
                option_of_vec_of_set,
                parent
            );
        }
    }
}
