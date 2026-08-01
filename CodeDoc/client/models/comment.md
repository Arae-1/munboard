---
title: Comment Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, comment]
sources: [comment.js]
---

# Comment Model

The `Comment` class represents a single comment on a post. It extends `EventTarget` and provides CRUD operations via the API.

## Constructor

Calls `super()` and `_updateFromResponse({})` to initialize all fields with defaults.

## Static Method: `create(postId)`

Creates a new unsaved `Comment` instance for the given `postId`. Sets `_postId` and returns the comment.

## Static Method: `fromResponse(response)`

Creates a `Comment` and populates it from an API response object. Returns the comment.

## Properties (Getters)

| Getter           | Return Type | Description                               |
|------------------|-------------|-------------------------------------------|
| `id`             | any         | The comment ID.                           |
| `postId`         | any         | The ID of the parent post.                |
| `text`           | String      | The comment text, or `""`.                |
| `user`           | String      | The username of the comment author.       |
| `creationTime`   | String      | ISO timestamp of creation.                |
| `lastEditTime`   | String      | ISO timestamp of last edit.               |
| `score`          | Number      | Total score of the comment.               |
| `ownScore`       | Number      | The current user's score on this comment. |

## Properties (Setters)

| Setter  | Type   | Description              |
|---------|--------|--------------------------|
| `text`  | String | Sets the comment text.   |

## Method: `save()`

Creates or updates the comment via the API.

- **If `this._id` exists**: sends `PUT /api/comment/<id>` with `version` and `text`.
- **If `this._id` is falsy**: sends `POST /api/comments` with `postId`, `version`, and `text`.

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete()`

Sends `DELETE /api/comment/<id>` with the `version` field. On success, dispatches a `"delete"` event.

## Method: `setScore(score)`

Sends `PUT /api/comment/<id>/score` with the `score` value. On success, calls `_updateFromResponse()` and dispatches a `"changeScore"` event.

## Method: `_updateFromResponse(response)`

Maps the API response fields to internal properties:

| API Field         | Internal Property |
|-------------------|-------------------|
| `response.version`| `_version`        |
| `response.id`     | `_id`             |
| `response.postId` | `_postId`         |
| `response.text`   | `_text`           |
| `response.user`   | `_user`           |
| `response.creationTime` | `_creationTime` |
| `response.lastEditTime` | `_lastEditTime` |
| `response.score`  | `_score` (parsed as int) |
| `response.ownScore`| `_ownScore` (parsed as int) |

## Events Emitted

| Event          | Detail                    |
|----------------|---------------------------|
| `change`       | `{ comment: this }`       |
| `delete`       | `{ comment: this }`       |
| `changeScore`  | `{ comment: this }`       |
