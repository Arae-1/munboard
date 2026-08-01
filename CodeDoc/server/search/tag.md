# tag.rs — Tag Search

This module provides search for tags.

## Token Enum

`tag.rs:18` — All search tokens for tags.

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `CreationTime` | `creation-date`, `creation-time` | `tag.creation_time` |
| `LastEditTime` | `edit-date`, `edit-time`, `last-edit-date`, `last-edit-time` | `tag.last_edit_time` |

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Name` | `name` | `tag_name.name` |
| `Category` | `category` | `tag_category.name` |
| `Description` | `description` | `tag.description` |

### Numeric Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `UsageCount` | `usages`, `post-count`, `usage-count` | `tag_statistics.usage_count` | `i64` |
| `ImplicationCount` | `implication-count` | `tag_statistics.implication_count` | `i64` |
| `SuggestionCount` | `suggestion-count` | `tag_statistics.suggestion_count` | `i64` |

### Relation Filters

| Token | Serialization | Description |
|-------|--------------|-------------|
| `Implies` | `implies` | Tags that imply a given tag |
| `Suggests` | `suggests` | Tags that suggest a given tag |

## `QueryBuilder` Struct

`tag.rs:41` — The tag search query builder.

### Constructor

`tag.rs:124` — The anonymous token is `Token::Name`. Bare words are treated as tag name filters.

### Anonymous User Blacklist

`tag.rs:126` — When the client is anonymous (`UserRank::Anonymous`), the user's `anonymous_preferences` are injected as additional negated filters:
- Each blacklisted tag name becomes a negated `Name` filter.
- Each blacklisted tag category becomes a negated `Category` filter.

This prevents anonymous users from seeing blacklisted tags.

### Count Optimization

`tag.rs:54` — If no filters are set, reads from `database_statistics.tag_count`.

### Base Query

`tag.rs:66` — The base query is:
```sql
SELECT tag.id
FROM tag
INNER JOIN tag_statistics ON ...
INNER JOIN tag_category ON ...
```

## `Name` Filter

`tag.rs:161` — Filters tags by name. Joins `tag_name` table. Uses the TEMP table cache system. The filter searches across all names (including all variants, not only primary names). Adds `DISTINCT` if multivalued.

## `Category` Filter

`tag.rs:80` — Filters tags by category name. Applied directly on the `tag_category` join from the base query. Does NOT use TEMP tables.

## `Description` Filter

`tag.rs:81` — Filters tags by description text. Applied directly. Does NOT use TEMP tables.

## `UsageCount` Filter

`tag.rs:82` — Filters tags by usage (post) count from tag statistics.

## `Implies` Filter

`tag.rs:174` — Filters tags that imply a given tag. Joins `tag_implication` with `tag_name` on `child_id`. The implication's `parent_id` is matched against the search condition. Uses TEMP tables.

Effect: `implies:sky` returns all tags that imply "sky".

## `Suggests` Filter

`tag.rs:190` — Filters tags that suggest a given tag. Joins `tag_suggestion` with `tag_name` on `child_id`. The suggestion's `parent_id` is matched against the search condition. Uses TEMP tables.

Effect: `suggests:sky` returns all tags that suggest "sky".

## Sorting

`tag.rs:91` — Default sort is `CreationTime` descending.

### Primary Name Requirement

`tag.rs:102` — Before sorting, the query filters to only primary tag names:
```rust
.inner_join(tag_name::table).filter(TagName::primary())
```

### Sortable Tokens

All tokens can be sorted. `ImplicationCount` and `Implies` share the same sort (by `implication_count`). `SuggestionCount` and `Suggests` share the same sort (by `suggestion_count`).

## Pagination

`tag.rs:115` — Standard OFFSET/LIMIT from `extra_args`.

## Default Token

The anonymous token for tag search is `Token::Name`. A bare word is treated as a tag name filter.
