//! these tests will be updated once the Swift emitter is converted
//! to use the new Emitter<Language> trait

use facet::Facet;

use crate::{
    generation::{
        CodeGeneratorConfig,
        indent::{IndentConfig, IndentedWriter},
        swift::{CodeGenerator, emitter::SwiftEmitter},
    },
    reflection::RegistryBuilder,
};

#[test]
fn unit_struct_1() {
    /// comments not yet supported
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let registry = RegistryBuilder::new().add_type::<UnitStruct>().build();
    let config = CodeGeneratorConfig::new("Root".to_string()).without_serialization();
    let generator = CodeGenerator::new(&config);
    let mut out = vec![];
    let mut emitter = SwiftEmitter {
        out: IndentedWriter::new(&mut out, IndentConfig::Space(4)),
        generator: &generator,
        current_namespace: Vec::new(),
    };
    for (name, format) in &registry {
        emitter.output_container(&name.name, format).unwrap();
    }

    let actual = String::from_utf8(out).unwrap();
    insta::assert_snapshot!(actual, @r"
    public struct UnitStruct: Hashable {

        public init() {
        }
    }
    ");
}

#[test]
fn enum_with_tuple_variant() {
    #[derive(Facet)]
    #[repr(C)]
    enum MyEnum {
        Variant1(String, i32),
        Variant2(f64),
    }

    let registry = RegistryBuilder::new().add_type::<MyEnum>().build();
    let config = CodeGeneratorConfig::new("Root".to_string()).without_serialization();
    let generator = CodeGenerator::new(&config);
    let mut out = vec![];
    let mut emitter = SwiftEmitter {
        out: IndentedWriter::new(&mut out, IndentConfig::Space(4)),
        generator: &generator,
        current_namespace: Vec::new(),
    };
    for (name, format) in &registry {
        emitter.output_container(&name.name, format).unwrap();
    }

    let actual = String::from_utf8(out).unwrap();
    insta::assert_snapshot!(actual, @r"
    indirect public enum MyEnum: Hashable {
        case variant1(String, Int32)
        case variant2(Double)
    }
    ");
}

#[test]
fn enum_with_tuple_variant_containing_vecs() {
    use facet::Facet;

    #[derive(Facet)]
    pub struct Test1 {
        field1: String,
        field2: i32,
    }

    #[derive(Facet)]
    pub struct Test2 {
        field1: String,
        field2: i32,
    }

    #[derive(Facet)]
    #[repr(C)]
    enum MyEnum {
        MyVariant(Vec<Test1>, Vec<Test2>),
    }

    let registry = RegistryBuilder::new().add_type::<MyEnum>().build();
    let config = CodeGeneratorConfig::new("Root".to_string()).without_serialization();
    let generator = CodeGenerator::new(&config);
    let mut out = vec![];
    let mut emitter = SwiftEmitter {
        out: IndentedWriter::new(&mut out, IndentConfig::Space(4)),
        generator: &generator,
        current_namespace: Vec::new(),
    };

    insta::assert_yaml_snapshot!(&registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        0:
          MyVariant:
            TUPLE:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: Test1
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: Test2
    ? namespace: ROOT
      name: Test1
    : STRUCT:
        - field1: STR
        - field2: I32
    ? namespace: ROOT
      name: Test2
    : STRUCT:
        - field1: STR
        - field2: I32
    ");

    for (name, format) in &registry {
        emitter.output_container(&name.name, format).unwrap();
    }

    let actual = String::from_utf8(out).unwrap();
    insta::assert_snapshot!(actual, @r"
    indirect public enum MyEnum: Hashable {
        case myVariant([Test1], [Test2])
    }

    public struct Test1: Hashable {
        @Indirect public var field1: String
        @Indirect public var field2: Int32

        public init(field1: String, field2: Int32) {
            self.field1 = field1
            self.field2 = field2
        }
    }

    public struct Test2: Hashable {
        @Indirect public var field1: String
        @Indirect public var field2: Int32

        public init(field1: String, field2: Int32) {
            self.field1 = field1
            self.field2 = field2
        }
    }
    ");
}
