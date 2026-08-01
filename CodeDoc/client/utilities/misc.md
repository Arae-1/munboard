---
title: Misc Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, misc, helpers]
sources: [misc.js]
---

# Misc Utility

The `misc.js` file provides general-purpose utility functions used throughout the client application.

## Function: `decamelize(str, sep)`

Converts a camelCase string to a lowercase-separated string. Uses `sep` as the separator (defaults to `"-"`).

- Input: `"myVariableName"` -> `"my-variable-name"`.
- Handles consecutive uppercase letters: `"XMLParser"` -> `"xml-parser"`.

## Function: `dateToRfc3339(dateStr)`

Converts a date-only string (`YYYY-MM-DD`) from an `<input type="date">` to an RFC 3339 UTC datetime string.

- If `dateStr` is falsy, returns `null`.
- Returns `"${dateStr}T00:00:00Z"`.

## Generator Function: `range(start = 0, end = null, step = 1)`

Yields numbers from `start` to `end` (exclusive) in steps of `step`. If `end` is `null`, `start` is treated as `end` and iteration starts from 0.

## Function: `_formatUnits(number, base, suffixes, callback)`

Internal function for formatting numbers with unit suffixes.

- Multiplies `number` by 1.0 to coerce to number.
- Divides by `base` while `number >= base` and suffixes remain, shifting suffixes each division.
- Calls `callback(number, suffix)` or a default callback.

## Function: `formatFileSize(fileSize)`

Formats a byte count into a human-readable string using binary suffixes (B, K, M, G). Uses `_formatUnits()` with base 1024. Shows one decimal place for values under 20 KB/MB/GB (unless the unit is bytes).

## Function: `formatRelativeTime(timeString)`

Formats an ISO time string into a relative time description.

- Returns `"never"` if the input is falsy.
- Compares `Date.parse(timeString)` with `Date.now()`.
- If the time is in the future, prepends `"in "`.
- If the time is in the past, appends `" ago"`.
- Special cases: `"yesterday"` and `"tomorrow"` replace `"a day ago"` and `"in a day"`.

## Function: `formatMarkdown(text)`

Delegates to `markdown.formatMarkdown(text)`.

## Function: `formatInlineMarkdown(text)`

Delegates to `markdown.formatInlineMarkdown(text)`.

## Function: `splitByWhitespace(str)`

Splits a string by whitespace and removes empty strings.

## Function: `unindent(callSite, ...args)`

Removes consistent leading whitespace from a multi-line string. Useful for tagged template literals.

- If `callSite` is a string, processes it directly.
- If `callSite` is a function, returns a function that processes the result.
- If `callSite` is an array (template literal), processes the concatenated result.

## Function: `enableExitConfirmation()`

Sets `window.onbeforeunload` to show a browser-native confirmation dialog when navigating away.

## Function: `disableExitConfirmation()`

Sets `window.onbeforeunload` to `null`.

## Function: `confirmPageExit()`

- If `window.onbeforeunload` is not set, returns `true`.
- If it is set, calls `window.confirm()` with the exit message. On confirm, disables the exit confirmation and returns `true`. Otherwise returns `undefined`.

## Function: `makeCssName(text, suffix)`

Generates a CSS-safe class name: `"<suffix>-<sanitized_text>"`. Replaces non-alphanumeric characters in `text` with underscores.

## Function: `arraysDiffer(source1, source2, orderImportant)`

Compares two arrays for differences.

- If `orderImportant` is `true`: compared element-by-element at same indices.
- If `orderImportant` is `false` (default): compares contents without order. Returns `true` if any element in one array is not in the other.

## Function: `objectsDiffer(source1, source2)`

Compares two plain objects by sorted key lists and values. Returns `true` if they differ.

## Function: `escapeSearchTerm(text)`

Escapes search term characters for safe use in search queries. Escapes `:` after word characters and `.` by backslash.

## Function: `dataURItoBlob(dataURI)`

Converts a data URI string to a `Blob` object.

- Splits the URI at the comma.
- Decodes base64 or URL-encoded content.
- Extracts the MIME type.
- Constructs a `Uint8Array` from the byte characters.
- Returns a `Blob` with the extracted MIME type.

## Function: `getPrettyName(tag)`

If the setting `tagUnderscoresAsSpaces` is enabled, replaces underscores with spaces. Otherwise, returns the tag unchanged.

## Function: `wildcardMatch(pattern, str, sensitive = false)`

Matches a string against a wildcard pattern. `*` in the pattern matches any sequence of characters. Case-insensitive by default. Escapes regex special characters in the pattern before converting `*` to `.*`.

## Function: `matchingNames(text, names)`

Finds names matching a text query in an array of names.

- First tries prefix match (`text*` wildcard).
- If no match and `text.length >= 3`, tries substring match (`*text*`).
- Returns the matched names, or all names if nothing matched.

## Function: `preloadPostImages(post)`

If the post type is `"image"` or `"animation"`, creates an `Image` object with `fetchPriority = "low"` and sets its `src` to the post's `contentUrl`. This preloads the image into the browser cache.

## Exports

```js
{
    range, formatRelativeTime, formatFileSize,
    formatMarkdown, formatInlineMarkdown,
    unindent, enableExitConfirmation, disableExitConfirmation,
    confirmPageExit, escapeHtml, makeCssName, splitByWhitespace,
    arraysDiffer, objectsDiffer, decamelize, dateToRfc3339,
    escapeSearchTerm, dataURItoBlob, getPrettyName,
    wildcardMatch, matchingNames, preloadPostImages,
}
```
