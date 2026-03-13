# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

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
