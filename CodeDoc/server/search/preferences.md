# preferences.rs — User Preference Filters

This module builds WHERE clauses that hide posts based on user preferences and rank.

## `has_preferences(ctx) -> bool`

`preferences.rs:16` — Returns true if the user's preferences should affect search results. True when:
- The client rank is `Anonymous` or `Restricted`.
- OR the server's anonymous_preferences is not empty.

This function is used as a fast path check before building preference-based filters.

## `hidden_posts(ctx, post_id_column) -> Option<BoxedQuery>`

`preferences.rs:21` — Returns a subquery that selects posts the user should not see. Returns `None` for regular users (rank >= `Regular`).

The returned subquery selects `0` (an integer literal) from an aliased `post` table, joined to `post_statistics`. It acts as an existence check used in `NOT EXISTS`.

### Restricted Users

`preferences.rs:34` — Restricted users cannot see unsafe posts:
```sql
WHERE inner_post.safety = 'unsafe'
  AND inner_post.id = <post_id_column>
```

### Anonymous Users — No Preferences

`preferences.rs:46` — If the server has no anonymous preferences configured, the subquery returns nothing (uses `WHERE 0 = 1`). No posts are hidden.

### Anonymous Users — With Preferences

`preferences.rs:49` — Builds a subquery with OR'd conditions:

| Preference | Condition |
|------------|-----------|
| `hide_sketchy` | `inner_post.safety = 'sketchy'` |
| `hide_unsafe` | `inner_post.safety = 'unsafe'` |
| `hide_untagged` | `post_statistics.tag_count = 0` |

### Tag/Category Blacklist

`preferences.rs:58` — If tag or category blacklists exist, adds an OR condition using EXISTS:

```sql
OR EXISTS (
  SELECT 0 FROM tag
  INNER JOIN tag_category ON ...
  INNER JOIN tag_name ON ...
  INNER JOIN post_tag ON ...
  WHERE tag_name.name IN (<blacklisted tags>)
     OR tag_category.name IN (<blacklisted categories>)
  AND inner_post.id = post_tag.post_id
)
```

This subquery checks if the post has any blacklisted tag or any tag in a blacklisted category.

## How Preferences Are Applied

In `post.rs` and `comment.rs`, after all search filters are built:
```rust
if let Some(hidden_posts) = preferences::hidden_posts(self.search.ctx, post::id) {
    query = query.filter(not(exists(hidden_posts)));
}
```

This adds a `WHERE NOT EXISTS (...)` clause to the main query. Posts that match the hidden_posts subquery are excluded from results.

## Aliased Table

`preferences.rs:11` — Uses Diesel's `alias!` macro to create `inner_post` as an alias of the `post` table. This allows the subquery to reference the post table independently of the outer query, even when the outer query already joins the post table.
