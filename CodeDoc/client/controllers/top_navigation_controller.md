# Top Navigation Controller

**File:** `client/js/controllers/top_navigation_controller.js`
**Class:** `TopNavigationController`

## Purpose

Controls the top navigation bar. This is a global singleton controller, not a route controller. It manages:
- Visibility of navigation items based on user privileges
- Login/logout state changes
- Active item highlighting
- User avatar and account link

## Initialization

The controller is exported as a singleton instance: `module.exports = new TopNavigationController()`.

This means it is created immediately when the module is loaded. There is no route registration.

---

## Constructor

**Code path:**
1. Calls `api.fetchConfig()` to get server configuration.

**On `fetchConfig` success:**
- Creates a `TopNavigationView` instance.
- Registers an `activate` event listener on the `topNavigation` model to handle `_evtActivate`.
- Registers `login` and `logout` event listeners on the `api` module to handle `_evtAuthChange`.
- Calls `_render()` to render the initial navigation state.

---

## Event Handlers

### `_evtAuthChange(e)`
**Trigger:** The `api` module fires a `login` or `logout` event.
**Code path:** Calls `_render()` to re-render the navigation with updated privileges and user state.

### `_evtActivate(e)`
**Trigger:** A route controller calls `topNavigation.activate(key)`.
**Parameters:** `e.detail.key` (the key of the navigation item to activate)
**Code path:** Calls `this._topNavigationView.activate(e.detail.key)` to highlight the active item in the rendered view.

---

## Private Methods

### `_updateNavigationFromPrivileges()`
**Code path:**
1. Sets the "account" item URL to `"user/" + api.userName`.
2. Sets the "account" item image URL to `api.user.avatarUrl` (or `null` if not logged in).
3. Calls `topNavigation.showAll()` to make all items visible.
4. Hides items based on missing privileges:

| Navigation Key | Required Privilege |
|---|---|
| `posts` | `post_list` |
| `upload` | `post_create` |
| `comments` | `comment_list` |
| `tags` | `tag_list` |
| `users` | `user_list` |
| `pools` | `pool_list` |
| `custom-fields` | `custom_field_list` |

5. If the user is logged in:
   - Hides "register" if the user lacks `user_create_any`.
   - Hides "login".
6. If the user is not logged in:
   - Hides "register" if the user lacks `user_create_self`.
   - Hides "account" and "logout".

### `_render()`
**Code path:**
1. Calls `_updateNavigationFromPrivileges()`.
2. Renders the navigation view with:
   - `items`: All navigation items from `topNavigation.getAll()`
   - `name`: The site name from `api.getName()`
3. Activates the currently active item in the view (or empty string if none).

---

## API Calls

| Call | Purpose |
|---|---|
| `api.fetchConfig()` | Fetches server configuration and user state |

---

## No API Calls

None besides `fetchConfig()`. All other operations use data already loaded in the `api` module.

---

## Dependencies

| Module | Use |
|---|---|
| `api` | User state, privileges, configuration |
| `top_navigation` | Navigation model (items, active state) |
| `TopNavigationView` | Renders the navigation bar in the DOM |
