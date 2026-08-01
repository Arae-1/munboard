# Home Controller

**File:** `client/js/controllers/home_controller.js`
**Class:** `HomeController`

## Purpose

Controls the home page (the root route `/`). Shows the site name, version, stats, and featured post.

## Routing

| Route | Controller | Handler |
|---|---|---|
| (empty / root) | `HomeController` | Constructor |

---

## Constructor

**Code path:**
1. Activates "home" in the top navigation bar.
2. Sets the page title to "Home".
3. Creates a `HomeView` with the following data:
   - `name`: The site name from `api.getName()`
   - `version`: The version from `config.meta.version`
   - `buildDate`: The build date from `config.meta.buildDate`
   - `canListSnapshots`: `api.hasPrivilege("snapshot_list")`
   - `canListPosts`: `api.hasPrivilege("post_list")`
   - `isDevelopmentMode`: `config.environment == "development"`
4. Calls `api.fetchConfig()` to get server configuration.

**On `fetchConfig` success:**
- Sets stats via `this._homeView.setStats()`:
  - `postCount`: `api.getPostCount()`
  - `diskUsage`: `api.getDiskUsage()`
- Sets the featured post via `this._homeView.setFeaturedPost()`:
  - `featuredPost`: Converts from API response using `Post.fromResponse()` if one exists, otherwise `null`
  - `featuringUser`: `api.getFeaturingUser()`

### Notes

- No privilege check for viewing the home page. It is always accessible.
- No event listeners. The view is read-only.

---

## Public Methods

### `showSuccess(message)`
**Parameters:** `message` (string)
**Code path:** Calls `this._homeView.showSuccess(message)`.

### `showError(message)`
**Parameters:** `message` (string)
**Code path:** Calls `this._homeView.showError(message)`.

---

## API Calls

| Call | Purpose |
|---|---|
| `api.fetchConfig()` | Fetches server configuration (post count, disk usage, featured post) |
| `api.getName()` | Gets the site name (from config, not an API call) |
| `api.getPostCount()` | Gets the total post count (from fetched config) |
| `api.getDiskUsage()` | Gets the disk usage (from fetched config) |
| `api.getFeaturedPost()` | Gets the featured post data (from fetched config) |
| `api.getFeaturingUser()` | Gets the user who set the featured post (from fetched config) |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Configuration and privilege checks |
| `config` | Meta information (version, build date, environment) |
| `Post` | Deserializes the featured post from the API response |
| `top_navigation` | Navigation bar activation and title |
| `HomeView` | Renders the home page |
