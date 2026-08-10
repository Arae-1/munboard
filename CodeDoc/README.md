# Oxibooru Code Documentation

## Project Summary

Oxibooru is an image board engine. It has a Rust backend and a JavaScript frontend. It uses PostgreSQL as the database. The backend is a rewrite of Szurubooru. The system gives high performance for large media archives.

## Architecture Overview

### System Components

```
                      Docker Compose (3 containers)
 ┌──────────────────────────────────────────────────────────────┐
 │                                                              │
 │   Browser ──> NGINX (client) ──proxy──> Axum (server)       │
 │                  :80                  :6666                  │
 │                    │                      │                  │
 │                    │     /data/           │                  │
 │                    │  (static files)      │                  │
 │                    └──────────────────────┘                  │
 │                                           │                  │
 │                                    PostgreSQL (sql)          │
 │                                    :5432                     │
 │                                           │                  │
 │                                    /var/lib/postgresql/data  │
 └──────────────────────────────────────────────────────────────┘
```

### Deployment Model

Three Docker containers run the application:

- **client** — NGINX on Alpine. It serves static HTML/JS/CSS. It proxies `/api` requests to the server. It serves `/data` files from a shared volume.
- **server** — Rust binary on Alpine. It serves the JSON REST API on port 6666. It uses Axum with tower middleware layers.
- **sql** — PostgreSQL 16 on Alpine. It stores all application data.

The client and server share a `/data` volume for media files. The server writes files to `/data`. NGINX reads files from `/data`.

## Directory Map

### Server Core
- [Core Module Overview](server/core/README.md)
- [main.rs — Binary Entry Point](server/core/main.md)
- [build.rs — Rebuild Triggers](server/core/build.md)
- [app.rs — Application State & Startup](server/core/app.md)
- [config.rs — Configuration](server/core/config.md)
- [db.rs — Async Connection Pool](server/core/db.md)
- [error.rs — Error Classifications](server/core/error.md)
- [extract.rs — Axum Extractors](server/core/extract.md)
- [filesystem.rs — File Storage](server/core/filesystem.md)
- [schema.rs — Diesel Schema](server/core/schema.md)
- [string.rs — String Types](server/core/string.md)
- [time.rs — DateTime Types](server/core/time.md)
- [test.rs — Test Infrastructure](server/core/test.md)

### Server API Layer
- [API Overview](server/api/README.md)
  - [Router & Shared Types](server/api/mod.md)
  - [Post Endpoints (13 routes)](server/api/post.md)
  - [Tag Endpoints (7 routes)](server/api/tag.md)
  - [User Endpoints (5 routes)](server/api/user.md)
  - [Comment Endpoints (6 routes)](server/api/comment.md)
  - [Pool Endpoints (6 routes)](server/api/pool.md)
  - [Custom Field Endpoints](server/api/custom_field.md)
  - [Pool Category Endpoints](server/api/pool_category.md)
  - [Tag Category Endpoints](server/api/tag_category.md)
  - [Snapshot Endpoints](server/api/snapshot.md)
  - [Password Reset Endpoints](server/api/password_reset.md)
  - [Upload Endpoints](server/api/upload.md)
  - [User Token Endpoints](server/api/user_token.md)
  - [Info Endpoint](server/api/info.md)
  - [Legacy Redirects](server/api/legacy.md)
  - [Middleware](server/api/middleware.md)
  - [API Error Responses](server/api/error.md)
  - [OpenAPI/Swagger Docs](server/api/doc.md)

### Server Data Models
- [Model Overview](server/models/README.md)
  - [User & UserToken](server/models/user.md)
  - [Post, PostSignature, PostRelation, PostNote](server/models/post.md)
  - [Comment](server/models/comment.md)
  - [Tag, TagName, TagImplication, TagSuggestion](server/models/tag.md)
  - [TagCategory](server/models/tag_category.md)
  - [Pool, PoolName](server/models/pool.md)
  - [PoolCategory](server/models/pool_category.md)
  - [CustomField](server/models/custom_field.md)
  - [PostCustomField](server/models/post_custom_field.md)
  - [Snapshot](server/models/snapshot.md)

### Server Search System
- [Search Framework Overview](server/search/README.md)
  - [Core: Builder, Criteria, Conditions](server/search/mod.md)
  - [Diesel Macros](server/search/macros.md)
  - [Query String Parsing](server/search/parse.md)
  - [Post Search (37 tokens)](server/search/post.md)
  - [Tag Search (10 tokens)](server/search/tag.md)
  - [Pool Search (5 tokens)](server/search/pool.md)
  - [Comment Search (7 tokens)](server/search/comment.md)
  - [Snapshot Search (5 tokens)](server/search/snapshot.md)
  - [User Search (3 tokens)](server/search/user.md)
  - [Preferences & Post Hiding](server/search/preferences.md)
  - [TEMP Tables](server/search/temp.md)

