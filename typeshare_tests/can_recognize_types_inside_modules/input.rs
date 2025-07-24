mod a {
    #[derive(Facet)]
    pub struct A {
        field: u32,
    }
    mod b {
        mod c {
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
