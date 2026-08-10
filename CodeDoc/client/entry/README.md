# Client Entry Points

The entry layer contains the core modules that bootstrap the client application. These modules handle startup, API communication, routing, events, configuration, category colors, and templates.

## Startup Sequence

The `main.js` file orchestrates the entire startup. The sequence is:

```
1. Load auto-generated config (version, build date, environment)
2. If development mode: connect live-reload WebSocket on port 8081
3. Load polyfills for NodeList, Promise, Number, Date
4. Import misc, views, and router modules
5. Set history.scrollRestoration = "manual"
6. Register router exit handler (save scroll position + confirm page exit)
7. Register router enter handler (reset keyboard shortcuts)
8. Apply dark theme from settings (class on document.body)
9. Call api.fetchConfig() — fetches /api/info for server configuration
    └── On success:
        a. Require and register all 26 controllers on the router
        b. Re-apply dark theme
        c. Set video autoplay from settings
        d. Call api.loginFromCookies() to restore session
            └── On success:
                - Refresh tag category color map (tags.refreshCategoryColorMap())
                - Refresh pool category color map (pools.refreshCategoryColorMap())
                - Call router.start() to begin routing
            └── On failure:
                - If URL contains "login": forget auth, start router
                - Otherwise: navigate to "/" and show login error
```

### Controller Registration Order

Controllers register on the router in this order. The `NotFoundController` registers last with a `null` path pattern (match-all fallback).

| # | Controller | Route |
|---|---|---|
| 1 | HomeController | `/` |
| 2 | HelpController | `help` |
| 3 | LoginController | `login` |
| 4 | LogoutController | `logout` |
| 5 | PasswordResetController | `password-reset` |
| 6 | PasswordResetFinishController | `password-reset/:descriptor` |
| 7 | CommentsController | `comments` |
| 8 | SnapshotsController | `history` |
| 9 | PostDetailController | `post/:id/merge` |
| 10 | PostMainController | `post/:id` and `post/:id/edit` |
| 11 | PostListController | `posts` |
| 12 | PostUploadController | `upload` |
| 13 | TagCreateController | `tag/create` |
| 14 | TagController | `tag/:name` |
| 15 | TagListController | `tags` |
| 16 | TagCategoriesController | `tag-categories` |
| 17 | PoolCreateController | `pool/create` |
| 18 | PoolController | `pool/:id` |
| 19 | PoolListController | `pools` |
| 20 | PoolCategoriesController | `pool-categories` |
| 21 | CustomFieldCreateController | `custom-field/create` |
| 22 | CustomFieldController | `custom-field/:id` |
| 23 | CustomFieldListController | `custom-fields` |
| 24 | SettingsController | `settings` |
| 25 | UserController | `user/:name` |
| 26 | UserListController | `users` |
| 27 | UserRegistrationController | `register` |
| 28 | NotFoundController | `null` (fallback) |

`TopNavigationController` is a singleton created at module load time. It does not register on the router.

## API Client

The `api.js` file exports a singleton `Api` instance. It wraps all HTTP communication with the server.

### Authentication

The API client supports two authentication methods:

1. **Token-based**: `userName` + `token` sent as base64-encoded `Authorization` header. Used after login or cookie restore.
2. **Basic auth**: `userName` + `password` sent as HTTP basic auth. Used during login.

The auth flow:
- `api.login(name, password, remember)` → basic auth → fetches user → creates token → stores cookie
- `api.loginFromCookies()` → reads `auth` cookie → calls `loginWithToken()`
- `api.loginWithToken(name, token, remember)` → sets cookie → fetches user
- `api.logout()` → deletes token → resets internal state
- `api.forget()` → removes auth cookie from browser

### Request Pipeline

Every API request goes through `_wrappedRequest()`:

1. Copy request data.
2. For each file in the request: if a file upload token is cached, insert it into data. Otherwise, upload the file via `POST /api/uploads` first, cache the token, then insert it.
3. Call `_rawRequest()` with the processed data.
4. If `MissingOrExpiredRequiredFileError`: clear file token cache, set friendly error message.

### GET Request Caching

GET requests are cached by URL. If the same URL is requested twice, the cached response is returned. POST, PUT, and DELETE requests clear the entire cache. The cache lives in memory only (not `localStorage`).

### File Uploads

File uploads use a two-step process:
1. `POST /api/uploads` with the file in the `content` field. Returns a file token.
2. The file token is stored in the module-level `fileTokens` cache, keyed by `file.name + file.size`.
3. On subsequent requests that reference the same file, the token is inserted directly.

### Progress Bar

