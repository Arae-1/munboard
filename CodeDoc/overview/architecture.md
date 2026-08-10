# Oxibooru Architecture

## System Components

Oxibooru has three primary components:

| Component | Language | Container | Purpose |
|-----------|----------|-----------|---------|
| Server | Rust | `server` | JSON REST API, content processing |
| Client | JavaScript | `client` | NGINX static file serving, API proxy |
| Database | N/A | `sql` | PostgreSQL 16 data storage |

The server and client share a `/data` volume for media files. The server writes files. NGINX reads and serves files directly.

## Docker Compose Setup

The application runs as three containers. The `docker-compose.yml` file defines these services:

### Server Container
- Builds from `server/Dockerfile`.
- Uses a multi-stage build: FFmpeg builder (custom static build with dav1d), cargo-chef planner, builder, and runtime.
- The runtime image is Alpine with `poppler-utils` installed for PDF thumbnail rendering.
- Includes a statically compiled FFmpeg binary for video processing.
- The Rust binary (`oxibooru_server`) is fully static (musl target).
- Runs on internal port 6666.
- Mounts `$MOUNT_DATA:/data` for media storage.
- Mounts `./server/config.toml:/opt/app/config.toml:ro` for configuration.
- Depends on the `sql` container.
- Stops with SIGINT for graceful shutdown.

### Client Container
- Builds from `client/Dockerfile`.
- Uses a multi-stage build: Node.js builder (npm install, browserify/stylus/terser build), and NGINX Alpine runtime.
- The build step runs `node build.js` which bundles JavaScript, compiles Stylus to CSS, minifies HTML templates, and gzips all assets.
- Serves static files from `/var/www`.
- Proxies `/api/*` requests to the server container.
- Serves `/data/*` files from the shared volume.
- Runs on port 80 (exposed as `$PORT` on the host).
- Depends on the `server` container.

### SQL Container
- Uses the `postgres:16-alpine` image directly.
- Stores database files at `$MOUNT_SQL:/var/lib/postgresql/data`.
- Internal port 5432, optionally mapped to host `127.0.0.1:$POSTGRES_PORT`.
- No build step needed.

### Environment Variables

The `.env` file configures:
- `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `POSTGRES_PORT` — Database credentials.
- `PORT` — Public HTTP port for the client container.
- `BASE_URL` — URL base path for the application.
- `MOUNT_DATA` — Host path for media file storage.
- `MOUNT_SQL` — Host path for PostgreSQL data.

## Request Lifecycle

### API Request Flow

```
Browser
  │
  │  HTTP GET /api/posts?query=sky
  ▼
NGINX (client container, port 80)
  │
  │  Location: ~ ^/api/(.*)$
  │  Strip /api/ prefix
  │  proxy_pass http://server:6666/posts?query=sky
  │  Headers: X-Forwarded-For, X-Real-IP
  │  CORS: same-origin only (no wildcard)
  │  Timeout: 60s
  ▼
Axum Server (server container, port 6666)
  │
  │  1. TraceLayer: log request
  │  2. TimeoutLayer: 60s timeout enforced
  │  3. Auth middleware: parse Authorization header
  │     - Basic (username:password) or Token (username:token-uuid)
  │     - Attach Client { id, rank } to context
  │  4. Route handler: post::list()
  │     - Parse search query string
  │     - Build Diesel SQL query with filters
  │     - Execute query against PostgreSQL
  │     - Map results through Resource layer (PostInfo)
  │  5. Post-to-webhooks middleware: send webhook event
  │  6. Log error middleware: log errors
  │
  │  HTTP 200 JSON response
  ▼
NGINX → Browser
```

### Static File Request Flow

```
Browser
  │
  │  HTTP GET /data/posts/000000/00/12345_abc123.jpg
  ▼
NGINX (client container, port 80)
  │
  │  Location: /data/
  │  root /data
  │  sendfile on, tcp_nopush on
  │  Serves file directly from shared volume
  │
  │  HTTP 200 with media content
  ▼
