---
title: Post Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, post]
sources: [post.js]
---

# Post Model

The `Post` class is the core model representing a single post (image, animation, document, etc.). It extends `EventTarget` and provides CRUD, scoring, favoriting, merge, reverse-search, and feature operations. It uses an `_orig` snapshot for diff-based save.

## Constructor

Calls `super()`. Initializes `_orig` as an empty object. For both `this` and `_orig`, initializes:
- `_tags` as `TagList`
- `_notes` as `NoteList`
- `_comments` as `CommentList`
- `_pools` as `PoolList`
- `_customFields` as empty object
- `_customFieldColors` as empty object

Calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter              | Return Type | Description                                            |
|---------------------|-------------|--------------------------------------------------------|
| `id`                | any         | Post ID.                                               |
| `type`              | String      | Post type (image, animation, document, etc.).          |
| `mimeType`          | String      | MIME type of the content.                              |
| `checksum`          | String      | Content checksum.                                      |
| `checksumMD5`       | String      | MD5 checksum.                                          |
| `creationTime`      | String      | ISO creation timestamp.                                |
| `user`              | String      | Uploader username.                                     |
| `description`       | String      | Post description text.                                 |
| `safety`            | String      | Safety rating (safe, sketchy, unsafe).                 |
| `contentUrl`        | String      | URL for the post content.                              |
| `fullContentUrl`    | String      | Absolute URL for the post content (resolved via `<base>` tag). |
| `thumbnailUrl`      | String      | URL for the thumbnail.                                 |
| `source`            | String      | Source URL(s), newline-separated.                      |
| `launchDate`        | String      | Launch date of the content.                            |
| `mediaCreatedDate`  | String      | Original media creation date.                          |
| `sourceSplit`       | Array       | Source split by newline, filtered for non-empty strings.|
| `canvasWidth`       | Number      | Canvas width: 850 for documents, `_canvasWidth` or 800.|
| `canvasHeight`      | Number      | Canvas height: 1100 for documents, `_canvasHeight` or 450.|
| `fileSize`          | Number      | File size in bytes (default 0).                        |
| `newContent`        | throws      | Always throws "Invalid operation".                     |
| `newThumbnail`      | throws      | Always throws "Invalid operation".                     |
| `flags`             | Array       | Post flags.                                            |
| `tags`              | TagList     | List of tags on this post.                             |
| `tagNames`          | Array       | First name of each tag.                                |
| `notes`             | NoteList    | List of annotation notes.                              |
| `comments`          | CommentList | List of comments.                                      |
| `relations`         | Array       | Related post IDs.                                      |
| `pools`             | PoolList    | List of pools containing this post.                    |
| `score`             | Number      | Total score.                                           |
| `commentCount`      | Number      | Number of comments.                                    |
| `favoriteCount`     | Number      | Number of favorites.                                   |
| `ownFavorite`       | Boolean     | Whether current user favorited this post.              |
| `ownScore`          | Number      | Current user's score on this post.                     |
| `hasCustomThumbnail`| Boolean     | Whether the post has a custom thumbnail.               |
| `customFields`      | Object      | Custom field key-value pairs.                          |
| `customFieldColors` | Object      | Custom field display colors keyed by name.             |

## Properties (Setters)

| Setter             | Type   | Description                                     |
|--------------------|--------|-------------------------------------------------|
| `flags`            | Array  | Sets flags.                                     |
| `description`      | String | Sets description.                               |
| `safety`           | String | Sets safety rating.                             |
| `relations`        | Array  | Sets relation IDs.                              |
| `newContent`       | File   | Sets new content file for upload replacment.    |
| `newThumbnail`     | File   | Sets new thumbnail file.                        |
| `source`           | String | Sets source URL(s).                             |
| `launchDate`       | String | Sets launch date.                               |
| `mediaCreatedDate` | String | Sets media creation date.                       |
| `customFields`     | Object | Sets custom fields.                             |
| `customFieldColors`| Object | Sets custom field colors.                       |

## Static Methods

### `fromResponse(response)`
Creates a `Post` and populates it from an API response.

### `reverseSearch(content)`
Sends `POST /api/posts/reverse-search` with a content file. Parses the response, converting `exactPost` and `similarPosts[*].post` into `Post` instances. Returns a promise with an `abort()` method.

### `get(id, options)`
Fetches a post by ID via `GET /api/post/<id>`. Returns a promise resolving to a `Post`.

## Method: `save(anonymous)`

Builds a diff between current state and `_orig`. Only sends changed fields:

