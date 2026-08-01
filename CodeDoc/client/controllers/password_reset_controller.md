# Password Reset Controller

**File:** `client/js/controllers/password_reset_controller.js`
**Classes:** `PasswordResetController`, `PasswordResetFinishController`

## Purpose

Controls two pages in the password reset flow:
1. **Password Reset Request** — The user enters their name or email to request a reset link.
2. **Password Reset Finish** — The user clicks the link from the email to complete the reset.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `password-reset` | `PasswordResetController` | Constructor |
| `password-reset/:descriptor` | `PasswordResetFinishController` | Constructor |

The `:descriptor` parameter is split into `name` and `token` by the colon delimiter (`name:token`).

---

## PasswordResetController

### Constructor

**Code path:**
1. Activates "login" in the top navigation bar.
2. Sets the page title to "Password reminder".
3. Creates a `PasswordResetView` instance.
4. Registers a `submit` event listener on the view for `_evtReset`.

### Notes

- No privilege check. The page is always accessible.

---

### Event Handlers

### `_evtReset(e)`
**Trigger:** User submits the reset request form.
**Parameters:** `e.detail.userNameOrEmail`

**Code path:**
1. Clears all messages on the view.
2. Disables the form.
3. Calls `api.forget()` to clear stored credentials.
4. Calls `api.logout()` to end any session.
5. Calls `api.get(uri.formatApiLink("password-reset", e.detail.userNameOrEmail))`.

**Success path:**
- Shows "E-mail has been sent. To finish the procedure, please click the link it contains.".

**Error path:**
- Shows the error message on the view.
- Enables the form.

---

## PasswordResetFinishController

### Constructor

**Parameters:** `name` (the user name), `token` (the reset token)

**Code path:**
1. Calls `api.forget()` to clear stored credentials.
2. Calls `api.logout()` to end any session.
3. Declares a `password` variable initialized to `null`.
4. Calls `api.post(uri.formatApiLink("password-reset", name), { token: token })`.

**Step 1 — Get new password:**
- On success: stores the new password from `response.password`. Returns `api.login(name, password, false)`.

**Step 2 — Auto-login with new password:**
- On success: routes to the home page via `router.show(uri.formatClientLink())`. Shows "New password: {password}".
- On error (either step): routes to the home page. Shows the error message.

---

## API Calls

| Call | Endpoint | Purpose |
|---|---|---|
| `api.get` | `/api/password-reset/{userNameOrEmail}` | Requests a password reset email |
| `api.post` | `/api/password-reset/{name}` with `{ token }` | Completes the reset and gets a new password |
| `api.login(name, password, false)` | Authentication | Logs the user in with the new password |
| `api.forget()` | (client) | Clears stored credentials |
| `api.logout()` | (server) | Ends the session |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | API calls and credential management |
| `uri` | Link formatting |
| `top_navigation` | Navigation bar activation and title |
| `PasswordResetView` | Renders the reset request form |
