# Auth Controller

**File:** `client/js/controllers/auth_controller.js`
**Classes:** `LoginController`, `LogoutController`

## Purpose

Controls the login page and the logout action. Manages user authentication through the API layer.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `login` | `LoginController` | Constructor |
| `logout` | `LogoutController` | Constructor |

---

## LoginController

### Constructor

1. Calls `api.forget()` to clear stored credentials.
2. Activates the "login" item in the top navigation bar.
3. Sets the page title to "Login".
4. Creates a `LoginView` instance.
5. Registers a `submit` event listener on the view.

### Event Handlers

#### `_evtLogin(e)`
**Trigger:** User submits the login form.
**Parameters:** `e.detail.name`, `e.detail.password`, `e.detail.remember`

**Code path:**
1. Clears all messages on the login view.
2. Disables the login form.
3. Calls `api.forget()` to clear any previous session.
4. Calls `api.login(name, password, remember)`.

**Success path:**
- Routes to the home page via `router.show(uri.formatClientLink())`.
- Shows the success message "Logged in" on the home controller.
- Refreshes the tag category color map via `tags.refreshCategoryColorMap()`.
- Refreshes the pool category color map via `pools.refreshCategoryColorMap()`.

**Error path:**
- Shows the error message on the login view.
- Re-enables the login form.

### API Calls

| Call | Purpose |
|---|---|
| `api.forget()` | Clears stored credentials from memory |
| `api.login(name, password, remember)` | Sends login credentials to the server |

---

## LogoutController

### Constructor

1. Calls `api.forget()` to clear stored credentials.
2. Calls `api.logout()` to end the session on the server.
3. Routes to the home page via `router.show(uri.formatClientLink())`.
4. Shows the success message "Logged out" on the home controller.

### API Calls

| Call | Purpose |
|---|---|
| `api.forget()` | Clears stored credentials from memory |
| `api.logout()` | Ends the server session |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | Authentication API calls |
| `tags` | Refreshes tag category color map after login |
| `pools` | Refreshes pool category color map after login |
| `uri` | Formats client links |
| `top_navigation` | Updates the navigation bar active item and title |
| `LoginView` | The login form view |
