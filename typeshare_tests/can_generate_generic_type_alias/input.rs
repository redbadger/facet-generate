#[derive(Facet)]
type GenericTypeAlias<T> = Vec<T>;

#[derive(Facet)]
type NonGenericAlias = GenericTypeAlias<Option<String>>;