### Server Snapshot System
- [Snapshot Overview](server/snapshot/overview.md)
  - [mod.md](server/snapshot/mod.md) — Snapshot trait, JSON diff engine, creation logic
  - [comment.md](server/snapshot/comment.md) — Comment snapshot tracking
  - [custom_field.md](server/snapshot/custom_field.md) — Custom field snapshot tracking
  - [pool.md](server/snapshot/pool.md) — Pool snapshot tracking and merge
  - [pool_category.md](server/snapshot/pool_category.md) — Pool category snapshot tracking
  - [post.md](server/snapshot/post.md) — Post snapshot tracking (12 fields)
  - [tag.md](server/snapshot/tag.md) — Tag snapshot tracking (implications/suggestions)
  - [tag_category.md](server/snapshot/tag_category.md) — Tag category snapshot tracking
  - [user.md](server/snapshot/user.md) — User snapshot tracking
  - [user_token.md](server/snapshot/user_token.md) — User token snapshot tracking

### Server Update Pipeline
- [Update Pipeline Overview](server/update/overview.md)
  - [mod.md](server/update/mod.md) — Update module overview
  - [pool.md](server/update/pool.md) — Pool update side-effects
  - [post.md](server/update/post.md) — Post update side-effects (merge, relations, thumbnail)
  - [tag.md](server/update/tag.md) — Tag implication chain resolution, post re-indexing
  - [user.md](server/update/user.md) — User update side-effects

### Server Content Processing
- [Content Processing Overview](server/content/overview.md)
  - [mod.md](server/content/mod.md) — Content module overview
  - [cache.md](server/content/cache.md) — Ring buffer upload cache
  - [decode.md](server/content/decode.md) — Image/video decoding
  - [download.md](server/content/download.md) — HTTP content download
  - [embed.md](server/content/embed.md) — OpenGraph embed pages for post links
  - [flash.md](server/content/flash.md) — Flash/SWF processing
  - [hash.md](server/content/hash.md) — Content hashing (blake3, MD5, SHA1, SHA256)
  - [signature.md](server/content/signature.md) — Image signature for reverse search
  - [thumbnail.md](server/content/thumbnail.md) — Thumbnail generation
  - [upload.md](server/content/upload.md) — Upload processing pipeline

### Server Authentication
- [Authentication Overview](server/auth/overview.md)
  - [header.md](server/auth/header.md) — Auth header parsing (Basic + Token)
  - [password.md](server/auth/password.md) — Argon2 password hashing
  - [token_crypto.md](server/auth/token_crypto.md) — Token at-rest encryption (AES-256-GCM)

### Server Admin CLI
- [Admin CLI Overview](server/admin/overview.md)
  - [mod.md](server/admin/mod.md) — CLI shell, command parsing
  - [database.md](server/admin/database.md) — Database admin commands
  - [input.md](server/admin/input.md) — Admin input parsing utilities
  - [post.md](server/admin/post.md) — Post admin operations
  - [user.md](server/admin/user.md) — User admin operations

### Server Math Utilities
- [Math Utilities Overview](server/math/overview.md)
  - [cartesian.md](server/math/cartesian.md) — Cartesian coordinate calculations
  - [interval.md](server/math/interval.md) — Interval math
  - [point.md](server/math/point.md) — Point structure
  - [rect.md](server/math/rect.md) — Rectangle operations

### Server Resource Layer
- [Resource Layer Overview](server/resource/overview.md)
  - [mod.md](server/resource/mod.md) — Resource trait framework
  - [field.md](server/resource/field.md) — Field serialization helpers
  - [comment.md](server/resource/comment.md) — Comment resource serialization
  - [custom_field.md](server/resource/custom_field.md) — Custom field resource serialization
  - [pool.md](server/resource/pool.md) — Pool resource serialization
  - [pool_category.md](server/resource/pool_category.md) — Pool category resource serialization
  - [post.md](server/resource/post.md) — Post resource serialization (30 fields, 20 helpers)
  - [snapshot.md](server/resource/snapshot.md) — Snapshot resource serialization
  - [tag.md](server/resource/tag.md) — Tag resource serialization
  - [tag_category.md](server/resource/tag_category.md) — Tag category resource serialization
  - [user.md](server/resource/user.md) — User resource serialization
  - [user_token.md](server/resource/user_token.md) — User token resource serialization

