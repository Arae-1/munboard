---
title: Build System
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, build, pipeline]
---

# Build Pipeline

## Overview

The build system is a Node.js script (`build.js`) that bundles all client assets into a `public/` directory for deployment. It uses Browserify for JavaScript bundling, Stylus for CSS preprocessing, and various optimization tools.

## Entry Point

**Command**: `node build.js [flags]`

**Flags**:
- `--watch` — Watch mode with live reload via WebSocket
- `--debug` — Debug mode (no minification, source maps)
- `--no-transpile` — Skip Babel transpilation
- `--no-binary-assets` — Skip copying fonts/images
- `--no-web-app-files` — Skip generating icons/splash screens
- `--no-html` — Skip HTML template bundling
- `--no-css` — Skip CSS bundling
- `--no-js` — Skip JS bundling (vendor and/or app)
- `--no-vendor-js` — Skip vendor JS bundle
- `--no-app-js` — Skip app JS bundle
- `--no-live-reload` — Disable WebSocket live reload
- `--gzip` — Generate gzipped (.gz) versions of output files

**Environment variables**:
- `BASE_URL` — Base URL for the application (default `/`)
- `BUILD_INFO` — Build version string (falls back to `git describe`)

## Build Pipeline Steps

### 1. `makeOutputDirs()`
Creates the output directory structure:
- `public/`
- `public/css/`
- `public/fonts/`
- `public/img/`
- `public/js/`

### 2. `bundleConfig()`
Generates `js/.config.autogen.json` with:
- `meta.version` — git describe or `BUILD_INFO` env var
- `meta.buildDate` — UTC date string
- `environment` — `"development"` or `"production"`

### 3. `bundleHtml()`
- Reads `html/index.htm`, replaces `<!-- Base HTML Placeholder -->` with `<base href="...">`
- Compiles all `html/**/*.tpl` files into `js/.templates.autogen.js`
  - Template names are derived from filenames: underscores become hyphens
  - Templates are processed by:
    1. Extracting `<%...%>` placeholders to prevent HTML minification from breaking them
    2. Minifying HTML with `html-minifier` (remove comments, collapse whitespace, conservative collapse)
    3. Restoring placeholders
    4. Compiling with `underscore.template()` (uses `variable: 'ctx'` parameter)
    5. Assigning to `templates['<name>']` in the output module
- Writes minified `public/index.htm`

### 4. `bundleCss()`
- Reads all `css/**/*.styl` files, renders each with Stylus
- Concatenates all output CSS
- Minifies with CSSO (`csso`)
- Writes to `public/css/app.min.css`
- Copies Font Awesome CSS from `node_modules/font-awesome/css/font-awesome.min.css` to `public/css/vendor.min.css`
- Optionally gzips both

### 5. `bundleJs()`
Two separate bundles:

#### Vendor Bundle (`public/js/vendor.min.js`)
- Uses Browserify to bundle external libraries:
  - `dompurify`, `js-cookie`, `marked`, `mousetrap`, `nprogress`, `superagent`, `underscore`
- Adds `babel-polyfill` if not `--no-transpile`
- Minifies with Terser (`unused: false` to preserve polyfill side effects)

#### App Bundle (`public/js/app.min.js`)
- Uses Browserify to bundle all `js/**/*.js` files
- Externalizes the vendor libraries (shared between bundles)
- Transforms with Babelify (unless `--no-transpile`)
- Minifies with Terser in production mode

### 6. `bundleBinaryAssets()`
- Copies `img/favicon.png` and `img/logo.png` to `public/img/`
- Copies `fonts/open_sans.woff2` to `public/fonts/`
- Copies all Font Awesome font files from `node_modules/font-awesome/fonts/`
- Optionally gzips all font files except `.woff2`

### 7. `bundleWebAppFiles()`
- Generates `public/manifest.json` with PWA manifest
- Uses Jimp to resize `img/app.png` into:
  - `android-chrome-192x192.png` (192px)
  - `android-chrome-512x512.png` (512px)
  - `apple-touch-icon.png` (180px)
  - `mstile-150x150.png` (150px)
- Uses Jimp to generate splash screens from `img/splash.png` for 7 different device sizes (640×1136 through 2048×2732)

## Watch Mode

When `--watch` is used:
- Starts a WebSocket server on port 8081 (development) or 8080 (production)
- Uses `chokidar` to watch for changes in:
  - `fonts/**/*` → rebundle assets, reload
  - `img/**/*` → regenerate webapp files, reload
  - `html/**/*.tpl` → rebundle HTML templates
  - `css/**/*.styl` → rebundle CSS, reload
- Uses `watchify` for incremental JS bundling (rebundles only changed modules)
- Sends "reload" message to connected WebSocket clients on changes

## Dependencies (devDependencies)

| Package | Purpose |
|---------|---------|
| `babel-core`, `babel-preset-env`, `babelify` | ES6+ to ES5 transpilation |
| `browserify`, `watchify` | JS module bundling |
| `chokidar` | File watching |
| `csso` | CSS minification |
| `glob` | File pattern matching |
| `html-minifier` | HTML minification |
| `jimp` | Image processing (icons, splash screens) |
| `pretty-error` | Error formatting |
| `stylus` | CSS preprocessing |
| `terser` | JS minification |
| `underscore` | Template engine |
| `ws` | WebSocket server for live reload |

## Dependencies (runtime)

| Package | Purpose |
|---------|---------|
| `dompurify` | HTML sanitization |
| `font-awesome` | Icon font |
| `ios-inner-height` | iOS viewport height fix |
| `js-cookie` | Cookie management |
| `marked` | Markdown rendering |
| `mousetrap` | Keyboard shortcut handling |
| `nprogress` | Loading progress bar |
| `superagent` | HTTP client |
| `turndown` | HTML to Markdown conversion |

## Babel Configuration

`.babelrc`: Uses `babel-preset-env` (transpiles to ES5 based on target environment).
