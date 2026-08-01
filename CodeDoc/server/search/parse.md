# parse.rs — Query String Parsing

This file parses raw search condition strings into typed `Condition` and `StrCondition` values. It handles escape sequences, wildcards, ranges, comma-separated lists, and time values.

## `Pattern` Trait

`parse.rs:9` — A trait for character matching. Like the standard library's `Pattern` trait (not yet stabilized).

### Implementors

| Implementor | `matches(self, c)` Behavior |
|-------------|---------------------------|
| `char` | `self == c` |
| `IsWhitespace` | `c.is_whitespace()` |

## `split_unescaped_whitespace(text) -> impl Iterator<Item = &str>`

`parse.rs:33` — Splits a string by whitespace characters. Whitespace that has a `\` before it is not treated as a separator. Empty terms are filtered out.

Example: `"hello world\\ here"` → `["hello", "world\\ here"]`

## `split_once<P: Pattern>(text, pattern) -> Option<(&str, &str)>`

`parse.rs:38` — Splits a string into two parts at the first unescaped match of the pattern. Returns `(left, right_without_pattern)`.

Example: `split_once("a:b:c", ':')` → `Some(("a", "b:c"))`

This is the core function for parsing `key:value` pairs.

## `str_condition(condition) -> StrCondition`

`parse.rs:45` — Parses a string-based condition.

**Logic:**
1. If the condition contains `*` → Creates `StrCondition::WildCard`.
   - Escape characters are removed (`unescape`).
   - `*` is replaced with `%` (SQL LIKE wildcard).
   - `_` is replaced with `\_` (escape SQL's single-character wildcard).
   - The result is lowercased.
2. If no `*` → Creates `StrCondition::Regular` using `parse_regular_str`.

## `time_condition(condition) -> ApiResult<Condition<Range<DateTime>>>`

`parse.rs:54` — Parses a time-based condition.

**Logic:**
1. If the condition contains `..`: Same as regular condition parsing — `a..` is GreaterEq, `..b` is LessEq, `a..b` is Range. Each time value is parsed with `parse_time`.
2. Otherwise: Splits on `,` and parses each value as a time. Wraps in `Condition::Values`.

## `condition<T>(condition) -> ApiResult<Condition<T>>`

`parse.rs:71` — Parses a generic typed condition.

**Logic:**
1. If the condition contains `..`: Returns `GreaterEq`, `LessEq`, or `Range` depending on which side is empty.
2. Otherwise: Splits on `,` and parses each value. Wraps in `Condition::Values`.

## `values<T>(condition) -> ApiResult<Vec<T>>`

`parse.rs:87` — Parses comma-separated values. Each value is parsed with `FromStr`.

## `is_multivalued(condition) -> bool`

`parse.rs:101` — Returns true if the condition contains a comma, `*`, or `..`. Used to decide whether to add `DISTINCT` to a query.

## `SplitUnescaped<'a, P>` Struct

`parse.rs:108` — An iterator that splits a string by a pattern. Characters with a `\` before them are not treated as delimiters.

| Field | Type | Description |
|-------|------|-------------|
| `text` | `&'a str` | The full text being split |
| `start` | `usize` | Index of the start of the current segment |
| `end` | `usize` | Index of the end of the current segment |
| `pattern` | `P` | The pattern to split on |

The iterator advances by finding the next unescaped pattern match, yielding the substring between `start` and `end`.

## `is_unescaped(text, index) -> bool`

`parse.rs:144` — Determines if the character at `index` is escaped. Counts `\` characters before the index. If the count is odd, the character is escaped. If even, it is not.

Example:
- `\:` → `:` is escaped (1 backslash, odd)
- `\\:` → `:` is not escaped (2 backslashes, even)
- `\\\:` → `:` is escaped (3 backslashes, odd)

## `next_unescaped_split<P: Pattern>(text, pattern) -> Option<usize>`

`parse.rs:155` — Finds the index of the first unescaped character that matches the pattern. Returns `None` if no match is found.

## `range_split(text) -> Option<(&str, &str)>`

`parse.rs:162` — Like `split_once` but specifically for `..` (two dots). The two dots must be consecutive characters at the same position. Checks both that the character is `.` and the next character is `.`, and that the position is unescaped.

## `unescape(text) -> String`

`parse.rs:177` — Removes escape characters (`\`) from text. Every odd `\` is removed. Even `\` are halved.

Example:
- `"\."` → `"."`
- `"\\\\."` → `"\\."`

## `parse_regular_str(filter) -> Condition<String>`

`parse.rs:191` — Parses a non-wildcard string condition.

**Logic:**
1. If there is a `..`: Returns GreaterEq, LessEq, or Range with unescaped values.
2. Otherwise: Splits by unescaped `,` and collects unescaped values.

## `parse_time(time) -> Result<Range<DateTime>, TimeParsingError>`

`parse.rs:202` — Parses a single time value into a range of `DateTime`.

### Special Values

| Value | Start | End |
|-------|-------|-----|
| `today` | Midnight today (UTC) | Midnight tomorrow (UTC) |
| `yesterday` | Midnight yesterday (UTC) | Midnight today (UTC) |

### Date Format

The format is `YYYY-M-D` or `YYYY-MM-DD`:
- Year is mandatory.
- Month is optional.
- Day is optional.

### Range Calculation

| Input | Start | End |
|-------|-------|-----|
| `2024` | `2024-01-01 00:00:00` | `2025-01-01 00:00:00` |
| `2024-7` | `2024-07-01 00:00:00` | `2024-08-01 00:00:00` |
| `2024-7-11` | `2024-07-11 00:00:00` | `2024-07-12 00:00:00` |
| `2024-2-29` | `2024-02-29 00:00:00` | `2024-03-01 00:00:00` |

The end date is exclusive (midnight of the next unit).

## `parse_month(text) -> Result<Month, TimeParsingError>`

`parse.rs:238` — Parses a month number (1-12) into a `time::Month` enum. First parses as `u8`, then converts.

## Escape Character Summary

The `\` character escapes the next character in a search string.

| Input | Meaning |
|-------|---------|
| `\ ` | Literal space, not a term separator |
| `\,` | Literal comma, not a value separator |
| `\:` | Literal colon, not a key:value separator |
| `\.` | Literal dot (first of two dots), not a range separator |
| `\\` | Literal backslash |

### Escape Examples

| Input | Parsed As |
|-------|-----------|
| `tag\ name` | Single term "tag name" |
| `a\,b` | Single value "a,b" |
| `key\:value` | Single term (no key:value split) |
| `1\.\.5` | Single value "1..5" (not a range) |
| `name\:my\ tag\ with\,comma` | `name` = `"my tag with,comma"` |
