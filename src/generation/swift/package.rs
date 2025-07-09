use indent::indent_all_with;
use indoc::formatdoc;

use crate::generation::Dependency;

impl Dependency {
    #[must_use]
    pub fn to_swift(self, level: usize) -> String {
        let name = self.name;
        let location = self.location;
        let version = self.version.unwrap_or_default();
        let indent_str = " ".repeat(4 * level);

        let base_string = formatdoc! {r#"
            .package(
                name: "{name}",
                url: "{location}",
                from: "{version}"
            )"#};

        indent_all_with(&indent_str, &base_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let dependency = Dependency {
            name: "SQLite.swift".to_string(),
            location: "https://github.com/stephencelis/SQLite.swift.git".to_string(),
            version: Some("0.12.2".to_string()),
        };

        insta::assert_snapshot!(dependency.to_swift(3), @r#"
            .package(
                name: "SQLite.swift",
                url: "https://github.com/stephencelis/SQLite.swift.git",
                from: "0.12.2"
            )
        "#);
    }
}
