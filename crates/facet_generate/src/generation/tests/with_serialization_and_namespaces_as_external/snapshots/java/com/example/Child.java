package com.example;


public final class Child {
    public final com.example2.other.OtherParent external;

    public Child(com.example2.other.OtherParent external) {
        java.util.Objects.requireNonNull(external, "external must not be null");
        this.external = external;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        external.serialize(serializer);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static Child deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.external = com.example2.other.OtherParent.deserialize(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static Child bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        Child value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Child other = (Child) obj;
        if (!java.util.Objects.equals(this.external, other.external)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.external != null ? this.external.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public com.example2.other.OtherParent external;

        public Child build() {
            return new Child(
                external
            );
        }
    }
}
