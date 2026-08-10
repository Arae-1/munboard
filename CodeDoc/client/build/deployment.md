---
title: Deployment
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, build, deployment, docker, nginx]
---

# Deployment

## Dockerfile

The Docker build uses a multi-stage build with three stages:

### Stage 1: `builder` (node:lts)
- Copies `package.json` and `package-lock.json`
- Runs `npm install`
- Copies all source files
- Builds with `BASE_URL="__BASEURL__" node build.js --gzip`
- Passes `BUILD_INFO` and `CLIENT_BUILD_ARGS` as build args

### Stage 2: `approot` (scratch)
- Copies `docker-start.sh` to `/`
- Copies `nginx.conf.docker` to `/etc/nginx/nginx.conf`
- Copies `public/` output from builder to `/var/www/`

### Stage 3: `release` (nginx:alpine)
- Installs `dumb-init` for proper signal handling
- Copies everything from `approot` stage
- CMD runs `/docker-start.sh`
- Volume mount at `/data`
- OCI labels for image metadata

## docker-start.sh

A shell script that:
1. Replaces `__BACKEND__` placeholder in nginx config with `BACKEND_HOST` environment variable
2. Replaces `__BASEURL__` placeholder in `index.htm` and `manifest.json` with `BASE_URL` environment variable (defaults to `/`)
3. Starts nginx as the foreground process

## nginx.conf.docker

### Server configuration:
- **Worker processes**: 1
- **User**: nginx
- **Logging**: error to stderr, access to stdout with custom log format (IP, request, status, referer, forwarded-for)
- **Upstream**: Backend server at `__BACKEND__:6666` (replaced at container startup)

### Location blocks:

| Location | Purpose |
|----------|---------|
| `~ ^/(docs\|apidoc)` | Proxy Swagger UI to backend. Private by default: only private address ranges (127.0.0.1, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) are allowed, all others are denied |
| `~ ^/api$` | Redirect `/api` to `/api/` |
| `~ ^/api/(.*)$` | API proxy with CORS headers, security headers, gzip, 8GB max body, backend error handling |
| `/data/` | Legacy redirect for old flat post URLs, then serves `/data` directory with sendfile. Unauthorized/not-found error pages |
| `/data/temporary-uploads/` | Returns 403. Temporary upload staging files are internal only. More specific than `/data/`, so it wins for every request under this path |
| `~ ^/post/[0-9]+/?$` | Proxies post URLs to the backend OpenGraph embed pages. Rewrites `/post/{id}` to `/embed/post/{id}` |
| `/` | Serves static files from `/var/www`, falls back to `index.htm`. gzip_static for pre-compressed files |

### Security headers:

Sent on every response from the server block, and repeated in the `/api/` location (a location with its own `add_header` does not inherit server-level ones):

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Strict-Transport-Security`: value comes from the `$hsts_header` map. The map sends `max-age=31536000; includeSubDomains` only when `X-Forwarded-Proto` is `https`; otherwise it sends an empty value. This keeps HSTS off for plain-HTTP requests
- `Content-Security-Policy`: `default-src 'self'; script-src 'self' https://static.cloudflareinsights.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' ws: wss:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'`
  - Scripts load from `'self'` and static.cloudflareinsights.com (Cloudflare Web Analytics beacon)
  - `style-src 'unsafe-inline'` because the client injects `<style>` rules at runtime
  - `connect-src` allows the WebSocket the client opens on port 8081
  - `img-src https:` allows gravatar/external avatar images

### CORS for API:
- `Access-Control-Allow-Origin` uses the `$cors_origin` map value, not a wildcard. The map echoes the request's `Origin` only when the origin matches the `Host` header the browser used (with optional port). Cross-site pages receive no `Access-Control-Allow-Origin` and cannot read API responses. The bundled client is same-origin
- OPTIONS preflight returns methods (GET, POST, PUT, DELETE, OPTIONS) and headers (Authorization, Content-Type)

### Static file delivery:
- `sendfile on`, `tcp_nopush on`, `tcp_nodelay on`
- `gzip_static on` — serves pre-compressed `.gz` files when available

## docker-start.sh Flow

```
Container starts
  → /docker-start.sh (via dumb-init)
    → sed replaces __BACKEND__ with $BACKEND_HOST in nginx.conf
    → sed replaces __BASEURL__ with $BASE_URL in index.htm and manifest.json
    → exec nginx (daemon off)
```

## .dockerignore

Excludes from Docker context:
- `node_modules/*`
- `public/` (generated during build)
- `Dockerfile`, `.dockerignore`
- All `.gitignore` files

## npm Scripts

| Script | Command | Purpose |
|--------|---------|---------|
| `build` | `node build.js` | Production build |
| `watch` | `node build.js --watch` | Development watch mode |
| `build-container` | `docker build -t szurubooru/client:dev .` | Build Docker image |
