# Writing a plugin

A plugin adds code to what `facet_generate` writes: extra methods on a type, declarations
beside it, imports, helper code at the top of a module, whole companion files, runtime files
and manifest entries. The serialization support is built this way —
[`BincodePlugin`](crate::generation::bincode::BincodePlugin) and
[`JsonPlugin`](crate::generation::json::JsonPlugin) are plugins like any other — and so is
everything [Crux](https://github.com/redbadger/crux) generates on top of the shared types.

This page covers how plugins fit into generation, what each hook does and which languages call
it, how to name types the way the emitter does, and how to test a plugin. Along the way it
builds a small plugin that gives every struct a constant holding its type name, for TypeScript
and Kotlin.

## How plugins fit

Generation has three layers:

- The **installer** (for example [`typescript::Installer`](crate::generation::typescript::Installer))
  splits the registry into one module per namespace, writes runtime files, generates each
  module and writes the package manifest.
- The **generator** (for example
  [`TypeScriptCodeGenerator`](crate::generation::typescript::TypeScriptCodeGenerator)) writes
  one module: its header (package or namespace declaration, imports, module helpers) followed by
  each type.
- The **emitter** writes each type, and calls the plugins at fixed points while it does.

A plugin is a type that implements
[`EmitterPlugin<L>`](crate::generation::plugin::EmitterPlugin) once for each language it
supports, where `L` is the language tag:
[`Swift`](crate::generation::swift::Swift),
[`Kotlin`](crate::generation::kotlin::Kotlin),
[`CSharp`](crate::generation::csharp::CSharp) or
[`TypeScript`](crate::generation::typescript::TypeScript). Every method has a default that does
nothing, so an implementation overrides only the hooks it needs. The trait requires `Debug`,
which the error for a bad [`referenced_types`](#declaring-the-types-you-reference) uses to name
the plugin.

Attach a plugin to an installer with its `plugin` method, for example
[`typescript::Installer::plugin`](crate::generation::typescript::Installer::plugin). Plugins
are called in the order they were added, and a plugin can sit beside `BincodePlugin` or
`JsonPlugin`:

**Rust**
```rust,no_run
# use facet_generate::generation::{bincode::BincodePlugin, plugin::EmitterPlugin, typescript::{self, TypeScript}};
# #[derive(Debug)]
# struct MyPlugin;
# impl EmitterPlugin<TypeScript> for MyPlugin {}
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let registry = facet_generate::Registry::new();
typescript::Installer::new("shared", "generated/typescript")
    .plugin(BincodePlugin)
    .plugin(MyPlugin)
    .generate(&registry)?;
# Ok(())
# }
```

To generate one module without an installer, pass the plugins to the generator's
`with_plugins`, as the [worked example](#a-worked-example) does. The generator's `output` calls
the module and type hooks. `runtime_files`, `manifest_dependencies` and `target_dependencies`
are called only by the installer, and `companion_files` by the generator's own
`companion_files` method (Swift, Kotlin and C#), which the installer calls to get the files it
writes.

A hook may be called more than once for the same module — the TypeScript installer calls
`module_helpers` and `imports` once to check for name collisions and again to write the module,
and the Swift installer asks for `imports` several times — so a hook should return the same
answer every time and have no side effects.

## The hooks

| Hook | Called | Swift | Kotlin | C# | TS |
|---|---|:-:|:-:|:-:|:-:|
| `imports` | module header | ✓ | ✓ | ✓ | ✓ |
| `module_helpers` | after imports, before types | ✓ | ✓ | ✓ | ✓ |
| `module_declarations` | module header | | ✓ | | |
| `referenced_types` | once per module, before the header | ✓ | ✓ | ✓ | ✓ |
| `type_annotations` | above a type | | ✓ | ✓ | |
| `type_conformances` | after the type name | ✓ | | ✓ | |
| `has_type_body` | before opening `{ }` | | ✓ | ✓ | |
| `type_body_preamble` | start of a sealed interface | | ✓ | | |
| `type_body` | end of a type body | ✓ | ✓ | ✓ | ✓ |
| `after_type` | after every top-level type | ✓ | ✓ | ✓ | ✓ |
| `field_annotations` | above a field | | ✓ | ✓ | |
| `enum_variant_annotations` | before an `enum class` constant | | ✓ | | |
| `runtime_files` | installer | ✓ | ✓ | ✓ | ✓ |
| `companion_files` | installer | ✓ | ✓ | ✓ | |
| `manifest_dependencies` | installer | ✓ | ✓ | | ✓ |
| `target_dependencies` | installer | ✓ | | | |

A hook a language does not call is simply ignored there, so a plugin can implement the same
hooks for every language and leave the unused ones in place.

### The context a type hook receives

The type and field hooks receive an [`EmitContext`](crate::generation::plugin::EmitContext):

- `ctx.container` is the type being emitted: its registry name and its
  [`ContainerFormat`](crate::reflection::format::ContainerFormat).
- `ctx.config` is the module's
  [`CodeGeneratorConfig`](crate::generation::CodeGeneratorConfig), with everything derived from
  the module's registry.
- `ctx.name()` is the name of the type (or variant) being emitted, `ctx.fields()` its fields
  normalised to a list (a newtype's single field is `value`, a tuple's are `field0`,
  `field1`, …), and `ctx.variants()` the variants when the type is an enum.
- `ctx.variant` is `Some` inside an enum variant. Only the Kotlin emitter builds such a
  context — for the `data class` or `data object` of each variant of a `sealed interface` —
  and there `ctx.container` is a stand-in named after the variant, not the enum. In every
  other language, and for every `after_type` call, `ctx.is_variant()` is `false`.

The writer the body hooks receive is a `&mut dyn`
[`IndentWrite`](crate::generation::indent::IndentWrite), already indented to the right level.
Use `writeln!`, `indent()` and `unindent()` on it, or
[`with_block`](crate::generation::indent::with_block) to write a `{ }` block.

### Module hooks

[`imports`](crate::generation::plugin::EmitterPlugin::imports) returns the imports to merge into
the module header; duplicates are removed. Each string takes the shape of the language's import:
a bare module name in Swift (`"Serde"`), a whole line in Kotlin
(`"import kotlinx.serialization.Serializable"`), a whole `using` directive in C#
(`"using System.Text.Json;"`), and a whole statement in TypeScript
(`r#"import * as $json from "./serde/json";"#`).

[`module_helpers`](crate::generation::plugin::EmitterPlugin::module_helpers) writes code after
the imports and before the first type: type aliases, helper functions, shared declarations.
[`module_declarations`](crate::generation::plugin::EmitterPlugin::module_declarations) is
Kotlin's companion to it: the names those helpers declare at module scope (a `typealias`, say),
so the emitter leaves out any import of the same name, which would otherwise hide the
declaration.

[`referenced_types`](crate::generation::plugin::EmitterPlugin::referenced_types) lists the types
your code names that the module does not otherwise reference; it has
[its own section](#declaring-the-types-you-reference).

All four take the module's config, so a plugin can act on one module only. Ask
[`CodeGeneratorConfig::generates`](crate::generation::CodeGeneratorConfig::generates) whether
this is the module that generates a type you know lives there, rather than comparing
`module_name()`, which each language spells differently (`kit` in Swift and TypeScript,
`com.example.kit` in Kotlin).

### Type hooks

[`type_annotations`](crate::generation::plugin::EmitterPlugin::type_annotations) returns lines
written above a type declaration (`@Serializable` in Kotlin, `[JsonConverter(…)]` in C#).
[`type_conformances`](crate::generation::plugin::EmitterPlugin::type_conformances) returns
protocols or interfaces appended to the declaration; the emitter supplies the separator. Swift
has no annotation hook, and Kotlin and TypeScript no conformance hook.

[`type_body`](crate::generation::plugin::EmitterPlugin::type_body) writes members at the end of
a type's body, after its fields or variants. Where it is called differs by language:

| Language | `type_body` is called inside |
|---|---|
| Swift | every `struct` and `enum` |
| Kotlin | `enum class` and `sealed interface` always; `data class` and `data object` (including a sealed interface's variants) only when a plugin's `has_type_body` is `true` |
| C# | the `abstract record` of an enum with data always; a `partial class` or unit `sealed record` only when `has_type_body` is `true` |
| TypeScript | every `class` (not the union type of an enum) |

A Kotlin `data class` with no body, or a C# unit record, has no braces to write into, so the
emitter asks [`has_type_body`](crate::generation::plugin::EmitterPlugin::has_type_body) first,
and opens a body only if some plugin says yes. For a C# class, `type_body` is called only on the
plugins that said yes. A plugin for Kotlin or C# that writes into structs therefore overrides
`has_type_body` as well; Swift and TypeScript never ask
([#239](https://github.com/redbadger/facet-generate/issues/239)).

[`type_body_preamble`](crate::generation::plugin::EmitterPlugin::type_body_preamble) writes at the
start of a Kotlin `sealed interface`, before the variant classes — the Bincode plugin declares
the abstract `serialize` method there.

[`after_type`](crate::generation::plugin::EmitterPlugin::after_type) writes after a type's
closing brace: extensions, free functions, companion declarations. It is called once for every
top-level type in all four languages — in TypeScript after an enum's union type and its helper
functions — and never for a variant. It is the hook to reach for when the body is not yours to
add to: Crux writes most of its output there. A plugin that only means to act on one shape of
type checks `ctx.container.format` and returns `Ok(())` for the rest.

### Field and variant hooks

[`field_annotations`](crate::generation::plugin::EmitterPlugin::field_annotations) returns lines
written above a field: in Kotlin, the properties of a `data class` (struct or variant); in C#,
the properties of a `partial class`.
[`enum_variant_annotations`](crate::generation::plugin::EmitterPlugin::enum_variant_annotations)
is Kotlin only: annotations written on the same line as an `enum class` constant
(`@SerialName("Variant1") VARIANT1`). It receives the variant's name, not a context.

### Installation hooks

These are for the package around the modules. The ones that take a config are asked once per
module, with that module's config.

[`runtime_files`](crate::generation::plugin::EmitterPlugin::runtime_files) returns
[`RuntimeFile`](crate::generation::plugin::RuntimeFile)s — paths relative to the output
directory, with their contents — that the generated code depends on. The first plugin to claim
a path wins. The Swift, Kotlin and TypeScript installers skip them when the serde runtime comes
from an external package (an [`ExternalPackage`](crate::generation::ExternalPackage) for the
`serde` namespace); the C# installer
writes them whenever it has plugins ([#232](https://github.com/redbadger/facet-generate/issues/232)).

[`companion_files`](crate::generation::plugin::EmitterPlugin::companion_files) returns
[`CompanionFile`](crate::generation::plugin::CompanionFile)s: whole source files that belong to
the module but sit beside its file. The generator puts the module's header in front of each one
(its imports merged with the file's own, no module helpers), and the installer writes it into
the module's directory. Swift, Kotlin and C# write them; a TypeScript module is a single file,
so the TypeScript installer ignores the hook, and a TypeScript plugin writes extra declarations
from `after_type` instead. Unlike runtime files, companion files are written even when the serde
runtime is external.

[`manifest_dependencies`](crate::generation::plugin::EmitterPlugin::manifest_dependencies)
returns entries for the package manifest, each in the manifest's own syntax: a Gradle
dependency line for Kotlin, an SPM `.package(...)` entry for Swift, a `"name": "version"` pair
for `package.json`. The C# installer does not consult it yet
([#134](https://github.com/redbadger/facet-generate/issues/134)).
[`target_dependencies`](crate::generation::plugin::EmitterPlugin::target_dependencies) is Swift
only: entries for the generated module's SPM target `dependencies:` array, such as
`.product(name: "Shared", package: "Shared")`.

The Swift installer makes every generated target depend on the `Serde` target as soon as it has
any plugin, and only a plugin with runtime files (`BincodePlugin` or `JsonPlugin`) writes that
target. A Swift package generated with only your own plugin does not build — `swift build`
reports ``product 'Serde' required by package … not found`` — so install it alongside one of
the serialization plugins. With a plugin attached, the Swift emitter also derives `Hashable` and
`Equatable` where it can, even if the plugin adds nothing ([#231](https://github.com/redbadger/facet-generate/issues/231)).

## Naming references correctly

The emitter does not write a type reference the way the registry spells it. Before it emits a
module, the generator rewrites every type name in the module's registry into its *emitter
spelling*: the qualified name the reference needs from that module. In Kotlin, a ROOT type
`Event` referenced from the namespaced module `com.example.kv` becomes `com.example.Event`; in
TypeScript it becomes `Example.Event`, reached through a namespace import; in C# and Kotlin a
type of another namespace is rooted at the package. The rules are on each language's
`requalify`.

The formats in [`EmitContext`](crate::generation::plugin::EmitContext) (field types, variant
payloads) are already in emitter spelling. A name the plugin knows from anywhere else is in
registry spelling: the key of `ctx.container`, a
[`QualifiedTypeName`](crate::reflection::format::QualifiedTypeName) the plugin built, or a
format from
[`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of). Before writing
such a name, or passing it to a helper that expects the emitter's spelling, respell it with the
language's `requalify` (a whole format with `requalify_format`):

| Language | Respell | Render |
|---|---|---|
| Swift | [`swift::requalify`](crate::generation::swift::requalify), [`requalify_format`](crate::generation::swift::requalify_format) | [`swift::render_type`](crate::generation::swift::render_type) |
| Kotlin | [`kotlin::requalify`](crate::generation::kotlin::requalify), [`requalify_format`](crate::generation::kotlin::requalify_format) | [`kotlin::render_type`](crate::generation::kotlin::render_type) |
| C# | [`csharp::requalify`](crate::generation::csharp::requalify), [`requalify_format`](crate::generation::csharp::requalify_format) | [`csharp::render_type`](crate::generation::csharp::render_type) |
| TypeScript | [`typescript::requalify`](crate::generation::typescript::requalify), [`requalify_format`](crate::generation::typescript::requalify_format) | [`typescript::render_type`](crate::generation::typescript::render_type) |

`render_type` writes a format as the type expression the emitter would use for a property of
that type (`Optional<Example.Event>`, `com.example.Event?`). The same applies to
[`CodeGeneratorConfig::is_enum`](crate::generation::CodeGeneratorConfig::is_enum) and
[`is_unit_enum`](crate::generation::CodeGeneratorConfig::is_unit_enum), whose sets are keyed in
emitter spelling: a lookup with a registry-spelled name can miss, and the plugin then treats an
enum as a class.

Respell a name exactly once. `requalify` is not idempotent in C#, Kotlin and TypeScript: a
second pass qualifies an already-qualified name again (`com.example.com.example.Event` in
Kotlin), and in a TypeScript namespaced module it qualifies the module's own types, which the
first pass left bare. Swift's happens to leave a respelled name alone, but is meant to be
applied once too. So never respell a format that came from `EmitContext`.

**Rust**
```rust
use facet_generate::{
    generation::{CodeGeneratorConfig, kotlin},
    reflection::format::{Format, QualifiedTypeName},
};

// The module of namespace `kv`, nested under the root package, as the installer makes it.
let config = CodeGeneratorConfig::new("kv".to_string()).with_parent("com.example");

let event = QualifiedTypeName::root("Event".to_string());
let mut format = Format::Seq(Box::new(Format::TypeName(event)));
kotlin::requalify_format(&config, &mut format);

assert_eq!(kotlin::render_type(&format, &config), "List<com.example.Event>");
```

The languages also export the rules the emitter uses to turn Rust names into identifiers, so a
plugin that refers to a field, a variant or a parameter spells it the same way:

| Language | Helpers |
|---|---|
| Swift | [`case_name`](crate::generation::swift::case_name) (`NotFound` → `notFound`), [`field_name`](crate::generation::swift::field_name), [`escape_identifier`](crate::generation::swift::escape_identifier) |
| Kotlin | [`variant_class_name`](crate::generation::kotlin::variant_class_name), [`enum_constant_name`](crate::generation::kotlin::enum_constant_name) (`Active` → `ACTIVE`), [`field_name`](crate::generation::kotlin::field_name), [`escape_identifier`](crate::generation::kotlin::escape_identifier) |
| C# | [`escape_identifier`](crate::generation::csharp::escape_identifier) (`class` → `@class`) |
| TypeScript | [`param_name`](crate::generation::typescript::param_name) (`default` → `default_`), [`is_reserved_word`](crate::generation::typescript::is_reserved_word) |

## Declaring the types you reference

A module imports what its own types reference: in Swift, the module (and target) of each type
from another namespace; in TypeScript, `import * as Ns from "./ns"`. A name that appears only
in a plugin's output is invisible to that analysis, so the import is missing and the code does
not compile. Declare such names from
[`referenced_types`](crate::generation::plugin::EmitterPlugin::referenced_types), and the
generator treats each as a reference the module makes:

| Language | What a reference into another module adds |
|---|---|
| Swift | the `import` of that module, and a dependency on its target in `Package.swift`, included in the check that the targets form no cycle |
| TypeScript | `import * as Ns from "<path>"` |
| Kotlin, C# | nothing — they write fully qualified names |

Return the names in registry spelling (as `QualifiedTypeName`s, not respelled), and don't return
the import or target dependency from `imports` or `target_dependencies` as well. A reference to
a type of the module itself adds nothing.

Every name must be a type in the registry, in all four languages. Otherwise generation fails
with an `InvalidInput` error that names the plugin (by its `Debug` output), the module and the
type:

**Error message**
```text
plugin Bad declares that module `shared` references `ROOT::Missing`, which is not a type in the registry
```

This plugin adds a TypeScript alias for a type of namespace `kit` to the root module, which
does not otherwise reference it, and gets the import it needs by declaring the reference:

**Rust**
```rust
use std::io;

use facet::Facet;
use facet_generate::{
    self as fg,
    generation::{
        CodeGeneratorConfig,
        indent::IndentWrite,
        plugin::EmitterPlugin,
        typescript::{self, TypeScript},
    },
    reflection::{
        RegistryBuilder,
        format::{Format, QualifiedTypeName},
    },
};

#[derive(Facet)]
#[facet(fg::namespace = "kit")]
struct Presence {
    online: bool,
}

#[derive(Facet)]
struct App {
    id: u32,
}

fn presence() -> QualifiedTypeName {
    QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string())
}

fn app() -> QualifiedTypeName {
    QualifiedTypeName::root("App".to_string())
}

#[derive(Debug)]
struct CurrentPresence;

impl EmitterPlugin<TypeScript> for CurrentPresence {
    fn referenced_types(&self, config: &CodeGeneratorConfig) -> Vec<QualifiedTypeName> {
        // Only the module that generates `App` names `Presence`.
        if config.generates(&app()) {
            vec![presence()]
        } else {
            vec![]
        }
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.generates(&app()) {
            let mut format = Format::TypeName(presence());
            typescript::requalify_format(config, &mut format);
            let presence = typescript::render_type(&format, config);
            writeln!(w, "export type CurrentPresence = {presence};")?;
        }
        Ok(())
    }
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = RegistryBuilder::new()
    .add_type::<App>()?
    .add_type::<Presence>()?
    .build()?;

let dir = std::env::temp_dir().join("facet_generate_guide_plugins_references");
# let _ = std::fs::remove_dir_all(&dir);
typescript::Installer::new("shared", &dir)
    .plugin(CurrentPresence)
    .generate(&registry)?;

let root = std::fs::read_to_string(dir.join("shared.ts"))?;
assert!(root.contains(r#"import * as Kit from "./kit";"#));
assert!(root.contains("export type CurrentPresence = Kit.Presence;"));
# std::fs::remove_dir_all(&dir)?;
# Ok(())
# }
```

## A worked example

This plugin gives every struct a constant holding its type name — `static readonly typeName`
in TypeScript, `const val TYPE_NAME` in a companion object in Kotlin — using only the public API.
It writes into the body with `type_body`, and for Kotlin also answers `has_type_body`, without
which the emitter would not open a body for a `data class` to write into. It skips enums and,
in Kotlin, the classes of an enum's variants.

**Rust**
```rust
use std::{io, sync::Arc};

use facet::Facet;
use facet_generate::{
    generation::{
        CodeGeneratorConfig,
        indent::IndentWrite,
        kotlin::{Kotlin, KotlinCodeGenerator},
        plugin::{EmitContext, EmitterPlugin},
        typescript::{TypeScript, TypeScriptCodeGenerator},
    },
    reflection::{RegistryBuilder, format::ContainerFormat},
};

/// Gives every struct a constant holding its type name.
#[derive(Debug)]
struct TypeNamePlugin;

impl TypeNamePlugin {
    /// A top-level struct: not an enum, and not the class of an enum's variant (Kotlin).
    fn applies(ctx: &EmitContext) -> bool {
        !ctx.is_variant() && matches!(ctx.container.format, ContainerFormat::Struct(..))
    }
}

impl EmitterPlugin<TypeScript> for TypeNamePlugin {
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if Self::applies(ctx) {
            writeln!(w)?;
            writeln!(w, r#"static readonly typeName = "{}";"#, ctx.name())?;
        }
        Ok(())
    }
}

impl EmitterPlugin<Kotlin> for TypeNamePlugin {
    // Kotlin opens a body for a `data class` only when a plugin asks for one.
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        Self::applies(ctx)
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if Self::applies(ctx) {
            writeln!(w, "companion object {{")?;
            w.indent();
            writeln!(w, r#"const val TYPE_NAME = "{}""#, ctx.name())?;
            w.unindent();
            writeln!(w, "}}")?;
        }
        Ok(())
    }
}

#[derive(Facet)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
enum Shape {
    Dot(Point),
    Empty,
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = RegistryBuilder::new().add_type::<Shape>()?.build()?;

let config = CodeGeneratorConfig::new("shapes".to_string());
let mut out = Vec::new();
TypeScriptCodeGenerator::new(&config)
    .with_plugins(vec![Arc::new(TypeNamePlugin)])
    .output(&mut out, &registry)?;
let typescript = String::from_utf8(out)?;

assert!(typescript.contains(
    r#"export class Point {
    constructor (public x: int32, public y: int32) {
    }

    static readonly typeName = "Point";
}"#
));

let config = CodeGeneratorConfig::new("com.example.shapes".to_string());
let mut out = Vec::new();
KotlinCodeGenerator::new(&config)
    .with_plugins(vec![Arc::new(TypeNamePlugin)])
    .output(&mut out, &registry)?;
let kotlin = String::from_utf8(out)?;

assert!(kotlin.contains(
    r#"data class Point(
    val x: Int,
    val y: Int,
) {
    companion object {
        const val TYPE_NAME = "Point"
    }
}"#
));
// The enum and its variant classes are left alone.
assert_eq!(kotlin.matches("TYPE_NAME").count(), 1);
# Ok(())
# }
```

For Swift the implementation is the TypeScript one with `public static let typeName = "…"`:
Swift calls `type_body` for every struct and enum without asking. For C# it is the Kotlin one
with a `public const string TypeName = "…";` — C# asks `has_type_body` before writing into a
`partial class`. To attach the plugin to a whole package, pass it to each language's installer
with `.plugin(TypeNamePlugin)`.

## Testing a plugin

Two checks catch most mistakes: a snapshot of what the plugin writes, and a build of the
generated code with the real compiler.

- **Snapshot the generated module.** Run the generator into a buffer, as the worked example
  does, and compare the result with [`insta`](https://docs.rs/insta) or
  [`expect_test`](https://docs.rs/expect_test). Going through the generator means the formats in
  `EmitContext` are respelled exactly as in real use. To snapshot one hook on its own, build an
  `EmitContext` with [`EmitContext::top_level`](crate::generation::plugin::EmitContext::top_level)
  and an [`IndentedWriter`](crate::generation::indent::IndentedWriter) over a `Vec<u8>` and call
  the hook directly; the registry and config's enum sets must then be respelled by hand first,
  as the generator would — Crux's
  [`tests.rs`](https://github.com/redbadger/crux/blob/master/crux_core/src/type_generation/facet/plugins/tests.rs)
  shows how.
- **Build what you generate.** Install into a temporary directory with the language's
  installer, then run the compiler over it: `swift build`, `gradle build`, `dotnet build` or
  `deno check`. This crate's own compilation tests do it, in `tests/swift_generation.rs`,
  `tests/kotlin_generation.rs`, `tests/csharp_generation.rs` and
  `tests/typescript_generation.rs`; `test_that_swift_code_naming_a_plugin_s_referenced_type_compiles`
  in the first is a test of a plugin's `referenced_types` and `module_helpers` in exactly this
  shape. Remember that a Swift package needs a serialization plugin beside yours to build.

The [contributing guide](crate::guide::contributing) describes how the crate's own tests are
layered.

## Examples to read

- The in-tree plugins, one file per language:
  [`bincode`](crate::generation::bincode) (`src/generation/bincode/*.rs`) and
  [`json`](crate::generation::json) (`src/generation/json/*.rs`). Between them they use nearly
  every hook, including `runtime_files`, `manifest_dependencies`, `field_annotations` and
  `enum_variant_annotations`.
- Crux's
  [plugins](https://github.com/redbadger/crux/tree/master/crux_core/src/type_generation/facet/plugins),
  which generate the shell side of a Crux app in all four languages. They write mostly through
  `after_type` (C# uses `type_body` for a property that has to be inside a record), add
  companion files, use `referenced_types` and `target_dependencies`, and define a small
  `Requalify` trait over the four languages' `requalify` functions so that code shared across
  languages can respell names generically.
