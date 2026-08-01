# Client Models

Models represent the data layer of the application. They wrap API data, provide property accessors, emit events on state changes, and handle API CRUD operations directly. All models extend `EventTarget` from the events module.

## Model Hierarchy

```
EventTarget (events module)
  ├── AbstractList ──────────── base for all list types
  │     ├── CommentList
  │     ├── CustomFieldList
  │     ├── NoteList
  │     ├── PointList
  │     ├── PoolList
  │     ├── PoolCategoryList
  │     ├── PostList
  │     ├── SnapshotList
  │     ├── TagList
  │     ├── TagCategoryList
  │     └── UserList
  ├── Comment
  ├── CustomField
  ├── Note
  ├── Point
  ├── Pool
  ├── PoolCategory
  ├── Post
  ├── Snapshot
  ├── Tag
  ├── TagCategory
  ├── User
  ├── UserToken
  ├── Settings (singleton)
  └── TopNavigation (singleton)
```

## AbstractList Base Class

`AbstractList` extends `EventTarget` and maintains an internal `_list` array. It provides the foundational pattern for all list-type models.

### Subclass Requirements

Each subclass must define two static properties:
- `_itemClass` — the constructor for individual items (e.g., `Comment`).
- `_itemName` — a string identifier used in event detail objects (e.g., `"comment"`).

### Core Methods

| Method | Description |
|---|---|
| `static fromResponse(response)` | Creates a list from an API response array. Instantiates each item via `_itemClass.fromResponse()`. Registers `delete` and `change` listeners on each item. |
| `sync(plainList)` | Clears and repopulates the list from a plain array. Used to synchronize embedded lists from model responses. |
| `add(item)` | Adds an item. Registers listeners. Dispatches `"add"` event. |
| `remove(item)` | Removes an item by reference identity. Dispatches `"remove"` event. |
| `clear()` | Removes all items. |
| `length` (getter) | Returns number of items. |
| `at(index)` | Returns item at index. |
| `map(...)` | Delegates to `_list.map()`. |
| `filter(...)` | Delegates to `_list.filter()`. |
| `[Symbol.iterator]()` | Makes the list iterable with `for...of`. |

### Events

| Event | Detail Key | When |
|---|---|---|
| `add` | `[this._itemName]` | After add() |
| `remove` | `[this._itemName]` | After remove() |
| `change` | forwarded from item | When a contained item dispatches change |

## Data Flow

```
API Response (JSON)
  → Model.static fromResponse(response)
    → Constructor
    → _updateFromResponse(response)
      → Map API fields to internal _properties
      → Copy to _orig for diff tracking (entity models only)
      → Sync embedded lists via .sync()
  → View reads getters
  → Control/user modifies via setters
  → model.save()
    → Compute diff between current state and _orig
    → Send only changed fields to API
    → On success: _updateFromResponse(), dispatch "change" event
    → View re-renders in response to "change" event
```

### Diff-Based Save Pattern

Entity models (Post, Tag, Pool, User, CustomField, UserToken, TagCategory, PoolCategory) use a diff-based save. Each model maintains an `_orig` snapshot. Before saving, the model compares current values against `_orig` and sends only changed fields. This minimizes request size and avoids privilege violations (sending fields the user cannot modify).

List models (extending AbstractList) do not have a `save()` method. They receive data from API responses and mutate items in place.

## Model Reference

### Post
**File:** `post.js` | **Extends:** `EventTarget`

The core model. Represents an image, animation, or document.

**Key properties:** `id`, `type`, `mimeType`, `checksum`, `checksumMD5`, `creationTime`, `user`, `description`, `safety`, `contentUrl`, `fullContentUrl`, `thumbnailUrl`, `source`, `sourceSplit`, `launchDate`, `mediaCreatedDate`, `canvasWidth`, `canvasHeight`, `fileSize`, `flags`, `tags` (TagList), `tagNames`, `notes` (NoteList), `comments` (CommentList), `relations`, `pools` (PoolList), `score`, `commentCount`, `favoriteCount`, `ownFavorite`, `ownScore`, `hasCustomThumbnail`, `customFields`, `customFieldColors`

**Key methods:** `save(anonymous)`, `delete()`, `setScore(score)`, `addToFavorites()`, `removeFromFavorites()`, `feature()`, `merge(targetId, useOldContent)`, `mutateContentUrl()`, `_savePoolPosts()`

**Static methods:** `fromResponse(response)`, `get(id, options)`, `reverseSearch(content)`

**Events:** `change`, `delete`, `changeContent`, `changeThumbnail`, `changeScore`, `changeFavorite`

---

### PostList
**File:** `post_list.js` | **Extends:** `AbstractList` | **Item:** `Post`

**Key methods:** `hasPostId(id)`, `addById(id)`, `removeById(id)`

