# User Registration Controller

**File:** `client/js/controllers/user_registration_controller.js`
**Class:** `UserRegistrationController`

## Purpose

Controls the user registration page. Supports both new user self-registration and administrators creating users while logged in.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `register` | `UserRegistrationController` | Constructor |

---

## Constructor

**Code path:**
1. Checks if the user has the `user_create_self` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "Registration is closed.".
   - Returns early.
3. Activates "register" in the top navigation bar.
4. Sets the page title to "Registration".
5. Creates a `RegistrationView` instance.
6. Registers a `submit` event listener on the view for `_evtRegister`.

---

## Event Handlers

### `_evtRegister(e)`
**Trigger:** User submits the registration form.
**Parameters:** `e.detail.name`, `e.detail.email`, `e.detail.password`

**Code path:**
1. Clears all messages on the view.
2. Disables the form.
3. Creates a new `User` model instance.
4. Sets `user.name`, `user.email`, and `user.password`.
5. Checks if there is a currently logged-in user via `api.isLoggedIn()`.
6. Calls `user.save()`.

**Success path:**
- If a user was already logged in (admin creating a user):
  - Resolves immediately (no auto-login).
  - Routes to the users list page via `router.show(uri.formatClientLink("users"))`.
  - Shows "User added!" on the list controller.
- If no user was logged in (self-registration):
  - Calls `api.forget()` to clear any residual state.
  - Calls `api.login(e.detail.name, e.detail.password, false)` to log in the new user.
  - Routes to the home page.
  - Shows "Welcome aboard!" on the home controller.

**Error path:**
- Shows the error message on the view.
- Enables the form.

### Dual Behavior

This controller handles two distinct use cases:
1. **Self-registration**: An anonymous user creates their own account. After creation, the controller logs them in automatically.
2. **Admin registration**: A logged-in administrator creates an account for someone else. After creation, the admin stays logged in and is redirected to the users list.

---

## API Calls

| Call | Purpose |
|---|---|
| `user.save()` | Creates the new user account |
| `api.login(name, password, false)` | Logs in the newly registered user (self-registration only) |
| `api.forget()` | Clears any stored credentials (self-registration only) |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | Privilege checks, login state, authentication |
| `uri` | Client link formatting |
| `User` | User model |
| `top_navigation` | Navigation bar activation and title |
| `RegistrationView` | Renders the registration form |
| `EmptyView` | Error view when registration is closed |
