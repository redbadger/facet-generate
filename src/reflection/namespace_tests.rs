use std::collections::HashMap;

use facet::Facet;

use crate::reflect;

#[test]
fn nested_namespaced_structs() {
    mod one {
        use facet::Facet;

        #[derive(Facet)]
        pub struct GrandChild {
            field: String,
        }

        #[derive(Facet)]
        #[facet(namespace = "one")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "two")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(Facet)]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect!(Parent);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: GrandChild
    : STRUCT:
        - - field:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: Parent
    : STRUCT:
        - - one:
              - TYPENAME:
                  namespace:
                    NAMED: one
                  name: Child
              - []
          - two:
              - TYPENAME:
                  namespace:
                    NAMED: two
                  name: Child
              - []
        - []
    ? namespace:
        NAMED: one
      name: Child
    : STRUCT:
        - - child:
              - TYPENAME:
                  namespace: ROOT
                  name: GrandChild
              - []
        - []
    ? namespace:
        NAMED: two
      name: Child
    : STRUCT:
        - - field:
              - STR
              - []
        - []
    ");
}

#[test]
fn nested_namespaced_enums() {
    mod one {
        #![allow(unused)]

        use facet::Facet;
        #[derive(Facet)]
        #[repr(C)]
        pub enum GrandChild {
            None,
        }

        #[derive(Facet)]
        #[facet(namespace = "one")]
        #[repr(C)]
        #[allow(unused)]
        pub enum Child {
            Data(GrandChild),
        }
    }
    mod two {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "two")]
        #[repr(C)]
        #[allow(unused)]
        pub enum Child {
            Data(String),
        }
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        One(one::Child),
        Two(two::Child),
    }

    let registry = reflect!(Parent);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: GrandChild
    : ENUM:
        - 0:
            None:
              - UNIT
              - []
        - []
    ? namespace: ROOT
      name: Parent
    : ENUM:
        - 0:
            One:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: one
                    name: Child
              - []
          1:
            Two:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: two
                    name: Child
              - []
        - []
    ? namespace:
        NAMED: one
      name: Child
    : ENUM:
        - 0:
            Data:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: GrandChild
              - []
        - []
    ? namespace:
        NAMED: two
      name: Child
    : ENUM:
        - 0:
            Data:
              - NEWTYPE: STR
              - []
        - []
    ");
}

#[test]
fn nested_namespaced_renamed_structs() {
    mod one {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(name = "GrandKid")]
        pub struct GrandChild {
            field: String,
        }

        #[derive(Facet)]
        #[facet(namespace = "one")]
        #[facet(name = "Kid")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "two")]
        #[facet(name = "Kid")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(Facet)]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect!(Parent);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: GrandKid
    : STRUCT:
        - - field:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: Parent
    : STRUCT:
        - - one:
              - TYPENAME:
                  namespace:
                    NAMED: one
                  name: Kid
              - []
          - two:
              - TYPENAME:
                  namespace:
                    NAMED: two
                  name: Kid
              - []
        - []
    ? namespace:
        NAMED: one
      name: Kid
    : STRUCT:
        - - child:
              - TYPENAME:
                  namespace: ROOT
                  name: GrandKid
              - []
        - []
    ? namespace:
        NAMED: two
      name: Kid
    : STRUCT:
        - - field:
              - STR
              - []
        - []
    ");
}

#[test]
fn namespaced_collections() {
    #[derive(Facet)]
    #[facet(namespace = "api")]
    pub struct User {
        id: String,
        name: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "api")]
    pub struct Group {
        users: Vec<User>,
    }

    #[derive(Facet)]
    struct Response {
        users: Vec<User>,
        user_arrays: [User; 5],
        optional_user: Option<User>,
        groups: Vec<Group>,
    }

    let registry = reflect!(Response);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Response
    : STRUCT:
        - - users:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: api
                    name: User
              - []
          - user_arrays:
              - TUPLEARRAY:
                  CONTENT:
                    TYPENAME:
                      namespace:
                        NAMED: api
                      name: User
                  SIZE: 5
              - []
          - optional_user:
              - OPTION:
                  TYPENAME:
                    namespace:
                      NAMED: api
                    name: User
              - []
          - groups:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: api
                    name: Group
              - []
        - []
    ? namespace:
        NAMED: api
      name: Group
    : STRUCT:
        - - users:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: api
                    name: User
              - []
        - []
    ? namespace:
        NAMED: api
      name: User
    : STRUCT:
        - - id:
              - STR
              - []
          - name:
              - STR
              - []
        - []
    ");
}