`progress.start()` is called before each HTTP request. `progress.done()` is called after each completes. A nesting counter prevents flicker during concurrent requests.

### Privilege Checking

`api.hasPrivilege(lookup)` takes a prefix like `"post_"`. It searches `remoteConfig.privileges` for keys starting with the prefix, finds the minimum rank required, and compares against the current user's rank.

Available ranks, from lowest to highest:
`anonymous` → `restricted` → `regular` → `power` → `moderator` → `administrator` → `nobody`

### Server Configuration

`api.fetchConfig()` calls `GET /api/info` and populates:
- `postCount`, `diskUsage`, `serverTime`
- `featuredPost`, `featuringTime`, `featuringUser`
- `remoteConfig` (full server configuration object)

Getters expose: site name, regex patterns (tag/pool/user/password names), contact email, cookie security, mail capability, safety toggle, and custom field default color.

## Router

The `router.js` file exports a singleton `Router` instance. It is a modified fork of the `page.js` library by visionmedia.

### Key Modifications from page.js
- Regexes changed to path components.
- Refactored to ES6 classes (`Context`, `Route`, `Router`).
- `.save()` renamed to `.replaceState()`.
- History state recovery on page refresh.
- Exposed `.url` property.

### Route Matching

Routes use an array of path components:
- Literal strings match exactly (e.g., `"post"`).
- Components starting with `:` are named parameters (e.g., `:id` → `ctx.parameters.id`).
- A trailing `;key=value` segment is parsed into a `variable` parameter.
- `null` matches everything (used for the 404 fallback).

### Navigation Methods

| Method | Description |
|---|---|
| `show(path, state, push)` | Navigate to a path. Dispatches enter/exit middleware. Pushes history state. |
| `replace(path, state, dispatch)` | Replace current URL without adding to history. Optionally dispatches middleware. |
| `showNoDispatch(path, state)` | Push history state without dispatching middleware. Used for search parameter updates. |
| `start()` | Begin listening for popstate and click events. Navigate to current URL. |
| `stop()` | Stop listening. |

### Link Interception

The router intercepts clicks on same-origin `<a>` elements. It routes them through `router.show()` instead of a full page load. It ignores clicks with:
- Meta, Ctrl, or Shift keys pressed.
- `download` attribute or `rel="external"`.
- `mailto:` protocol.
- A `target` attribute.
- Hash-only links to the same path.

## Event System

The `events.js` file provides:

### EventTarget Class

A lightweight implementation using a `DocumentFragment` as the backing target. All models extend `EventTarget` to emit `change`, `delete`, and other events.

### proxyEvent Function

Forwards events from one target to another:
```js
proxyEvent(sourceControl, this, "submit", "submit");
```

### Message Type Constants

`Success`, `Error`, `Info` — CSS class strings used for message display in views.

## Configuration

The `config.js` file wraps `.config.autogen.json`, an auto-generated file with:

| Field | Purpose |
|---|---|
| `meta.version` | Git hash + dirty flag (e.g., `"1660cfe6-dirty"`) |
| `meta.buildDate` | Build date and time |
| `environment` | `"production"` or `"development"` |

Used by `main.js` to decide whether to connect a live-reload WebSocket (development mode only).

## Tags Module

The `tags.js` file manages the dynamic CSS stylesheet for tag category colors.

### refreshCategoryColorMap()

1. Fetches `TagCategoryList.get()` from the API.
2. Removes any previous `<style>` element from `<head>`.
3. Creates a new `<style>` element.
4. For each tag category, inserts a CSS rule:
   ```css
   .tag-category_name { color: <category.color>; }
   ```
5. The CSS class name is generated by `misc.makeCssName(category.name, "tag")`.

Called during startup (after login) and after tag category edits in `TagCategoriesController`.

## Pools Module

The `pools.js` file mirrors `tags.js` for pool category colors.

### refreshCategoryColorMap()

Same pattern but uses `PoolCategoryList.get()` and CSS class prefixes of `"pool"`. Produces rules like:
```css
.pool-category_name { color: <category.color>; }
```

Called during startup and after pool category edits in `PoolCategoriesController`.

## Templates Module

The `templates.js` file re-exports the auto-generated compiled templates module:

```js
module.exports = require("./.templates.autogen.js");
```

The `.templates.autogen.js` file contains compiled Underscore templates as named functions. Template names include: `comment`, `comment-list`, `post-content`, `post-edit-sidebar`, `post-readonly-sidebar`, `expander`, `file-dropper`, `tag-input`, `pool-input`, `custom-field-input`, `score`, `fav`, and others.

The `views.js` utility's `getTemplate()` function resolves template names through this module and renders them with enriched context objects.
