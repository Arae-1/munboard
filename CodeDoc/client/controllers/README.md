# Client Controllers

The controller layer manages the application logic for each page. Controllers handle routing, privilege checks, API calls, and event coordination between models and views.

## Architecture

Each controller registers one or more route handlers on the client router. The router calls the controller constructor when the URL matches. The constructor performs privilege checks, fetches data, creates views, and binds event listeners.

There are 29 controllers spread across two types: route controllers (most) and utility controllers (one).

### Controller Hierarchy

```
BasePostController
  ├── PostDetailController
  └── PostMainController

PageController (utility, not route-based)
  └── Used by: PostListController, TagListController, PoolListController,
               UserListController, CommentsController, SnapshotsController,
               CustomFieldListController
```

`BasePostController` checks the `post_view` privilege, activates the "posts" navigation item, and sets the page title. `PostDetailController` and `PostMainController` extend it for merge and view/edit operations.

`PageController` provides pagination logic. It delegates to either `EndlessPageView` or `ManualPageView` based on the `endlessScroll` user setting.

### Shared Patterns

**Privilege checks.** Every route controller starts with a privilege check. If the user lacks the required privilege, the controller creates an `EmptyView` and shows a "You don't have privileges..." message. All other initialization stops.

**Exit confirmation.** Controllers use `misc.enableExitConfirmation()` and `misc.disableExitConfirmation()` to warn users before leaving a page with unsaved changes. Enable is called on form `change` events. Disable is called on successful save.

**Form state.** Controllers follow a consistent pattern for form submissions:
1. Clear all messages on the view.
2. Disable the form.
3. Update model properties or make API calls.
4. On success: show a success message and enable the form.
5. On error: show the error message and enable the form.

**Message queue.** The `UserController` implements a message queue pattern. Messages sent before the view is ready are stored in `this._successMessages` and `this._errorMessages` arrays. When the view becomes available, queued messages flush to it.

## Routing Table

| URL Pattern | Controller | Section/Handler |
|---|---|---|
| (root) | HomeController | constructor |
| `login` | LoginController | constructor |
| `logout` | LogoutController | constructor |
| `register` | UserRegistrationController | constructor |
| `password-reset` | PasswordResetController | constructor |
| `password-reset/:descriptor` | PasswordResetFinishController | constructor |
| `help` | HelpController | constructor |
| `help/:section` | HelpController | constructor |
| `help/:section/:subsection` | HelpController | constructor |
| `settings` | SettingsController | constructor |
| `posts` | PostListController | constructor |
| `upload` | PostUploadController | constructor |
| `post/:id` | PostMainController | editMode=false |
| `post/:id/edit` | PostMainController | editMode=true |
| `post/:id/merge` | PostDetailController | section="merge" |
| `comments` | CommentsController | constructor |
| `history` | SnapshotsController | constructor |
| `tags` | TagListController | constructor |
| `tag/create` | TagCreateController | constructor |
| `tag/:name` | TagController | section="summary" |
| `tag/:name/edit` | TagController | section="edit" |
| `tag/:name/merge` | TagController | section="merge" |
| `tag/:name/delete` | TagController | section="delete" |
| `tag-categories` | TagCategoriesController | constructor |
| `pools` | PoolListController | constructor |
| `pool/create` | PoolCreateController | constructor |
| `pool/:id` | PoolController | section="summary" |
| `pool/:id/edit` | PoolController | section="edit" |
| `pool/:id/merge` | PoolController | section="merge" |
| `pool/:id/delete` | PoolController | section="delete" |
| `pool-categories` | PoolCategoriesController | constructor |
| `custom-fields` | CustomFieldListController | constructor |
| `custom-field/create` | CustomFieldCreateController | constructor |
| `custom-field/:id` | CustomFieldController | section="summary" |
| `custom-field/:id/edit` | CustomFieldController | section="edit" |
| `custom-field/:id/delete` | CustomFieldController | section="delete" |
| `users` | UserListController | constructor |
| `user/:name` | UserController | section="summary" |
| `user/:name/edit` | UserController | section="edit" |
| `user/:name/list-tokens` | UserController | section="list-tokens" |
| `user/:name/delete` | UserController | section="delete" |
| (fallback, null) | NotFoundController | constructor |

