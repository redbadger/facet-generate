# Changelog

All notable changes to this project will be documented in this file.

## [0.22.0] - 2026-09-23

References **across namespaces** now generate code that compiles in every language: from the root namespace into a named one, from a named namespace back to the root, and between two named namespaces. Reflection now names every reference the way its type is registered, whatever wraps it, and rejects a reference to a type that isn't registered. Plugins get each generator's `requalify` and `requalify_format`, so they can name a type the way the emitter does, and can declare the types they name, so the module imports them. There's one exception. In Swift, the root target and a namespace that reference each other form a dependency cycle, which SwiftPM can't build, so generation now fails with an error naming the types involved. Output for anything that already compiled is unchanged, apart from a small change to builtin shadowing in C# and Swift, and to the namespace of a type held by a namespaced transparent wrapper (see Bug Fixes). The enum fix below is also available for the 0.19 line as [0.19.1](#0191---2026-09-23). `facet` stays pinned at `=0.46.5`, and `facet-generate-attrs` is unchanged at 0.18.0.

Reported downstream as [redbadger/crux#603](https://github.com/redbadger/crux/issues/603).

### 💥 Breaking Changes

- **`CodeGeneratorConfig::enum_type_names` and `unit_variant_enums` are now `BTreeSet<QualifiedTypeName>`** instead of `BTreeSet<String>`. They're keyed by qualified name, and they cover every enum in the registry, not just the current module's. Query them with the new `CodeGeneratorConfig::is_enum(&QualifiedTypeName)` and `is_unit_enum`, passing the name as the emitter sees it (after the generator rewrites references) [#155](https://github.com/redbadger/facet-generate/pull/155)
- **`CodeGeneratorConfig` gained a public `registry_type_names: BTreeSet<QualifiedTypeName>` field**: every container in the registry that the module was split from. A struct literal of `CodeGeneratorConfig` needs the new field [#155](https://github.com/redbadger/facet-generate/pull/155)
- **`CodeGeneratorConfig` gained a public `namespace: Namespace` field**: the namespace whose types the module generates, set when the registry is split into modules. Ask it with the new `CodeGeneratorConfig::generates(&QualifiedTypeName)`, which says whether the module generates a given type, so a plugin can pick its module without copying the language's module-naming rules. `CodeGeneratorConfig::new` sets it to the root namespace. A struct literal of `CodeGeneratorConfig` needs the new field [#173](https://github.com/redbadger/facet-generate/pull/173)
- **The Swift installer rejects a cycle between targets.** If the root target and a namespace reference each other, or two namespaces do, SwiftPM can't build the package, whatever the imports. That's the usual shape when a root `ViewModel` holds namespaced types that refer back to root types. Until now, generation succeeded and wrote a `Package.swift` that SwiftPM rejected. It now fails with an `InvalidInput` error naming the types on each edge, and suggests moving the shared types into a namespace of their own. It doesn't write `Package.swift`. **Code generation that calls the Swift installer now fails for such a registry** [#165](https://github.com/redbadger/facet-generate/pull/165)
- **`Error` gained a `DanglingTypeReference` variant**: `RegistryBuilder::build` now fails if a type reference names no registered container, giving the referring container, the field and the missing name. A dangling reference is always a reflection bug, so please report one. The check turns it into an error at reflection, where it used to become generated code that doesn't compile. It adds a variant to the exhaustive `Error` enum [#169](https://github.com/redbadger/facet-generate/pull/169)

### 🚀 Features

- **`typescript::requalify`, `csharp::requalify`, `kotlin::requalify` and `swift::requalify`**, and a `requalify_format` beside each: the spelling the emitter gives a reference to a registry name from the current module. `requalify` rewrites one name, and `requalify_format` every name in a `Format`, as the generator does to each container before emitting it. A plugin must pass any name it knows in registry spelling, or a format from `RegistryBuilder::format_of`, through one of them exactly once before `render_type`, `write_serialize_value`, `is_enum` or `is_unit_enum`. Neither is idempotent: in C# and Kotlin a second pass nests the root package. Formats that come through `EmitContext` already have it. Without this, the 0.22 enum lookups missed for a name such as an app's `Event` in a dotted C# package, and wrote `Event.Serialize(...)` for an all-unit enum [#168](https://github.com/redbadger/facet-generate/pull/168)
- **`EmitterPlugin::referenced_types`**: a plugin can declare, in registry spelling, the types its output names in a module. Each one is treated as a reference the module makes. In Swift, that module gets the `import` and the target dependency, and the edge takes part in the check for cycles between targets. In TypeScript, it gets the `import * as Ns`. Kotlin and C# need nothing, because they write fully qualified names. Without it, a type that only a plugin names, such as an operation's output in crux's generated effect dispatcher, got no import when nothing else in the module referred to its namespace. A declared type that isn't in the registry fails generation. It's a default method, so existing plugins are unaffected (reported as [redbadger/crux#614](https://github.com/redbadger/crux/issues/614)) [#173](https://github.com/redbadger/facet-generate/pull/173)

### 🐛 Bug Fixes

- **Swift: no root target when the root module has no types**: when every type is in a named namespace, the installer still declared a target named after the package, with no sources, and SwiftPM rejected the package. It now declares only the namespace targets, and the library product lists the top-level one [#158](https://github.com/redbadger/facet-generate/issues/158), [#176](https://github.com/redbadger/facet-generate/pull/176)
- **An enum from another namespace is serialized through its own functions (TypeScript, C#)**: each namespace is generated as its own module, and a module only knew its own enums. So a type holding an enum from a different namespace serialized it as if it were a class: `this.presence.serialize(serializer)` / `Kit.Presence.deserialize(deserializer)` in TypeScript (unit and data enums), and `Presence.Serialize(serializer)` in C# (all-unit enums). None of that compiles. TypeScript now calls `Kit.serializePresence(value, serializer)` / `Kit.deserializePresence(deserializer)` in both the bincode and JSON plugins. C# calls the enum's `…Bincode` helper, qualified the same way as the type [#154](https://github.com/redbadger/facet-generate/issues/154), [#155](https://github.com/redbadger/facet-generate/pull/155)
- **Builtin shadowing counts types from other modules that are in scope (C#, Swift)**: a C# namespace also sees the root namespace's types, and a Swift module also sees the types of the modules it imports. So a root `Unit` broke `()` fields in a C# namespace, and a `kit` type called `Set` broke `BTreeSet` fields in the Swift root module. The shadowing check now counts both, and writes the builtin fully qualified [#155](https://github.com/redbadger/facet-generate/pull/155)
- **TypeScript: a namespaced module imports the root types it references**: it wrote a root type's name bare, but only imported named namespaces. It now writes `import * as Example from "./example"` and `Example.Shared`, and enum functions follow as `Example.serializeLevel(...)`. The root module and a namespace now import each other. Generated code evaluates nothing across modules at load time, and a runtime round trip in both import orders confirms that's safe [#150](https://github.com/redbadger/facet-generate/issues/150), [#161](https://github.com/redbadger/facet-generate/pull/161)
- **C#: references to root and sibling types are qualified from the root package**: they were built from the current module's name, so a root type referenced from `Example.Kv` became `Example.Kv.Shared`, and a type in `b` referenced from `Example.A` became `Example.A.B.Status`. Neither namespace exists. They're now `Example.Shared` and `Example.B.Status`, and the `…Bincode` helpers follow [#149](https://github.com/redbadger/facet-generate/issues/149), [#163](https://github.com/redbadger/facet-generate/pull/163)
- **Kotlin: a root type referenced from a namespaced module lives in the root package**: it was written `com.example.kv.Level`, and is now `com.example.Level` [#148](https://github.com/redbadger/facet-generate/issues/148), [#165](https://github.com/redbadger/facet-generate/pull/165)
- **Swift: a namespaced target can reference root types**: it wrote them bare, without depending on or importing the root target. It now writes `import Example` and `Example.Level`, and depends on the root target. A same-named local type can no longer capture the reference. When a namespace references root, the library product lists the top-level target (`["Kv"]`), and consumers can still `import Example` [#151](https://github.com/redbadger/facet-generate/issues/151), [#165](https://github.com/redbadger/facet-generate/pull/165)
- **A reference records the namespace its type is registered under, whatever wraps it**: references were computed in about ten places, each with its own idea of the namespace context. So an `Option<_>` of a type that inherits its namespace was recorded as a `Root` type that doesn't exist ([#167](https://github.com/redbadger/facet-generate/issues/167)), and a ROOT-pinned type nested in a generic took the container's namespace ([#160](https://github.com/redbadger/facet-generate/issues/160)). Once ROOT references from namespaced modules were qualified, such a phantom reference became a Swift target cycle, or a root type that doesn't exist in the other languages. One walker now names every reference, including `Option`, sequences, sets, maps, tuples, arrays and transparent wrappers, with the function that names a type when it's registered [#169](https://github.com/redbadger/facet-generate/pull/169)
- **Four more phantom references, found by the new dangling-reference check**: a type reached through a chain of transparent wrappers was never registered; a newtype variant's tuple payload became the type name `(…)`; an anonymous tuple field never registered its elements; and an array in a tuple position was pushed twice. Also, an enum inside a newtype or tuple struct took its last variant's payload (`struct W(E)` came out as `NewTypeStruct(Seq(…))`) [#169](https://github.com/redbadger/facet-generate/pull/169)
- **A transparent wrapper's `fg::namespace` applies to the type it wraps in every position**: a type is generated as the one its chain of transparent wrappers finally wraps, in the namespace of the innermost wrapper that has one, unless the type pins its own. It used to be registered in the wrapper's namespace but referred to from the root namespace, and a field of the wrapper's type in a `Vec`, an `Option` or a map registered a second copy in the root namespace. So a direct field of the wrapper's type didn't compile, and one that sat beside such a field compiled against the root copy. That case now refers to the wrapper's namespace, and the root copy is gone [#169](https://github.com/redbadger/facet-generate/pull/169)
- **A renamed type under a field's `fg::namespace` keeps its new name**: a `#[facet(rename = "…")]` type used in a field with its own `fg::namespace` was registered under its Rust name and referred to by its new one [#169](https://github.com/redbadger/facet-generate/pull/169)
- **`Range`, `PhantomData` and `Infallible` no longer panic in nested positions**: these are user structs that facet describes as scalars, and they're now reflected as containers everywhere. An unsupported type such as `Result`, anywhere in a field's type, now skips the field in every position, as it already did in most. Some positions used to panic, and an `Option` of one failed reflection; a newtype variant whose payload is skipped becomes a unit variant [#169](https://github.com/redbadger/facet-generate/pull/169)

### 🧪 Tests

- **A new `with_enums_across_namespaces` fixture** covers root → namespace, namespace → namespace, namespace → root and cross-module builtin shadowing, for unit and data enums, in all four languages, with bincode and JSON. Every language now compiles the output (deno, dotnet, gradle, swift build; Swift JSON is excluded until [#157](https://github.com/redbadger/facet-generate/issues/157)). New runtime round trips against Rust bincode cover root ↔ namespace in TypeScript, C#, Kotlin and Swift.
- **Reference positions:** `reflection/reference_tests.rs` checks a type referenced from 21 positions, whether it inherits its namespace, is pinned to ROOT, or is explicitly namespaced. The #167 shape is compiled by all four installers.

## [0.21.1] - 2026-09-22

### 🐛 Bug Fixes

- **A Kotlin variant named after an import its enum never uses is no longer rejected** —
  the reserved-name pre-pass that 0.21.0 introduced ran the import-collision rule on every
  variant of a data-carrying enum, because Kotlin and C# emit each one as a nested class.
  But a nested class only shadows the file's imports inside the enum that declares it, and
  `Bytes`, `UUID`, `BigInteger`, `Int128` and the `NTupleN` helpers are only imported or
  written at all beside a field of that format. `crux_kv`'s `Value::Bytes(Vec<u8>)` — a
  `List<UByte>` payload in an enum with no bytes field — was therefore refused with
  `type Bytes collides with the Bytes import`, which made 0.21.0 unable to generate Kotlin
  for a released crux capability. Such names are now *format-bound*: a top-level type is
  rejected only when the module has a field of that format, and a variant only when its own
  enum has one. `Serializer`, `Deserializer` and the other names every generated type
  mentions are still always rejected, as are the same names in every other position. The
  shared shadowing fixture now carries crux_kv's `Value` enum, so the Kotlin, C#, Swift and
  TypeScript compile tests cover a nested `Bytes` beside a module-level `Bytes` import
  [#147](https://github.com/redbadger/facet-generate/pull/147)

## [0.21.0] - 2026-09-22

Extensibility work for **out-of-tree `EmitterPlugin` implementations**. Everything a
plugin needs to emit code *about* a type — where to hook in, how to name it, how to
render it, how to serialize it — is now part of the public API, so a plugin no longer
has to re-implement (and drift from) the emitters' own naming and rendering rules.
Nothing the in-tree plugins generate changes for input that already produced complete
output: every existing snapshot and expect-file is byte-for-byte identical. Input that
used to lose a type silently is now rejected (see Bug Fixes), and generated code no
longer trips over the target language's reserved words or builtin type names. `facet`
stays pinned at `=0.46.5` (no newer stable release exists), the lockfile is refreshed to
the latest semver-compatible version of every dependency, and `facet-generate-attrs` is
unchanged and stays at 0.18.0.

Motivated by, but not specific to, the effect-handler code generation in
[redbadger/crux#581](https://github.com/redbadger/crux/pull/581).

### 💥 Breaking Changes

- **`CodeGeneratorConfig` gained a public `declared_type_names: BTreeSet<String>` field.**
  As with `parent` in 0.20.0, the struct has all-public fields and is not
  `#[non_exhaustive]`, so a struct literal or exhaustive destructuring no longer compiles;
  use `CodeGeneratorConfig::new()` and the builder methods. The field is filled in by
  `update_from` with every type and data-carrying variant the module declares, and is what
  lets `render_type` decide whether a builtin needs qualifying [#126](https://github.com/redbadger/facet-generate/pull/126)
- **`swift::case_name` now escapes Swift keywords**, so a plugin that used it for a
  variant named `Struct` or `Default` sees `` `struct` `` / `` `default` `` where it
  previously saw the bare (uncompilable) word [#126](https://github.com/redbadger/facet-generate/pull/126)
- **`EmitterPlugin::target_dependencies` now takes the module's `&CodeGeneratorConfig`.**
  The Swift installer asks every plugin for its SPM target edges once per module, but
  gave the plugin nothing to say which module it was being asked about, so an edge meant
  for one target (Crux's `.product(name: "Shared", package: "Shared")` for the app's FFI
  bridge) was written onto every namespaced feature target as well. A plugin whose edge
  belongs to one module compares `CodeGeneratorConfig::module_name` and returns nothing
  for the rest; a plugin whose edge every target needs ignores the argument.
  `manifest_dependencies` keeps its signature, because the manifest's `dependencies:` are
  the package's and are asked for once. Any plugin implementing `target_dependencies`
  (Swift-only) must add the parameter [#136](https://github.com/redbadger/facet-generate/pull/136)

### 🚀 Features

- **`RegistryBuilder::format_of::<T>()`** — the `Format` a struct field of type `T`
  would be given, so a plugin can name and serialize a type it did not receive as the
  container being emitted. Named containers become `TypeName(qualified)` honouring
  `rename` / `fg::namespace` / `transparent`, `()` becomes `Unit`, and `Option`, `Vec`,
  maps, tuples and primitives become the corresponding structural format. `T`'s
  container types should be added with `add_type` first [#123](https://github.com/redbadger/facet-generate/pull/123)
- **Public `write_serialize_value` per language** — `generation::bincode::{swift, kotlin, typescript, csharp}::write_serialize_value(w, value_expr, format, config)` emits exactly the bincode serialization statements the plugin writes for a field of that format. Each documents its precondition: a `serializer` variable of the language's conventional type in scope, and container depth managed by the caller [#123](https://github.com/redbadger/facet-generate/pull/123)
- **Public naming and type-rendering helpers** so plugins reproduce emitter naming exactly: `generation::{swift,kotlin,typescript,csharp}::render_type(format, config)`, plus `swift::case_name`, `kotlin::variant_class_name`, `kotlin::enum_constant_name` and `csharp::escape_identifier` (which was private in `bincode::csharp`) [#123](https://github.com/redbadger/facet-generate/pull/123)
- **Reserved words are escaped in every language.** A field or variant whose generated
  identifier is a keyword (`default`, `in`, `class`, `where`, …) is now written with the
  language's own escape — backticks in Swift and Kotlin, `@` in C#, and in TypeScript a
  `_`-suffixed parameter or local while the property keeps its name — everywhere the
  emitters and the bincode/JSON plugins spell it: declarations, initialiser labels,
  bindings, patterns and property accesses. Escaping is pure quoting, so wire names are
  untouched. New public helpers `swift::field_name`, `swift::escape_identifier`,
  `kotlin::property_name`, `kotlin::escape_identifier`, `typescript::param_name` and
  `typescript::is_reserved_word` let plugins reproduce the emitters' spelling exactly;
  the word lists live in one `naming` module per language with a shared driver [#126](https://github.com/redbadger/facet-generate/pull/126)
- **Builtin type names are qualified when a generated type shadows them.** A Rust type
  called `Set`, `List`, `Map`, `String`, `Dictionary`… is emitted under its own name, and
  every reference to the builtin of the same name in that module is written fully
  qualified instead — `kotlin.collections.Set<T>`, `Swift.Set<T>`,
  `global::System.Collections.Generic.HashSet<T>`, `globalThis.Map<K, V>` — including the
  module-level serialization helpers. Prompted by `crux_kv`'s `Set` operation struct, which
  previously hid `kotlin.collections.Set` for the whole generated package. Nothing changes
  when no name is shadowed [#126](https://github.com/redbadger/facet-generate/pull/126)
- **Names that can be neither escaped nor qualified are rejected before anything is
  written.** A type named after an explicit import the generated code depends on
  (`Serializer`, `Bytes`, `UUID`, the TypeScript `str`/`Seq` aliases…), or a field that
  would become a member the language or the generated code already provides (`toString`,
  `copy`, `hashCode`, `GetHashCode`, a C# property named like its class, `serializer`,
  `deserializer`), now fails generation with an `InvalidInput` error that names the
  language, the offending type or field, why it clashes, and suggests
  `#[facet(rename = "...")]` [#126](https://github.com/redbadger/facet-generate/pull/126)
- **`EmitContext::variants()`** — the container's variants keyed by discriminant, or `None` when it is not an enum [#123](https://github.com/redbadger/facet-generate/pull/123)

### 🐛 Bug Fixes

- **Keyword-named fields and variants generated code that did not compile** — a Rust
  `r#default` field came out as `public var default: String` in Swift and `val in: Int` in
  Kotlin, a `Default` variant as `case default`, and a TypeScript constructor took a
  parameter literally named `class`. All four targets now compile such types; the
  `generate_types_with_keywords` fixture (dormant since the typeshare days) is live again
  for every language, and Swift, Kotlin and TypeScript gained compile tests alongside the
  existing C# one [#126](https://github.com/redbadger/facet-generate/pull/126)
- **fix(kotlin): `#[facet(bytes)]` fields compile under the JSON plugin** — the emitter
  writes such a field as `Bytes`, but the JSON plugin never imported it, and the runtime's
  `com.novi.serde.Bytes` is not `@Serializable` anyway, so `gradle build` failed with
  `Unresolved reference 'Bytes'`. The plugin now emits a `BytesSerializer` and a
  `typealias Bytes` that binds it, on the same pattern as `UUID`, encoding the value as a
  JSON array of bytes [#126](https://github.com/redbadger/facet-generate/pull/126)
- **The Kotlin compile test compiles something now.** The installer writes the package tree
  at the project root while Gradle reads `src/main/kotlin`, so `test_that_kotlin_code_compiles`
  had always ended in `compileKotlin NO-SOURCE` and asserted nothing. All Kotlin compile tests
  now share one setup that moves the sources into the source set, pins the JVM target, and
  fails unless `compileKotlin` genuinely ran. Making it real exposed two generator bugs that
  are recorded in the test rather than fixed here: Kotlin JSON cannot serialise `u128`/`i128`
  because the unconditional `import java.math.BigInteger` outranks the plugin's
  `typealias BigInteger`, and Kotlin bincode output for the main fixture does not compile
  (128-bit integers, `char`, and `Vec<()>` / maps of unit) [#126](https://github.com/redbadger/facet-generate/pull/126)
- **`after_type` now fires for every top-level type, in every language** — it was only called for TypeScript enums and C# all-unit enums, which made it unusable as the "emit something alongside this type" hook it is documented to be. It is now called after every top-level container: Swift structs and enums, Kotlin `data class` / `data object` / `enum class` / `sealed interface`, TypeScript classes (as well as enums), and C# classes, sealed records and `abstract record` variant hierarchies (as well as enums). It is still never called for an individual enum variant, and the context is always a top-level one [#123](https://github.com/redbadger/facet-generate/pull/123)

  **A third-party plugin implementing `after_type` will now be called at call sites it
  never saw before**, and must guard on the container shape (`ctx.container.format`, or
  the new `ctx.variants()`) and return `Ok(())` for the rest — as the in-tree bincode
  and JSON plugins already did. The hook table in the `generation::plugin` module docs
  records exactly where it fires.
- **Two Rust types that generate the same name are now an error** — `RegistryBuilder`
  kept its "already processed" set by generated name alone, so when a second, different
  Rust type reflected to a name already in the registry (an app's `secret::Delete` beside
  `crux_kv`'s `Delete`, say) it was treated as a recursive visit and silently left out.
  The builder now remembers which Rust type claimed each name and returns
  `Error::DuplicateTypeName`, naming both types by their Rust path and the two ways to
  resolve it: `#[facet(rename = "...")]` or `#[facet(fg::namespace = "...")]`. This covers
  types added directly and types reached through fields and variants, in the root and in
  named namespaces, and collisions caused by `rename`, `type_tag` or a field-level
  namespace override. The same identity check fixes two neighbours: a `rename` on one type
  no longer applies to an unrelated type with the same Rust identifier, and the check for
  a generic used with different parameters is keyed by declaration, so `a::Foo<A>` and
  `b::Foo<B>` are reported as a name collision rather than an unsupported generic.
  `Error` gains a variant, so an exhaustive `match` on it needs a new arm. Fixes
  [redbadger/crux#601](https://github.com/redbadger/crux/issues/601) [#137](https://github.com/redbadger/facet-generate/pull/137)

## [0.20.0] - 2026-08-26

Two generated-output fixes, released as a minor bump because one of them widens the
public API. `facet` stays pinned at `=0.46.5`, and `facet-generate-attrs` is unchanged
and stays at 0.18.0.

### 💥 Breaking Changes

- **`CodeGeneratorConfig` gained a public `parent: Option<String>` field.** The struct
  has all-public fields and is not `#[non_exhaustive]`, so any code building one with a
  struct literal (or destructuring it exhaustively) no longer compiles. Use
  `CodeGeneratorConfig::new()` and the `with_*` builder methods, which is what the field
  is populated by. Nothing else in the API moved [#122](https://github.com/redbadger/facet-generate/pull/122)

### 🐛 Bug Fixes

- **fix(kotlin): root a sibling namespace's package at the parent, not the module** — a type in a *different* named namespace was qualified with the current module's full name, so a reference from `feature` to a type in `kit` came out as `com.example.feature.kit.Row` when the type is declared in `com.example.kit`, generating source that does not compile. `CodeGeneratorConfig` now records the `parent` set by `with_parent` and a new `root_package()` accessor returns it (falling back to `module_name`), so a root module and its namespaced children agree on where namespace packages live [#122](https://github.com/redbadger/facet-generate/pull/122)
- **fix(typescript): use type-only imports for runtime interfaces** — generated projects no longer report TS1484 for `Serializer` and `Deserializer` when `verbatimModuleSyntax` is enabled [#119](https://github.com/redbadger/facet-generate/pull/119)

## [0.19.1] - 2026-09-23

A patch release for crux_core 0.20.x. It carries one fix from the 0.22 line, re-implemented for 0.19: there's no public API change, and output only changes for code that didn't compile before. `facet-generate-attrs` is unchanged at 0.18.0.

### 🐛 Bug Fixes

- **An enum from another namespace is serialized through its own functions (TypeScript, C#)**:
  the installers generate each namespace as its own module, and a module only knew about its own
  enums. So a type holding an enum from a different namespace serialized it as if it were a class:
  `this.presence.serialize(serializer)` and `Kit.Presence.deserialize(deserializer)` in TypeScript
  (for unit and data enums), and `Presence.Serialize(serializer)` in C# (for all-unit enums). None of
  that compiles.
  - TypeScript now calls the enum's free functions through the namespace import:
    `Kit.serializePresence(value, serializer)` and `Kit.deserializePresence(deserializer)`.
  - C# now calls the enum's `…Bincode` helper, qualified the same way as the type.
  - Enums in the same namespace, and everything else, generate exactly as in 0.19.0.
  - The fix applies to generation through `Installer::generate`. It was reported downstream as
    [redbadger/crux#603](https://github.com/redbadger/crux/issues/603).
  [#154](https://github.com/redbadger/facet-generate/issues/154)

## [0.19.0] - 2026-08-06

A dependency-only release: `facet` moves from `=0.44` to `=0.46.5`. No generation
behaviour changes — output is byte-for-byte identical to 0.18.0.

`facet-generate-attrs` is released alongside as **0.18.0** for the same reason.

### 💥 Breaking Changes

- **Requires `facet` 0.46.5** (was 0.44). `facet` is a public dependency of both crates
  — `facet_generate` takes `Shape`, `Field` and `Variant` in its API, and
  `facet-generate-attrs` builds its attribute grammar with `facet::define_attr_grammar!`
  — so a consumer pinning `facet =0.44` cannot use this version, and vice versa. Both
  crates need to move together with whatever pins `facet` downstream.

### ⚙️ Miscellaneous Tasks

- No source changes were required for the upgrade. The facet 0.44 → 0.46 delta is
  additive and confined to parts of facet this crate does not use: `facet-core` gained
  list `pop`/`swap` operations, `char` and `()` now advertise `Clone` in their
  `TypeOps`, `Result`'s hand-written drop glue moved, and `facet-reflect` grew its
  `poke` module (map, set, option, result, tuple, pointer, ndarray, list_like,
  dynamic_value). `facet-macros` and `facet-macro-parse` are byte-identical, so there
  are no new derive attributes. The `Shape`/`Field`/`Variant` surface this crate reads
  is unchanged, which is why no snapshots or expect-files moved.

## [0.18.0] - 2026-08-03

TypeScript enums are now generated as **discriminated union types** instead of abstract class hierarchies. This is a breaking change for any code that was constructing or matching TypeScript enum values, but the new output is far more idiomatic and integrates naturally with TypeScript's type narrowing.

### 💥 Breaking Changes

- **TypeScript enums are no longer emitted as abstract classes.** Each enum is now a `type` alias over a union of object literals. Code that used `new FooVariant(...)` or `instanceof` checks must be updated to use the generated constructor functions and `matchX` helpers described below. [#106](https://github.com/redbadger/facet-generate/pull/106)

### 🚀 Features

- **feat(typescript): discriminated union types for enums** — Rust enums now generate a `type` alias, a typed constructor arrow function per variant, and an exhaustive `matchX<R>(...)` helper [#106](https://github.com/redbadger/facet-generate/pull/106)
- **feat(typescript): `EnumTagging` in format AST** — `#[facet(tag = "...")]` and `#[facet(tag = "...", content = "...")]` are now reflected into the format as `EnumTagging::Internal` and `EnumTagging::Adjacent`, controlling the shape of each variant's object literal [#106](https://github.com/redbadger/facet-generate/pull/106)
- **feat(typescript): standalone serialize/deserialize for enums** — the Bincode and JSON plugins now emit `serializeX(value, serializer)` / `deserializeX(deserializer)` functions alongside each enum type rather than expecting a `.serialize()` method on the value [#106](https://github.com/redbadger/facet-generate/pull/106)

### 🐛 Bug Fixes

- **fix(typescript): quote non-identifier property keys in match function** — variant names that are not valid JavaScript identifiers (e.g. `"number-array"` from `rename_all = "kebab-case"`) are now correctly quoted as string keys in the `matchX` cases object type [#106](https://github.com/redbadger/facet-generate/pull/106)
- **fix(typescript): correct `i64`/`i128` deserialization when the low limb has its high bit set** — the previous BigInt `|` of signed halves sign-extended the low limb and dropped the upper one, so any `i64` at or above `2^31` (including every current epoch-millis timestamp) decoded incorrectly [#110](https://github.com/redbadger/facet-generate/pull/110)
- **fix(csharp): escape reserved C# identifiers in bincode output** — generated local variable names that collide with C# keywords (e.g. a field named `event` or `string`) are now prefixed with `@`, so the emitted bincode serializers compile [#109](https://github.com/redbadger/facet-generate/pull/109)
- **fix(typescript): reject truncated input instead of decoding a short value** — the binary deserializer's `read()` returned however many bytes were left in the buffer, so a length-prefixed field (`String`, `Vec<u8>`) whose payload was cut short silently deserialized as a shorter value, while a truncated fixed-width field surfaced as an opaque `RangeError` from `DataView`. Both now throw an error naming the offset and the shortfall [#112](https://github.com/redbadger/facet-generate/pull/112)

### 🧪 Tests

- Enabled TypeScript output for 21 previously-disabled or untested expect-file test cases covering structs, unit enums, externally/internally/adjacently tagged enums, skipped variants, readonly fields, deprecation notices, and anonymous struct variants [#106](https://github.com/redbadger/facet-generate/pull/106)
- Added edge-case round-trip coverage for every multi-limb integer in the TypeScript runtime (`i64`, `u64`, `i128`, `u128` at zero, ±1, the type extremes, and the 32-/64-bit limb boundaries), checked against bincode's encoding rather than against the runtime itself [#111](https://github.com/redbadger/facet-generate/pull/111)

### ⚙️ Miscellaneous Tasks

- chore(typescript): The `serde` runtime now reads and writes 64-bit integers with the native `DataView` `BigInt64`/`BigUint64` accessors instead of splitting them into 32-bit limbs, making `U64`/`U128` consistent with the `I64`/`I128` fix and retiring the now-unused `BIG_32` constants [#111](https://github.com/redbadger/facet-generate/pull/111)

---

#### Usage examples

Given this Rust enum:

```rust
#[derive(Facet)]
#[repr(C)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Point,
}
```

The generated TypeScript is:

```typescript
export type Shape =
    | { kind: "Circle"; radius: float64 }
    | { kind: "Rectangle"; width: float64; height: float64 }
    | { kind: "Point" };

export const shapeCircle = (radius: float64): Shape => ({ kind: "Circle", radius });
export const shapeRectangle = (width: float64, height: float64): Shape => ({ kind: "Rectangle", width, height });
export const shapePoint = (): Shape => ({ kind: "Point" });

export function matchShape<R>(value: Shape, cases: {
    Circle: (v: Extract<Shape, { kind: "Circle" }>) => R;
    Rectangle: (v: Extract<Shape, { kind: "Rectangle" }>) => R;
    Point: (v: Extract<Shape, { kind: "Point" }>) => R;
}): R {
    return cases[value.kind as Shape["kind"]](value as never);
}
```

**Constructing values:**

```typescript
const circle = shapeCircle(3.14);
const rect = shapeRectangle(10, 20);
const point = shapePoint();
```

**Exhaustive pattern matching:**

```typescript
const area = matchShape(shape, {
    Circle: ({ radius }) => Math.PI * radius ** 2,
    Rectangle: ({ width, height }) => width * height,
    Point: () => 0,
});
```

**Type narrowing without `matchShape`** — TypeScript's built-in narrowing works directly on the `kind` field:

```typescript
if (shape.kind === "Circle") {
    console.log(shape.radius); // TypeScript knows this is a Circle
}
```

**Internally tagged enums** (`#[facet(tag = "type")]`) inline the tag alongside the payload fields:

```rust
#[facet(tag = "type")]
pub enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
}
```

```typescript
export type Event =
    | { type: "Click"; x: int32; y: int32 }
    | { type: "KeyPress"; key: str };
```

**Adjacently tagged enums** (`#[facet(tag = "type", content = "data")]`) wrap the payload under a content field:

```rust
#[facet(tag = "type", content = "data")]
pub enum Message {
    Text(String),
    Image { url: String, alt: String },
}
```

```typescript
export type Message =
    | { type: "Text"; data: str }
    | { type: "Image"; data: { url: str; alt: str; } };
```

## [0.17.2] - 2026-06-25

### 🐛 Bug Fixes

- fix(swift): Types containing a map field are now only marked as `Hashable` when both the map key and value are `Hashable`; previously a non-hashable value type was not checked, causing the containing type to be incorrectly marked as conformant [#105](https://github.com/redbadger/facet-generate/pull/105)
- fix(swift): Types in named namespaces are now correctly recognised as `Hashable` [#105](https://github.com/redbadger/facet-generate/pull/105)

### ⚙️ Miscellaneous Tasks

- chore: Update Rust dependencies

## [0.17.1] - 2026-06-19

### 🚀 Features

- feat(uuid): Add native UUID support for Kotlin, Swift, TypeScript, and C# [#100](https://github.com/redbadger/facet-generate/pull/100)

### 🐛 Bug Fixes

- fix(typescript): Fix import path when using `fg::namespace="other"` [#99](https://github.com/redbadger/facet-generate/pull/99)
- fix(swift): `Map` is `Hashable` when its key is `Hashable` and its value is `Hashable` [#102](https://github.com/redbadger/facet-generate/pull/102)
- fix(swift): Generate both `Hashable` and `Equatable` conformances when possible [#102](https://github.com/redbadger/facet-generate/pull/102)
- fix(swift): Check all `TypeName` fields for `Hashable`/`Equatable` regardless of declaration order [#102](https://github.com/redbadger/facet-generate/pull/102)

### ⚙️ Miscellaneous Tasks

- chore: Mark `Format` and `Feature` as `#[non_exhaustive]` so future variants are not breaking changes

### 📝 Documentation

- Updated README output snippets to reflect current generated code [#103](https://github.com/redbadger/facet-generate/pull/103)

## [0.17.0] - 2026-04-19

This is a major release that introduces a new **plugin-based emitter architecture**, removes Java code generation entirely, drops Deno support for TypeScript, and brings significant improvements to Swift native type generation.

### 💥 Breaking Changes

- **New plugin-based emitter architecture** — emitters are now built around an `EmitterPlugin<Lang>` trait, enabling modular and composable code generation per language [#88](https://github.com/redbadger/facet-generate/pull/88)
- **Removed Java code generation** — Java support has been fully removed as it had diverged too far; use Kotlin instead
- **Dropped Deno support for TypeScript** — modern Deno works with Node packages, so dedicated Deno support is no longer needed [#89](https://github.com/redbadger/facet-generate/pull/89)
- **Removed `Encoding` from plugins** — encoding is no longer part of the plugin configuration
- **Renamed `CodeGen` to `CodeGenerator`** — and associated implementors [#86](https://github.com/redbadger/facet-generate/pull/86)

### 🚀 Features

- feat(plugins): Introduce `EmitterPlugin<Lang>` trait with plugin store for modular emitter composition [#88](https://github.com/redbadger/facet-generate/pull/88)
- feat(plugins): Migrate Kotlin, Swift, TypeScript, and C# emitters to plugin-based architecture
- feat(plugins): Plugin-based configuration system replacing the previous encoding-based approach
- feat(writer): Child `IndentedWriter` support for nested code generation
- feat(swift): Detect `Hashable`, `Equatable`, and `Indirect` conformance automatically [#91](https://github.com/redbadger/facet-generate/pull/91)
- feat(swift): Remove remaining wrapper types (`Slice`, `Int128`, `UInt128`) in favour of native Swift types [#91](https://github.com/redbadger/facet-generate/pull/91)
- feat(swift): Add compilation conformance tests
- feat(swift): Use automatic test discovery on Linux with Swift 6

### 🐛 Bug Fixes

- fix(swift): Handling of 128-bit integers (`Int128`, `UInt128`) [#97](https://github.com/redbadger/facet-generate/pull/97)
- fix(swift): Problem with Tuple handling in JSON serialization [#90](https://github.com/redbadger/facet-generate/pull/90)
- fix(csharp): Conform C-style enum discovery to new `Language` model
- fix: Don't strip comments for Bincode
- fix: Error on bad namespace attribute [#82](https://github.com/redbadger/facet-generate/pull/82)

### ⚙️ Miscellaneous Tasks

- chore(ci): Run swift tests on Unix only [#97](https://github.com/redbadger/facet-generate/pull/97)
- chore: Rename `CodeGen` to `CodeGenerator` and associated implementors [#86](https://github.com/redbadger/facet-generate/pull/86)
- chore: Pass `Language` by reference throughout emitters
- chore: Inline Kotlin feature-based code into relevant plugins
- chore: Improve docs and fix warnings [#82](https://github.com/redbadger/facet-generate/pull/82)

## [0.16.0] - 2026-03-13

This is a major release with several breaking changes, including a new simplified public API, and an upgrade to facet v0.44. It also introduces C# code generation (still experimental), deprecates Java in favor of Kotlin and removes BCS support.

### 💥 Breaking Changes

- **Upgraded to facet v0.44** — this changes the underlying reflection framework
- **New simplified public API** — the API surface has been streamlined for ease of use
- **Removed BCS (Binary Canonical Serialization) support**

### 🚀 Features

- feat(csharp): Support experimental C# type generation [#73](https://github.com/redbadger/facet-generate/pull/73)
- feat(typescript): Refactor TypeScript generator to multi-target architecture with `Module::write` [#67](https://github.com/redbadger/facet-generate/pull/67)
- feat: Support `rename` and `rename_all` attributes for containers and fields
- feat(java): Deprecated Java code generation now that Kotlin is available [#69](https://github.com/redbadger/facet-generate/pull/69)
- feat: Crates moved into a workspace structure (`crates/facet_generate`, `crates/facet-generate-attrs`)

### 🐛 Bug Fixes

- fix: Use absolute paths with `include_dir!` for Bazel compatibility
- fix: Respect `external_packages` when emitting serde runtime
- fix: Build and test on Windows (#62, #63)

### ⚙️ Miscellaneous Tasks

- chore: Refactored emitters to use `crate::emit` macro
- chore: Refactored tests for new API
- chore: Updated README and documentation

## [0.15.0] - 2026-03-02

This is a potential breaking change.

Updates the Swift code generation, as a second implementation of the Emitter pattern that is used for Kotlin generation. The generated code is slightly different, but is more idiomatic Swift and should be a drop in replacement (except for code generated when serialization support is not needed, which is simpler and fully Swift native).

### 🚀 Features

- feat(swift): Complete rewrite of the Swift emitter, following the architecture pattern used for Kotlin (`Emitter<Language>` trait with phantom type parameter)
- feat(swift): Introduce `Module` abstraction for writing preamble and organizing output
- feat(swift): RAII block guards for cleaner code generation
- feat(swift): Move encoding configuration to the `Language` struct
- feat(swift): Improved Bincode support implementation in new Swift emitter
- feat(swift): Use native Swift types when not serializing, removing unnecessary serde runtime dependency

### 🐛 Bug Fixes

- fix(swift): Don't qualify types that are in the same module
- fix(swift): Correct namespacing in deserialize expressions
- fix(swift): Don't call `.init()` explicitly
- fix(swift): Fix self-imports and add newline after imports
- fix(swift): Move serialization helpers to features in emitter
- fix: Streamlined build processes

### 🧪 Tests

- test(swift): Enable 20 previously ignored Swift JSON tests
- test(swift): All insta and expect_file snapshots updated to match new emitter output

### ⚙️ Miscellaneous Tasks

- chore: Update Rust dependencies

## [0.14.0] - 2026-02-02

### 🚀 Features

- feat(kotlin): Kotlin codegen now uses the native runtime that was added in v0.13.2
- feat(kotlin): Use ByteArray instead of Bytes for better native type support
- feat(kotlin): Add value class Bytes to implement equals and hashCode
- feat(kotlin): Only emit import for Bytes if needed

### 🐛 Bug Fixes

- fix(ci): Remove if condition

## [0.13.2] - 2026-01-30

### 🚀 Features

- feat: Allow type with same name in ROOT and other namespace
- Add runtime for Kotlin native (note it is not yet used in Kotlin codegen, which will be fixed in the next release v0.14)

### 🐛 Bug Fixes

- fix(swift): Fix external type always used regardless of namespace
- fix(typescript): Fix external type always used regardless of namespace
- fix(kotlin): resolve Kotlin generation issues
- fix(kotlin): fmt, tweaks and tests
- fix(kotlin): revert native type changes for serde compatibility

## [0.13.1] - 2026-01-17

### 🐛 Bug Fixes

- fix(kotlin): fully qualify type references
- fix(kotlin): respect external packages
- fix(kotlin): fully qualify types when calling deserialize
- fix(kotlin): use property accessor instead of get method

## [0.13.0] - 2025-12-17

#### -  `#[facet(transparent)]` being ignored on multiple cases

Transparent NewTypes over scalar values would generate some named type without a definition. In typescript it would generate `MyType: any`. Now something like

```rust
#[facet(transparent)]
struct Wrapper(String);
```

will correctly be considered a `String` in generation

#### - More support for byte types 

- `bytes::Bytes` (requires `#[facet(bytes)]`)
- `Option<bytes::Bytes>` (requires `#[facet(bytes)]`)
- `Option<Vec<u8>>`
- `Option<&[u8]>`
- `[u8; N]`

#### - Support for `#[facet(bytes)]` and `#[facet(transparent)]` in enum variants

#### - Fixed transparent newtypes with `#[facet(bytes)]` not triggering the bytes generation

#### - Bumped `facet`

Many thanks to [Jeremy](https://github.com/o0Ignition0o) and [Felix](https://github.com/ManevilleF) for this contribution.

## [0.12.1] - 2025-11-16

Support for `#[facet(opaque)]` on struct fields. This attribute can be used to indicate that a field should be treated as opaque, meaning that it should not be reflected or serialized. You can use this when a struct field references another type that does not implement the `Facet` trait.

The intermediate representation will describe a struct without opaque fields. If this results in a struct with no fields, it will be represented as a unit struct. This applies to both structs and struct variants of enums.

Example, in a struct:

```rust
struct WithoutFacet;
#[derive(Facet)]
struct WithFacet {
    #[facet(opaque)]
    ignore: WithoutFacet,
}
```

Example, in an enum struct variant:

```rust
struct WithoutFacet;
#[derive(Facet)]
#[repr(C)]
enum WithFacet {
    WithNonFacetType {
        #[facet(opaque)]
        ignore: WithoutFacet,
    },
}
```

Example, in a tuple struct variant:

```rust
struct WithoutFacet;
#[derive(Facet)]
#[repr(C)]
enum WithFacet {
    WithNonFacetType(String, #[facet(opaque)] WithoutFacet),
}
```

## [0.12.0] - 2025-10-20

### Breaking changes!!

1. Namespaces are now propagated (inherited by child types) regardless of whether they are specified at the type level or the field level (call site). Explicit annotation overrides any namespace inheritance. When overriding a namespace, the new namespace can be the root (use `#[facet(namespace = None)]`) or named (use `#[facet(namespace = "new_namespace")]`). The new namespace will then continue to be propagated to subsequent types. If there is ambiguity (e.g. a type would be in multiple namespaces), the type generation will produce an error indicating the conflicting namespaces.

2. Type reflection no longer panics and instead returns an error if:
   * there is a problem building the registry,
   * non-special generic types are used with different type parameters,
   * there is an unsupported layout,
   * namespaces are conflicting,
   * namespaces have invalid names, or
   * attributes are malformed.

## [0.11.7] - 2025-10-15

Introduces namespace propagation (currently only when using call-site annotations). See https://github.com/redbadger/facet-generate/pull/40.

Propagation of type-level annotations is a breaking change, so will be introduced in 0.12.0.

## [0.11.6] - 2025-10-12

Adds support for specifying namespaces at the call-site. This allows you to specify that the type (struct or enum) that a field points to is in another (possibly external) namespace. See https://github.com/redbadger/facet-generate/pull/39

## [0.11.5] - 2025-10-06

### 🐛 Bug Fixes

* fixes a bug where a target's dependencies sometimes were not in UpperCamelCase.
* fixes a bug where there were extraneous imports in a module's preamble

see https://github.com/redbadger/facet-generate/pull/38

## [0.11.4] - 2025-10-03

### 🐛 Bug Fixes

[Fixes a bug](https://github.com/redbadger/facet-generate/pull/35) when generating function names in Swift for serialization.

## [0.11.3] - 2025-10-02

### 🐛 Bug Fixes

[Fixes a bug](https://github.com/redbadger/facet-generate/pull/34) with module splitting by namespace.

## [0.11.2] - 2025-09-08

### 🐛 Bug Fixes

There was a problem reflecting enums inside struct variants, which is now fixed.

Also changed the handling of generic types, which are supported if:
- `Arc`, `Rc`, `Box`, which are reflected as the inner type
- `Option`, which is reflected as `OPTION`
- `Vec`, `HashSet`, `BTreeSet`, reflected as `SEQ`
- `HashMap`, `BTreeMap`, reflected as `MAP`
- `DateTime`, reflected as `STR`, for serialization as RFC3339 (using the serde feature of DateTime)
- other generic types are reflected as `TYPENAME`, with the type parameters currently removed. This means that if the type is used more than once with different parameters, the reflection will panic.

## [0.11.1] - 2025-09-03

### 🐛 Bug Fixes

- fixes a bug when generating typescript, and the serde runtime is not an external package, the import should be relative to the current directory

## [0.11.0] - 2025-08-29

### 🚀 Features

- adds support for generating code in Kotlin. The generated code is idiomatic and clean (and passes `ktlint` checks). Using Kotlin instead of Java can be more ergonomic with, for example, exhaustive when statements for enums.

## [0.10.4] - 2025-09-03

### 🐛 Bug Fixes

- fixes a bug when generating typescript, and the serde runtime is not an external package, the import should be relative to the current directory

## [0.10.3] - 2025-08-15

### 🐛 Bug Fixes

- fixes a bug when reflecting over enums that have tuple variants with more than one value _and_ are user structs nested within option or sequence types.

## [0.10.2] - 2025-08-06

### 🚀 Features

- support for sets (e.g. HashSet, BTreeSet, as sequences, following serde, for now) https://github.com/redbadger/facet-generate/pull/24

## [0.10.1] - 2025-07-31

### 🚀 Features

- removes some unneeded dependencies, adds some doc comments, and tidies up a bit

## [0.10.0] - 2025-07-30

### 🚀 Features

- fixes typescript and java generation to generate code that handles external dependencies better

## [0.9.0] - 2025-07-28

### 🚀 Features

Improves TypeScript generation to include support for namespaces (internal and external dependencies) and package.json generation.

## [0.8.0] - 2025-07-25

### 🚀 Features

Updates to `facet` v0.28.0 and allows installer methods to take parameters by reference

## [0.7.2] - 2025-07-21

### 🚀 Features

Adds a new Config object with associated builder for clients to use when configuring the generation process.

## [0.7.1] - 2025-07-19

### 🐛 Bug Fixes

- fix handling of disjoint namespaces, which could be orphaned and are now referenced as top level library targets.

## [0.7.0] - 2025-07-15

### 🚀 Features

- Support (Swift-only for now) for emitting namespaces as separate SPM packages with relevant dependencies, including Serde

### 🐛 Bug Fixes

- Always import Serde for now (in order to support `Indirect` attribute)

### ⚙️ Miscellaneous Tasks

- Test to show Serde as separate package

## [0.6.0] - 2025-07-07

### 🚀 Features

- Add namespace support for reflection
- Split registry on namespace
- Swift module generation
- Qualified typenames are now strongly typed
- Support self-referencing types
- UpperCamelCase for swift namespaces and types
- Tree_with_mutual_recursion
- Enum struct variants with single field
- Java package from namespaces
- Remove namespace inheritance
- QualifiedTypename to Typename
- Registry key is a QualifiedTypeName
- Generation::Module and cleanup
- Registry builder
- Ability to reflect over multiple types

### 🐛 Bug Fixes

- *(facet_generate)* Use "name = " instead of "rename = "
- Swift import UpperCamelCase
- Fix capitalisation of TypeScript and Swift namespaces
- Bug in qualifying typenames in Swift generation

### ⚙️ Miscellaneous Tasks

- *(facet_generate)* Add CI
- *(facet_generate)* Tidy
- *(facet_generate)* 0.2.0 and readme
- *(facet_generate)* Deps and v0.2.1
- Vendor serde crates
- 0.3.0, + mit license
- Add swift and deno to build.yaml
- Update readme
- Refactor
- Extra namespace tests
- Update to facet 0.27.15
- Update heck to latest version
- Refactor
- Reflection and generation modules
- 0.4.2
- 0.4.0
- Facet 0.27.16
- Move integration tests and remove mutexes

<!-- generated by git-cliff -->
