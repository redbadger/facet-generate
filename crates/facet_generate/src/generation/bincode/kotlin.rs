//! `EmitterPlugin<Kotlin>` implementation for the [`BincodePlugin`].
//!
//! Provides bincode-specific imports and feature helper snippets for Kotlin
//! code generation. The heavy lifting (serialize / deserialize method bodies)
//! still lives in the Kotlin emitter; this plugin covers the module-level
//! concerns that are straightforward to express through the plugin API.

use std::io;

use crate::generation::{
    CodeGeneratorConfig, Feature, indent::IndentWrite, kotlin::Kotlin, plugin::EmitterPlugin,
};

use super::BincodePlugin;

// Feature helper snippets — the same `include_bytes!` constants that used to
// live in the Kotlin emitter.  They are Kotlin source files embedded at
// compile time and written into the module header when the corresponding
// [`Feature`] flag is active.
const FEATURE_LIST_OF_T: &[u8] =
    include_bytes!("../../generation/kotlin/emitter/features/ListOfT.kt");
const FEATURE_MAP_OF_T: &[u8] =
    include_bytes!("../../generation/kotlin/emitter/features/MapOfT.kt");
const FEATURE_OPTION_OF_T: &[u8] =
    include_bytes!("../../generation/kotlin/emitter/features/OptionOfT.kt");
const FEATURE_SET_OF_T: &[u8] =
    include_bytes!("../../generation/kotlin/emitter/features/SetOfT.kt");

impl EmitterPlugin<Kotlin> for BincodePlugin {
    /// Bincode / serde imports for a Kotlin module.
    ///
    /// Returns the base set of imports that every bincode-enabled module
    /// needs, plus feature-specific imports (e.g. `Bytes`, `Int128`).
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let bp = &self.bincode_package;
        let sp = &self.serde_package;

        let mut imports = vec![
            format!("import {bp}.BincodeDeserializer"),
            format!("import {bp}.BincodeSerializer"),
            format!("import {sp}.DeserializationError"),
            format!("import {sp}.Deserializer"),
            format!("import {sp}.Serializer"),
        ];

        // Feature-driven imports
        for feature in &config.features {
            match feature {
                Feature::Bytes => {
                    imports.push(format!("import {sp}.Bytes"));
                }
                Feature::BigInt => {
                    // BigInteger is JVM-only; kept for backward compat.
                    imports.push("import java.math.BigInteger".to_string());
                    imports.push(format!("import {sp}.Int128"));
                }
                // Other features add helper *code* (via module_helpers),
                // not imports.
                _ => {}
            }
        }

        imports
    }

    /// Bincode feature helper snippets for a Kotlin module.
    ///
    /// These are small Kotlin source fragments (extension functions on
    /// `Serializer` / `Deserializer`) that teach the serde runtime how to
    /// handle generic containers (`List<T>`, `Set<T>`, `Map<K,V>`,
    /// `Optional<T>`).  They are written into the module header, after
    /// imports but before any type declarations.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        for feature in &config.features {
            match feature {
                Feature::ListOfT => {
                    w.write_all(FEATURE_LIST_OF_T)?;
                    writeln!(w)?;
                }
                Feature::OptionOfT => {
                    w.write_all(FEATURE_OPTION_OF_T)?;
                    writeln!(w)?;
                }
                Feature::SetOfT => {
                    w.write_all(FEATURE_SET_OF_T)?;
                    writeln!(w)?;
                }
                Feature::MapOfT => {
                    w.write_all(FEATURE_MAP_OF_T)?;
                    writeln!(w)?;
                }
                // BigInt, Bytes, TupleArray are handled elsewhere
                // (BigInt for *JSON* has a helper but that belongs to a
                // future JSON plugin; TupleArray is encoding-independent
                // and stays in the emitter).
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::CodeGeneratorConfig;
    use crate::generation::indent::{IndentConfig, IndentedWriter};
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    #[test]
    fn base_imports_are_present() {
        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("BincodeSerializer")));
        assert!(imports.iter().any(|i| i.contains("BincodeDeserializer")));
        assert!(imports.iter().any(|i| i.contains("Serializer")));
        assert!(imports.iter().any(|i| i.contains("Deserializer")));
        assert!(imports.iter().any(|i| i.contains("DeserializationError")));
    }

    #[test]
    fn bytes_feature_adds_import() {
        let cfg = make_config(&[Feature::Bytes]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("Bytes")));
    }

    #[test]
    fn bigint_feature_adds_imports() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("BigInteger")));
        assert!(imports.iter().any(|i| i.contains("Int128")));
    }

    #[test]
    fn module_helpers_emit_list_of_t() {
        let cfg = make_config(&[Feature::ListOfT]);
        let plugin = BincodePlugin::from_config(&cfg);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(4));
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        // The ListOfT.kt snippet defines extension functions — just check
        // it's non-empty and contains a recognizable marker.
        assert!(!output.is_empty());
    }
}
