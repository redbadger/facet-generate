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
    insta::assert_yaml_snapshot!(registry, @r"
    GrandChild:
      STRUCT:
        - field: STR
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
              namespace: ROOT
              name: GrandChild
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
        #[allow(unused)]
        pub enum Child {
            Data(GrandChild),
        }
    }
    mod two {
        #[derive(facet::Facet)]
        #[facet(namespace = "two")]
        #[repr(C)]
        #[allow(unused)]
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
    insta::assert_yaml_snapshot!(registry, @r"
    GrandChild:
      ENUM:
        0:
          None: UNIT
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
                namespace: ROOT
                name: GrandChild
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
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry, @r"
    GrandKid:
      STRUCT:
        - field: STR
    Parent:
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
    one.Kid:
      STRUCT:
        - child:
            QUALIFIEDTYPENAME:
              namespace: ROOT
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
        user_arrays: [User; 5],
        optional_user: Option<User>,
        groups: Vec<Group>,
    }

    let registry = reflect::<Response>();
    insta::assert_yaml_snapshot!(registry, @r"
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
              SIZE: 5
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
        user_counts: HashMap<String, u32>,
    }

    let registry = reflect::<Database>();
    insta::assert_yaml_snapshot!(registry, @r"
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
              VALUE: U32
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
        #[allow(unused)]
        #[allow(clippy::enum_variant_names)]
        pub enum Event {
            UserCreated(UserData),
            UserUpdated { old: UserData, new: UserData },
            SystemEvent(SystemData),
            Simple,
        }
    }

    #[derive(facet::Facet)]
    struct EventLog {
        events: Vec<events::Event>,
    }

    let registry = reflect::<EventLog>();
    insta::assert_yaml_snapshot!(registry, @r"
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
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: events
                name: SystemData
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
    insta::assert_yaml_snapshot!(registry, @r"
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
    #[facet(namespace = "storage")]
    struct Record {
        entity: Entity,
        request: Request,
    }

    #[derive(facet::Facet)]
    struct System {
        records: Vec<Record>,
    }

    let registry = reflect::<System>();
    insta::assert_yaml_snapshot!(registry, @r"
    System:
      STRUCT:
        - records:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace:
                  NAMED: storage
                name: Record
    api.Request:
      STRUCT:
        - entity:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: entities
              name: Entity
        - metadata: STR
    entities.Entity:
      STRUCT:
        - id: STR
    storage.Record:
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
    insta::assert_yaml_snapshot!(registry, @r"
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
    insta::assert_yaml_snapshot!(registry, @r"
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
fn transparent_struct_explicit_namespace() {
    // Test transparent structs with explicit namespace annotations
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
    insta::assert_yaml_snapshot!(registry, @r"
    Container:
      STRUCT:
        - wrapped_id:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: UserId
    UserId:
      NEWTYPESTRUCT: STR
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
fn redundant_namespace_declarations() {
    // This test demonstrates explicit namespace declarations
    mod api_example {
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
    }

    mod events_example {
        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        pub struct UserData {
            id: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        pub struct SystemData {
            timestamp: u64,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "events")]
        #[repr(C)]
        #[allow(unused)]
        pub enum Event {
            UserCreated(UserData),
            SystemEvent(SystemData),
        }
    }

    #[derive(facet::Facet)]
    struct ApiContainer {
        #[facet(name = "user")]
        user: api_example::User,
        group: api_example::Group,
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
        api_data: ApiContainer,
        event: events_example::Event,
        efficient: InheritedGroup,
    }

    let registry = reflect::<TestContainer>();
    insta::assert_yaml_snapshot!(registry, @r"
    ApiContainer:
      STRUCT:
        - user:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: api
              name: User
        - group:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: api
              name: Group
    InheritedGroup:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: InheritedUser
    InheritedUser:
      STRUCT:
        - id: STR
        - name: STR
    TestContainer:
      STRUCT:
        - api_data:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: ApiContainer
        - event:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: events
              name: Event
        - efficient:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: InheritedGroup
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
}

#[test]
fn collections_with_explicit_namespace() {
    // Test that types in collections go to root namespace when no explicit namespace is given
    #[derive(facet::Facet)]
    struct UnnamedUser {
        name: String,
    }

    #[derive(facet::Facet)]
    struct UnnamedRole {
        permissions: Vec<String>,
    }

    // Container with explicit namespace
    #[derive(facet::Facet)]
    #[facet(namespace = "system")]
    struct UserManager {
        users: Vec<UnnamedUser>,
        admins: [UnnamedUser; 2],
        optional_user: Option<UnnamedUser>,
        role_map: HashMap<String, UnnamedRole>,
        nested_lists: Vec<Vec<UnnamedUser>>,
    }

    let registry = reflect::<UserManager>();
    insta::assert_yaml_snapshot!(registry, @r"
    UnnamedRole:
      STRUCT:
        - permissions:
            SEQ: STR
    UnnamedUser:
      STRUCT:
        - name: STR
    system.UserManager:
      STRUCT:
        - users:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: UnnamedUser
        - admins:
            TUPLEARRAY:
              CONTENT:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: UnnamedUser
              SIZE: 2
        - optional_user:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: UnnamedUser
        - role_map:
            MAP:
              KEY: STR
              VALUE:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: UnnamedRole
        - nested_lists:
            SEQ:
              SEQ:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: UnnamedUser
    ");
}

#[test]
fn enums_with_explicit_namespace() {
    // Test that enum variant types go to root namespace when no explicit namespace is given
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
        estimate: ErrorData,
    }

    // Enum with explicit namespace
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
    insta::assert_yaml_snapshot!(registry, @r"
    ErrorData:
      STRUCT:
        - code: U32
        - message: STR
    ProcessingData:
      STRUCT:
        - progress: F32
        - estimate:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: ErrorData
    SuccessData:
      STRUCT:
        - result: STR
    api.Response:
      ENUM:
        0:
          Success:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: SuccessData
        1:
          Error:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: ErrorData
        2:
          Processing:
            STRUCT:
              - data:
                  QUALIFIEDTYPENAME:
                    namespace: ROOT
                    name: ProcessingData
              - extra:
                  QUALIFIEDTYPENAME:
                    namespace: ROOT
                    name: SuccessData
        3:
          Multipart:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: ErrorData
              - QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: SuccessData
        4:
          Empty: UNIT
    ");
}

#[test]
fn nested_structs_with_explicit_namespace() {
    // Test that deeply nested structs go to root namespace when no explicit namespace is given
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

    // Container with explicit namespace
    #[derive(facet::Facet)]
    #[facet(namespace = "nested")]
    struct Container {
        top: TopLayer,
        middle_direct: MiddleLayer,
        inner_direct: DeepInner,
    }

    let registry = reflect::<Container>();
    insta::assert_yaml_snapshot!(registry, @r"
    DeepInner:
      STRUCT:
        - value: I32
    MiddleLayer:
      STRUCT:
        - inner:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: DeepInner
        - inner_list:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: DeepInner
    TopLayer:
      STRUCT:
        - middle:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: MiddleLayer
        - direct_inner:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: DeepInner
    nested.Container:
      STRUCT:
        - top:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: TopLayer
        - middle_direct:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: MiddleLayer
        - inner_direct:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: DeepInner
    ");
}

#[test]
fn transparent_struct_chains() {
    // Test transparent struct chains - they should resolve to the final non-transparent type
    #[derive(facet::Facet, Clone)]
    struct CoreId(String);

    #[derive(facet::Facet, Clone)]
    #[facet(transparent)]
    struct WrapperId(CoreId);

    #[derive(facet::Facet, Clone)]
    #[facet(transparent)]
    struct DoubleWrapperId(WrapperId);

    // Container with explicit namespace
    #[derive(facet::Facet)]
    #[facet(namespace = "identity")]
    struct NamespacedWrapper(DoubleWrapperId);

    #[derive(facet::Facet)]
    struct IdContainer {
        id: NamespacedWrapper,
    }

    let registry = reflect::<IdContainer>();
    insta::assert_yaml_snapshot!(registry, @r"
    CoreId:
      NEWTYPESTRUCT: STR
    IdContainer:
      STRUCT:
        - id:
            QUALIFIEDTYPENAME:
              namespace:
                NAMED: identity
              name: NamespacedWrapper
    identity.NamespacedWrapper:
      NEWTYPESTRUCT:
        QUALIFIEDTYPENAME:
          namespace: ROOT
          name: DoubleWrapperId
    ");
}

#[test]
fn mixed_containers_with_explicit_namespace() {
    // Test that various container types correctly reference root namespace types
    #[derive(facet::Facet)]
    struct Item {
        id: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "storage")]
    struct MixedContainer {
        single: Item,
        vector: Vec<Item>,
        array: [Item; 3],
        option: Option<Item>,
        tuple: (Item, String),
        nested_option: Option<Vec<Item>>,
        complex_map: HashMap<String, Vec<Option<Item>>>,
    }

    let registry = reflect::<MixedContainer>();
    insta::assert_yaml_snapshot!(registry, @r"
    Item:
      STRUCT:
        - id: STR
    storage.MixedContainer:
      STRUCT:
        - single:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: Item
        - vector:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Item
        - array:
            TUPLEARRAY:
              CONTENT:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Item
              SIZE: 3
        - option:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Item
        - tuple:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Item
              - STR
        - nested_option:
            OPTION:
              SEQ:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Item
        - complex_map:
            MAP:
              KEY: STR
              VALUE:
                SEQ:
                  OPTION:
                    QUALIFIEDTYPENAME:
                      namespace: ROOT
                      name: Item
    ");
}

#[test]
fn no_namespace_pollution() {
    // Test that types without explicit namespaces don't get duplicated across multiple namespaces
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
    insta::assert_yaml_snapshot!(registry, @r"
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
              namespace: ROOT
              name: SharedType
    beta.BetaContainer:
      STRUCT:
        - shared:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: SharedType
    ");
}

#[test]
fn explicit_namespace_behavior_summary() {
    // Summary test documenting explicit namespace behavior
    #[derive(facet::Facet)]
    struct BaseType {
        value: String,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "first")]
    struct FirstContainer {
        item: BaseType,
    }

    #[derive(facet::Facet)]
    #[facet(namespace = "second")]
    struct SecondContainer {
        item: BaseType,
    }

    #[derive(facet::Facet)]
    struct Root {
        first: FirstContainer,
        second: SecondContainer,
        direct: BaseType,
    }

    let registry = reflect::<Root>();
    insta::assert_yaml_snapshot!(registry, @r"
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
    first.FirstContainer:
      STRUCT:
        - item:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: BaseType
    second.SecondContainer:
      STRUCT:
        - item:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: BaseType
    ");
}
