# time.rs — DateTime Wrapper and Timer Utility

## Purpose

This file defines a `Timer` utility for measuring elapsed time in debug logs, and a `DateTime` wrapper around `OffsetDateTime` that integrates with serde (RFC 3339), Diesel (`Timestamptz`), and utoipa (OpenAPI schema).

## Struct: `Timer<'a>`

```rust
pub struct Timer<'a> {
    name: &'a str,
    start: std::time::Instant,
}
```

A utility for timing code blocks. Logs the elapsed time when dropped.

| Field | Type | Purpose |
|-------|------|---------|
| `name` | `&'a str` | Label for the timed operation |
| `start` | `Instant` | Timestamp when the timer was created |

### Method: `Timer::new(name: &'a str) -> Self`

Creates a timer with the given name, recording the current instant.

### Trait: `Drop for Timer<'_>`

When the timer is dropped, logs the elapsed time using `tracing::info!`. The time unit is chosen automatically based on the magnitude:

| Elapsed Time | Format |
|-------------|--------|
| < 1 μs | `{:.1}ns` |
| 1 μs to 1 ms | `{:.3}μs` |
| 1 ms to 1 s | `{:.3}ms` |
| 1 s to 1000 s | `{:.3}s` |
| >= 1000 s | `{:.0}s` |

The unit selection uses `elapsed.as_nanos().ilog10()` to determine the order of magnitude.

## Struct: `DateTime`

```rust
pub struct DateTime(#[serde(with = "rfc3339")] OffsetDateTime);
```

A wrapper around `time::OffsetDateTime` that serializes and deserializes in RFC 3339 format. Integrates with Diesel as `Timestamptz` and with utoipa for OpenAPI schema generation.

**Derived traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`, `AsExpression`, `FromSqlRow`, `ToSchema`.

The `unsafe_derive_deserialize` clippy warning is suppressed because the `rfc3339` serde module ensures correct deserialization.

### Method: `DateTime::now() -> Self`

Returns the current UTC time.

### Method: `DateTime::today() -> Self`

Returns the start (midnight) of the current UTC day.

### Method: `DateTime::yesterday() -> Self`

Returns the start (midnight) of the previous UTC day. Falls back to `Date::MIN` if there is no previous day.

### Method: `DateTime::tomorrow() -> Self`

Returns the start (midnight) of the next UTC day. Falls back to `Date::MAX` if there is no next day.

### Method: `DateTime::from_date(year, month, day) -> Result<Self, ComponentRange>`

Constructs a `DateTime` from a calendar date at midnight UTC.

**Parameters:**
- `year: i32`
- `month: Month`
- `day: u8`

**Returns:** Midnight UTC on the given date, or `ComponentRange` error if the date is invalid.

### Method: `DateTime::test_date() -> Self` (test only)

Returns a fixed test date: `2008-09-15 00:00 UTC`.

### Trait: `Deref for DateTime`

Derefs to `&OffsetDateTime`, allowing access to all `OffsetDateTime` methods.

### Trait: `DerefMut for DateTime`

Derefs mutably to `&mut OffsetDateTime`.

### Trait: `From<OffsetDateTime> for DateTime`

Converts from an `OffsetDateTime`.

### Trait: `ToSql<Timestamptz, Pg> for DateTime`

Serializes to PostgreSQL `Timestamptz` by delegating to `<OffsetDateTime as ToSql>`.

### Trait: `FromSql<Timestamptz, Pg> for DateTime`

Deserializes from PostgreSQL `Timestamptz` by delegating to `OffsetDateTime::from_sql` and wrapping the result.
