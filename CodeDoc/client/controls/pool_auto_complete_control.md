# PoolAutoCompleteControl

## UI Element

Extends `AutoCompleteControl`. Provides autocomplete suggestions for pool names. Renders pool names with their category color and post count inside the parent dropdown.

## Helper Function

### `_poolListToMatches(text, pools, options)`
Takes search text and a pool list. Sorts pools by post count descending. Maps each pool to a `{caption, value}` object. The caption is a `<span>` with:
- A CSS class derived from the pool category via `misc.makeCssName(category, "pool")`.
- The best-matching name (from `misc.matchingNames`) followed by the post count in parentheses.
- HTML-escaping applied.

## Constructor

### `constructor(input, options)`
- `input`: The `<input>` element.
- `options`: Standard `AutoCompleteControl` options.

Sets `minLengthForPartialSearch` to 3.

Overrides `options.getMatches` to:
1. Determine the search query mode: `*term*` (substring) if text is 3 or more characters; otherwise `term*` (prefix).
2. Append ` sort:post-count` to the query.
3. Call `PoolList.search(query, 0, maxResults, ["id", "names", "category", "postCount", "version"], {noProgress: true})`.
4. Map results through `_poolListToMatches`.

Calls the parent constructor.

## Override

### `_getActiveSuggestion()`
Overrides the parent method. Returns the pool object (not a string). Calls `misc.matchingNames` to find the best matching pool name for the current text. Returns `null` if no result is active.

## Communication

- **Queries**: `PoolList.search()` on every autocomplete request. Uses fields: `id`, `names`, `category`, `postCount`, `version`. Passes `{noProgress: true}` to suppress the loading indicator.

## Autocomplete Logic

- For text shorter than 3 characters: prefix search (`term*`).
- For text 3 characters or longer: substring search (`*term*`).
- Results are sorted by post count descending.
- The active suggestion returns a pool model object.

## State Management

Inherits all state from `AutoCompleteControl`.

## Keyboard Shortcuts

Inherits all keyboard shortcuts from `AutoCompleteControl`.
