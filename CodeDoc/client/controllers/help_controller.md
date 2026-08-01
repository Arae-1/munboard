# Help Controller

**File:** `client/js/controllers/help_controller.js`
**Class:** `HelpController`

## Purpose

Controls the help page. Displays static help content with optional section and subsection navigation.

## Routing

| Route | Controller | Parameters |
|---|---|---|
| `help` | `HelpController` | (none) |
| `help/:section` | `HelpController` | `section` |
| `help/:section/:subsection` | `HelpController` | `section`, `subsection` |

---

## Constructor

**Parameters:** `section` (optional string), `subsection` (optional string)

**Code path:**
1. Activates "help" in the top navigation bar.
2. Sets the page title to "Help".
3. Creates a `HelpView` with the optional `section` and `subsection`.

### Notes

- No privilege check. The help page is public.
- No event listeners. The view handles its own navigation.
- No API calls. The help content is static.
- No error handling. The view handles its own display.

---

## Dependencies

| Module | Use |
|---|---|
| `top_navigation` | Navigation bar activation and title |
| `HelpView` | Renders the help content |
