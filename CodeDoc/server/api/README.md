# API Layer — README

## Architecture

The API layer uses Axum with utoipa for OpenAPI generation and tower middleware for cross-cutting concerns. Diesel provides database access. Each resource entity is a separate sub-router module.

## Module Map

| Order | Sub-router | Doc File |
|--------|------------|-----------|
| 1 | `comment` | [comment.md](comment.md) |
| 2 | `custom_field` | [custom_field.md](custom_field.md) |
| 3 | `info` | [info.md](info.md) |
| 4 | `legacy` | [legacy.md](legacy.md) |
| 5 | `password_reset` | [password_reset.md](password_reset.md) |
| 6 | `pool` | [pool.md](pool.md) |
| 7 | `pool_category` | [pool_category.md](pool_category.md) |
| 8 | `post` | [post.md](post.md) |
| 9 | `snapshot` | [snapshot.md](snapshot.md) |
| 10 | `tag` | [tag.md](tag.md) |
| 11 | `tag_category` | [tag_category.md](tag_category.md) |
| 12 | `upload` | [upload.md](upload.md) |
| 13 | `user` | [user.md](user.md) |
| 14 | `user_token` | [user_token.md](user_token.md) |

Support modules: [mod.md](mod.md), [middleware.md](middleware.md), [error.md](error.md), [doc.md](doc.md)

## Shared Types

Defined in [mod.md](mod.md):

| Type | Purpose |
|------|---------|
| `RatingBody` | Request body for applying a score (-1, 0, 1) |
| `DeleteBody` | Request body for deleting a resource (includes `version`) |
| `MergeBody<T>` | Request body for merging two resources (remove, merge_to, versions) |
| `ResourceParams<F>` | Query parameters for search (query string + field selection) |
| `PageParams` | Pagination parameters (offset, limit) |
| `PagedResponse<T>` | Paginated search result (query, offset, limit, total, results) |
| `UnpagedResponse<T>` | Unpaged list result |

## Middleware Layers

Applied outermost-first in [mod.md](mod.md):

| Order | Layer | Purpose |
|-------|-------|---------|
| 1 | `TraceLayer` | HTTP request tracing |
| 2 | `TimeoutLayer` (408, 1 min) | Request timeout |
| 3 | `auth` | Authentication via `Authorization` header (Basic or Token) |
| 4 | `post_to_webhooks` | Sends new snapshots to configured webhook URLs |
| 5 | `log_error` | Logs errors from downstream handlers |

## Common Patterns

- **Auth:** Every handler verifies privilege via `ctx.verify_privilege(Action::...)` or `api::verify_privilege(client, rank)`.
- **Version checking:** PUT and DELETE requests include a `version` field. The server rejects the request if the version does not match the current resource timestamp (optimistic locking).
- **Field selection:** All endpoints accept a `?fields=` query parameter for partial response selection.
- **Pagination:** List endpoints accept `offset` and `limit` query parameters. All list endpoints cap `limit` at 1000.
- **Multipart uploads:** Some endpoints accept both `application/json` and `multipart/form-data`. The `JsonOrMultipart<T>` extractor handles content-type dispatch.
- **Snapshots:** Every create, update, delete, and merge operation records a snapshot for auditing and webhooks.
- **Webhooks:** After POST/PUT/DELETE requests, new snapshots are sent to all configured webhook URLs via HTTP POST.
- **Tag mutex:** Post endpoints that modify tags acquire a global `POST_TAG_MUTEX` to prevent uniqueness violations and deadlocks from concurrent tag updates.
- **Anonymous preferences:** Anonymous users have posts, tags, and tag categories filtered based on server-configured blacklists.

## Helper Functions

| Function | Purpose |
|----------|---------|
| `verify_privilege(client, rank)` | Checks if client rank meets required rank |
| `verify_version(current, client)` | Checks optimistic locking version match |
| `verify_matches_regex(config, haystack, type)` | Checks regex validation |
| `verify_valid_email(email)` | Validates email address format |
| `deserialize_some` | Custom serde deserializer for optional fields (present-with-null becomes `Some(None)`) |

## Error Response Codes

| Status | Error Variants |
|--------|---------------|
| 400 | `ContentTypeMismatch`, `HeaderDeserialization`, `InvalidMagicBytes`, `MissingContent`, `MissingContentType`, `MissingFormData`, `MissingMetadata`, JSON syntax errors |
| 401 | `NotLoggedIn`, `Password`, `UnauthorizedPasswordReset`, authentication failures |
| 403 | `Hidden`, `InsufficientPrivileges` |
| 404 | `NotFound` |
| 409 | `AlreadyExists`, `ResourceModified` |
| 415 | `UnsupportedExtension` |
| 422 | `BlockedUrl`, `CyclicDependency`, `DeleteDefault`, `FileTooLarge`, `ExpressionFailsRegex`, `InvalidSort`, `InvalidTime`, `NoNamesGiven`, `SelfMerge`, image/ffmpeg errors |
| 500 | `FailedQuery`, `StdIo`, internal errors |
| 503 | `FailedConnection` |

Error response body: `{"name": "ErrorName", "title": "Error Category", "description": "Detailed message"}`

## Endpoint Summary — Posts (13 routes)

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `GET` | `/posts` | `list` | `PostList` |
| `GET` | `/post/{id}` | `get` | `PostView` |
| `GET` | `/post/{id}/around` | `get_neighbors` | `PostList` |
| `GET` | `/featured-post` | `get_featured` | `PostViewFeatured` |
| `POST` | `/featured-post` | `feature` | `PostFeature` |
| `POST` | `/posts/reverse-search` | `reverse_search` | `PostReverseSearch` |
| `POST` | `/posts` | `create` | `PostCreateIdentified` / `PostCreateAnonymous` |
| `POST` | `/post-merge` | `merge` | `PostMerge` |
| `POST` | `/post/{id}/favorite` | `favorite` | `PostFavorite` |
| `DELETE` | `/post/{id}/favorite` | `unfavorite` | `PostFavorite` |
| `PUT` | `/post/{id}/score` | `rate` | `PostScore` |
| `PUT` | `/post/{id}` | `update` | Various field-specific |
| `DELETE` | `/post/{id}` | `delete` | `PostDelete` |