#[test]
fn namespaced_maps() {
    mod models {
        use facet::Facet;

        #[derive(Facet, Clone, Hash, Eq, PartialEq)]
        #[facet(namespace = "models")]
        pub struct UserId(String);

        #[derive(Facet)]
        #[facet(namespace = "models")]
        pub struct UserProfile {
            name: String,
            active: bool,
        }
    }

    #[derive(Facet)]
    struct Database {
        user_profiles: HashMap<models::UserId, models::UserProfile>,
        user_counts: HashMap<String, u32>,
    }

    let registry = reflect!(Database);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Database
    : STRUCT:
        - - user_profiles:
              - MAP:
                  KEY:
                    TYPENAME:
                      namespace:
                        NAMED: models
                      name: UserId
                  VALUE:
                    TYPENAME:
                      namespace:
                        NAMED: models
                      name: UserProfile
              - []
          - user_counts:
              - MAP:
                  KEY: STR
                  VALUE: U32
              - []
        - []
    ? namespace:
        NAMED: models
      name: UserId
    : NEWTYPESTRUCT:
        - STR
        - []
    ? namespace:
        NAMED: models
      name: UserProfile
    : STRUCT:
        - - name:
              - STR
              - []
          - active:
              - BOOL
              - []
        - []
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
fn complex_namespaced_enums() {
    mod events {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "events")]
        pub struct UserData {
            id: String,
            email: String,
        }

        #[derive(Facet)]
        #[facet(namespace = "events")]
        pub struct SystemData {
            timestamp: u64,
        }

        #[derive(Facet)]
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

    #[derive(Facet)]
    struct EventLog {
        events: Vec<events::Event>,
    }

    let registry = reflect!(EventLog);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: EventLog
    : STRUCT:
        - - events:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: events
                    name: Event
              - []
        - []
    ? namespace:
        NAMED: events
      name: Event
    : ENUM:
        - 0:
            UserCreated:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: events
                    name: UserData
              - []
          1:
            UserUpdated:
              - STRUCT:
                  - old:
                      - TYPENAME:
                          namespace:
                            NAMED: events
                          name: UserData
                      - []
                  - new:
                      - TYPENAME:
                          namespace:
                            NAMED: events
                          name: UserData
                      - []
              - []
          2:
            SystemEvent:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: events
                    name: SystemData
              - []
          3:
            Simple:
              - UNIT
              - []
        - []
    ? namespace:
        NAMED: events
      name: SystemData
    : STRUCT:
        - - timestamp:
              - U64
              - []
        - []
    ? namespace:
        NAMED: events
      name: UserData
    : STRUCT:
        - - id:
              - STR
              - []
          - email:
              - STR
              - []
        - []
    ");
}

#[test]
fn namespaced_transparent_structs() {
    #[derive(Facet, Clone)]
    #[facet(namespace = "wrappers")]
    pub struct UserId(String);

    #[derive(Facet)]
    #[facet(namespace = "wrappers")]
    #[facet(transparent)]
    pub struct TransparentWrapper(UserId);

    #[derive(Facet)]
    struct Container {
        direct_id: UserId,
        wrapped_id: TransparentWrapper,
    }

    let registry = reflect!(Container);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Container
    : STRUCT:
        - - direct_id:
              - TYPENAME:
                  namespace:
                    NAMED: wrappers
                  name: UserId
              - []
          - wrapped_id:
              - TYPENAME:
                  namespace:
                    NAMED: wrappers
                  name: UserId
              - []
        - []
    ? namespace:
        NAMED: wrappers
      name: UserId
    : NEWTYPESTRUCT:
        - STR
        - []
    ");
}

