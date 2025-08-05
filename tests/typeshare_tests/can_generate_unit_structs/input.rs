// This test verifies that unit structs created without bracket syntax can still be generated.

use facet::Facet;

#[derive(Facet)]
struct UnitStruct;