Search tokens: `id`, `tag`, `tag-category`, `score`, `uploader`, `comment`, `fav`, `pool`, `pool-category`, `tag-count`, `comment-count`, `fav-count`, `note-count`, `note-text`, `relation-count`, `feature-count`, `type`, `content-checksum`, `flag`, `source`, `file-size`, `image-width`, `image-height`, `image-area`, `image-aspect-ratio`, `creation-date`, `last-edit-date`, `comment-date`, `fav-date`, `feature-date`, `safety`/`rating`. Special: `liked`, `disliked`, `fav`, `tumbleweed`.

## Endpoint Summary — Tags (7 routes)

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `GET` | `/tags` | `list` | `TagList` |
| `GET` | `/tag/{name}` | `get` | `TagView` |
| `GET` | `/tag-siblings/{name}` | `get_siblings` | `TagView` |
| `POST` | `/tags` | `create` | `TagCreate` |
| `POST` | `/tag-merge` | `merge` | `TagMerge` |
| `PUT` | `/tag/{name}` | `update` | Various field-specific |
| `DELETE` | `/tag/{name}` | `delete` | `TagDelete` |

## Endpoint Summary — Users (5 routes)

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `GET` | `/users` | `list` | `UserList` |
| `GET` | `/user/{name}` | `get` | Self or `UserView` |
| `POST` | `/users` | `create` | `UserCreateSelf` / `UserCreateAny` |
| `PUT` | `/user/{name}` | `update` | Various field-specific (Self/Any) |
| `DELETE` | `/user/{name}` | `delete` | `UserDeleteSelf` / `UserDeleteAny` |

## Endpoint Summary — Comments (6 routes)

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `GET` | `/comments` | `list` | `CommentList` |
| `GET` | `/comment/{id}` | `get` | `CommentView` |
| `POST` | `/comments` | `create` | `CommentCreate` |
| `PUT` | `/comment/{id}` | `update` | `CommentEditOwn` / `CommentEditAny` |
| `PUT` | `/comment/{id}/score` | `rate` | `CommentScore` |
| `DELETE` | `/comment/{id}` | `delete` | `CommentDeleteOwn` / `CommentDeleteAny` |

## Endpoint Summary — Pools (6 routes)

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `GET` | `/pools` | `list` | `PoolList` |
| `GET` | `/pool/{id}` | `get` | `PoolView` |
| `POST` | `/pool` | `create` | `PoolCreate` |
| `POST` | `/pool-merge` | `merge` | `PoolMerge` |
| `PUT` | `/pool/{id}` | `update` | Various field-specific |
| `DELETE` | `/pool/{id}` | `delete` | `PoolDelete` |

## Other Endpoints

| Module | Routes | Doc |
|--------|--------|-----|
| Custom Field | GET `/custom-fields`, POST `/custom-fields`, GET/PUT/DELETE `/custom-field/{id}` | [custom_field.md](custom_field.md) |
| Pool Category | GET/POST `/pool-categories`, GET/PUT/DELETE `/pool-category/{name}`, PUT `/{name}/default` | [pool_category.md](pool_category.md) |
| Tag Category | GET/POST `/tag-categories`, GET/PUT/DELETE `/tag-category/{name}`, PUT `/{name}/default` | [tag_category.md](tag_category.md) |
| Snapshot | GET `/snapshots` | [snapshot.md](snapshot.md) |
| Password Reset | GET `/password-reset/{id}` (request), POST `/password-reset/{id}` (confirm) | [password_reset.md](password_reset.md) |
| Upload | POST `/uploads` | [upload.md](upload.md) |
| User Token | GET/POST `/user-tokens/{username}`, PUT/DELETE `/user-token/{username}/{token}` | [user_token.md](user_token.md) |
| Info | GET `/info` | [info.md](info.md) |
| Legacy | GET `/legacy/redirect/{filename}` | [legacy.md](legacy.md) |

## Documentation Files

| File | Content |
|------|---------|
| [mod.md](mod.md) | Router construction, shared types, helper functions |
| [middleware.md](middleware.md) | Auth, error logging, and webhook middleware |
| [error.md](error.md) | `ApiError` enum, HTTP status mapping, DB error mappers |
| [doc.md](doc.md) | OpenAPI tags, `ApiDoc` struct, documentation text constants |
| [post.md](post.md) | Post endpoints (13 routes) |
| [tag.md](tag.md) | Tag endpoints (7 routes) |
| [user.md](user.md) | User endpoints (5 routes) |
| [comment.md](comment.md) | Comment endpoints (6 routes) |
| [pool.md](pool.md) | Pool endpoints (6 routes) |
| [custom_field.md](custom_field.md) | Custom field endpoints (5 routes) |
| [pool_category.md](pool_category.md) | Pool category endpoints (6 routes) |
| [tag_category.md](tag_category.md) | Tag category endpoints (6 routes) |
| [snapshot.md](snapshot.md) | Snapshot list endpoint |
| [password_reset.md](password_reset.md) | Password reset flow (2 routes) |
| [upload.md](upload.md) | Temporary file upload endpoint |
| [user_token.md](user_token.md) | User token endpoints (4 routes) |
| [info.md](info.md) | Server information endpoint |
| [legacy.md](legacy.md) | Legacy URL redirect endpoint |
