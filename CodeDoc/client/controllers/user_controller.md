# User Controller

**File:** `client/js/controllers/user_controller.js`
**Class:** `UserController`

## Purpose

Controls the user detail/profile page. Supports viewing, editing, deleting a user, and managing user authentication tokens. Handles both the logged-in user's own profile and other users' profiles.

## Routing

| Route | Controller | Section |
|---|---|---|
| `user/:name` | `UserController` | `"summary"` |
| `user/:name/edit` | `UserController` | `"edit"` |
| `user/:name/list-tokens` | `UserController` | `"list-tokens"` |
| `user/:name/delete` | `UserController` | `"delete"` |

---

## Constructor

**Parameters:** `ctx` (the router context), `section` (one of `"summary"`, `"edit"`, `"list-tokens"`, `"delete"`)

**Code path:**
1. Checks if the user has the `user_view` privilege OR is logged in as the target user.
2. If neither condition is met:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view users.".
   - Returns early.
3. Initializes message queues: `this._successMessages = []`, `this._errorMessages = []`.
4. If section is `"list-tokens"`:
   - Fetches user tokens via `UserToken.get(userName)`.
   - Marks each token: sets `token.isCurrentAuthToken` using `api.isCurrentAuthToken(token)`.
   - On token fetch error: returns an empty array (tokens still display but none are shown).
5. Sets the page title to "User {userName}".
6. Calls `Promise.all([userTokenPromise, User.get(userName)])`.

**Success path:**
- Determines if the current user is the target user via `api.isLoggedIn(user)`.
- Sets `infix` to `"self"` if logged in, `"any"` otherwise. This infix determines privilege suffixes (e.g., `user_edit_self_name` vs `user_edit_any_name`).
- Stores the original name in `this._name`.
- Registers a `change` event listener on the user model for `_evtSaved`.
- Builds a rank map: includes only ranks at or below the current user's rank. Excludes "anonymous".
- Activates "account" in navigation if viewing own profile, or "users" if viewing another user's profile.
- Creates a `UserView` with user data, section, login state, privileges, ranks, and tokens.
- Registers six event listeners: `change`, `submit`, `delete`, `create-token`, `delete-token`, `update-token`.
- Flushes any queued success/error messages to the view.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privilege Check Details

The privilege check for viewing is:
```javascript
if (!api.hasPrivilege("user_view") && !api.isLoggedIn({ name: userName }))
```

Users can always view their own profile. Other profiles require `user_view`.

### Privilege Infix Logic

The `infix` is `"self"` when the logged-in user views their own profile, and `"any"` when viewing another user's profile. This is used to construct privilege names:
- `user_edit_{infix}_name`
- `user_edit_{infix}_pass`
- `user_edit_{infix}_email`
- `user_edit_{infix}_rank`
- `user_edit_{infix}_avatar`
- `user_edit_{infix}` (anything)
- `user_token_list_{infix}`
- `user_token_create_{infix}`
- `user_token_edit_{infix}`
- `user_token_delete_{infix}`
- `user_delete_{infix}`

### Rank Map Construction

The controller builds a map of available ranks:
1. Gets the current user's rank index via `api.allRanks.indexOf(api.user.rank)`.
2. Iterates over all ranks.
3. Skips the "anonymous" rank.
4. Skips ranks above the current user's rank (user cannot assign a rank higher than their own).
5. Maps rank identifiers to display names using `api.rankNames`.

### Privileges Passed to the View

| Context Property | Privilege Pattern |
|---|---|
| `isLoggedIn` | `api.isLoggedIn(user)` |
| `canEditName` | `user_edit_{infix}_name` |
| `canEditPassword` | `user_edit_{infix}_pass` |
| `canEditEmail` | `user_edit_{infix}_email` |
| `canEditRank` | `user_edit_{infix}_rank` |
| `canEditAvatar` | `user_edit_{infix}_avatar` |
| `canEditAnything` | `user_edit_{infix}` |
| `canListTokens` | `user_token_list_{infix}` |
| `canCreateToken` | `user_token_create_{infix}` |
| `canEditToken` | `user_token_edit_{infix}` |
| `canDeleteToken` | `user_token_delete_{infix}` |
| `canDelete` | `user_delete_{infix}` |
| `ranks` | Rank map (filtered) |
| `tokens` | User tokens array |

---

## Public Methods

### `showSuccess(message)`
**Parameters:** `message` (string)

**Code path:**
1. If `this._view` is not yet defined (view still loading):
   - Pushes the message to `this._successMessages` queue.
2. If the view exists:
   - Calls `this._view.showSuccess(message)`.

This queuing mechanism handles messages sent before the view is ready (e.g., from token creation redirects).

