# Panic Tests for Variable Resolution

This module contains tests that demonstrate scenarios where `Variable` placeholders are not resolved during the reflection process, leading to panics in `RegistryBuilder::build()`.

## Background

The `RegistryBuilder` uses `Variable` placeholders (created by `Format::unknown()` and `VariantFormat::unknown()`) as temporary values while processing types. These variables must be resolved to actual formats before the registry is built, or the `build()` method will panic with:

```
should not have any remaining placeholders: Incomplete reflection detected
```

## Root Cause

The panic occurs when:

1. A temporary container is created with `Box::default()` (which creates `Format::unknown()`)
2. `process_type()` is called to populate the variable
3. `process_type()` encounters an unhandled case and doesn't call any `update_container_format*` method
4. The `Variable` remains unresolved (`None`)
5. `build()` calls `visit()` on all formats, which returns `Error::UnknownFormat` for unresolved variables
6. The error is converted to a panic

## Problematic Code Paths

### 1. Unhandled `Def` variants in `format_from_def_system`

**File**: `src/reflection/mod.rs:174-193`

```rust
fn format_from_def_system(&mut self, shape: &Shape, namespace: Option<&str>) {
    match shape.def {
        Def::Scalar => self.format_scalar(shape),
        // ... other handled cases ...
        _ => {} // ⚠️ PROBLEM: Silently ignores unhandled Def variants
    }
}
```

**Issue**: The catch-all `_ => {}` means:
- `Def::Set(_)` - ignored
- `Def::Pointer(PointerDef { pointee: None, .. })` - ignored  
- Any future `Def` variants - ignored

### 2. Unhandled primitive types in `handle_undefined_def`

**File**: `src/reflection/mod.rs:206-223`

```rust
fn handle_undefined_def(&mut self, shape: &Shape, namespace: Option<&str>) {
    match &shape.ty {
        Type::Primitive(primitive) => match primitive {
            PrimitiveType::Boolean
            | PrimitiveType::Numeric(NumericType::Float)
            | PrimitiveType::Textual(TextualType::Str) => {} // ⚠️ PROBLEM: No update call
            // ...
        },
        // ...
    }
}
```

**Issue**: These primitive types don't call `update_container_format*` methods.

### 3. Failed sequence type handling

**File**: `src/reflection/mod.rs:146-172`

If `SequenceType::Slice` doesn't have `Def::Slice` (or `Array`/`Array`), `handle_sequence_type` returns `false`, then `format_from_def_system` is called and may hit the catch-all.

### 4. `Def::Set` causes immediate panic

**File**: `src/reflection/mod.rs:1142-1147`

```rust
Def::Set(_set_def) => todo!(), // ⚠️ PROBLEM: Panics immediately
```

## Test Cases

The tests in this module demonstrate these scenarios:

- `test_unresolved_variable_causes_panic` - Direct demonstration of unresolved Variable
- `test_temp_container_with_unresolved_variable` - Simulates temporary container workflow
- `test_nested_unresolved_variables` - Shows unresolved Variables in struct fields
- `test_enum_with_unresolved_variant` - Shows unresolved Variables in enum variants
- `test_variable_visit_returns_unknown_format_error` - Shows the underlying error
- `test_format_from_def_system_catch_all_issue` - Demonstrates the catch-all problem
- `test_error_message_from_unresolved_variable` - Verifies the panic message

## Solutions

To fix these issues:

1. **Replace catch-all with explicit handling**:
   ```rust
   match shape.def {
       // ... existing cases ...
       Def::Set(set_def) => self.handle_set(set_def, namespace),
       Def::Pointer(PointerDef { pointee: None, .. }) => self.handle_null_pointer(shape),
       // Add other missing cases
   }
   ```

2. **Add update calls for primitive types**:
   ```rust
   PrimitiveType::Boolean => {
       let format = Format::Bool;
       self.update_container_format(format);
   }
   ```

3. **Add fallback handling for sequence type mismatches**

4. **Implement `Def::Set` handling instead of `todo!()`**

## When This Occurs in Practice

This panic typically happens when:
- Processing Rust types that map to unhandled `Def` variants
- Using newer versions of the `facet` crate with new `Def` variants
- Processing complex nested types with unusual pointer/reference patterns
- Using types that have inconsistent `Type` vs `Def` representations