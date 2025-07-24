#[derive(Facet)]
struct GenericType<K, V> {
    key: K,
    value: V,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
enum GenericEnum<K, V> {
    Variant { key: K, value: V },
}
