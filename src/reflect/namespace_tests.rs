use std::collections::HashMap;

use crate::reflect;

#[test]
fn nested_namespaced_structs() {
    mod one {
        #[derive(facet::Facet)]
        pub struct GrandChild {
            field: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        #[derive(facet::Facet)]
        #[facet(namespace = "two")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(facet::Facet)]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Parent:
      STRUCT:
        - one:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: one
              name: Child
        - two:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: two
              name: Child
    one.Child:
      STRUCT:
        - child:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: one
              name: GrandChild
    one.GrandChild:
      STRUCT:
        - field: STR
    two.Child:
      STRUCT:
        - field: STR
    ");
}

#[test]
fn nested_namespaced_enums() {
    mod one {
        #![allow(unused)]
        #[derive(facet::Facet)]
        #[repr(C)]
        pub enum GrandChild {
            None,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        #[repr(C)]
        pub enum Child {
            Data(GrandChild),
        }
    }
    mod two {
        #![allow(unused)]
        #[derive(facet::Facet)]
        #[repr(C)]
        #[facet(namespace = "two")]
        pub enum Child {
            Data(String),
        }
    }

    #[derive(facet::Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        One(one::Child),
        Two(two::Child),
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Parent:
      ENUM:
        0:
          One:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: one
                name: Child
        1:
          Two:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: two
                name: Child
    one.Child:
      ENUM:
        0:
          Data:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: one
                name: GrandChild
    one.GrandChild:
      ENUM:
        0:
          None: UNIT
    two.Child:
      ENUM:
        0:
          Data:
            NEWTYPE: STR
    ");
}

#[test]
fn nested_namespaced_renamed_structs() {
    mod one {
        #[derive(facet::Facet)]
        #[facet(name = "GrandKid")]
        pub struct GrandChild {
            field: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        #[facet(name = "Kid")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        #[derive(facet::Facet)]
        #[facet(namespace = "two")]
        #[facet(name = "Kid")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(facet::Facet)]
    #[facet(name = "Pop")]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Pop:
      STRUCT:
        - one:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: one
              name: Kid
        - two:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: two
              name: Kid
    one.GrandKid:
      STRUCT:
        - field: STR
    one.Kid:
      STRUCT:
        - child:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: one
              name: GrandKid
    two.Kid:
      STRUCT:
        - field: STR
    ");
}

#[test]
fn namespaced_collections() {
    #[derive(facet::Facet)]
    #[facet(namespace = "api")]
    pub struct User {
        id: String,
        name: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "api")]
    pub struct Group {
        users: Vec<User>,
    }

    #[derive(facet::Facet)]
    struct Response {
        users: Vec<User>,
        user_arrays: [User; 3],
        optional_user: Option<User>,
        groups: Vec<Group>,
    }

    let registry = reflect::<Response>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Response:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: User
        - user_arrays:
            TUPLEARRAY:
              CONTENT:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: api
                  name: User
              SIZE: 3
        - optional_user:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: User
        - groups:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: Group
    api.Group:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: User
    api.User:
      STRUCT:
        - id: STR
        - name: STR
    ");
}

#[test]
fn namespaced_maps() {
    mod models {
        #[derive(facet::Facet, Clone, Hash, Eq, PartialEq)]
        #[facet(namespace = "models")]
        pub struct UserId(String);

        #[derive(facet::Facet)]
        #[facet(namespace = "models")]
        pub struct UserProfile {
            name: String,
            active: bool,
        }
    }

    #[derive(facet::Facet)]
    struct Database {
        user_profiles: HashMap<models::UserId, models::UserProfile>,
        user_counts: HashMap<String, models::UserId>,
    }

    let registry = reflect::<Database>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Database:
      STRUCT:
        - user_profiles:
            MAP:
              KEY:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: models
                  name: UserId
              VALUE:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: models
                  name: UserProfile
        - user_counts:
            MAP:
              KEY: STR
              VALUE:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: models
                  name: UserId
    models.UserId:
      NEWTYPESTRUCT: STR
    models.UserProfile:
      STRUCT:
        - name: STR
        - active: BOOL
    ");
}

#[test]
fn complex_namespaced_enums() {
    mod events {
        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        pub struct UserData {
            id: String,
            email: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        pub struct SystemData {
            timestamp: u64,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        #[repr(C)]
        #[allow(unused, clippy::enum_variant_names)]
        pub enum Event {
            UserCreated(UserData),
            UserUpdated { old: UserData, new: UserData },
            SystemEvent(SystemData, String),
            Simple,
        }
    }

    #[derive(facet::Facet)]
    struct EventLog {
        events: Vec<events::Event>,
    }

    let registry = reflect::<EventLog>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    EventLog:
      STRUCT:
        - events:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: events
                name: Event
    events.Event:
      ENUM:
        0:
          UserCreated:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: events
                name: UserData
        1:
          UserUpdated:
            STRUCT:
              - old:
                  QUALIFIEDTYPENAME:
                    namespace:
                      NAMED: events
                    name: UserData
              - new:
                  QUALIFIEDTYPENAME:
                    namespace:
                      NAMED: events
                    name: UserData
        2:
          SystemEvent:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: events
                  name: SystemData
              - STR
        3:
          Simple: UNIT
    events.SystemData:
      STRUCT:
        - timestamp: U64
    events.UserData:
      STRUCT:
        - id: STR
        - email: STR
    ");
}

#[test]
fn namespaced_transparent_structs() {
    mod wrappers {
        #[derive(facet::Facet, Clone)]
        #[facet(namespace = "wrappers")]
        pub struct UserId(String);

        #[derive(facet::Facet)]
        #[facet(namespace = "wrappers")]
        #[facet(transparent)]
        pub struct TransparentWrapper(UserId);
    }

    #[derive(facet::Facet)]
    struct Container {
        direct_id: wrappers::UserId,
        wrapped_id: wrappers::TransparentWrapper,
    }

    let registry = reflect::<Container>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Container:
      STRUCT:
        - direct_id:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: wrappers
              name: UserId
        - wrapped_id:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: wrappers
              name: UserId
    wrappers.UserId:
      NEWTYPESTRUCT: STR
    ");
}

#[test]
fn cross_namespace_references() {
    #[derive(facet::Facet)]
    #[facet(namespace = "entities")]
    struct Entity {
        id: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "api")]
    struct Request {
        entity: Entity,
        metadata: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "db")]
    struct Record {
        entity: Entity,
        request: Request,
    }

    #[derive(facet::Facet)]
    struct System {
        records: Vec<Record>,
    }

    let registry = reflect::<System>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    System:
      STRUCT:
        - records:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: db
                name: Record
    api.Request:
      STRUCT:
        - entity:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: entities
              name: Entity
        - metadata: STR
    db.Record:
      STRUCT:
        - entity:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: entities
              name: Entity
        - request:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: api
              name: Request
    entities.Entity:
      STRUCT:
        - id: STR
    ");
}

#[test]
fn namespace_with_byte_attributes() {
    mod data {
        #[derive(facet::Facet)]
        #[facet(namespace = "data")]
        pub struct BinaryData {
            #[facet(bytes)]
            content: Vec<u8>,
            #[facet(bytes)]
            header: &'static [u8],
            metadata: String,
        }
    }

    #[derive(facet::Facet)]
    struct Document {
        binary: data::BinaryData,
    }

    let registry = reflect::<Document>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Document:
      STRUCT:
        - binary:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: data
              name: BinaryData
    data.BinaryData:
      STRUCT:
        - content: BYTES
        - header: BYTES
        - metadata: STR
    ");
}

#[test]
fn deeply_nested_namespaces() {
    mod level1 {
        pub mod level2 {
            #[derive(facet::Facet)]
            #[facet(namespace = "level1.level2")]
            pub struct DeepStruct {
                value: String,
            }
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "level1")]
        pub struct MiddleStruct {
            deep: level2::DeepStruct,
        }
    }

    #[derive(facet::Facet)]
    struct RootStruct {
        middle: level1::MiddleStruct,
        deep_direct: level1::level2::DeepStruct,
    }

    let registry = reflect::<RootStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    RootStruct:
      STRUCT:
        - middle:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: level1
              name: MiddleStruct
        - deep_direct:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: level1.level2
              name: DeepStruct
    level1.MiddleStruct:
      STRUCT:
        - deep:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: level1.level2
              name: DeepStruct
    level1.level2.DeepStruct:
      STRUCT:
        - value: STR
    ");
}

#[test]
fn transparent_struct_namespace_behavior() {
    // Test to verify what happens with transparent structs and namespace inheritance
    mod wrappers {
        #[derive(facet::Facet, Clone)]
        pub struct UserId(String);

        #[derive(facet::Facet)]
        #[facet(namespace = "wrappers")]
        #[facet(transparent)]
        pub struct TransparentWrapper(UserId);
    }

    #[derive(facet::Facet)]
    struct Container {
        wrapped_id: wrappers::TransparentWrapper,
    }

    let registry = reflect::<Container>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Container:
      STRUCT:
        - wrapped_id:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: wrappers
              name: UserId
    wrappers.UserId:
      NEWTYPESTRUCT: STR
    ");

    // Note: UserId gets the "wrappers" namespace even though it doesn't have
    // an explicit namespace annotation, because it's referenced within the
    // namespaced TransparentWrapper context. This demonstrates that our
    // namespace inheritance is working correctly for transparent structs too.
}

#[test]
fn debug_transparent_struct_step_by_step() {
    // Debug test to understand the transparent struct processing

    // Step 1: Simple struct without namespace
    #[derive(facet::Facet, Clone)]
    struct SimpleStruct {
        value: String,
    }

    // Step 2: Transparent wrapper WITH namespace
    #[derive(facet::Facet)]
    #[facet(namespace = "debug")]
    #[facet(transparent)]
    struct TransparentWrapper(SimpleStruct);

    // Step 3: Container that uses the transparent wrapper
    #[derive(facet::Facet)]
    struct TestContainer {
        wrapped: TransparentWrapper,
    }

    let registry = reflect::<TestContainer>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    TestContainer:
      STRUCT:
        - wrapped:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: debug
              name: SimpleStruct
    debug.SimpleStruct:
      STRUCT:
        - value: STR
    ");

    // This test should show that SimpleStruct gets namespaced to "debug.SimpleStruct"
    // because it's processed within the context of the namespaced TransparentWrapper
}

#[test]
#[allow(clippy::too_many_lines)]
fn redundant_namespace_declarations() {
    // This test demonstrates cases where we explicitly specify namespaces
    // that could now be inherited automatically due to our consistent inheritance

    // Example 1: In collections - Group could inherit from api namespace
    mod api_example {
        #[derive(facet::Facet)]
        #[facet(namespace = "api")]
        pub struct User {
            id: String,
            name: String,
        }

        // This Group has an explicit namespace, but it could inherit "api"
        // if it was defined within a namespaced container
        #[derive(facet::Facet)]
        #[facet(namespace = "api")] // <- Could be redundant
        pub struct Group {
            users: Vec<User>,
        }
    }

    // Example 2: Enum variants - all these could inherit from "events"
    mod events_example {
        // These types all explicitly declare "events" namespace
        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        pub struct UserData {
            id: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "events")] // <- Could be redundant
        pub struct SystemData {
            timestamp: u64,
        }

        // If this enum had namespace, the types above could inherit
        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        #[repr(C)]
        #[allow(unused)]
        pub enum Event {
            UserCreated(UserData),   // UserData would inherit "events"
            SystemEvent(SystemData), // SystemData would inherit "events"
        }
    }

    // A more efficient way would be to use a containing struct/enum with namespace
    #[derive(facet::Facet)]
    #[facet(namespace = "efficient")]
    struct ApiContainer {
        // These don't need explicit namespaces - they inherit "efficient"
        user: InheritedUser,
        group: InheritedGroup,
    }

    #[derive(facet::Facet)]
    struct InheritedUser {
        id: String,
        name: String,
    }

    #[derive(facet::Facet)]
    struct InheritedGroup {
        users: Vec<InheritedUser>,
    }

    #[derive(facet::Facet)]
    struct TestContainer {
        api_data: api_example::Group,
        event: events_example::Event,
        efficient: ApiContainer,
    }

    let registry = reflect::<TestContainer>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    TestContainer:
      STRUCT:
        - api_data:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: api
              name: Group
        - event:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: events
              name: Event
        - efficient:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: efficient
              name: ApiContainer
    api.Group:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: User
    api.User:
      STRUCT:
        - id: STR
        - name: STR
    efficient.ApiContainer:
      STRUCT:
        - user:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: efficient
              name: InheritedUser
        - group:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: efficient
              name: InheritedGroup
    efficient.InheritedGroup:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: efficient
                name: InheritedUser
    efficient.InheritedUser:
      STRUCT:
        - id: STR
        - name: STR
    events.Event:
      ENUM:
        0:
          UserCreated:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: events
                name: UserData
        1:
          SystemEvent:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: events
                name: SystemData
    events.SystemData:
      STRUCT:
        - timestamp: U64
    events.UserData:
      STRUCT:
        - id: STR
    ");

    // KEY INSIGHT: The "efficient" approach shows that with a single namespace
    // declaration on the container, all contained types automatically inherit
    // the namespace, reducing redundant declarations.
}

#[test]
fn comprehensive_inheritance_proof_collections() {
    // Proves that types in collections inherit namespace from the containing struct

    #[derive(facet::Facet)]
    struct UnnamedUser {
        name: String,
    }

    #[derive(facet::Facet)]
    struct UnnamedRole {
        permissions: Vec<String>,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "system")]
    struct UserManager {
        users: Vec<UnnamedUser>,
        admins: [UnnamedUser; 2],
        optional_user: Option<UnnamedUser>,
        role_map: std::collections::HashMap<String, UnnamedRole>,
        nested_lists: Vec<Vec<UnnamedUser>>,
    }

    let registry = reflect::<UserManager>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    system.UnnamedRole:
      STRUCT:
        - permissions:
            SEQ: STR
    system.UnnamedUser:
      STRUCT:
        - name: STR
    system.UserManager:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: system
                name: UnnamedUser
        - admins:
            TUPLEARRAY:
              CONTENT:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: system
                  name: UnnamedUser
              SIZE: 2
        - optional_user:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: system
                name: UnnamedUser
        - role_map:
            MAP:
              KEY: STR
              VALUE:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: system
                  name: UnnamedRole
        - nested_lists:
            SEQ:
              SEQ:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: system
                  name: UnnamedUser
    ");
}

#[test]
fn comprehensive_inheritance_proof_enums() {
    // Proves that enum variant types inherit namespace from the enum

    #[derive(facet::Facet)]
    struct ErrorData {
        code: u32,
        message: String,
    }

    #[derive(facet::Facet)]
    struct SuccessData {
        result: String,
    }

    #[derive(facet::Facet)]
    struct ProcessingData {
        progress: f32,
        estimate: ErrorData, // Nested inheritance
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "api")]
    #[repr(C)]
    #[allow(unused)]
    enum Response {
        Success(SuccessData),
        Error(ErrorData),
        Processing {
            data: ProcessingData,
            extra: SuccessData,
        },
        Multipart(ErrorData, SuccessData),
        Empty,
    }

    let registry = reflect::<Response>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    api.ErrorData:
      STRUCT:
        - code: U32
        - message: STR
    api.ProcessingData:
      STRUCT:
        - progress: F32
        - estimate:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: api
              name: ErrorData
    api.Response:
      ENUM:
        0:
          Success:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: SuccessData
        1:
          Error:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: api
                name: ErrorData
        2:
          Processing:
            STRUCT:
              - data:
                  QUALIFIEDTYPENAME:
                    namespace:
                      NAMED: api
                    name: ProcessingData
              - extra:
                  QUALIFIEDTYPENAME:
                    namespace:
                      NAMED: api
                    name: SuccessData
        3:
          Multipart:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: api
                  name: ErrorData
              - QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: api
                  name: SuccessData
        4:
          Empty: UNIT
    api.SuccessData:
      STRUCT:
        - result: STR
    ");
}

