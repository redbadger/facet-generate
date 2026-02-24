use indent::indent_all_with;
use indoc::formatdoc;

use crate::generation::{ExternalPackage, PackageLocation};

impl ExternalPackage {
    #[must_use]
    pub fn to_swift(self, level: usize) -> String {
        let indent_str = " ".repeat(4 * level);
        let base_string = match self.location {
            PackageLocation::Path(location) => {
                formatdoc! {r#"
                .package(
                    path: "{location}"
                )"#}
            }
            PackageLocation::Url(location) => {
                let version = self.version.unwrap_or_default();

                formatdoc! {r#"
                .package(
                    url: "{location}",
                    from: "{version}"
                )"#}
            }
        };

        indent_all_with(&indent_str, &base_string)
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::PackageLocation;

    use super::*;

    #[test]
    fn remote_package() {
        let dependency = ExternalPackage {
            for_namespace: "SQLite.swift".to_string(),
            location: PackageLocation::Url(
                "https://github.com/stephencelis/SQLite.swift.git".to_string(),
            ),
            module_name: None,
            version: Some("0.12.2".to_string()),
        };

        insta::assert_snapshot!(dependency.to_swift(3), @r#"
        .package(
            url: "https://github.com/stephencelis/SQLite.swift.git",
            from: "0.12.2"
        )
        "#);
    }

    #[test]
    fn local_package() {
        let dependency = ExternalPackage {
            for_namespace: "SQLite.swift".to_string(),
            location: PackageLocation::Path("path/to/SQLite.swift".to_string()),
            module_name: None,
            version: None,
        };

        insta::assert_snapshot!(dependency.to_swift(3), @r#"
        .package(
            path: "path/to/SQLite.swift"
        )
        "#);
    }
}
