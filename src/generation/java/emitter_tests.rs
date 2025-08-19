use std::collections::HashMap;

use facet::Facet;

use crate::generation::{
    CodeGeneratorConfig, Encoding,
    indent::{IndentConfig, IndentedWriter},
    java::{CodeGenerator, emitter::JavaEmitter},
};

macro_rules! emit {
    ($($ty:ident),* as $encoding:path) => {
        || -> anyhow::Result<String> {
            let mut out = Vec::new();
            let w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let config = CodeGeneratorConfig::new("com.example".to_string())
                .with_encoding($encoding);
            let generator = CodeGenerator::new(&config);
            let mut emitter = JavaEmitter {
                out: w,
                generator: &generator,
                current_namespace: Vec::new(),
                current_reserved_names: HashMap::new(),
            };
            let registry = $crate::reflect!($($ty),*);
            for (name, format) in &registry {
                emitter.output_container(&name.name, format).unwrap();
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @r"
    public final class UnitStruct {
        public UnitStruct() {
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            UnitStruct other = (UnitStruct) obj;
            return true;
        }

        public int hashCode() {
            int value = 7;
            return value;
        }

        public static final class Builder {
            public UnitStruct build() {
                return new UnitStruct(
                );
            }
        }
    }
    ");

    let actual = emit!(UnitStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class UnitStruct {
        public UnitStruct() {
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.decrease_container_depth();
        }

        public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static UnitStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static UnitStruct bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
            UnitStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            UnitStruct other = (UnitStruct) obj;
            return true;
        }

        public int hashCode() {
            int value = 7;
            return value;
        }

        public static final class Builder {
            public UnitStruct build() {
                return new UnitStruct(
                );
            }
        }
    }
    "#);
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit!(NewType as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class NewType {
        public final String value;

        public NewType(String value) {
            java.util.Objects.requireNonNull(value, "value must not be null");
            this.value = value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            NewType other = (NewType) obj;
            if (!java.util.Objects.equals(this.value, other.value)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String value;

            public NewType build() {
                return new NewType(
                    value
                );
            }
        }
    }
    "#);

    let actual = emit!(NewType as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class NewType {
        public final String value;

        public NewType(String value) {
            java.util.Objects.requireNonNull(value, "value must not be null");
            this.value = value;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(value);
            serializer.decrease_container_depth();
        }

        public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static NewType deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.value = deserializer.deserialize_str();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static NewType bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
            NewType value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            NewType other = (NewType) obj;
            if (!java.util.Objects.equals(this.value, other.value)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String value;

            public NewType build() {
                return new NewType(
                    value
                );
            }
        }
    }
    "#);
}
