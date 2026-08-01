---
title: Settings Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, settings, preferences]
sources: [settings.js]
---

# Settings Model

The `Settings` class manages user preferences stored in `localStorage`. It extends `EventTarget`. The file exports a singleton instance.

## Default Settings

The `defaultSettings` constant defines:

| Key                     | Type    | Default Value                          |
|-------------------------|---------|----------------------------------------|
| `listPosts`             | Object  | `{ safe: true, sketchy: true, unsafe: false }` |
| `upscaleSmallPosts`     | Boolean | `false`                                |
| `endlessScroll`         | Boolean | `false`                                |
| `keyboardShortcuts`     | Boolean | `true`                                 |
| `transparencyGrid`      | Boolean | `true`                                 |
| `fitMode`               | String  | `"fit-both"`                           |
| `tagSuggestions`        | Boolean | `true`                                 |
| `autoplayVideos`        | Boolean | `false`                                |
| `postsPerPage`          | Number  | `42`                                   |
| `tagUnderscoresAsSpaces`| Boolean | `false`                                |
| `darkTheme`             | Boolean | `true`                                 |
| `postFlow`              | Boolean | `false`                                |
| `accentColor`           | String  | `"#BC3736"`                            |

## Constructor

Calls `super()`. Loads settings from `localStorage` via `_getFromLocalStorage()`. Applies the accent color via `accent.apply()`.

## Method: `_getFromLocalStorage()`

- Starts with a copy of `defaultSettings`.
- Attempts to parse `localStorage.getItem("settings")` as JSON and merge it over the defaults.
- If parsing fails, silently continues with defaults.

Returns the merged settings object.

## Method: `save(newSettings, silent)`

- Merges `newSettings` into `this.cache`.
- Stores the merged settings as JSON in `localStorage` under the key `"settings"`.
- Toggles the `"darktheme"` class on `document.body` based on `darkTheme`.
- Applies the accent color via `accent.apply()`.
- Reloads settings from localStorage into `this.cache`.
- Dispatches a `"change"` event unless `silent` is `true`.

## Method: `get()`

Returns `this.cache` (the current settings object).

## Events Emitted

| Event    | Detail                          | When                       |
|----------|---------------------------------|----------------------------|
| `change` | `{ settings: this.cache }`      | After `save()` (non-silent).|

## Exports

The file exports `new Settings()`, a singleton instance.
