# Settings Controller

**File:** `client/js/controllers/settings_controller.js`
**Class:** `SettingsController`

## Purpose

Controls the browsing settings page. Manages user preferences such as endless scroll toggle and posts per page.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `settings` | `SettingsController` | Constructor |

---

## Constructor

**Code path:**
1. Activates "settings" in the top navigation bar.
2. Sets the page title to "Browsing settings".
3. Creates a `SettingsView` with `settings: settings.get()` (current settings object).
4. Registers a `submit` event listener on the view.

### Notes

- No privilege check. The settings page is always accessible. Settings are stored client-side (localStorage).
- No API calls.

---

## Event Handlers

### `_evtSubmit(e)`
**Trigger:** User submits the settings form.
**Parameters:** `e.detail` (the settings object from the form)

**Code path:**
1. Clears all messages on the view.
2. Calls `settings.save(e.detail)` to persist the new settings.
3. Shows "Settings saved." on the view.

---

## API Calls

None. Settings are stored client-side only.

---

## Dependencies

| Module | Use |
|---|---|
| `settings` | Gets and saves browsing settings |
| `top_navigation` | Navigation bar activation and title |
| `SettingsView` | Renders the settings form |