### Server Database Migrations
- [Migrations Overview](server/migrations/overview.md)

### Server Procedural Macro
- [non_nullable_options Macro](server/macros.md)

### Client Entry Points
- [Entry Points Overview](client/entry/README.md)
  - [main.md — Startup Sequence](client/entry/main.md)
  - [router.md — Client-Side Router](client/entry/router.md)
  - [api.md — API Client](client/entry/api.md)
  - [config.md — Build Configuration](client/entry/config.md)
  - [templates.md — Template Export](client/entry/templates.md)
  - [tags.md — Tag Category CSS](client/entry/tags.md)
  - [pools.md — Pool Category CSS](client/entry/pools.md)
  - [events.md — Event System](client/entry/events.md)

### Client Controllers
- [Controllers Overview](client/controllers/README.md)
  - [auth_controller.md — Login & Logout](client/controllers/auth_controller.md)
  - [base_post_controller.md — Base Post Controller](client/controllers/base_post_controller.md)
  - [comments_controller.md — Comment Listing](client/controllers/comments_controller.md)
  - [custom_field_controller.md — Custom Field Edit](client/controllers/custom_field_controller.md)
  - [custom_field_create_controller.md — Custom Field Create](client/controllers/custom_field_create_controller.md)
  - [custom_field_list_controller.md — Custom Field List](client/controllers/custom_field_list_controller.md)
  - [help_controller.md — Help Pages](client/controllers/help_controller.md)
  - [home_controller.md — Home Page](client/controllers/home_controller.md)
  - [not_found_controller.md — 404 Page](client/controllers/not_found_controller.md)
  - [page_controller.md — Pagination Utility](client/controllers/page_controller.md)
  - [password_reset_controller.md — Password Reset](client/controllers/password_reset_controller.md)
  - [pool_categories_controller.md — Pool Categories](client/controllers/pool_categories_controller.md)
  - [pool_controller.md — Pool Edit](client/controllers/pool_controller.md)
  - [pool_create_controller.md — Pool Create](client/controllers/pool_create_controller.md)
  - [pool_list_controller.md — Pool List](client/controllers/pool_list_controller.md)
  - [post_detail_controller.md — Post Merge](client/controllers/post_detail_controller.md)
  - [post_list_controller.md — Post List](client/controllers/post_list_controller.md)
  - [post_main_controller.md — Post View](client/controllers/post_main_controller.md)
  - [post_upload_controller.md — Post Upload](client/controllers/post_upload_controller.md)
  - [settings_controller.md — User Settings](client/controllers/settings_controller.md)
  - [snapshots_controller.md — History](client/controllers/snapshots_controller.md)
  - [tag_categories_controller.md — Tag Categories](client/controllers/tag_categories_controller.md)
  - [tag_controller.md — Tag Edit](client/controllers/tag_controller.md)
  - [tag_create_controller.md — Tag Create](client/controllers/tag_create_controller.md)
  - [tag_list_controller.md — Tag List](client/controllers/tag_list_controller.md)
  - [top_navigation_controller.md — Navigation Bar](client/controllers/top_navigation_controller.md)
  - [user_controller.md — User Edit](client/controllers/user_controller.md)
  - [user_list_controller.md — User List](client/controllers/user_list_controller.md)
  - [user_registration_controller.md — Register](client/controllers/user_registration_controller.md)

### Client Controls
- [Controls Overview](client/controls/README.md)
  - [auto_complete_control.md — Autocomplete Base](client/controls/auto_complete_control.md)
  - [tag_auto_complete_control.md — Tag Autocomplete](client/controls/tag_auto_complete_control.md)
  - [pool_auto_complete_control.md — Pool Autocomplete](client/controls/pool_auto_complete_control.md)
  - [custom_field_auto_complete_control.md — Custom Field Autocomplete](client/controls/custom_field_auto_complete_control.md)
  - [tag_input_control.md — Tag Input Editor](client/controls/tag_input_control.md)
  - [pool_input_control.md — Pool Input Editor](client/controls/pool_input_control.md)
  - [custom_field_input_control.md — Custom Field Input Editor](client/controls/custom_field_input_control.md)
  - [post_content_control.md — Post Media Display](client/controls/post_content_control.md)
  - [post_notes_overlay_control.md — Post Notes SVG](client/controls/post_notes_overlay_control.md)
  - [post_edit_sidebar_control.md — Post Edit Sidebar](client/controls/post_edit_sidebar_control.md)
  - [post_readonly_sidebar_control.md — Post View Sidebar](client/controls/post_readonly_sidebar_control.md)
  - [comment_control.md — Single Comment](client/controls/comment_control.md)
  - [comment_list_control.md — Comment List](client/controls/comment_list_control.md)
  - [file_dropper_control.md — File Upload UI](client/controls/file_dropper_control.md)
  - [expander_control.md — Collapsible Sections](client/controls/expander_control.md)

