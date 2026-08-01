# TagAutoCompleteControl

## UI Element

Extends `AutoCompleteControl`. Provides autocomplete suggestions for tag names. Renders tag names with category color, post count, and negation support inside the parent dropdown.

## Helper Function

### `_tagListToMatches(text, tags, options, negated)`
Takes search text and a tag list. Sorts tags by usage count descending. For each tag:
1. Computes `matchingNames` via `misc.matchingNames`.
2. Derives a CSS class from `misc.makeCssName(tag.category, "tag")`.
3. Adds `" disabled"` class if the tag is already tagged (via `options.isTaggedWith`).
4. If negated, prepends `"-"` to all names and matching names.
5. The caption is a `<span>` with the category CSS class, the first matching name, and the post count in parentheses.

Returns an array of `{caption, value}` where `value` is the full tag object.

## Constructor

### `constructor(input, options)`
- `input`: The `<input>` element.
- `options`: Extended `AutoCompleteControl` options with:
  - `isTaggedWith`: function(tagName) returning true if the tag is already present. Defaults to `() => false`.
  - `isNegationAllowed`: whether the `-` prefix is treated as a negation search. Defaults to `false`.

Overrides `options.getMatches` to:
1. Detect if the text starts with `-` and `isNegationAllowed` is true. If negated, strips the leading `-` and sets `negated = true`.
2. If text is empty after stripping, returns a promise that resolves to null.
3. Determines search mode: `*term*` (substring) if text is `>= minLengthForPartialSearch` (3), or if negation is not allowed and the first character is `-`. Otherwise, uses `term*` (prefix).
4. Appends ` sort:usages` to the query.
5. Calls `TagList.search(query, 0, maxResults, ["names", "category", "usages"], {noProgress: true})`.
6. Maps results through `_tagListToMatches`.

Calls the parent constructor.

## Override

### `_getActiveSuggestion()`
Overrides the parent method. Returns the tag object (not a string). Calls `misc.matchingNames` to find the best matching tag name for the current text. Returns `null` if no result is active.

## Communication

- **Queries**: `TagList.search()` on every autocomplete request. Uses fields: `names`, `category`, `usages`. Passes `{noProgress: true}`.

## Autocomplete Logic

- For text shorter than 3 characters (and no negation): prefix search (`term*`).
- For text 3 characters or longer (or negation): substring search (`*term*`).
- Results are sorted by tag usage count descending.
- Tags already in the current tag list show with a `disabled` CSS class.
- When `isNegationAllowed` is true and the text starts with `-`, the search operates on the text after the `-`, and returned names are prefixed with `-`.

## State Management

Inherits all state from `AutoCompleteControl`.

## Keyboard Shortcuts

Inherits all keyboard shortcuts from `AutoCompleteControl`.
