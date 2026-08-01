---
title: UserList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, user, list]
sources: [user_list.js]
---

# UserList Model

The `UserList` class extends `AbstractList`. It adds search functionality for users.

## Static Properties

- `_itemClass` = `User`
- `_itemName` = `"user"`

## Static Method: `search(text, offset, limit)`

Searches users via `GET /api/users` with parameters `query`, `offset`, and `limit`. Returns a promise resolving to the response object where `results` is a `UserList` instance.
