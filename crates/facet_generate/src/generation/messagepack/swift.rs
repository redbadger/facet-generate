//! `EmitterPlugin<Swift>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports, `Codable` conformance, and
//! convenience serialize/deserialize wrappers for Swift code generation,
//! using the `MessagePacker` SPM package.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports`               | `import MessagePacker` |
//! | `type_conformances`     | `Codable` appended to struct/enum declarations |
//! | `has_type_body`         | `true` |
//! | `type_body`             | `msgPackSerialize()` / `msgPackDeserialize()` wrappers |
//! | `manifest_dependencies` | SPM `.package(url:from:)` entry for `MessagePacker` |
//!
//! # Swift usage
//!
//! The generated code delegates to `MessagePackEncoder` / `MessagePackDecoder`
//! from the `MessagePacker` package.  Because Swift's `Codable` machinery does
//! all the heavy lifting, this plugin does **not** need to emit
//! `serialize` / `deserialize` protocol methods — only the convenience wrappers.

use std::io;

use indoc::writedoc;

use crate::generation::{
    CodeGeneratorConfig,
    indent::IndentWrite,
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
    swift::Swift,
};

use super::MessagePackPlugin;

impl EmitterPlugin<Swift> for MessagePackPlugin {
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec!["MessagePacker".to_string()]
    }

    fn type_conformances(&self, _ctx: &EmitContext) -> Vec<String> {
        vec!["Codable".to_string()]
    }

    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let name = ctx.name();
        writeln!(w)?;
        writedoc!(
            w,
            r"
            public func msgPackSerialize() throws -> Data {{
                return try MessagePackEncoder().encode(self)
            }}
            "
        )?;
        writeln!(w)?;
        writedoc!(
            w,
            r"
            public static func msgPackDeserialize(input: Data) throws -> {name} {{
                return try MessagePackDecoder().decode({name}.self, from: input)
            }}
            "
        )
    }

    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            r#".package(url: "https://github.com/hirotakan/MessagePacker.git", from: "0.4.7")"#
                .to_string(),
        ]
    }

    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        CodeGeneratorConfig, Container,
        indent::IndentedWriter,
        plugin::{EmitContext, EmitterPlugin},
        swift::Swift,
    };
    use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

    fn make_ctx<'a>(
        container: &'a Container<'a>,
        config: &'a CodeGeneratorConfig,
    ) -> EmitContext<'a> {
        EmitContext::top_level(container, config)
    }

    #[test]
    fn imports_returns_message_packer() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;
        let imports = plugin.imports(&cfg);
        assert!(
            imports.contains(&"MessagePacker".to_string()),
            "imports should contain 'MessagePacker', got: {imports:?}"
        );
    }

    #[test]
    fn type_conformances_returns_codable() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = make_ctx(&container, &cfg);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;
        let conformances = plugin.type_conformances(&ctx);
        assert!(
            conformances.contains(&"Codable".to_string()),
            "type_conformances should contain 'Codable', got: {conformances:?}"
        );
    }

    #[test]
    fn has_type_body_is_true() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = make_ctx(&container, &cfg);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;
        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn type_body_emits_msg_pack_methods() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("MyType".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = make_ctx(&container, &cfg);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(
            output.contains("msgPackSerialize"),
            "type_body should contain 'msgPackSerialize', got:\n{output}"
        );
        assert!(
            output.contains("msgPackDeserialize"),
            "type_body should contain 'msgPackDeserialize', got:\n{output}"
        );
        assert!(
            output.contains("MessagePackEncoder"),
            "type_body should contain 'MessagePackEncoder', got:\n{output}"
        );
        assert!(
            output.contains("MessagePackDecoder"),
            "type_body should contain 'MessagePackDecoder', got:\n{output}"
        );
    }

    #[test]
    fn manifest_dependency_contains_message_packer() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;
        let deps = plugin.manifest_dependencies();
        let combined = deps.join("\n");
        assert!(
            combined.contains("MessagePacker"),
            "manifest_dependencies should contain 'MessagePacker', got: {deps:?}"
        );
        assert!(
            combined.contains("0.4.7"),
            "manifest_dependencies should contain '0.4.7', got: {deps:?}"
        );
    }

    #[test]
    fn runtime_files_is_empty() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Swift>;
        assert!(
            plugin.runtime_files().is_empty(),
            "runtime_files should be empty for MessagePackPlugin (Swift)"
        );
    }
}
