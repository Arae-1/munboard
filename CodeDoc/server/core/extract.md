# extract.rs — Request Extractor Wrappers

## Purpose

This file defines custom Axum request extractors that convert standard Axum rejection types into `ApiError` instead, providing consistent error handling across all API routes.

## Struct: `Ctx`

```rust
pub struct Ctx(pub Context, pub Arc<AsyncConnectionPool>);
```

The primary per-request context extractor. Contains both the authentication context (`Context`) and a reference to the database connection pool.

### Trait: `Deref for Ctx`

Derefs to `Context`, so methods on `Context` (such as `has_privilege` and `verify_privilege`) can be called directly on `Ctx`.

### Trait: `FromRequestParts<S> for Ctx`

Constructs a `Ctx` from request parts.

**Process:**
1. Extracts `AppState` from request state.
2. Extracts the `Client` extension (set by authentication middleware).
3. Calls `state.make_context(client)`.

**Error:** Returns `ApiError` on failure.

## Struct: `Json<T>`

```rust
pub struct Json<T>(pub T);
```

A wrapper over Axum's `Json` extractor that maps `JsonRejection` to `ApiError`.

### Trait: `FromRequest<S> for Json<T>`

Extracts JSON body, mapping rejection to `ApiError::from`.

### Trait: `IntoResponse for Json<T>`

Serializes back to JSON response. Wraps `AxumJson(self.0).into_response()`.

Requires `T: Serialize`.

## Enum: `JsonOrMultipart<T>`

```rust
pub enum JsonOrMultipart<T> {
    Json(T),
    Multipart(AxumMultipart),
}
```

Used for request bodies that can be either JSON or multipart form data, such as upload endpoints. The content type header determines which variant is created.

| Variant | Purpose |
|---------|---------|
| `Json(T)` | The body was parsed as JSON |
| `Multipart(AxumMultipart)` | The body is multipart form data |

### Trait: `FromRequest<S> for JsonOrMultipart<T>`

**Process:**
1. Reads the `Content-Type` header.
2. If it starts with `application/json`, deserializes as JSON.
3. If it starts with `multipart/form-data`, returns the raw multipart stream.
4. If the content type is missing or unrecognized, returns `ApiError::JsonRejection(MissingJsonContentType)`.

## Struct: `Path<T>`

```rust
pub struct Path<T>(pub T);
```

A wrapper over Axum's `Path` extractor that maps `PathRejection` to `ApiError`.

### Trait: `FromRequestParts<S> for Path<T>`

Extracts path parameters, mapping rejection to `ApiError::from`.

## Struct: `Query<T>`

```rust
pub struct Query<T>(pub T);
```

A wrapper over Axum's `Query` extractor that maps `QueryRejection` to `ApiError`.

### Trait: `FromRequestParts<S> for Query<T>`

Extracts query string parameters, mapping rejection to `ApiError::from`.
