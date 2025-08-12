#![expect(clippy::upper_case_acronyms)]

use facet::Facet;

use a::{
    A,
    b::{AB, c::ABC},
};

mod a {
    use facet::Facet;

    #[derive(Facet)]
    pub struct A {
        field: u32,
    }

    pub mod b {
        use facet::Facet;

        pub mod c {
            use facet::Facet;

            #[derive(Facet)]
            pub struct ABC {
                field: u32,
            }
        }

        #[derive(Facet)]
        pub struct AB {
            field: u32,
        }
    }
}

#[derive(Facet)]
pub struct OutsideOfModules {
    field: u32,
}

crate::test! {
    A, AB, ABC, OutsideOfModules for kotlin
}
