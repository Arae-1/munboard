# Procedural Macro: `non_nullable_options`

**File**: `server/macros/src/lib.rs`

## Purpose

The `non_nullable_options` attribute macro automatically adds `#[schema(nullable = false)]` to every named field in a struct, unless the field already has an explicit `#[schema(nullable...)]` attribute.

This is used on resource structs (e.g., `PostInfo`, `TagInfo`, `PoolInfo`, etc.) to generate correct OpenAPI/utoipa schema documentation. By default, all `Option<T>` fields would be documented as nullable. This macro ensures they are marked as non-nullable unless explicitly overridden.

## How It Works

1. Parses the input as a `DeriveInput` (a struct).
2. Extracts the named fields from the struct data.
3. For each field:
   - Checks if the field already has a `#[schema(...)]` attribute containing the word "nullable".
   - If not, appends `#[schema(nullable = false)]` to the field's attributes.
4. Outputs the modified struct as a token stream.

## Usage Example

```rust
#[non_nullable_options]
#[skip_serializing_none]
#[derive(Serialize, ToSchema)]
pub struct PostInfo {
    pub id: Option<i64>,           // Becomes: #[schema(nullable = false)]
    #[schema(nullable)]
    pub user: Option<Option<MicroUser>>,  // Left unchanged (already has nullable)
    pub file_size: Option<i64>,    // Becomes: #[schema(nullable = false)]
}
```

## Dependencies
- `proc_macro`: Rust's built-in procedural macro system.
- `quote`: For generating token streams.
- `syn`: For parsing Rust source code into an AST.
