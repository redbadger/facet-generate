#[derive(Facet)]
pub struct OtherType {}

/// This is a comment.
#[derive(Facet)]
#[facet(rename = "PersonTwo")]
pub struct Person {
    pub name: String,
    pub age: u8,
    #[facet(rename = "extraSpecialFieldOne")]
    pub extra_special_field1: i32,
    #[facet(rename = "extraSpecialFieldTwo")]
    pub extra_special_field2: Option<Vec<String>>,
    #[facet(rename = "nonStandardDataType")]
    pub non_standard_data_type: OtherType,
    #[facet(rename = "nonStandardDataTypeInArray")]
    pub non_standard_data_type_in_array: Option<Vec<OtherType>>,
}
