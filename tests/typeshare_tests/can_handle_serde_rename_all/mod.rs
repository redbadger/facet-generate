use facet::Facet;

/// This is a Person struct with camelCase rename
#[derive(Facet)]
#[facet(default, rename_all = "camelCase")]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub extra_special_field1: i32,
    pub extra_special_field2: Option<Vec<String>>,
}

/// This is a Person2 struct with `SCREAMING_SNAKE_CASE` rename
#[derive(Facet)]
#[facet(default, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Person2 {
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
}