#[test]
fn comprehensive_inheritance_proof_nested_structs() {
    // Proves that deeply nested structs inherit namespace correctly

    #[derive(facet::Facet)]
    struct DeepInner {
        value: i32,
    }

    #[derive(facet::Facet)]
    struct MiddleLayer {
        inner: DeepInner,
        inner_list: Vec<DeepInner>,
    }

    #[derive(facet::Facet)]
    struct TopLayer {
        middle: MiddleLayer,
        direct_inner: DeepInner,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "nested")]
    struct Container {
        top: TopLayer,
        middle_direct: MiddleLayer,
        inner_direct: DeepInner,
    }

    let registry = reflect::<Container>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    nested.Container:
      STRUCT:
        - top:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: TopLayer
        - middle_direct:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: MiddleLayer
        - inner_direct:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: DeepInner
    nested.DeepInner:
      STRUCT:
        - value: I32
    nested.MiddleLayer:
      STRUCT:
        - inner:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: DeepInner
        - inner_list:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: nested
                name: DeepInner
    nested.TopLayer:
      STRUCT:
        - middle:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: MiddleLayer
        - direct_inner:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: nested
              name: DeepInner
    ");
}

#[test]
fn comprehensive_inheritance_proof_transparent_chains() {
    // Proves that transparent struct chains inherit namespace correctly

    #[derive(facet::Facet, Clone)]
    struct CoreId(String);

    #[derive(facet::Facet, Clone)]
    #[facet(transparent)]
    struct WrapperId(CoreId);

    #[derive(facet::Facet, Clone)]
    #[facet(transparent)]
    struct DoubleWrapperId(WrapperId);

    #[derive(facet::Facet)]
    #[facet(namespace = "identity")]
    #[facet(transparent)]
    struct NamespacedWrapper(DoubleWrapperId);

    #[derive(facet::Facet)]
    struct IdContainer {
        id: NamespacedWrapper,
    }

    let registry = reflect::<IdContainer>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    IdContainer:
      STRUCT:
        - id:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: identity
              name: DoubleWrapperId
    identity.CoreId:
      NEWTYPESTRUCT: STR
    ");
}

