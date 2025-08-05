#![allow(non_snake_case)]

use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[facet(tag = "type", content = "content", rename_all = "camelCase")]
#[repr(C)]
pub enum AnonymousStructWithRename {
    List {
        list: Vec<String>,
    },
    LongFieldNames {
        // Note that the `#[serde(rename_all)]` attribute applied to the overall enum
        // does not apply to these anonymous struct variant fields.
        //
        // These fields should rename in snake_case.
        some_long_field_name: String,
        and: bool,
        but_one_more: Vec<String>,
    },
    #[facet(rename_all = "kebab-case")]
    KebabCase {
        // Similar to the above, the `#[serde(rename_all)]` attribute applied to
        // this enum variant will apply, rather than the one applied to the overall
        // enum.
        anotherList: Vec<String>,
        // However, this even more specific `#[serde(rename)]` attribute should
        // cause this field to remain in camelCase.
        #[facet(rename = "camelCaseStringField")]
        camelCaseStringField: String,
        something_else: bool,
    },
}
