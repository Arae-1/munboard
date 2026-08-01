---
title: CustomField Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, custom-field]
sources: [custom_field.js]
---

# CustomField Model

The `CustomField` class represents a custom metadata field that can be attached to posts. It extends `EventTarget` and supports CRUD operations.

## Constructor

Calls `super()`. Initializes `_orig` as an empty object. Calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter         | Return Type | Description                           |
|----------------|-------------|---------------------------------------|
| `id`           | any         | Custom field ID.                      |
| `name`         | String      | Field name.                           |
| `description`  | String      | Field description.                    |
| `color`        | String      | Display color (hex).                  |
| `creationTime` | String      | ISO creation timestamp.               |
| `lastEditTime` | String      | ISO last-edit timestamp.              |

## Properties (Setters)

| Setter        | Type   | Description                    |
|---------------|--------|--------------------------------|
| `name`        | String | Sets the field name.           |
| `description` | String | Sets the field description.    |
| `color`       | String | Sets the display color.        |

## Static Methods

### `fromResponse(response)`
Creates a `CustomField` and populates it from an API response.

### `get(id)`
Fetches a single custom field by ID via `GET /api/custom-field/<id>`. Returns a promise resolving to a `CustomField`.

## Method: `save()`

Builds a diff. Sends only changed fields (`name`, `description`, `color`).

**If `_id` exists**: `PUT /api/custom-field/<id>`.
**If `_id` is falsy**: `POST /api/custom-fields`.

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete()`

Sends `DELETE /api/custom-field/<id>` with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field               | Internal Property |
|-------------------------|-------------------|
| `response.id`           | `_id`             |
| `response.version`      | `_version`        |
| `response.name`         | `_name`           |
| `response.description`  | `_description`    |
| `response.color`        | `_color`          |
| `response.creationTime` | `_creationTime`   |
| `response.lastEditTime` | `_lastEditTime`   |

Copies values to both `this` and `_orig`.

## Events Emitted

| Event    | Detail                      |
|----------|-----------------------------|
| `change` | `{ customField: this }`     |
| `delete` | `{ customField: this }`     |
