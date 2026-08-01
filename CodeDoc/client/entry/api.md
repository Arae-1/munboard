---
title: API Client
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, api, http, auth]
sources: [api.js]
---

# API Client

The `api.js` file defines the `Api` class. This class wraps HTTP requests to the server API. It handles authentication, file uploads, caching, progress indication, and request abortion. The file exports a singleton `Api` instance.

## Module-Level Variables

| Variable              | Description                                          |
|-----------------------|------------------------------------------------------|
| `fileTokens`          | Cache of file upload tokens keyed by file identifier.|
| `remoteConfig`        | The server configuration fetched at startup.         |
| `remoteConfigPromise` | The in-flight promise for `fetchConfig()`.           |
| `postCount`           | Total number of posts on the server.                 |
| `diskUsage`           | Total disk usage in bytes.                           |
| `serverTime`          | Current server time string.                          |
| `featuredPost`        | The current featured post object.                    |
| `featuringTime`       | When the post was featured.                          |
| `featuringUser`       | The user who featured the post.                      |

## Class: Api

Extends `events.EventTarget`.

### Constructor

- Initializes `user`, `userName`, `userPassword`, `token` to `null`.
- Initializes `cache` as an empty object (for GET request caching).
- Initializes `allRanks` as an array of rank strings: `["anonymous", "restricted", "regular", "power", "moderator", "administrator", "nobody"]`.
- Initializes `rankNames` as a Map from rank key to human-readable name.

### Method: `get(url, options)`

- If `url` exists in `this.cache`, returns a resolved promise with the cached response.
- Otherwise, calls `_wrappedRequest()` with `request.get`. After success, caches the response and resolves.

### Method: `post(url, data, files, options)`

Clears the request cache. Calls `_wrappedRequest()` with `request.post`.

### Method: `put(url, data, files, options)`

Clears the request cache. Calls `_wrappedRequest()` with `request.put`.

### Method: `delete(url, data, options)`

Clears the request cache. Calls `_wrappedRequest()` with `request.delete`.

### Method: `fetchConfig()`

- If `remoteConfig` is already fetched, returns a resolved promise (no-op).
- If a fetch is in progress, returns the existing `remoteConfigPromise`.
- Otherwise, calls `this.get(uri.formatApiLink("info"))`. This constructs the URL `/api/info`. On success, populates `remoteConfig`, `postCount`, `diskUsage`, `serverTime`, `featuredPost`, `featuringTime`, `featuringUser`.

### Getters for Server Info

- `getPostCount()` — returns `postCount`.
- `getDiskUsage()` — returns `diskUsage`.
- `getServerTime()` — returns `serverTime`.
- `getFeaturedPost()` — returns `featuredPost`.
- `getFeaturingTime()` — returns `featuringTime`.
- `getFeaturingUser()` — returns `featuringUser`.
- `getName()` — returns `remoteConfig.name`.
- `getTagNameRegex()` — returns `remoteConfig.tagNameRegex`.
- `getPoolNameRegex()` — returns `remoteConfig.poolNameRegex`.
- `getPasswordRegex()` — returns `remoteConfig.passwordRegex`.
- `getUserNameRegex()` — returns `remoteConfig.userNameRegex`.
- `getContactEmail()` — returns `remoteConfig.contactEmail`.
- `getSecureCookie()` — returns `true` if `remoteConfig.secureCookie` is truthy.
- `canSendMails()` — returns `true` if `remoteConfig.canSendMails` is truthy.
- `safetyEnabled()` — returns `true` if `remoteConfig.enableSafety` is truthy.
- `getCustomFieldDefaultColor()` — returns `remoteConfig.customFieldDefaultColor` or `"#888888"`.

### Method: `hasPrivilege(lookup)`

Takes a privilege name prefix (e.g., `"post_"`). Searches `remoteConfig.privileges` for keys starting with the prefix. Finds the minimum rank index required for any matching privilege. Compares the current user's rank index with the minimum required. Returns `true` if the user has sufficient rank. Throws an error if no matching privilege is found.