#[test]
fn cross_namespace_references() {
    #[derive(Facet)]
    #[facet(namespace = "entities")]
    struct Entity {
        id: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "api")]
    struct Request {
        entity: Entity,
        metadata: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "storage")]
    struct Record {
        entity: Entity,
        request: Request,
    }

    #[derive(Facet)]
    struct System {
        records: Vec<Record>,
    }

    let registry = reflect!(System);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: System
    : STRUCT:
        - - records:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: storage
                    name: Record
              - []
        - []
    ? namespace:
        NAMED: api
      name: Request
    : STRUCT:
        - - entity:
              - TYPENAME:
                  namespace:
                    NAMED: entities
                  name: Entity
              - []
          - metadata:
              - STR
              - []
        - []
    ? namespace:
        NAMED: entities
      name: Entity
    : STRUCT:
        - - id:
              - STR
              - []
        - []
    ? namespace:
        NAMED: storage
      name: Record
    : STRUCT:
        - - entity:
              - TYPENAME:
                  namespace:
                    NAMED: entities
                  name: Entity
              - []
          - request:
              - TYPENAME:
                  namespace:
                    NAMED: api
                  name: Request
              - []
        - []
    ");
}

#[test]
fn namespace_with_byte_attributes() {
    mod data {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "data")]
        pub struct BinaryData {
            #[facet(bytes)]
            content: Vec<u8>,
            #[facet(bytes)]
            header: &'static [u8],
            metadata: String,
        }
    }

    #[derive(Facet)]
    struct Document {
        binary: data::BinaryData,
    }

    let registry = reflect!(Document);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Document
    : STRUCT:
        - - binary:
              - TYPENAME:
                  namespace:
                    NAMED: data
                  name: BinaryData
              - []
        - []
    ? namespace:
        NAMED: data
      name: BinaryData
    : STRUCT:
        - - content:
              - BYTES
              - []
          - header:
              - BYTES
              - []
          - metadata:
              - STR
              - []
        - []
    ");
}

#[test]
fn deeply_nested_namespaces() {
    mod level1 {
        use facet::Facet;

        pub mod level2 {
            use facet::Facet;

            #[derive(Facet)]
            #[facet(namespace = "level1.level2")]
            pub struct DeepStruct {
                value: String,
            }
        }

        #[derive(Facet)]
        #[facet(namespace = "level1")]
        pub struct MiddleStruct {
            deep: level2::DeepStruct,
        }
    }

    #[derive(Facet)]
    struct RootStruct {
        middle: level1::MiddleStruct,
        deep_direct: level1::level2::DeepStruct,
    }

    let registry = reflect!(RootStruct);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: RootStruct
    : STRUCT:
        - - middle:
              - TYPENAME:
                  namespace:
                    NAMED: level1
                  name: MiddleStruct
              - []
          - deep_direct:
              - TYPENAME:
                  namespace:
                    NAMED: level1.level2
                  name: DeepStruct
              - []
        - []
    ? namespace:
        NAMED: level1
      name: MiddleStruct
    : STRUCT:
        - - deep:
              - TYPENAME:
                  namespace:
                    NAMED: level1.level2
                  name: DeepStruct
              - []
        - []
    ? namespace:
        NAMED: level1.level2
      name: DeepStruct
    : STRUCT:
        - - value:
              - STR
              - []
        - []
    ");
}