Browser
```

### Client Page Request Flow

```
Browser
  │
  │  HTTP GET /posts
  ▼
NGINX (client container, port 80)
  │
  │  Location: /
  │  root /var/www
  │  try_files $uri /index.htm
  │  Serves pre-gzipped index.htm (fallback for SPA routing)
  │
  │  HTTP 200 with HTML
  ▼
Browser loads index.htm
  │
  │  <script> loads app.js (browserify bundle)
  │  Router matches /posts → PostListController
  │  PostListController calls api.get("/api/posts?...")
  │  API call goes through NGINX → Server → PostgreSQL
  │  JSON response updates DOM
  ▼
Rendered page
```

## Authentication Flow

### Login

1. The user submits a username and password on the login page.
2. The client calls `api.login(name, password)`.
3. The API client sends an HTTP request with an `Authorization: Basic <base64(name:password)>` header to `/api/user/<name>`.
4. The server's auth middleware decodes the header. It queries the user by name.
5. The server verifies the password against the stored Argon2id hash using the `password_secret` from config.
6. If successful, the server returns user data. The client stores the credentials in a cookie.
7. The user can also generate authentication tokens. Each token has a UUID. The client can authenticate with `Authorization: Token <base64(name:token-uuid)>`.

### Token Management

- Tokens are stored in the `user_token` table.
- Each token has: user_id, UUID, note, enabled flag, optional expiration time, and last usage time.
- Token-based auth queries the `user_token` table joined with `user`.
- The server checks the token is enabled and not expired.

### Anonymous Access

- Requests without an Authorization header get a `Client { id: None, rank: Anonymous }`.
- Anonymous users have restricted visibility based on `anonymous_preferences` config.
- They can view posts but may have blacklisted tag categories hidden.

## File Storage Structure

The `/data` directory has this layout:

```
/data/
  posts/              — Post content files
    000000/           — Outer bucket (post_id / 1,000,000)
      00/             — Inner bucket ((post_id / 10,000) % 100)
        12345_<hash>/ — Post file: {post_id}_{blake3_hash}
  generated-thumbnails/ — Auto-generated thumbnails
    000000/00/12345_<hash>.jpg
  custom-thumbnails/  — Custom user-uploaded thumbnails
    000000/00/12345_<hash>.jpg
  avatars/            — Custom user avatars
    000000/00/12345_<hash>.jpg
  temp/               — Temporary upload files
    <uuid>.<ext>      — Files awaiting processing
