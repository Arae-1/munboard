# string.rs — Custom String Types

## Purpose

This file defines two custom string types optimized for different use cases: `SmallString` for short, frequently-used strings (like tag names), and `LargeString` for potentially large, immutable strings (like post descriptions).

## Struct: `SmallString`

```rust
pub struct SmallString(CompactString);
```

A wrapper over `CompactString` that implements Small String Optimization (SSO). Does not allocate on the heap if the string is 24 bytes or fewer. Ideal for tags, usernames, passwords, and other short identifiers.

**Derived traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Serialize`, `Deserialize`, `AsExpression`, `FromSqlRow`, `ToSchema`.

**Diesel attributes:** Compatible with both `Text` and `Citext` SQL types.

### Method: `SmallString::new(text: impl AsRef<str>) -> Self`

Constructs a `SmallString` from any string-like input.

### Method: `SmallString::to_lowercase() -> Self`

Returns a new `SmallString` with all characters converted to lowercase.

### Trait: `Deref for SmallString`

Derefs to `&str`, so all standard string operations work directly.

### Trait: `FromStr for SmallString`

Parses from a string slice. Infallible (`Err = Infallible`).

### Trait: `From<String> for SmallString`

Converts from a `String`.

### Trait: `From<Cow<'_, str>> for SmallString`

Converts from a `Cow<str>`.

### Trait: `From<i64> for SmallString`

Converts an `i64` to its string representation.

### Trait: `Display for SmallString`

Delegates to inner `CompactString`'s `Display`.

### Trait: `ToSql<Text, Pg> for SmallString`

Serializes to PostgreSQL `Text` by delegating to `<str as ToSql<Text, Pg>>`.

### Trait: `ToSql<Citext, Pg> for SmallString`

Serializes to PostgreSQL `Citext` by delegating to `<str as ToSql<Citext, Pg>>`.

### Trait: `FromSql<T, Pg> for SmallString`

Deserializes from any PostgreSQL text type that `String` can deserialize from. Uses `CompactString::from_utf8` for zero-copy where possible.

## Struct: `LargeString`

```rust
pub struct LargeString(Arc<str>);
```

A wrapper over `Arc<str>` for immutable strings. Cheap to clone (only increments reference count). Can be shared across threads. Meant for potentially large strings like post descriptions or source URLs.

**Derived traits:** `Debug`, `Clone`, `Default`, `Serialize`, `Deserialize`, `AsExpression`, `FromSqlRow`, `ToSchema`.

**Diesel attributes:** Compatible with `Text` SQL type.

### Method: `LargeString::new() -> Self`

Constructs an empty `LargeString`.

### Method: `LargeString::trimmed() -> &str`

Returns the string with leading and trailing whitespace removed.

### Trait: `Deref for LargeString`

Derefs to `&str`.

### Trait: `Display for LargeString`

Delegates to inner `Arc<str>`'s `Display`.

### Trait: `FromSql<Text, Pg> for LargeString`

Deserializes from PostgreSQL `Text`. Converts bytes to string and wraps in `Arc<str>`.

### Trait: `ToSql<Text, Pg> for LargeString`

Serializes to PostgreSQL `Text`. Trims whitespace before serialization.

### `PartialEq` (test only)

In test builds, equality comparison trims both sides before comparing. This ensures that whitespace differences in test assertions do not cause false failures.