### TopNavigationController

Not a route controller. A singleton initialized at module load time. Manages the top navigation bar visibility based on user privileges, handles login/logout state changes, and activates highlighted items.

## Entity Detail Controllers

Four controllers share an identical pattern for viewing, editing, merging, and deleting single entities:

| Controller | Entity | Fetch Dependencies | View |
|---|---|---|---|
| TagController | Tag | Tag + TagCategoryList | TagView |
| PoolController | Pool | Pool + PoolCategoryList | PoolView |
| CustomFieldController | CustomField | CustomField only | CustomFieldView |
| UserController | User | User + UserToken list | UserView |

These controllers all:
1. Check the view privilege.
2. Fetch the entity by ID/name.
3. Store the original identifier for URL update after rename.
4. Listen to `change` events on the model for URL replacement.
5. Expose `showSuccess()` and `showError()` methods.

### UserController Special Case

Uses a self/any infix pattern for privilege names:
- Own profile: `user_edit_self_name`, `user_delete_self`, etc.
- Other profile: `user_edit_any_name`, `user_delete_any`, etc.

Builds a filtered rank map that excludes ranks above the current user's rank.

## Create Controllers

Three create controllers share an identical pattern:

| Controller | Endpoint | Models |
|---|---|---|
| TagCreateController | `POST /api/tags` | TagCategoryList |
| PoolCreateController | `POST /api/pool` | PoolCategoryList |
| CustomFieldCreateController | `POST /api/custom-fields` | (none) |

## List Controllers

Six list controllers use `PageController` for pagination:

| Controller | Default Limit | Search Method | Cache |
|---|---|---|---|
| PostListController | settings.postsPerPage | PostList.search() | No, API search |
| TagListController | 50 | TagList.search() | No, API search |
| PoolListController | 50 | PoolList.search() | No, API search |
| UserListController | 30 | UserList.search() | No, API search |
| CommentsController | 10 | PostList.search() | No, API search |
| SnapshotsController | 25 | SnapshotList.search() | No, API search |
| CustomFieldListController | 50 | CustomFieldList.get() | Yes, client-side |

The `CustomFieldListController` is unique. It fetches all custom fields once, caches them, and performs filtering and sorting on the client side. It supports filter tokens (`name:`, `color:`, negation with `-`), sort tokens (`sort:name`, `sort:creation-time`), and free-text search.

## Post Controllers

### PostMainController

The most complex controller. Manages:
- Post viewing and editing (tags, safety, description, source, relations, content, thumbnail, launch date, media created date, custom fields, flags)
- Post scoring and favoriting
- Post featuring and deletion
- Comment creation, editing, scoring, deletion
- Previous/next post navigation with preloading
- Query parameter cleanup from URL to history state
- Fit mode persistence via settings

### PostDetailController

Handles the post merge section only. Allows selecting a target post and merging.

### PostUploadController

Manages the upload page:
- Sequential upload processing via `Array.reduce`
- Reverse image search for duplicate detection (with `Post.reverseSearch()`)
- Abortable uploads via `CancellablePromise`
- Skip-duplicates and similar-post confirmation
- Dual behavior for files and URLs

### PostListController

Manages the post listing page with bulk operations:
- Bulk tag add/remove
- Bulk safety change
- Bulk deletion with confirmation dialog
- Safety filtering via `api.safetyEnabled()`

## Authentication Controllers

| Controller | Purpose |
|---|---|
| LoginController | Login form, stores credentials, refreshes category color maps |
| LogoutController | Clears session, routes to home |
| PasswordResetController | Sends reset email |
| PasswordResetFinishController | Completes reset, auto-logs in |
| UserRegistrationController | Self-registration or admin-creation |

`UserRegistrationController` has dual behavior:
- Anonymous users self-register, get auto-logged in, and route to home.
- Logged-in admins create accounts for others and stay logged in.

## Error Handling Pattern

