# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

## [0.18.0] - unreleased

TypeScript enums are now generated as **discriminated union types** instead of abstract class hierarchies. This is a breaking change for any code that was constructing or matching TypeScript enum values, but the new output is far more idiomatic and integrates naturally with TypeScript's type narrowing.

### 💥 Breaking Changes

- **TypeScript enums are no longer emitted as abstract classes.** Each enum is now a `type` alias over a union of object literals. Code that used `new FooVariant(...)` or `instanceof` checks must be updated to use the generated constructor functions and `matchX` helpers described below. [#106](https://github.com/redbadger/facet-generate/pull/106)

### 🚀 Features

- **feat(typescript): discriminated union types for enums** — Rust enums now generate a `type` alias, a typed constructor arrow function per variant, and an exhaustive `matchX<R>(...)` helper [#106](https://github.com/redbadger/facet-generate/pull/106)
- **feat(typescript): `EnumTagging` in format AST** — `#[facet(tag = "...")]` and `#[facet(tag = "...", content = "...")]` are now reflected into the format as `EnumTagging::Internal` and `EnumTagging::Adjacent`, controlling the shape of each variant's object literal [#106](https://github.com/redbadger/facet-generate/pull/106)
- **feat(typescript): standalone serialize/deserialize for enums** — the Bincode and JSON plugins now emit `serializeX(value, serializer)` / `deserializeX(deserializer)` functions alongside each enum type rather than expecting a `.serialize()` method on the value [#106](https://github.com/redbadger/facet-generate/pull/106)

### 🐛 Bug Fixes

- **fix(typescript): quote non-identifier property keys in match function** — variant names that are not valid JavaScript identifiers (e.g. `"number-array"` from `rename_all = "kebab-case"`) are now correctly quoted as string keys in the `matchX` cases object type [#106](https://github.com/redbadger/facet-generate/pull/106)

### 🧪 Tests

- Enabled TypeScript output for 21 previously-disabled or untested expect-file test cases covering structs, unit enums, externally/internally/adjacently tagged enums, skipped variants, readonly fields, deprecation notices, and anonymous struct variants [#106](https://github.com/redbadger/facet-generate/pull/106)

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