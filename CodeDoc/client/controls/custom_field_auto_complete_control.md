# CustomFieldAutoCompleteControl

## UI Element

Extends `AutoCompleteControl`. Provides autocomplete suggestions for custom field names. Renders matching custom field names with highlighted search terms inside the parent dropdown.

## Constructor

### `constructor(input, options)`
- `input`: The `<input>` element.
- `options`: Standard `AutoCompleteControl` options.

Overrides `options.getMatches` to:
1. Query `CustomFieldList.get(["id", "name", "description", "color"])` from the API.
2. Filter results where the custom field name includes the typed text (case-insensitive).
3. Limit to `maxResults`.
4. Map each match to `{caption, value}`. The caption is escaped HTML with the matching substring wrapped in `<b>` tags.
5. The `value` is the full custom field object.

Calls the parent constructor with the modified options.

## Module-Level Functions

### `_loadFieldNames()`
Caches custom field names in `_fieldNamesCache` (a `Set`). Calls `CustomFieldList.get(["name"])` once. Returns the cached promise if a request is in flight.

### `isValidFieldName(name)`
Returns true if the given name (case-insensitive) exists in the loaded field names cache. Returns false if the cache is null.

### `ensureFieldNamesLoaded()`
Returns a promise that resolves when `_loadFieldNames` completes. Used to pre-warm the cache, for example by `CustomFieldInputControl` before validation.

## Communication

- **Queries**: `CustomFieldList.get()` to search custom fields on every autocomplete request.
- **Module exports**: `isValidFieldName` and `ensureFieldNamesLoaded` are exposed for use by `CustomFieldInputControl`.

## Autocomplete Logic

The `getMatches` function filters by substring match. Search term matching uses a case-insensitive regex with HTML escaping. Matched text is wrapped in `<b>` tags for visual highlighting.

## State Management

Module-level cache:
- `_fieldNamesCache`: A `Set` of lowercased custom field names. Populated once and reused.
- `_fieldNamesPromise`: Stores the in-flight promise to prevent duplicate requests.

## Keyboard Shortcuts

Inherits all keyboard shortcuts from `AutoCompleteControl`.