Every controller follows the same error flow:
1. Controller checks `api.hasPrivilege()`. On failure, creates `EmptyView` and returns.
2. API fetch errors show on the view with the error message.
3. Form submit errors show on the form and re-enable the form.
4. Some non-critical errors use `window.alert()` (score actions, bulk operations).

## All API Endpoints Used

| HTTP Method | Endpoint | Used By |
|---|---|---|
| GET | `/api/info` | HomeController, TopNavigationController |
| GET | `/api/post/{id}` | PostMainController, PostDetailController |
| GET | `/api/post/{id}/around` | PostMainController |
| GET | `/api/posts` | PostListController, CommentsController |
| POST | `/api/posts/reverse-search` | PostUploadController |
| POST | `/api/posts` | PostUploadController |
| PUT | `/api/post/{id}` | PostMainController |
| DELETE | `/api/post/{id}` | PostMainController, PostListController |
| POST | `/api/post/{id}/favorite` | PostMainController |
| DELETE | `/api/post/{id}/favorite` | PostMainController |
| PUT | `/api/post/{id}/score` | PostMainController |
| POST | `/api/featured-post` | PostMainController |
| POST | `/api/post-merge` | PostDetailController |
| GET | `/api/tag/{name}` | TagController, TagInputControl |
| GET | `/api/tags` | TagListController |
| POST | `/api/tags` | TagCreateController |
| PUT | `/api/tag/{name}` | TagController |
| DELETE | `/api/tag/{name}` | TagController |
| POST | `/api/tag-merge` | TagController |
| GET | `/api/tag-categories` | TagCategoriesController, TagController, tags module |
| PUT | `/api/tag-category/{name}` | TagCategoriesController |
| POST | `/api/tag-categories` | TagCategoriesController |
| DELETE | `/api/tag-category/{name}` | TagCategoriesController |
| PUT | `/api/tag-category/{name}/default` | TagCategoriesController |
| GET | `/api/tag-siblings/{name}` | TagInputControl |
| GET | `/api/pool/{id}` | PoolController |
| GET | `/api/pools` | PoolListController |
| POST | `/api/pool` | PoolCreateController |
| PUT | `/api/pool/{id}` | PoolController, PoolListController |
| DELETE | `/api/pool/{id}` | PoolController |
| POST | `/api/pool-merge` | PoolController |
| GET | `/api/pool-categories` | PoolCategoriesController, PoolController, pools module |
| PUT | `/api/pool-category/{name}` | PoolCategoriesController |
| POST | `/api/pool-categories` | PoolCategoriesController |
| DELETE | `/api/pool-category/{name}` | PoolCategoriesController |
| PUT | `/api/pool-category/{name}/default` | PoolCategoriesController |
| GET | `/api/custom-field/{id}` | CustomFieldController |
| GET | `/api/custom-fields` | CustomFieldListController |
| POST | `/api/custom-fields` | CustomFieldCreateController |
| PUT | `/api/custom-field/{id}` | CustomFieldController |
| DELETE | `/api/custom-field/{id}` | CustomFieldController |
| GET | `/api/user/{name}` | UserController, Api.login/relogin |
| GET | `/api/users` | UserListController |
| POST | `/api/users` | UserRegistrationController |
| PUT | `/api/user/{name}` | UserController |
| DELETE | `/api/user/{name}` | UserController |
| GET | `/api/user-tokens/{name}` | UserController |
| POST | `/api/user-token/{name}` | UserController, Api.login |
| PUT | `/api/user-token/{name}/{token}` | UserController |
| DELETE | `/api/user-token/{name}/{token}` | UserController, Api.logout |
| GET | `/api/comments` | CommentsController |
| POST | `/api/comments` | PostMainController, CommentsController |
| PUT | `/api/comment/{id}` | PostMainController, CommentsController |
| DELETE | `/api/comment/{id}` | PostMainController, CommentsController |
| PUT | `/api/comment/{id}/score` | PostMainController, CommentsController |
| GET | `/api/snapshots` | SnapshotsController |
| GET | `/api/password-reset/{name}` | PasswordResetController |
| POST | `/api/password-reset/{name}` | PasswordResetFinishController |
| POST | `/api/uploads` | Api._upload |
