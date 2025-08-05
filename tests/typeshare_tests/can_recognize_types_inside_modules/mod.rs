#![expect(clippy::upper_case_acronyms)]

use facet::Facet;

mod a {
    use facet::Facet;

    #[derive(Facet)]
    pub struct A {
        field: u32,
    }
    mod b {
        use facet::Facet;

        mod c {
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