### Client Models
- [Models Overview](client/models/README.md)
  - [abstract_list.md — List Base Class](client/models/abstract_list.md)
  - [post.md — Post Model](client/models/post.md)
  - [tag.md — Tag Model](client/models/tag.md)
  - [pool.md — Pool Model](client/models/pool.md)
  - [user.md — User Model](client/models/user.md)
  - [comment.md — Comment Model](client/models/comment.md)
  - [snapshot.md — Snapshot Model](client/models/snapshot.md)
  - [tag_category.md — Tag Category Model](client/models/tag_category.md)
  - [pool_category.md — Pool Category Model](client/models/pool_category.md)
  - [point.md — 2D Point Model](client/models/point.md)
  - [note.md — Annotation Note Model](client/models/note.md)
  - [post_list.md — Post List](client/models/post_list.md)
  - [tag_list.md — Tag List](client/models/tag_list.md)
  - [pool_list.md — Pool List](client/models/pool_list.md)
  - [user_list.md — User List](client/models/user_list.md)
  - [comment_list.md — Comment List](client/models/comment_list.md)
  - [snapshot_list.md — Snapshot List](client/models/snapshot_list.md)
  - [note_list.md — Note List](client/models/note_list.md)
  - [point_list.md — Point List](client/models/point_list.md)
  - [custom_field_list.md — Custom Field List](client/models/custom_field_list.md)
  - [tag_category_list.md — Tag Category List](client/models/tag_category_list.md)
  - [pool_category_list.md — Pool Category List](client/models/pool_category_list.md)
  - [custom_field.md — Custom Field Model](client/models/custom_field.md)
  - [settings.md — User Settings](client/models/settings.md)
  - [user_token.md — Auth Token Model](client/models/user_token.md)
  - [top_navigation.md — Navigation Model](client/models/top_navigation.md)

### Client Views
- [views/common.md — Shared Views](client/views/common.md)
- [views/home.md — Home Page View](client/views/home.md)
- [views/help.md — Help Page View](client/views/help.md)
- [views/settings.md — Settings View](client/views/settings.md)
- [views/posts.md — Post Views](client/views/posts.md)
- [views/tags.md — Tag Views](client/views/tags.md)
- [views/pools.md — Pool Views](client/views/pools.md)
- [views/users.md — User Views](client/views/users.md)
- [views/comments.md — Comment Views](client/views/comments.md)
- [views/custom_fields.md — Custom Field Views](client/views/custom_fields.md)
- [views/snapshots.md — Snapshot Views](client/views/snapshots.md)

### Client Templates
- [templates/common.md — Shared Templates](client/templates/common.md)
- [templates/help.md — Help Templates](client/templates/help.md)
- [templates/settings.md — Settings Templates](client/templates/settings.md)
- [templates/posts.md — Post Templates](client/templates/posts.md)
- [templates/tags.md — Tag Templates](client/templates/tags.md)
- [templates/pools.md — Pool Templates](client/templates/pools.md)
- [templates/users.md — User Templates](client/templates/users.md)
- [templates/comments.md — Comment Templates](client/templates/comments.md)
- [templates/custom_fields.md — Custom Field Templates](client/templates/custom_fields.md)
- [templates/snapshots.md — Snapshot Templates](client/templates/snapshots.md)

### Client Stylesheets
- [styles/core.md — Core Layout](client/styles/core.md)
- [styles/common.md — Shared Elements](client/styles/common.md)
- [styles/posts.md — Post Styles](client/styles/posts.md)
- [styles/tags.md — Tag Styles](client/styles/tags.md)
- [styles/pools.md — Pool Styles](client/styles/pools.md)
- [styles/users.md — User Styles](client/styles/users.md)
- [styles/comments.md — Comment Styles](client/styles/comments.md)
- [styles/custom_fields.md — Custom Field Styles](client/styles/custom_fields.md)
- [styles/snapshots.md — Snapshot Styles](client/styles/snapshots.md)