### `showError(message)`
**Parameters:** `message` (string)

**Code path:** Same queuing pattern as `showSuccess`, using `this._errorMessages`.

---

## Event Handlers

### `_evtChange(e)`
**Trigger:** Any change to the view form.
**Code path:** Calls `misc.enableExitConfirmation()`.

### `_evtSaved(e, section)`
**Trigger:** The user model fires a `change` event after a successful save.
**Parameters:** `e.detail.user`

**Code path:**
1. Calls `misc.disableExitConfirmation()`.
2. If the user name changed from the original, calls `router.replace()` to update the URL.

### `_evtUpdate(e)`
**Trigger:** User submits the edit form.
**Parameters:** `e.detail.user`, `e.detail.name`, `e.detail.email`, `e.detail.rank`, `e.detail.password`, `e.detail.avatarStyle`, `e.detail.avatarContent`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Determines `isLoggedIn` and `infix`.
4. Updates user properties (only if not `undefined`/`null`):
   - `name` — sets `e.detail.user.name`
   - `email` — sets `e.detail.user.email`
   - `rank` — sets `e.detail.user.rank`
   - `password` — sets `e.detail.user.password`
   - `avatarStyle` — sets `e.detail.user.avatarStyle`. If `avatarContent` exists, also sets `e.detail.user.avatarContent`.
5. Calls `e.detail.user.save()`.
6. If the user is logged in (editing own profile):
   - After save, re-authenticates via `api.login(e.detail.name || api.userName, e.detail.password || api.userPassword, false)`.
   - This is necessary because changing the name or password invalidates the current session.
7. If the user is not logged in (admin editing another user):
   - Resolves immediately after save.

**Success path:** Shows "Settings updated." and enables the form.
**Error path:** Shows the error message and enables the form.

### `_evtDelete(e)`
**Trigger:** User confirms account deletion.
**Parameters:** `e.detail.user`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Determines `isLoggedIn`.
4. Calls `e.detail.user.delete()`.

**Success path:**
- If the user was logged in (deleted own account):
  - Calls `api.forget()` and `api.logout()`.
- If the user has `user_list` privilege:
  - Routes to the users list page.
- Otherwise:
  - Routes to the home page.
- Shows "Account deleted.".

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtCreateToken(e)`
**Trigger:** User creates a new authentication token.
**Parameters:** `e.detail.user`, `e.detail.note`, `e.detail.expirationTime`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `UserToken.create(user.name, note, expirationTime)`.

**Success path:**
- Routes to the user's token list page.
- Shows "Token {token} created.".

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtDeleteToken(e)`
**Trigger:** User deletes an authentication token.
**Parameters:** `e.detail.user`, `e.detail.userToken`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Checks if the token is the current authentication token via `api.isCurrentAuthToken(e.detail.userToken)`.

**If current token:**
- Routes to the logout page via `router.show(uri.formatClientLink("logout"))`.
- This logs the user out since their current token is being deleted.

**If not current token:**
- Calls `e.detail.userToken.delete(e.detail.user.name)`.
- On success: routes to the token list page. Shows "Token {token} deleted.".
- On error: shows the error message and enables the form.

### `_evtUpdateToken(e)`
**Trigger:** User edits a token's note.
**Parameters:** `e.detail.user`, `e.detail.userToken`, `e.detail.note`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. If `e.detail.note` is not `undefined`/`null`, sets `e.detail.userToken.note = e.detail.note`.
4. Calls `e.detail.userToken.save(e.detail.user.name)`.

**Success path:**
- Routes to the token list page.
- Shows "Token {token} updated.".

**Error path:**
- Shows the error message.
- Enables the form.

---

## API Calls

| Call | Purpose |
|---|---|
| `UserToken.get(userName)` | Fetches all authentication tokens for the user |
| `User.get(userName)` | Fetches the user profile |
| `user.save()` | Saves user profile edits |
| `user.delete()` | Deletes the user account |
| `api.login(name, password, false)` | Re-authenticates after profile changes |
| `api.forget()` | Clears stored credentials |
| `api.logout()` | Ends the session |
| `UserToken.create(userName, note, expirationTime)` | Creates a new authentication token |
| `userToken.delete(userName)` | Deletes an authentication token |
| `userToken.save(userName)` | Updates a token's note |
| `api.isCurrentAuthToken(token)` | Checks if a token is the one currently in use |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | User state, privileges, authentication |
| `uri` | Client link formatting |
| `misc` | Exit confirmation management |
| `views` | (Imported but not used directly) |
| `User` | User model |
| `UserToken` | Authentication token model |
| `top_navigation` | Navigation bar activation and title |
| `UserView` | Renders the user profile page |
| `EmptyView` | Error view |
