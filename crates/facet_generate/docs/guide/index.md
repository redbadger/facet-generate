
Read these in order if you're new; each one stands on its own if you're not.

1. [Why `facet_generate`, and how it compares](crate::guide::motivation) — the problem it solves, and how it differs from UniFFI, typeshare, serde-generate, ts-rs and specta.
2. [Supported types](crate::guide::supported_types) — every Rust type the reflector accepts, and what it becomes in Swift, Kotlin, C# and TypeScript.
3. [Serialization: bincode and JSON](crate::guide::serialization) — what the two plugins generate, and how their bytes and JSON line up with Rust's `bincode` and `serde_json`.
4. The target languages, one page each: [Swift](crate::guide::swift), [Kotlin](crate::guide::kotlin), [C#](crate::guide::csharp), [TypeScript](crate::guide::typescript) — the installer, the package it writes, runtimes, namespaces, external packages and toolchains.
5. [Writing a plugin](crate::guide::plugins) — extending the generated code through [`EmitterPlugin`](crate::generation::plugin::EmitterPlugin).
6. [Contributing](crate::guide::contributing) — how this crate's own tests are organised.