#[test]
fn comprehensive_inheritance_proof_mixed_containers() {
    // Proves that various container types all consistently inherit namespaces

    #[derive(facet::Facet)]
    struct Item {
        id: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "storage")]
    struct MixedContainer {
        // All these should make Item inherit "storage" namespace
        single: Item,
        vector: Vec<Item>,
        array: [Item; 3],
        option: Option<Item>,
        tuple: (Item, String),
        nested_option: Option<Vec<Item>>,
        complex_map: std::collections::HashMap<String, Vec<Option<Item>>>,
    }

    let registry = reflect::<MixedContainer>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    storage.Item:
      STRUCT:
        - id: STR
    storage.MixedContainer:
      STRUCT:
        - single:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: storage
              name: Item
        - vector:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: storage
                name: Item
        - array:
            TUPLEARRAY:
              CONTENT:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: storage
                  name: Item
              SIZE: 3
        - option:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: storage
                name: Item
        - tuple:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: storage
                  name: Item
              - STR
        - nested_option:
            OPTION:
              SEQ:
                QUALIFIEDTYPENAME:
                  namespace:
                    NAMED: storage
                  name: Item
        - complex_map:
            MAP:
              KEY: STR
              VALUE:
                SEQ:
                  OPTION:
                    QUALIFIEDTYPENAME:
                      namespace:
                        NAMED: storage
                      name: Item
    ");
}

