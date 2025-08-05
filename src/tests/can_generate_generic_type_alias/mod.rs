#![expect(unused)]

type GenericTypeAlias<T> = Vec<T>;

type NonGenericAlias = GenericTypeAlias<Option<String>>;
