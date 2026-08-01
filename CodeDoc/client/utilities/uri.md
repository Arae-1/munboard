---
title: URI Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, uri, links, routing]
sources: [uri.js]
---

# URI Utility

The `uri.js` file provides functions for building API URLs, client-side URLs, and parsing/manipulating URIs.

## Function: `formatApiLink(...values)`

Builds an API URL path from a list of values. Each value is URL-encoded.

- If a value is an `Object`, it is treated as query parameters. Keys with truthy values are appended as `?key=encodedValue&...`. This object must be the last argument.
- All values are joined with `/`.

Example: `formatApiLink("post", 123, "score")` -> `"/post/123/score"`.

## Function: `escapeParam(text)`

Returns `encodeURIComponent(text)`.

## Function: `unescapeParam(text)`

Returns `decodeURIComponent(text)`.

## Function: `formatClientLink(...values)`

Builds a client-side URL path. Same structure as `formatApiLink()` but:
- Query parameters are joined with `;` instead of `&`.
- Uses `escapeParam()` for encoding.

## Function: `extractHostname(url)`

Extracts the hostname from a URL string. Handles URLs with and without protocol.

## Function: `extractRootDomain(url)`

Extracts the root domain from a URL. Handles subdomains and country-code TLDs (e.g., `example.co.uk` returns `example.co.uk`).

## Function: `escapeTagName(text)`

Escapes special characters in tag names for safe use in search queries. Escapes `:` and `.` with backslashes.

## Exports

```js
{
    formatClientLink, formatApiLink,
    escapeTagName, escapeParam, unescapeParam,
    extractHostname, extractRootDomain,
}
```