**Static methods:** `search(text, offset, limit, fields)`, `getAround(id, query, options)`, `_decorateSearchQuery(text)`

The `_decorateSearchQuery` method applies safety filtering by prepending `-rating:safe,sketchy,...` clauses based on browsing settings.

---

### Tag
**File:** `tag.js` | **Extends:** `EventTarget`

**Key properties:** `names`, `category`, `description`, `suggestions` (TagList), `implications` (TagList), `postCount`, `creationTime`, `lastEditTime`

**Key methods:** `save()`, `delete()`, `merge(targetName, addAlias)`

**Static methods:** `fromResponse(response)`, `get(name)`

**Events:** `change`, `delete`

---

### TagList
**File:** `tag_list.js` | **Extends:** `AbstractList` | **Item:** `Tag`

**Key methods:** `isTaggedWith(testName)`, `addByName(tagName, addImplications)`, `removeByName(testName)`

**Static methods:** `search(text, offset, limit, fields, options)`

`addByName` has recursive implication resolution. If tag A implies B and B implies C, adding A adds B and then C automatically.

---

### TagCategory
**File:** `tag_category.js` | **Extends:** `EventTarget`

**Key properties:** `name`, `color`, `order`, `tagCount`, `isDefault`, `isTransient`

**Key methods:** `save()`, `delete()`

**Static methods:** `fromResponse(response)`

**Events:** `change`, `delete`

---

### TagCategoryList
**File:** `tag_category_list.js` | **Extends:** `AbstractList` | **Item:** `TagCategory`

**Key properties:** `defaultCategory`

**Key methods:** `save()` (batch save with delete tracking)

**Static methods:** `fromResponse(response)`, `get()`

Tracks deleted categories in `_deletedCategories`. On `save()`, collects save, delete, and default-update promises, then runs `Promise.all()`.

---

### Pool
**File:** `pool.js` | **Extends:** `EventTarget`

**Key properties:** `id`, `names`, `searchableName`, `category`, `description`, `posts` (PostList), `postCount`, `creationTime`, `lastEditTime`, `launchDate`

**Key methods:** `save()`, `delete()`, `merge(targetId, addAlias)`

**Static methods:** `fromResponse(response)`, `get(id)`

**Events:** `change`, `delete`

---

### PoolList
**File:** `pool_list.js` | **Extends:** `AbstractList` | **Item:** `Pool`

**Key methods:** `hasPoolId(poolId)`, `removeById(poolId)`

**Static methods:** `search(text, offset, limit, fields, options)`

---

### PoolCategory
**File:** `pool_category.js` | **Extends:** `EventTarget`

**Key properties:** `name`, `color`, `poolCount`, `isDefault`, `isTransient`

**Key methods:** `save()`, `delete()`

**Static methods:** `fromResponse(response)`

**Events:** `change`, `delete`

---

### PoolCategoryList
**File:** `pool_category_list.js` | **Extends:** `AbstractList` | **Item:** `PoolCategory`

**Key properties:** `defaultCategory`

**Key methods:** `save()` (batch save with delete tracking)

**Static methods:** `fromResponse(response)`, `get()`

Mirrors `TagCategoryList` in structure. Tracks deleted categories and the default category for batch `save()`.

---

### Comment
**File:** `comment.js` | **Extends:** `EventTarget`

**Key properties:** `id`, `postId`, `text`, `user`, `creationTime`, `lastEditTime`, `score`, `ownScore`

**Key methods:** `save()`, `delete()`, `setScore(score)`

**Static methods:** `fromResponse(response)`, `create(postId)`

**Events:** `change`, `delete`, `changeScore`

---

### CommentList
**File:** `comment_list.js` | **Extends:** `AbstractList` | **Item:** `Comment`

No additional methods beyond AbstractList.

---

### User
**File:** `user.js` | **Extends:** `EventTarget`

**Key properties:** `name`, `rank`, `email`, `avatarStyle`, `avatarUrl`, `creationTime`, `lastLoginTime`, `commentCount`, `favoritePostCount`, `uploadedPostCount`, `likedPostCount`, `dislikedPostCount`, `rankName`

**Key methods:** `save()`, `delete()`

**Static methods:** `fromResponse(response)`, `get(name)`

Setters for `password` and `avatarContent` throw errors. They are write-only and routed to the API during `save()`.

**Events:** `change`, `delete`

---

### UserList
**File:** `user_list.js` | **Extends:** `AbstractList` | **Item:** `User`

**Static methods:** `search(text, offset, limit)`

---

### UserToken
**File:** `user_token.js` | **Extends:** `EventTarget`

**Key properties:** `token`, `note`, `enabled`, `version`, `expirationTime`, `creationTime`, `lastEditTime`, `lastUsageTime`