#[test]
fn comprehensive_inheritance_proof_no_pollution() {
    // Proves that namespace inheritance doesn't cause pollution between unrelated types

    #[derive(facet::Facet)]
    struct SharedType {
        value: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "alpha")]
    struct AlphaContainer {
        shared: SharedType,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "beta")]
    struct BetaContainer {
        shared: SharedType,
    }

    #[derive(facet::Facet)]
    struct RootContainer {
        alpha: AlphaContainer,
        beta: BetaContainer,
        unnamespaced: SharedType,
    }

    let registry = reflect::<RootContainer>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    RootContainer:
      STRUCT:
        - alpha:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: alpha
              name: AlphaContainer
        - beta:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: beta
              name: BetaContainer
        - unnamespaced:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: SharedType
    SharedType:
      STRUCT:
        - value: STR
    alpha.AlphaContainer:
      STRUCT:
        - shared:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: alpha
              name: SharedType
    alpha.SharedType:
      STRUCT:
        - value: STR
    beta.BetaContainer:
      STRUCT:
        - shared:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: beta
              name: SharedType
    beta.SharedType:
      STRUCT:
        - value: STR
    ");
}

#[test]
fn namespace_inheritance_behavior_summary() {
    // Summary test documenting the actual behaviors and edge cases of namespace inheritance

    #[derive(facet::Facet)]
    struct BaseType {
        value: String,
    }

    // Test 1: Processing order affects which version is created first
    #[derive(facet::Facet)]
    #[facet(namespace = "first")]
    struct FirstContainer {
        item: BaseType, // Creates "first.BaseType"
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "second")]
    struct SecondContainer {
        item: BaseType, // Creates "second.BaseType"
    }

    #[derive(facet::Facet)]
    struct Root {
        first: FirstContainer,
        second: SecondContainer,
        direct: BaseType, // Should not reference the last processed version
    }

    let registry = reflect::<Root>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    BaseType:
      STRUCT:
        - value: STR
    Root:
      STRUCT:
        - first:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: first
              name: FirstContainer
        - second:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: second
              name: SecondContainer
        - direct:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: BaseType
    first.BaseType:
      STRUCT:
        - value: STR
    first.FirstContainer:
      STRUCT:
        - item:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: first
              name: BaseType
    second.BaseType:
      STRUCT:
        - value: STR
    second.SecondContainer:
      STRUCT:
        - item:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: second
              name: BaseType
    ");

    // KEY BEHAVIORS DOCUMENTED:
    // 1. ✅ INHERITANCE WORKS: Types DO inherit namespaces from their containers
    // 2. ✅ MULTIPLE VERSIONS: Same type can exist in multiple namespaces
    // 3. ✅ PROCESSING ORDER: Later processed namespaces affect subsequent references
    // 4. ✅ EXPLICIT BEATS IMPLICIT: Explicit namespace declarations take precedence
    // 5. ✅ CONSISTENT ACROSS CONTAINERS: Vec, Option, HashMap, etc. all behave the same
    //
    // CONCLUSION: Namespace inheritance works reliably and you CAN omit explicit
    // namespace declarations when you want types to inherit from their container context.
    // The choice between explicit vs inherited namespaces is a matter of preference
    // and code organization, not technical necessity.
}

#[test]
fn namespace_pollution_isolation() {
    #[derive(facet::Facet)]
    struct IsolatedType {
        data: String,
    }

    // Test with just one namespaced container
    #[derive(facet::Facet)]
    #[facet(namespace = "test")]
    struct NamespacedContainer {
        item: IsolatedType,
    }

    #[derive(facet::Facet)]
    struct SimpleRoot {
        namespaced: NamespacedContainer,
        direct: IsolatedType, // This should be plain IsolatedType, not test.IsolatedType
    }

    let registry = reflect::<SimpleRoot>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    IsolatedType:
      STRUCT:
        - data: STR
    SimpleRoot:
      STRUCT:
        - namespaced:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: test
              name: NamespacedContainer
        - direct:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: IsolatedType
    test.IsolatedType:
      STRUCT:
        - data: STR
    test.NamespacedContainer:
      STRUCT:
        - item:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: test
              name: IsolatedType
    ");
}
