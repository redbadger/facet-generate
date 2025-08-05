#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
struct GenericType<K, V> {
    key: K,
    value: V,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
enum GenericEnum<K, V> {
    Variant { key: K, value: V },
}