**Key methods:** `save(userName)`, `delete(userName)`

**Static methods:** `fromResponse(response)`, `get(userName)`, `create(userName, note, expirationTime)`

`fromResponse` handles both single-token and list responses.

**Events:** `change`, `delete`

---

### CustomField
**File:** `custom_field.js` | **Extends:** `EventTarget`

**Key properties:** `id`, `name`, `description`, `color`, `creationTime`, `lastEditTime`

**Key methods:** `save()`, `delete()`

**Static methods:** `fromResponse(response)`, `get(id)`

**Events:** `change`, `delete`

---

### CustomFieldList
**File:** `custom_field_list.js` | **Extends:** `AbstractList` | **Item:** `CustomField`

**Static methods:** `get(fields)` — fetches all custom fields with optional field selection.

---

### Note
**File:** `note.js` | **Extends:** `EventTarget`

**Key properties:** `text`, `polygon` (PointList)

No `save()` or `delete()` methods. Notes are serialized as part of `Post.save()`.

**Static methods:** `fromResponse(response)`

---

### NoteList
**File:** `note_list.js` | **Extends:** `AbstractList` | **Item:** `Note`

No additional methods beyond AbstractList.

---

### Point
**File:** `point.js` | **Extends:** `EventTarget`

**Key properties:** `x`, `y` (both with getters/setters)

Setting `x` or `y` dispatches a `"change"` event. Used as polygon vertices for note annotations.

**Events:** `change`

---

### PointList
**File:** `point_list.js` | **Extends:** `AbstractList` | **Item:** `Point`

**Additional getters:** `firstPoint`, `secondLastPoint`, `lastPoint`

Used by the notes overlay for polygon drawing state management.

---

### Snapshot
**File:** `snapshot.js` | **Extends:** `EventTarget`

**Key properties:** `operation`, `type`, `id`, `user`, `data`, `time`

Read-only model. No `save()` or `delete()` methods.

**Static methods:** `fromResponse(response)`

---

### SnapshotList
**File:** `snapshot_list.js` | **Extends:** `AbstractList` | **Item:** `Snapshot`

**Static methods:** `search(text, offset, limit)`

---

### Settings
**File:** `settings.js` | **Extends:** `EventTarget` | **Singleton**

**Key properties (via `get()`):**

| Key | Default | Description |
|---|---|---|
| `listPosts` | `{ safe: true, sketchy: true, unsafe: false }` | Safety filters for post listing |
| `upscaleSmallPosts` | `false` | Enlarge images beyond native size |
| `endlessScroll` | `false` | Use endless scroll instead of page buttons |
| `keyboardShortcuts` | `true` | Enable global keyboard shortcuts |
| `transparencyGrid` | `true` | Show checkerboard behind transparent images |
| `fitMode` | `"fit-both"` | Default image fit mode |
| `tagSuggestions` | `true` | Show tag sibling suggestions |
| `autoplayVideos` | `false` | Auto-play video posts |
| `postsPerPage` | `42` | Number of posts per page |
| `tagUnderscoresAsSpaces` | `false` | Display underscores as spaces in tag names |
| `darkTheme` | `true` | Enable dark theme |
| `postFlow` | `false` | Flow layout for post listing |
| `accentColor` | `"#BC3736"` | UI accent color |

**Key methods:** `get()`, `save(newSettings, silent)`

`save()` persists to `localStorage`, toggles `darktheme` class on `document.body`, applies accent color, and dispatches `"change"` unless silent.

**Events:** `change`

---

### TopNavigation
**File:** `top_navigation.js` | **Extends:** `EventTarget` | **Singleton**

14 navigation items defined by the `_makeTopNavigation()` factory:

| Key | Access Key | Title | URL |
|---|---|---|---|
| `home` | H | Home | `""` |
| `posts` | P | Posts | `"posts"` |
| `upload` | U | Upload | `"upload"` |
| `comments` | C | Comments | `"comments"` |
| `tags` | T | Tags | `"tags"` |
| `pools` | O | Pools | `"pools"` |
| `custom-fields` | F | Custom Fields | `"custom-fields"` |
| `users` | S | Users | `"users"` |
| `account` | A | Account | `"user/{me}"` |
| `register` | R | Register | `"register"` |
| `login` | L | Log in | `"login"` |
| `logout` | O | Logout | `"logout"` |
| `help` | E | Help | `"help"` |
| `settings` | null | Cog icon | `"settings"` |

**Key methods:** `getAll()`, `get(key)`, `add(key, item)`, `activate(key)`, `setTitle(title)`, `showAll()`, `show(key)`, `hide(key)`

**Events:** `activate`

The `TopNavigationItem` class holds: `accessKey`, `title`, `url`, `available`, `imageUrl`, `key`.
