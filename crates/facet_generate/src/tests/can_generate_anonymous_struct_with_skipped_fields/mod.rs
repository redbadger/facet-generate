#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
#[repr(C)]
pub enum SomeEnum {
    AnonymousStruct {
        all: bool,
        #[facet(skip)]
        none: bool,
        // TODO: #[facet(swift(skip))]
        #[facet(skip)]
        except_swift: bool,
        // TODO: #[facet(kotlin(skip))]
        #[facet(skip)]
        except_kotlin: bool,
        // TODO: #[facet(typescript(skip))]
        #[facet(skip)]
        except_ts: bool,
    },
}