```

### Path Construction

The `PostHash` struct constructs paths:
- `content`: `{data_dir}/posts/{outer:06}/{inner:02}/{post_id}_{blake3_hash}`
- `thumbnail`: `{data_dir}/generated-thumbnails/{outer:06}/{inner:02}/{post_id}_{blake3_hash}.jpg`
- `custom_thumbnail`: `{data_dir}/custom-thumbnails/{outer:06}/{inner:02}/{post_id}_{blake3_hash}.jpg`

The outer bucket is `post_id / 1_000_000`. The inner bucket is `(post_id / 10_000) % 100`.

### Cleanup

The filesystem module runs a periodic cleanup task. It removes stale temporary upload files. It uses a two-pass system:
1. First pass: Record all files in the temp directory.
2. Second pass (next tick): Delete files that were present in the first pass. This prevents deleting files that are being uploaded.

## Build Process

### Server Build

The server Dockerfile uses a four-stage build:

1. **FFmpeg Builder** — Compiles a minimal static FFmpeg with dav1d support. This FFmpeg handles AVIF decoding and video frame extraction for thumbnails.

2. **Planner** — Uses `cargo-chef` to analyze crate dependencies and create a recipe file. This enables caching of built dependencies.

3. **Builder** — Uses `cargo-chef cook` to build dependencies from the recipe (cached). Then builds the full application with `cargo build --release --no-default-features`. The `TARGET_CPU` argument controls CPU-specific optimizations. The final binary is fully static (musl).

4. **Runtime** — Minimal Alpine image with `poppler-utils` (for PDF thumbnails) and the statically compiled FFmpeg and Rust binaries.

### Client Build

The client build script (`client/build.js`) processes:

1. **JavaScript**: Browserify bundles all `client/js/*` modules. Babel transforms ES6 to ES5. Terser minifies the output. The entry point is `client/js/main.js`.

2. **CSS**: Stylus compiles `.styl` files to CSS. CSSO minifies the output.

3. **HTML**: Underscore templates from `client/html/` are precompiled into JavaScript. HTML Minifier minifies template literals.

4. **Output**: All built files go to `client/public/`. Files larger than 100 bytes are gzipped with `--gzip`.

5. **Watch mode**: `node build.js --watch` uses Chokidar for hot reload. WebSocket notifies the browser to reload on changes.

The client Dockerfile runs `npm install` then `node build.js --gzip`. The output is copied to the NGINX runtime image.

### Development Workflow

#### Server Development

1. Install Rust toolchain and diesel_cli.
2. Set up a local PostgreSQL database.
3. Copy `example.env` and configure database credentials.
4. Create `server/config.toml` from the example configuration.
5. Run `diesel migration run` to apply database migrations.
6. Start the server with `cargo run`.
7. The server serves the API on the configured port (default 6666).

#### Client Development

1. Install Node.js and npm.
2. Run `npm install` in the `client/` directory.
3. Start the watch mode with `npm run watch`.
4. The build script watches `client/js/`, `client/html/`, and `client/css/` for changes.
5. A WebSocket server (port 6667) notifies the browser to reload.
6. Open the application in a browser. The client uses the built files.

#### Full Stack Development

1. Start the PostgreSQL database.
2. Start the Rust server with `cargo run`.
3. Start the client build in watch mode with `npm run watch`.
4. Point the browser to the client's development URL.
5. API requests proxy through NGINX or directly to the server.

#### Docker Development

1. Copy `example.env` to `.env` and configure it.
2. Copy `server/config.example.toml` to `server/config.toml` and configure it.
3. Run `docker compose up --build`.
4. The application starts on the configured `$PORT`.

### Server Configuration

The server reads `config.toml` at startup. Key configuration sections:

- `database_url` — PostgreSQL connection string (constructed from environment variables).
- `thumbnails` — Post and avatar thumbnail dimensions.
- `smtp` — Email settings for password reset.
- `privileges` — Maps each `Action` to a minimum `UserRank`.
- `anonymous_preferences` — Visibility rules for anonymous users.
- `password_secret` — Secret key for Argon2id password hashing.
- `tag_regex`, `pool_regex`, `user_name_regex` — Validation patterns.

## Database Schema

The database has 31 tables across 29 migrations. Key tables:

| Table | Purpose |
|-------|---------|
| `user` | User accounts |
| `user_token` | API authentication tokens |
| `post` | Uploaded media posts |
| `post_signature` | Perceptual image signatures |
| `post_relation` | Parent-child post relationships |
| `post_note` | Polygon annotations on posts |
| `post_tag` | Tag-to-post associations |
| `post_favorite` | User post favorites |
| `post_feature` | Featured posts |
| `post_score` | User post scores |
| `post_custom_field` | Custom field values per post |
| `comment` | Post comments |
| `comment_score` | Comment scores |
| `tag` | Tag entities |
| `tag_name` | Tag names and aliases |
| `tag_category` | Tag category definitions |
| `tag_implication` | Tag implication rules |
| `tag_suggestion` | Tag suggestion rules |
| `pool` | Pool entities |
| `pool_name` | Pool names |
| `pool_category` | Pool category definitions |
| `pool_post` | Pool-to-post associations |
| `custom_field` | Custom field definitions |
| `snapshot` | Resource change history |
| `database_statistics` | Global counters |
| `*_statistics` | Denormalized statistics tables (6 tables) |

PostgreSQL triggers maintain the statistics tables automatically on INSERT, UPDATE, and DELETE operations.