- `description`, `safety`, `source`, `launchDate`, `mediaCreatedDate` — direct comparison.
- `flags`, `tags`, `relations`, `notes` — compared via `misc.arraysDiffer()`.
- Tags serialize as `tag.names[0]`.
- Notes serialize as `{ polygon: [[x, y], ...], text: ... }`.
- Files: `content` from `_newContent`, `thumbnail` from `_newThumbnail`.
- `customFields` via `misc.objectsDiffer()`.
- If `anonymous` is true, adds `anonymous: true`.

**If `_id` exists**: `PUT /api/post/<id>`.
**If `_id` is falsy**: `POST /api/posts`.

After the API call, if pools changed, calls `_savePoolPosts()`. On success, updates from response, dispatches `"change"`, and conditionally dispatches `"changeContent"` or `"changeThumbnail"`.

On `PostAlreadyUploadedError`, rewrites the error message to include the duplicat post ID.

### Method: `_savePoolPosts()`

Computes the set difference between current pools and `_orig` pools. For each added pool, fetches the full pool, adds this post's ID if missing, and saves. For each removed pool, fetches the full pool, removes this post's ID if present, and saves.

### Method: `feature()`

Sends `POST /api/featured-post` with `{ id: this._id }` to feature this post.

### Method: `delete()`

Sends `DELETE /api/post/<id>` with `version`. On success, dispatches a `"delete"` event.

### Method: `merge(targetId, useOldContent)`

1. Fetches the target post via `Post.get(targetId)`.
2. Sends `POST /api/post-merge` with `removeVersion`, `remove`, `mergeToVersion`, `mergeTo`, and `replaceContent`.
3. On success, updates from response and dispatches `"change"`.

### Method: `setScore(score)`

Sends `PUT /api/post/<id>/score` with `{ score }`. On success, updates from response. Dispatches `"changeFavorite"` if `ownFavorite` changed, then dispatches `"changeScore"`.

### Method: `addToFavorites()`

Sends `POST /api/post/<id>/favorite`. On success, updates from response. Dispatches `"changeScore"` if `ownScore` changed, then dispatches `"changeFavorite"`.

### Method: `removeFromFavorites()`

Sends `DELETE /api/post/<id>/favorite`. On success, updates from response. Dispatches `"changeScore"` if `ownScore` changed, then dispatches `"changeFavorite"`.

### Method: `mutateContentUrl()`

Appends a random cache-busting query parameter to `_contentUrl`. Useful for forcing a content reload.

### Method: `_updateFromResponse(response)`

Maps API fields to internal properties. Returns a `map()` function that produces:

| API Field                | Internal Property     |
|--------------------------|-----------------------|
| `response.version`       | `_version`            |
| `response.id`            | `_id`                 |
| `response.type`          | `_type`               |
| `response.mimeType`      | `_mimeType`           |
| `response.checksum`      | `_checksum`           |
| `response.checksumMD5`   | `_checksumMD5`        |
| `response.creationTime`  | `_creationTime`       |
| `response.user`          | `_user`               |
| `response.description`   | `_description`        |
| `response.safety`        | `_safety`             |
| `response.contentUrl`    | `_contentUrl`         |
| resolved full URL        | `_fullContentUrl`     |
| `response.thumbnailUrl`  | `_thumbnailUrl`       |
| `response.source`        | `_source`             |
| `response.launchDate`    | `_launchDate`         |
| `response.mediaCreatedDate`| `_mediaCreatedDate` |
| `response.canvasWidth`   | `_canvasWidth`        |
| `response.canvasHeight`  | `_canvasHeight`       |
| `response.fileSize`      | `_fileSize`           |
| `response.flags`         | `_flags` (copied array)|
| `response.relations`     | `_relations` (copied array)|
| `response.score`         | `_score`              |
| `response.commentCount`  | `_commentCount`       |
| `response.favoriteCount` | `_favoriteCount`      |
| `response.ownScore`      | `_ownScore`           |
| `response.ownFavorite`   | `_ownFavorite`        |
| `response.hasCustomThumbnail`| `_hasCustomThumbnail`|

For both `this` and `_orig`:
- `_tags.sync(response.tags)`
- `_notes.sync(response.notes)`
- `_comments.sync(response.comments)`
- `_pools.sync(response.pools)`
- `_customFields` and `_customFieldColors` populated from `response.customFields` array.

## Events Emitted

| Event             | Detail              | When                                         |
|-------------------|---------------------|----------------------------------------------|
| `change`          | `{ post: this }`    | After save or merge.                         |
| `changeContent`   | `{ post: this }`    | When new content was uploaded during save.   |
| `changeThumbnail` | `{ post: this }`    | When new thumbnail was uploaded during save. |
| `delete`          | `{ post: this }`    | After deletion.                              |
| `changeScore`     | `{ post: this }`    | After score change or favorite toggle.       |
| `changeFavorite`  | `{ post: this }`    | When favorite status changes.                |