#[test]
fn transparent_struct_explicit_namespace() {
    // Test transparent structs with explicit namespace annotations
    mod wrappers {
        use facet::Facet;

        #[derive(Facet, Clone)]
        pub struct UserId(String);

        #[derive(Facet)]
        #[facet(namespace = "wrappers")]
        #[facet(transparent)]
        pub struct TransparentWrapper(UserId);
    }

    #[derive(Facet)]
    struct Container {
        wrapped_id: wrappers::TransparentWrapper,
    }

    let registry = reflect!(Container);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Container
    : STRUCT:
        - - wrapped_id:
              - TYPENAME:
                  namespace: ROOT
                  name: UserId
              - []
        - []
    ? namespace: ROOT
      name: UserId
    : NEWTYPESTRUCT:
        - STR
        - []
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
fn explicit_namespace_declarations() {
    mod api_example {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "api")]
        pub struct User {
            id: String,
            name: String,
        }

        #[derive(Facet)]
        #[facet(namespace = "api")]
        pub struct Group {
            users: Vec<User>,
        }
    }

    mod events_example {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "events")]
        pub struct UserData {
            id: String,
        }

        #[derive(Facet)]
        #[facet(namespace = "events")]
        pub struct SystemData {
            timestamp: u64,
        }

        #[derive(Facet)]
        #[facet(namespace = "events")]
        #[repr(C)]
        #[allow(unused)]
        pub enum Event {
            UserCreated(UserData),
            SystemEvent(SystemData),
        }
    }

    #[derive(Facet)]
    struct ApiContainer {
        #[facet(name = "user")]
        user: api_example::User,
        group: api_example::Group,
    }

    #[derive(Facet)]
    struct RootUser {
        id: String,
        name: String,
    }

    #[derive(Facet)]
    struct RootGroup {
        users: Vec<RootUser>,
    }

    #[derive(Facet)]
    struct RootContainer {
        api_data: ApiContainer,
        event: events_example::Event,
        efficient: RootGroup,
    }

    let registry = reflect!(RootContainer);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: ApiContainer
    : STRUCT:
        - - user:
              - TYPENAME:
                  namespace:
                    NAMED: api
                  name: User
              - []
          - group:
              - TYPENAME:
                  namespace:
                    NAMED: api
                  name: Group
              - []
        - []
    ? namespace: ROOT
      name: RootContainer
    : STRUCT:
        - - api_data:
              - TYPENAME:
                  namespace: ROOT
                  name: ApiContainer
              - []
          - event:
              - TYPENAME:
                  namespace:
                    NAMED: events
                  name: Event
              - []
          - efficient:
              - TYPENAME:
                  namespace: ROOT
                  name: RootGroup
              - []
        - []
    ? namespace: ROOT
      name: RootGroup
    : STRUCT:
        - - users:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: RootUser
              - []
        - []
    ? namespace: ROOT
      name: RootUser
    : STRUCT:
        - - id:
              - STR
              - []
          - name:
              - STR
              - []
        - []
    ? namespace:
        NAMED: api
      name: Group
    : STRUCT:
        - - users:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: api
                    name: User
              - []
        - []
    ? namespace:
        NAMED: api
      name: User
    : STRUCT:
        - - id:
              - STR
              - []
          - name:
              - STR
              - []
        - []
    ? namespace:
        NAMED: events
      name: Event
    : ENUM:
        - 0:
            UserCreated:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: events
                    name: UserData
              - []
          1:
            SystemEvent:
              - NEWTYPE:
                  TYPENAME:
                    namespace:
                      NAMED: events
                    name: SystemData
              - []
        - []
    ? namespace:
        NAMED: events
      name: SystemData
    : STRUCT:
        - - timestamp:
              - U64
              - []
        - []
    ? namespace:
        NAMED: events
      name: UserData
    : STRUCT:
        - - id:
              - STR
              - []
        - []
    ");
}

