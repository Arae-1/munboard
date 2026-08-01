---
title: User Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, user]
sources: [user.js]
---

# User Model

The `User` class represents a user account. It extends `EventTarget` and supports CRUD, diff-based save, avatar upload, and rank management.

## Constructor

Calls `super()`. Initializes `_orig` as an empty object. Calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter              | Return Type | Description                                      |
|---------------------|-------------|--------------------------------------------------|
| `name`              | String      | Username.                                        |
| `rank`              | String      | Rank key (anonymous, regular, moderator, etc.).  |
| `email`             | String      | Email address.                                   |
| `avatarStyle`       | String      | Avatar display style.                            |
| `avatarUrl`         | String      | Avatar image URL.                                |
| `creationTime`      | String      | ISO creation timestamp.                          |
| `lastLoginTime`     | String      | ISO last login timestamp.                        |
| `commentCount`      | Number      | Total comments by this user.                     |
| `favoritePostCount` | Number      | Number of favorited posts.                       |
| `uploadedPostCount` | Number      | Number of uploaded posts.                        |
| `likedPostCount`    | Number      | Number of liked posts.                           |
| `dislikedPostCount` | Number      | Number of disliked posts.                        |
| `rankName`          | String      | Human-readable rank name from `api.rankNames`.   |
| `avatarContent`     | throws      | Always throws "Invalid operation".               |
| `password`          | throws      | Always throws "Invalid operation".               |

## Properties (Setters)

| Setter          | Type   | Description                                    |
|-----------------|--------|------------------------------------------------|
| `name`          | String | Sets username.                                 |
| `rank`          | String | Sets rank key.                                 |
| `email`         | String | Sets email (null for empty).                   |
| `avatarStyle`   | String | Sets avatar style.                             |
| `avatarContent` | File   | Sets the avatar file for upload.               |
| `password`      | String | Sets a new password.                           |

## Static Methods

### `fromResponse(response)`
Creates a `User` and populates it from an API response.

### `get(name)`
Fetches a user by name via `GET /api/user/<name>`. Returns a promise resolving to a `User` instance.

## Method: `save()`

Builds a diff between current state and `_orig`. Only sends changed fields:
- `name`, `email`, `rank`, `avatarStyle` — direct comparison.
- `avatarContent` — if set, includes `avatarStyle` and the avatar file in `files`.
- `password` — if set, includes in detail.

**If `_orig._name` exists**: `PUT /api/user/<origName>`.
**If `_orig._name` is falsy**: `POST /api/users` (registration).

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete()`

Sends `DELETE /api/user/<orig._name>` with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field                   | Internal Property        |
|-----------------------------|--------------------------|
| `response.version`          | `_version`               |
| `response.name`             | `_name`                  |
| `response.rank`             | `_rank`                  |
| `response.email`            | `_email`                 |
| `response.avatarStyle`      | `_avatarStyle`           |
| `response.avatarUrl`        | `_avatarUrl`             |
| `response.creationTime`     | `_creationTime`          |
| `response.lastLoginTime`    | `_lastLoginTime`         |
| `response.commentCount`     | `_commentCount`          |
| `response.favoritePostCount`| `_favoritePostCount`     |
| `response.uploadedPostCount`| `_uploadedPostCount`     |
| `response.likedPostCount`   | `_likedPostCount`        |
| `response.dislikedPostCount`| `_dislikedPostCount`     |

After mapping, copies values to both `this` and `_orig`. Clears `_password` and `_avatarContent`.

## Events Emitted

| Event    | Detail             |
|----------|--------------------|
| `change` | `{ user: this }`   |
| `delete` | `{ user: this }`   |
