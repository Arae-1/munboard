---
title: CustomFieldList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, custom-field, list]
sources: [custom_field_list.js]
---

# CustomFieldList Model

The `CustomFieldList` class extends `AbstractList`. It adds a static get method for fetching custom fields.

## Static Properties

- `_itemClass` = `CustomField`
- `_itemName` = `"custom_field"`

## Static Method: `get(fields)`

Fetches custom fields from `GET /api/custom-fields` with a `fields` query parameter (joined by comma). Returns a promise resolving to the response object where `results` is a `CustomFieldList` instance.