#[test]
fn collections_with_explicit_namespace() {
    // Test that types in collections go to root namespace when no explicit namespace is given
    #[derive(Facet)]
    struct UnnamedUser {
        name: String,
    }

    #[derive(Facet)]
    struct UnnamedRole {
        permissions: Vec<String>,
    }

    // Container with explicit namespace
    #[derive(Facet)]
    #[facet(namespace = "system")]
    struct UserManager {
        users: Vec<UnnamedUser>,
        admins: [UnnamedUser; 2],
        optional_user: Option<UnnamedUser>,
        role_map: HashMap<String, UnnamedRole>,
        nested_lists: Vec<Vec<UnnamedUser>>,
    }

    let registry = reflect!(UserManager);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: UnnamedRole
    : STRUCT:
        - - permissions:
              - SEQ: STR
              - []
        - []
    ? namespace: ROOT
      name: UnnamedUser
    : STRUCT:
        - - name:
              - STR
              - []
        - []
    ? namespace:
        NAMED: system
      name: UserManager
    : STRUCT:
        - - users:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: UnnamedUser
              - []
          - admins:
              - TUPLEARRAY:
                  CONTENT:
                    TYPENAME:
                      namespace: ROOT
                      name: UnnamedUser
                  SIZE: 2
              - []
          - optional_user:
              - OPTION:
                  TYPENAME:
                    namespace: ROOT
                    name: UnnamedUser
              - []
          - role_map:
              - MAP:
                  KEY: STR
                  VALUE:
                    TYPENAME:
                      namespace: ROOT
                      name: UnnamedRole
              - []
          - nested_lists:
              - SEQ:
                  SEQ:
                    TYPENAME:
                      namespace: ROOT
                      name: UnnamedUser
              - []
        - []
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
fn enums_with_explicit_namespace() {
    // Test that enum variant types go to root namespace when no explicit namespace is given
    #[derive(Facet)]
    struct ErrorData {
        code: u32,
        message: String,
    }

    #[derive(Facet)]
    struct SuccessData {
        result: String,
    }

    #[derive(Facet)]
    struct ProcessingData {
        progress: f32,
        estimate: ErrorData,
    }

    // Enum with explicit namespace
    #[derive(Facet)]
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

    let registry = reflect!(Response);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: ErrorData
    : STRUCT:
        - - code:
              - U32
              - []
          - message:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: ProcessingData
    : STRUCT:
        - - progress:
              - F32
              - []
          - estimate:
              - TYPENAME:
                  namespace: ROOT
                  name: ErrorData
              - []
        - []
    ? namespace: ROOT
      name: SuccessData
    : STRUCT:
        - - result:
              - STR
              - []
        - []
    ? namespace:
        NAMED: api
      name: Response
    : ENUM:
        - 0:
            Success:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: SuccessData
              - []
          1:
            Error:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: ErrorData
              - []
          2:
            Processing:
              - STRUCT:
                  - data:
                      - TYPENAME:
                          namespace: ROOT
                          name: ProcessingData
                      - []
                  - extra:
                      - TYPENAME:
                          namespace: ROOT
                          name: SuccessData
                      - []
              - []
          3:
            Multipart:
              - TUPLE:
                  - TYPENAME:
                      namespace: ROOT
                      name: ErrorData
                  - TYPENAME:
                      namespace: ROOT
                      name: SuccessData
              - []
          4:
            Empty:
              - UNIT
              - []
        - []
    ");
}

#[test]
fn nested_structs_with_explicit_namespace() {
    // Test that deeply nested structs go to root namespace when no explicit namespace is given
    #[derive(Facet)]
    struct DeepInner {
        value: i32,
    }

    #[derive(Facet)]
    struct MiddleLayer {
        inner: DeepInner,
        inner_list: Vec<DeepInner>,
    }

    #[derive(Facet)]
    struct TopLayer {
        middle: MiddleLayer,
        direct_inner: DeepInner,
    }

    // Container with explicit namespace
    #[derive(Facet)]
    #[facet(namespace = "nested")]
    struct Container {
        top: TopLayer,
        middle_direct: MiddleLayer,
        inner_direct: DeepInner,
    }

    let registry = reflect!(Container);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: DeepInner
    : STRUCT:
        - - value:
              - I32
              - []
        - []
    ? namespace: ROOT
      name: MiddleLayer
    : STRUCT:
        - - inner:
              - TYPENAME:
                  namespace: ROOT
                  name: DeepInner
              - []
          - inner_list:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: DeepInner
              - []
        - []
    ? namespace: ROOT
      name: TopLayer
    : STRUCT:
        - - middle:
              - TYPENAME:
                  namespace: ROOT
                  name: MiddleLayer
              - []
          - direct_inner:
              - TYPENAME:
                  namespace: ROOT
                  name: DeepInner
              - []
        - []
    ? namespace:
        NAMED: nested
      name: Container
    : STRUCT:
        - - top:
              - TYPENAME:
                  namespace: ROOT
                  name: TopLayer
              - []
          - middle_direct:
              - TYPENAME:
                  namespace: ROOT
                  name: MiddleLayer
              - []
          - inner_direct:
              - TYPENAME:
                  namespace: ROOT
                  name: DeepInner
              - []
        - []
    ");
}

#[test]
fn transparent_struct_chains() {
    // Test transparent struct chains - they should resolve to the final non-transparent type
    #[derive(Facet, Clone)]
    struct CoreId(String);

    #[derive(Facet, Clone)]
    #[facet(transparent)]
    struct WrapperId(CoreId);

    #[derive(Facet, Clone)]
    #[facet(transparent)]
    struct DoubleWrapperId(WrapperId);

    // Container with explicit namespace
    #[derive(Facet)]
    #[facet(namespace = "identity")]
    struct NamespacedWrapper(DoubleWrapperId);

    #[derive(Facet)]
    struct IdContainer {
        id: NamespacedWrapper,
    }

    let registry = reflect!(IdContainer);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: CoreId
    : NEWTYPESTRUCT:
        - STR
        - []
    ? namespace: ROOT
      name: IdContainer
    : STRUCT:
        - - id:
              - TYPENAME:
                  namespace:
                    NAMED: identity
                  name: NamespacedWrapper
              - []
        - []
    ? namespace:
        NAMED: identity
      name: NamespacedWrapper
    : NEWTYPESTRUCT:
        - TYPENAME:
            namespace: ROOT
            name: DoubleWrapperId
        - []
    ");
}

#[test]
fn mixed_containers_with_explicit_namespace() {
    // Test that various container types correctly reference root namespace types
    #[derive(Facet)]
    struct Item {
        id: String,
    }

    #[derive(Facet)]
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

    let registry = reflect!(MixedContainer);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: Item
    : STRUCT:
        - - id:
              - STR
              - []
        - []
    ? namespace:
        NAMED: storage
      name: MixedContainer
    : STRUCT:
        - - single:
              - TYPENAME:
                  namespace: ROOT
                  name: Item
              - []
          - vector:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: Item
              - []
          - array:
              - TUPLEARRAY:
                  CONTENT:
                    TYPENAME:
                      namespace: ROOT
                      name: Item
                  SIZE: 3
              - []
          - option:
              - OPTION:
                  TYPENAME:
                    namespace: ROOT
                    name: Item
              - []
          - tuple:
              - TUPLE:
                  - TYPENAME:
                      namespace: ROOT
                      name: Item
                  - STR
              - []
          - nested_option:
              - OPTION:
                  SEQ:
                    TYPENAME:
                      namespace: ROOT
                      name: Item
              - []
          - complex_map:
              - MAP:
                  KEY: STR
                  VALUE:
                    SEQ:
                      OPTION:
                        TYPENAME:
                          namespace: ROOT
                          name: Item
              - []
        - []
    ");
}

#[test]
fn no_namespace_pollution() {
    // Test that types without explicit namespaces don't get duplicated across multiple namespaces
    #[derive(Facet)]
    struct SharedType {
        value: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "alpha")]
    struct AlphaContainer {
        shared: SharedType,
    }

    #[derive(Facet)]
    #[facet(namespace = "beta")]
    struct BetaContainer {
        shared: SharedType,
    }

    #[derive(Facet)]
    struct RootContainer {
        alpha: AlphaContainer,
        beta: BetaContainer,
        unnamespaced: SharedType,
    }

    let registry = reflect!(RootContainer);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: RootContainer
    : STRUCT:
        - - alpha:
              - TYPENAME:
                  namespace:
                    NAMED: alpha
                  name: AlphaContainer
              - []
          - beta:
              - TYPENAME:
                  namespace:
                    NAMED: beta
                  name: BetaContainer
              - []
          - unnamespaced:
              - TYPENAME:
                  namespace: ROOT
                  name: SharedType
              - []
        - []
    ? namespace: ROOT
      name: SharedType
    : STRUCT:
        - - value:
              - STR
              - []
        - []
    ? namespace:
        NAMED: alpha
      name: AlphaContainer
    : STRUCT:
        - - shared:
              - TYPENAME:
                  namespace: ROOT
                  name: SharedType
              - []
        - []
    ? namespace:
        NAMED: beta
      name: BetaContainer
    : STRUCT:
        - - shared:
              - TYPENAME:
                  namespace: ROOT
                  name: SharedType
              - []
        - []
    ");
}

