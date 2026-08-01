---
title: Common Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, common]
---

# Common Views

## TopNavigationView (`js/views/top_navigation_view.js`)

**Template**: `top-navigation`

**Purpose**: Renders the top navigation bar with site name, logo, and menu items.

**Constructor**: No ctx. Host node is `#top-navigation-holder`.

**Render method** (`render(ctx)`):
- `ctx.items` — array of navigation items with `key`, `url`, `title`, `accessKey`, `imageUrl`, `available`
- `ctx.name` — site name
- Binds mobile navigation toggle behavior

**Key methods**:
- `activate(key)` — highlights the active navigation item by `data-name`
- `_bindMobileNavigationEvents()` — toggle click opens/closes menu on mobile; link clicks close menu

---

## EndlessPageView (`js/views/endless_page_view.js`)

**Templates**: `endless-pager`, `endless-pager-page`

**Purpose**: Infinite scroll page container that loads pages as the user scrolls.

**Constructor** (`ctx`): Renders the holder template into `#content-holder`.

**Run method** (`run(ctx)`):
- Sets up a timer (250ms interval) to probe page loads
- `_probePageLoad(ctx)` — checks if top or bottom page guard is visible, loads adjacent pages
- `_loadPage(ctx, offset, limit, append)` — fetches a page and renders it
- `_syncUrl(ctx)` — updates browser URL as user scrolls through pages
- `_renderPage(ctx, append, response)` — creates page node with page template, calls `ctx.pageRenderer`

**Properties**:
- `pageHeaderHolderNode` — for header views (e.g., posts header)
- Capped guards at top and bottom of the page list

---

## ManualPageView (`js/views/manual_page_view.js`)

**Templates**: `manual-pager`, `manual-pager-nav`

**Purpose**: Paged navigation with page number links.

**Constructor** (`ctx`): Renders the holder template into `#content-holder`.

**Run method** (`run(ctx)`):
- Fetches the requested page via `ctx.requestPage(offset, limit)`
- Renders content via `ctx.pageRenderer`
- Generates navigation with visible page numbers (current ± 2, first few, last few, with ellipsis gaps)
- Binds keyboard `a`/`left` for previous, `d`/`right` for next
- Handles removal events to adjust pagination

**Helper functions**:
- `_getVisiblePageNumbers(currentPage, totalPages)` — calculates which page numbers to show
- `_getPages(currentPage, pageNumbers, limit, defaultLimit, removedItems)` — creates page map with offset/limit/active/ellipsis entries

**Properties**:
- `pageHeaderHolderNode` — for header views

---

## EmptyView (`js/views/empty_view.js`)

**Purpose**: Renders an empty wrapper used as a placeholder when permissions prevent a real view from loading.

**Constructor**: Renders `<div class="wrapper"><div class="messages"></div></div>` into `#content-holder`.

**Key methods**:
- `showError(message)` — shows an error in the messages area

---

## NotFoundView (`js/views/not_found_view.js`)

**Template**: `not-found`

**Purpose**: 404 page showing "Not found" with the invalid path and a link back to the main page.

**Constructor** (`path`):
- Renders the template with `{path: path}`

---

## LoginView (`js/views/login_view.js`)

**Template**: `login`

**Purpose**: Login form with user name, password, and "Remember me" checkbox.

**Constructor**: Renders template with `userNamePattern` and `passwordPattern` from the API. Decorates form validator.

**Events dispatched**:
- `submit` — with `{name, password, remember}`

---

## RegistrationView (`js/views/registration_view.js`)

**Template**: `user-registration`

**Purpose**: User registration form with name, password, email fields plus anti-autocomplete hidden fields.

**Constructor**: Renders template with `userNamePattern`, `passwordPattern`. Decorates validator.

**Events dispatched**:
- `submit` — with `{name, password, email}`

---

## PasswordResetView (`js/views/password_reset_view.js`)

**Template**: `password-reset`

**Purpose**: Password reset request form. If mail is disabled, shows contact email instead.

**Constructor**: Renders template with `canSendMails`, `contactEmail`. Decorates validator.

**Events dispatched**:
- `submit` — with `{userNameOrEmail}`