### Method: `loginFromCookies()`

Reads the `auth` cookie as JSON. If it contains `user` and `token`, calls `loginWithToken()`. Otherwise, resolves immediately (no-op).

### Method: `loginWithToken(userName, token, doRemember)`

- Stores `userName` and `token`.
- Calls `this.get("/user/" + userName + "?bump-login=true")`.
- On success, sets the `auth` cookie (with optional 365-day expiry, Secure flag, SameSite=Strict), stores the user, dispatches a `"login"` event.
- On failure, rejects and calls `logout()`.

### Method: `createToken(userName, options)`

Creates a new user token via `POST /user-token/<userName>`. Sets expiry based on `options.expires`. On success, stores the cookie, userName, and token.

### Method: `deleteToken(userName, userToken)`

Calls `DELETE /user-token/<userName>/<userToken>`. On success, sets the auth cookie with a null token.

### Method: `login(userName, userPassword, doRemember)`

- Stores `userName` and `userPassword`.
- Calls `this.get("/user/" + userName + "?bump-login=true")`.
- On success, calls `createToken()` and dispatches a `"login"` event.
- On failure, rejects and calls `logout()`.

### Method: `logout()`

Calls `deleteToken()`. On both success and failure, calls `_logout()`.

### Method: `_logout()`

Resets `user`, `userName`, `userPassword`, `token` to `null`. Dispatches a `"logout"` event.

### Method: `forget()`

Removes the `auth` cookie.

### Method: `isLoggedIn(user)`

- If a `user` argument is given, checks if `this.userName` matches `user.name` (case-insensitive).
- Otherwise, checks if `this.userName` is not null.

### Method: `isCurrentAuthToken(userToken)`

Returns `true` if `userToken.token` equals `this.token`.

### Method: `_getFullUrl(url)`

Prepends `"api/"` to the URL. Normalizes double slashes. Splits the URL into the base path and a query string fragment. Returns `[baseUrl, query]`.

### Method: `_getFileId(file)`

If `file` is a string, returns it as-is. Otherwise, returns `file.name + file.size` as a unique identifier.

### Method: `_wrappedRequest(url, requestFactory, data, files, options)`

The core request pipeline:

1. Copies `data`.
2. For each file in `files`:
   - If the file token is cached, inserts `key + "Token"` into data.
   - Otherwise, calls `_upload()` to upload the file, gets a token, and inserts it.
3. Calls `_rawRequest()` with the final data.
4. If the response error is `MissingOrExpiredRequiredFileError`, clears the cached file tokens and sets a user-friendly error message.

Returns a promise with an `abort()` method that calls the current abort function.

### Method: `_upload(file, options)`

Uploads a single file via `POST /api/uploads` with the file in the `content` field. Returns a promise that resolves with the file token. Has an `abort()` method.

### Method: `_rawRequest(url, requestFactory, data, files, options)`

Makes the actual HTTP request:

1. Builds the full URL via `_getFullUrl()`.
2. Creates the request object using `requestFactory`.
3. Sets the `Accept: application/json` header.
4. If a query string fragment exists, adds it via `req.query()`.
5. Attaches files from the `files` parameter. String values are sent as URL references (`key + "Url"`). Blob/File values are attached directly.
6. If `data` is present:
   - With files: attaches metadata as a JSON blob.
   - Without files: sets `Content-Type: application/json` and sends JSON.
7. Handles authentication:
   - If `userName` and `token` exist, sets the `Authorization` header with a base64-encoded `userName:token`.
   - If `userName` and `userPassword` exist, uses HTTP basic auth with URL-decoded password.
8. Starts the progress bar (unless `options.noProgress` is set).
9. Defines the `abort` function to abort the request and stop the progress bar.
10. On response:
    - Stops the progress bar.
    - On error: constructs an error from `response.body.description` or `"Unknown error"`, attaches the response body, and rejects.
    - On success: resolves with `response.body`.

## Exports

The file exports `new Api()`, a singleton instance.
