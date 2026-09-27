//! `reflect!` works in a crate that doesn't depend on `anyhow` (#230).
//!
//! This crate's tests can see `anyhow`, so the caller below shadows the name with an empty module
//! instead: a bare `anyhow::` path in the macro's expansion would fail to compile here.

mod caller {
    #[allow(dead_code)]
    mod anyhow {}

    use facet::Facet;
    use facet_generate::reflect;

    #[derive(Facet)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[test]
    fn reflect_does_not_need_anyhow_in_scope() {
        let registry = reflect!(Point).unwrap();
        assert_eq!(registry.len(), 1);
    }
}