#[test]
fn explicit_namespace_behavior_summary() {
    #[derive(Facet)]
    struct BaseType {
        value: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "first")]
    struct FirstContainer {
        item: BaseType,
    }

    #[derive(Facet)]
    #[facet(namespace = "second")]
    struct SecondContainer {
        item: BaseType,
    }

    #[derive(Facet)]
    struct Root {
        first: FirstContainer,
        second: SecondContainer,
        direct: BaseType,
    }

    let registry = reflect!(Root);
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: BaseType
    : STRUCT:
        - - value:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: Root
    : STRUCT:
        - - first:
              - TYPENAME:
                  namespace:
                    NAMED: first
                  name: FirstContainer
              - []
          - second:
              - TYPENAME:
                  namespace:
                    NAMED: second
                  name: SecondContainer
              - []
          - direct:
              - TYPENAME:
                  namespace: ROOT
                  name: BaseType
              - []
        - []
    ? namespace:
        NAMED: first
      name: FirstContainer
    : STRUCT:
        - - item:
              - TYPENAME:
                  namespace: ROOT
                  name: BaseType
              - []
        - []
    ? namespace:
        NAMED: second
      name: SecondContainer
    : STRUCT:
        - - item:
              - TYPENAME:
                  namespace: ROOT
                  name: BaseType
              - []
        - []
    ");
}