### Client Utilities
- [Utilities Overview](client/utilities/README.md)
  - [views.md — DOM & HTML Utilities](client/utilities/views.md)
  - [misc.md — Formatting Utilities](client/utilities/misc.md)
  - [markdown.md — Markdown Renderer](client/utilities/markdown.md)
  - [uri.md — URL Construction](client/utilities/uri.md)
  - [keyboard.md — Keyboard Shortcuts](client/utilities/keyboard.md)
  - [touch.md — Swipe Gestures](client/utilities/touch.md)
  - [optimized_resize.md — Resize Handler](client/utilities/optimized_resize.md)
  - [search.md — Search Input](client/utilities/search.md)
  - [accent.md — Accent Color](client/utilities/accent.md)
  - [progress.md — Loading Bar](client/utilities/progress.md)
  - [polyfill.md — Browser Polyfills](client/utilities/polyfill.md)

### Client Build System
- [build.md — Build Process](client/build/build.md)
- [deployment.md — Deployment](client/build/deployment.md)

### Architecture
- [Architecture Document](overview/architecture.md)

## Data Flow Summary

This is the request flow for a typical API call:

1. The browser sends an HTTP request. The user enters a URL or clicks a link.
2. NGINX (client container) receives the request on port 80.
3. If the path starts with `/api/`, NGINX removes the prefix and proxies the request to the server on port 6666.
4. If the path starts with `/data/`, NGINX serves the file from the shared `/data` volume.
5. For all other paths, NGINX serves the static client files or falls back to `/index.htm`.
6. The server (Axum) receives the API request. Tower middleware layers process it in this order:
   - TraceLayer: Logs the request.
   - TimeoutLayer: Enforces a 60-second timeout.
   - Auth middleware: The authorization header is validated. A `Client` is attached to the context.
   - The route handler processes the request. It uses the Diesel ORM to query or modify PostgreSQL.
   - Post-to-webhooks middleware: Sends webhook events after a successful response.
   - Log error middleware: Logs errors.
7. The server returns a JSON response through NGINX to the browser.
8. The client JavaScript updates the DOM based on the response.

## Key Concepts

### Tags
Tags are labels applied to posts. Each tag has one or more names (aliases). The first name is the primary name. Tags belong to tag categories (for example, "artist", "character", "meta"). Tags can imply other tags. Tags can suggest other tags.

### Posts
A post is an uploaded media file. It can be an image, animation, video, Flash animation, or document. Posts have safety ratings, dimensions, checksums, descriptions, tags, notes, relations to other posts, custom fields, scores, and favorites.

### Pools
A pool is an ordered collection of posts. Pools belong to pool categories. Each pool has names, a description, and a launch date.

### Comments
Comments are text messages attached to posts. Users can create, edit, delete, and score comments.

### Users
A user is an account with a name, password, email, rank, and avatar. User ranks control privileges: Anonymous, Restricted, Regular, Power, Moderator, Administrator.

### Custom Fields
Custom fields are named numeric values assigned to posts. Each field has a name, description, and color. Users can create custom field definitions. Posts can have custom field values.

### Snapshots
Snapshots record every change to database resources. They store the user who made the change, the operation type, the resource type and ID, and JSON data. The snapshot system gives a full audit trail.

## Quick Reference

### Add a new API endpoint
1. Define the route handler in `server/src/api/<module>.rs`.
2. Register the route in the module's router function.
3. Merge the sub-router into the top-level router in `server/src/api/mod.rs` (function `routes()`).
4. Add the privilege action to `Action` enum in `server/src/config.rs` if needed.
5. Add the resource info struct in `server/src/resource/<resource>.rs` if needed.

### Add a new search filter
1. Add a new token variant to the entity's token enum in `server/src/search/<entity>.rs`.
2. Add the filter function that maps the token to a Diesel query expression.
3. Register the token in the entity's `SearchTokenIter` implementation.
4. Add the token name and description to the entity's `token_name()` match arm.

### Add a new client page
1. Create a controller in `client/js/controllers/`.
2. Register the route in the controller's exported function.
3. Create the view class in `client/js/views/`.
4. Create the Underscore template in `client/html/`.
5. Create the Stylus stylesheet if needed.
6. The build system (`client/build.js`) automatically picks up new files.

### Add a new database table
1. Create a new Diesel migration in `server/migrations/`.
2. Define the model struct in `server/src/model/<name>.rs`.
3. Add the table to `server/src/schema.rs` (auto-generated by Diesel CLI).
4. Add statistics triggers if the table has counts.

## License

[GNU General Public License v3.0](../LICENSE.md)
