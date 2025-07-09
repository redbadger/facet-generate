package com.example;


public final class Child {
    public final ExternalParent external;

    public Child(ExternalParent external) {
        java.util.Objects.requireNonNull(external, "external must not be null");
        this.external = external;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        external.serialize(serializer);
        serializer.decrease_container_depth();
    }

    public static Child deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.external = ExternalParent.deserialize(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
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
        public ExternalParent external;

        public Child build() {
            return new Child(
                external
            );
        }
    }
}
