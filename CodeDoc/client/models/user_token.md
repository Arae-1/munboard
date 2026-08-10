---
title: UserToken Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, user, token, auth]
sources: [user_token.js]
---

# UserToken Model

The `UserToken` class represents an authentication token assigned to a user. It extends `EventTarget` and supports CRUD operations via the API.

## Constructor

Calls `super()`. Initializes `_orig` as an empty object. Calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter           | Return Type | Description                                |
|------------------|-------------|--------------------------------------------|
| `id`             | String      | The internal token id.                     |
| `token`          | String      | The token string.                          |
| `note`           | String      | Optional note describing the token.        |
| `enabled`        | Boolean     | Whether the token is active.               |
| `version`        | any         | Token version for optimistic locking.      |
| `expirationTime` | String      | ISO timestamp of expiration.               |
| `creationTime`   | String      | ISO creation timestamp.                    |
| `lastEditTime`   | String      | ISO last-edit timestamp.                   |
| `lastUsageTime`  | String      | ISO last usage timestamp.                  |

## Properties (Setters)

| Setter | Type   | Description                         |
|--------|--------|-------------------------------------|
| `note` | String | Sets the token's note description.  |
| `enabled` | Boolean | Sets whether the token is active. |

## Static Methods

### `fromResponse(response)`

If `response.results` is defined (list response), iterates through `response.results`, creates a `UserToken` for each, and returns an array of tokens.

If `response.results` is not defined (single token response), creates a single `UserToken` and returns it.

### `get(userName)`

Fetches all tokens for a user via `GET /api/user-tokens/<userName>`. Returns a promise resolving to the result of `UserToken.fromResponse()`.

### `create(userName, note, expirationTime)`

Creates a new token via `POST /api/user-token/<userName>` with `enabled: true`, optional `note`, and optional `expirationTime`. Returns a promise resolving to a `UserToken` instance.

## Method: `save(userName)`

Builds a diff with `version`. Adds `note` when it differs from the original note. Adds `enabled` when it differs from the original enabled state. Sends `PUT /api/user-token/<userName>/<origId>`, where `<origId>` is the original internal token id. On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete(userName)`

Sends `DELETE /api/user-token/<userName>/<origId>`, where `<origId>` is the original internal token id, with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field                  | Internal Property    |
|----------------------------|----------------------|
| `response.id`              | `_id`                |
| `response.token`           | `_token`             |
| `response.note`            | `_note`              |
| `response.enabled`         | `_enabled`           |
| `response.expirationTime`  | `_expirationTime`    |
| `response.version`         | `_version`           |
| `response.creationTime`    | `_creationTime`      |
| `response.lastEditTime`    | `_lastEditTime`      |
| `response.lastUsageTime`   | `_lastUsageTime`     |

Copies values to both `this` and `_orig`. Public-only responses omit `id` and `token`. For these responses, the method deletes `_id` and `_token` from the map so the current values stay unchanged.

## Events Emitted

| Event    | Detail                 |
|----------|------------------------|
| `change` | `{ userToken: this }`  |
| `delete` | `{ userToken: this }`  |
