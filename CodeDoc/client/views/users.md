---
title: User Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, users]
---

# User Views

## UsersPageView (`js/views/users_page_view.js`)

**Template**: `users-page`

**Purpose**: Renders a flexbox list of user cards with avatar, name, registration time, last login time.

**Render method**: Simple `views.replaceContent` with the template.

---

## UsersHeaderView (`js/views/users_header_view.js`)

**Template**: `users-header`

**Purpose**: Header with search form.

**Events dispatched**:
- `navigate` — on form submit with `{query, page: 1}`

---

## UserView (`js/views/user_view.js`)

**Template**: `user`

**Purpose**: Top-level user detail view with tabbed sections (summary, edit, list-tokens, delete).

**Constructor** (`ctx`):
- `ctx.user` — User model
- `ctx.section` — `'summary'`, `'edit'`, `'list-tokens'`, or `'delete'`

**Render method**: Renders template with tab navigation. Creates inner view:
- `summary` → UserSummaryView
- `edit` → UserEditView
- `list-tokens` → UserTokensView
- `delete` → UserDeleteView

Listens to user `change` event.

---

## UserSummaryView (`js/views/user_summary_view.js`)

**Template**: `user-summary`

**Purpose**: Displays user avatar, registration time, last login time, rank, and quick links (uploads, favorites, comments, liked/disliked posts).

---

## UserEditView (`js/views/user_edit_view.js`)

**Template**: `user-edit`

**Purpose**: Edit form for a user.

**Form fields**: User name (with regex pattern), Password (with regex, optional), Email, Rank (select), Avatar (Gravatar/Manual radio + FileDropperControl for manual upload).

**Events dispatched**:
- `submit` — with `{user, name, email, rank, avatarStyle, password, avatarContent}`
- `change` — on any form input change

---

## UserTokensView (`js/views/user_tokens_view.js`)

**Template**: `user-tokens`

**Purpose**: Lists user authentication tokens and provides token creation, deletion, note editing, and enabled-state toggling.

**Constructor** (`ctx`):
- `ctx.user` — User model
- `ctx.tokens` — array of token objects

**Key methods**:
- `_tokenById(id)` — finds a token in `_tokens` by its internal id
- `_decorateTokenToggleLinks()` — binds each `(toggle)` link; clicking dispatches `update` with `enabled: !userToken.enabled`

**Events dispatched**:
- `submit` / `create-token` — on create token form (note, expirationTime)
- `delete` / `delete-token` — on token delete (userToken)
- `update` / `update-token` — on token note change (userToken, note) or enabled-state toggle (userToken, enabled)

---

## UserDeleteView (`js/views/user_delete_view.js`)

**Template**: `user-delete`

**Purpose**: Delete confirmation form for a user account.

**Events dispatched**:
- `submit` — with `{user}`
