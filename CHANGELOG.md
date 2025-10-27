# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

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
