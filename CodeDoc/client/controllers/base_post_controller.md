# Base Post Controller

**File:** `client/js/controllers/base_post_controller.js`
**Class:** `BasePostController`

## Purpose

Provides a base class for all post-related controllers. Checks the `post_view` privilege. Activates the "posts" navigation item. Sets the page title.

This controller does **not** register route handlers. It is a base class extended by `PostDetailController` and `PostMainController`.

## Constructor

**Parameters:** `ctx` (the router context with `parameters.id`)

**Code path:**
1. Checks if the user has the `post_view` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows the message "You don't have privileges to view posts.".
   - Returns early. No further initialization occurs.
3. If the user has the privilege:
   - Activates the "posts" item in the top navigation bar.
   - Sets the page title to "Post #{id}" using `ctx.parameters.id`.

### Privilege Check

| Privilege | Effect When Missing |
|---|---|
| `post_view` | Shows an error and stops initialization |

### API Calls

None. This controller delegates API calls to its subclasses.

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege check (`hasPrivilege`) |
| `top_navigation` | Navigation bar activation and title |
| `EmptyView` | Shows the error view when privilege is missing |
