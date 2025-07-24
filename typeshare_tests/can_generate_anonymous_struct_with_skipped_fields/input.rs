#[derive(Facet)]
pub enum SomeEnum {
    AnonymousStruct {
        all: bool,
        #[facet(skip)]
        none: bool,
        #[facet(swift(skip))]
        except_swift: bool,
        #[facet(kotlin(skip))]
        except_kotlin: bool,
        #[facet(typescript(skip))]
        except_ts: bool,
    },
}
