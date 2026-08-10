---
title: User Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, users]
---

# User Templates

## `users_page.tpl`
**Used by**: UsersPageView
**Variables**: `ctx.response.results`, `ctx.canViewUsers`, `ctx.formatClientLink`, `ctx.makeThumbnail`, `ctx.makeRelativeTime`, `ctx.makeFlexboxAlign`
**Structure**: Flexbox list of user cards. Each card has a thumbnail (linked if `canViewUsers`), and details: name (linked), registration time, last login time.

---

## `users_header.tpl`
**Used by**: UsersHeaderView
**Variables**: `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeTextInput`
**Structure**: Horizontal form with search text input, submit button, syntax help link. No auto-complete control (simpler than tags/pools headers).

---

## `user.tpl`
**Used by**: UserView
**Variables**: `ctx.user.name`, `ctx.formatClientLink`, `ctx.canEditAnything`, `ctx.canListTokens`, `ctx.canDelete`
**Structure**: Content wrapper with user name heading, tab navigation (Summary, conditional Settings/Login tokens/Delete tabs), and `user-content-holder` div.

---

## `user_summary.tpl`
**Used by**: UserSummaryView
**Variables**: `ctx.user` (avatarUrl, name, creationTime, lastLoginTime, rankName, uploadedPostCount, favoritePostCount, commentCount, likedPostCount, dislikedPostCount), `ctx.isLoggedIn`, `ctx.makeThumbnail`, `ctx.makeRelativeTime`, `ctx.formatClientLink`
**Structure**: Avatar thumbnail, basic info list (registered, last seen, rank), quick links (uploads, favorites, comments), and conditional "only visible to you" section (liked/disliked posts).

---

## `user_edit.tpl`
**Used by**: UserEditView
**Variables**: `ctx.user`, `ctx.userNamePattern`, `ctx.passwordPattern`, `ctx.ranks`, `ctx.canEditName`, `ctx.canEditPassword`, `ctx.canEditEmail`, `ctx.canEditRank`, `ctx.canEditAvatar`, `ctx.makeTextInput`, `ctx.makePasswordInput`, `ctx.makeEmailInput`, `ctx.makeSelect`, `ctx.makeRadio`
**Structure**: Form with anti-autocomplete hidden fields. Conditional fields: User name (pattern-validated), Password (pattern-validated, placeholder "leave blank"), Email, Rank (select), Avatar (Gravatar/Manual radios + file dropper). Messages and save button.

---

## `user_delete.tpl`
**Used by**: UserDeleteView
**Variables**: `ctx.makeCheckbox`
**Structure**: Confirmation checkbox, messages, delete account submit button.

---

## `user_tokens.tpl`
**Used by**: UserTokensView
**Variables**: `ctx.tokens` (array), `ctx.makeRelativeTime`, `ctx.makeTextInput`, `ctx.makeDateInput`, `_` (underscore)
**Structure**: Token list using flex layout. Each token has a labels column and a values column. The values column shows:
- Internal id: first 8 characters plus an ellipsis, inside `<code class='token-id'>`; the full id is in the `title` attribute
- Token value: shown only when `token.token` is present, inside `<code class='token-value'>` with escaped interpolation (`<%-`)
- Note (with change link) — escaped with `<%-` when present
- Creation time, expiration time ("No expiration" when absent), last used time
- Status row: "Enabled" or "Disabled", with a `(toggle)` link
- Delete form (warns if it is current auth token)

The `data-token-id` attributes on the delete form, the note change link, and the toggle link carry the token's internal id, not the array index. Create-token form at bottom with Note text input and Expires date input.
