package com.example;

final class TraitHelpers {
    static void serialize_map_str_to_i32(java.util.Map<String, Integer> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        int[] offsets = new int[value.size()];
        int count = 0;
        for (java.util.Map.Entry<String, Integer> entry : value.entrySet()) {
            offsets[count++] = serializer.get_buffer_offset();
            serializer.serialize_str(entry.getKey());
            serializer.serialize_i32(entry.getValue());
        }
        serializer.sort_map_entries(offsets);
    }

    static java.util.Map<String, Integer> deserialize_map_str_to_i32(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.Map<String, Integer> obj = new java.util.HashMap<String, Integer>();
        int previous_key_start = 0;
        int previous_key_end = 0;
        for (long i = 0; i < length; i++) {
            int key_start = deserializer.get_buffer_offset();
            String key = deserializer.deserialize_str();
            int key_end = deserializer.get_buffer_offset();
            if (i > 0) {
                deserializer.check_that_key_slices_are_increasing(
                    new com.novi.serde.Slice(previous_key_start, previous_key_end),
                    new com.novi.serde.Slice(key_start, key_end));
            }
            previous_key_start = key_start;
            previous_key_end = key_end;
            Integer value = deserializer.deserialize_i32();
            obj.put(key, value);
        }
        return obj;
    }

    static void serialize_map_str_to_vector_i32(java.util.Map<String, java.util.List<Integer>> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        int[] offsets = new int[value.size()];
        int count = 0;
        for (java.util.Map.Entry<String, java.util.List<Integer>> entry : value.entrySet()) {
            offsets[count++] = serializer.get_buffer_offset();
            serializer.serialize_str(entry.getKey());
            TraitHelpers.serialize_vector_i32(entry.getValue(), serializer);
        }
        serializer.sort_map_entries(offsets);
    }

    static java.util.Map<String, java.util.List<Integer>> deserialize_map_str_to_vector_i32(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.Map<String, java.util.List<Integer>> obj = new java.util.HashMap<String, java.util.List<Integer>>();
        int previous_key_start = 0;
        int previous_key_end = 0;
        for (long i = 0; i < length; i++) {
            int key_start = deserializer.get_buffer_offset();
            String key = deserializer.deserialize_str();
            int key_end = deserializer.get_buffer_offset();
            if (i > 0) {
                deserializer.check_that_key_slices_are_increasing(
                    new com.novi.serde.Slice(previous_key_start, previous_key_end),
                    new com.novi.serde.Slice(key_start, key_end));
            }
            previous_key_start = key_start;
            previous_key_end = key_end;
            java.util.List<Integer> value = TraitHelpers.deserialize_vector_i32(deserializer);
            obj.put(key, value);
        }
        return obj;
    }

    static void serialize_option_vector_set_str(java.util.Optional<java.util.List<java.util.List<String>>> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        if (value.isPresent()) {
            serializer.serialize_option_tag(true);
            TraitHelpers.serialize_vector_set_str(value.get(), serializer);
        } else {
            serializer.serialize_option_tag(false);
        }
    }

    static java.util.Optional<java.util.List<java.util.List<String>>> deserialize_option_vector_set_str(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        boolean tag = deserializer.deserialize_option_tag();
        if (!tag) {
            return java.util.Optional.empty();
        } else {
            return java.util.Optional.of(TraitHelpers.deserialize_vector_set_str(deserializer));
        }
    }

    static void serialize_set_str(java.util.List<String> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (String item : value) {
            serializer.serialize_str(item);
        }
    }

    static java.util.List<String> deserialize_set_str(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<String> obj = new java.util.ArrayList<String>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(deserializer.deserialize_str());
        }
        return obj;
    }

    static void serialize_vector_i32(java.util.List<Integer> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (Integer item : value) {
            serializer.serialize_i32(item);
        }
    }

    static java.util.List<Integer> deserialize_vector_i32(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<Integer> obj = new java.util.ArrayList<Integer>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(deserializer.deserialize_i32());
        }
        return obj;
    }

    static void serialize_vector_set_str(java.util.List<java.util.List<String>> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (java.util.List<String> item : value) {
            TraitHelpers.serialize_set_str(item, serializer);
        }
    }

    static java.util.List<java.util.List<String>> deserialize_vector_set_str(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<java.util.List<String>> obj = new java.util.ArrayList<java.util.List<String>>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(TraitHelpers.deserialize_set_str(deserializer));
        }
        return obj;
    }

}

