---
title: TopNavigation Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, navigation, top-bar]
sources: [top_navigation.js]
---

# TopNavigation Model

The `TopNavigation` module manages the top navigation bar. It defines two classes and exports a singleton navigation instance built by a factory function.

## Class: TopNavigationItem

Represents one entry in the top navigation bar.

### Constructor: `TopNavigationItem(accessKey, title, url, available, imageUrl)`

| Parameter   | Type            | Description                                      |
|-------------|-----------------|--------------------------------------------------|
| `accessKey` | String or null  | Keyboard access key character.                   |
| `title`     | String          | Display label (supports HTML like `<i>` tags).   |
| `url`       | String          | Relative client URL for the link.                |
| `available` | Boolean         | Whether the item is shown. Defaults to `true`.   |
| `imageUrl`  | String or null  | Optional image URL. Defaults to `null`.          |

Properties set on the instance: `accessKey`, `title`, `url`, `available`, `imageUrl`, `key` (set later by `TopNavigation.add()`).

## Class: TopNavigation

Extends `EventTarget`.

### Constructor

Initializes `activeItem` to `null`, `_keyToItem` as a new `Map()`, and `_items` as an empty array.

### Method: `getAll()`

Returns the `_items` array.

### Method: `get(key)`

Returns the item from `_keyToItem` by key. Throws if the key does not exist.

### Method: `add(key, item)`

Sets `item.key = key`. Throws if the key already exists. Adds the item to `_keyToItem` map and `_items` array.

### Method: `activate(key)`

Sets `activeItem` to `null`. Dispatches an `"activate"` event with `{ key, item }` where `item` is resolved from `_keyToItem` (or `null` if key is falsy).

### Method: `setTitle(title)`

Calls `api.fetchConfig()`. Sets `document.title` to the server name, optionally followed by `" \u2013 " + title`. Also resets `document.oldTitle` to `null`.

### Method: `showAll()`

Sets `available = true` on every item in `_items`.

### Method: `show(key)`

Sets `available = true` on the item with the given key.

### Method: `hide(key)`

Sets `available = false` on the item with the given key.

### Events Emitted

| Event      | Detail                        |
|------------|-------------------------------|
| `activate` | `{ key: key, item: item }`    |

## Factory Function: `_makeTopNavigation()`

Creates a `TopNavigation` instance and adds the standard navigation items:

| Key            | Access Key | Title             | URL                      |
|----------------|-----------|-------------------|--------------------------|
| `home`         | H         | Home              | `""`                     |
| `posts`        | P         | Posts             | `"posts"`                |
| `upload`       | U         | Upload            | `"upload"`               |
| `comments`     | C         | Comments          | `"comments"`             |
| `tags`         | T         | Tags              | `"tags"`                 |
| `pools`        | O         | Pools             | `"pools"`                |
| `custom-fields`| F         | Custom Fields     | `"custom-fields"`        |
| `users`        | S         | Users             | `"users"`                |
| `account`      | A         | Account           | `"user/{me}"`            |
| `register`     | R         | Register          | `"register"`             |
| `login`        | L         | Log in            | `"login"`                |
| `logout`       | O         | Logout            | `"logout"`               |
| `help`         | E         | Help              | `"help"`                 |
| `settings`     | null      | `<i class='fa fa-cog'></i>` | `"settings"`   |

## Exports

The file exports the result of `_makeTopNavigation()`, a singleton `TopNavigation` instance.
