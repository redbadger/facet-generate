//! `EmitterPlugin<TypeScript>` implementation for [`MessagePackPlugin`].
//!
//! Provides module-level encode/decode helpers and manifest dependencies for
//! TypeScript code generation, using `@msgpack/msgpack`.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `import { encode, decode } from "@msgpack/msgpack"` |
//! | `module_helpers` | `msgPackEncode` / `msgPackDecode` generic arrow-function constants |
//! | `manifest_dependencies` | `"@msgpack/msgpack": "^3.1.3"` |
//! | `runtime_files` | Nothing — the library arrives via npm |
//!
//! All per-type hooks (`has_type_body`, `type_body`, etc.) use the default
//! no-op implementations. The plugin produces no per-type body content.

use std::io;

use super::MessagePackPlugin;
use crate::generation::{
    CodeGeneratorConfig,
    indent::IndentWrite,
    plugin::{EmitterPlugin, RuntimeFile},
    typescript::TypeScript,
};

impl EmitterPlugin<TypeScript> for MessagePackPlugin {
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![r#"import { encode, decode } from "@msgpack/msgpack";"#.to_string()]
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        _config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        // `useBigInt64: true` lets the underlying library round-trip
        // JavaScript `BigInt` values as MessagePack int64 / uint64, which is
        // required because the generator maps Rust `u64` / `i64` to TypeScript
        // `bigint`. Without this option `@msgpack/msgpack` v3 throws at
        // runtime when it encounters a `BigInt`.
        writeln!(w)?;
        writeln!(
            w,
            "export const msgPackEncode = <T>(value: T): Uint8Array =>\n    encode(value, {{ useBigInt64: true }});"
        )?;
        writeln!(
            w,
            "export const msgPackDecode = <T>(bytes: Uint8Array): T =>\n    decode(bytes, {{ useBigInt64: true }}) as T;"
        )?;
        Ok(())
    }

    fn manifest_dependencies(&self) -> Vec<String> {
        vec![r#""@msgpack/msgpack": "^3.1.3""#.to_string()]
    }

    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{CodeGeneratorConfig, Container, plugin::EmitContext};
    use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

    fn render_helpers(
        plugin: &dyn EmitterPlugin<TypeScript>,
        config: &CodeGeneratorConfig,
    ) -> String {
        let mut buf = Vec::new();
        {
            use crate::generation::indent::IndentedWriter;
            let mut w = IndentedWriter::new(&mut buf, config.indent);
            plugin.module_helpers(&mut w, config).unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn imports_returns_msgpack_import() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<TypeScript>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let imports = plugin.imports(&config);
        assert!(
            imports.iter().any(|s| s.contains("@msgpack/msgpack")),
            "expected @msgpack/msgpack in imports, got: {imports:?}"
        );
    }

    #[test]
    fn module_helpers_emits_encode_and_decode() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<TypeScript>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let out = render_helpers(plugin, &config);
        assert!(
            out.contains("msgPackEncode"),
            "missing msgPackEncode:\n{out}"
        );
        assert!(
            out.contains("msgPackDecode"),
            "missing msgPackDecode:\n{out}"
        );
    }

    #[test]
    fn manifest_dependency_contains_msgpack() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<TypeScript>;
        let deps = plugin.manifest_dependencies();
        assert!(
            deps.iter()
                .any(|s| s.contains("@msgpack/msgpack") && s.contains("3.1.3")),
            "expected @msgpack/msgpack and 3.1.3 in deps, got: {deps:?}"
        );
    }

    #[test]
    fn runtime_files_is_empty() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<TypeScript>;
        assert!(plugin.runtime_files().is_empty());
    }

    #[test]
    fn has_type_body_is_false() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<TypeScript>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(!plugin.has_type_body(&ctx));
    }
}
