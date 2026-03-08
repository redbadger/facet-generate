// This test verifies that unit structs created without bracket syntax can still be generated.

use facet::Facet;

#[derive(Facet)]
struct UnitStruct;

// TODO: enable swift, typescript (expect files need updating for no-encoding output)
crate::test! {
    UnitStruct for kotlin
}
