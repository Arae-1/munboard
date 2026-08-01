---
title: Help Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, help]
---

# Help Templates

## `help.tpl`
**Used by**: HelpView
**Variables**: `ctx.formatClientLink`
**Structure**: Content wrapper with primary navigation tabs (About, Tagging guidelines, Keyboard, Search syntax, Comments, Terms of service). Content div for section content.

## `help_about.tpl`
**Used by**: HelpView (about section)
**Variables**: `ctx.name` (not directly used, text is hardcoded)
**Structure**: Static text about Munboard, registration, and account deletion.

## `help_guidelines.tpl`
**Used by**: HelpView (guidelines section)
**Structure**: Placeholder text ("FILL THIS OUT LATER PLEASE").

## `help_keyboard.tpl`
**Used by**: HelpView (keyboard section)
**Structure**: Table of keyboard shortcuts:
- `Q` — focus search field
- `A`/`D` or arrows — previous/next page or post
- `F` — cycle post fit mode
- `E` — edit post
- `P` — focus first post in list
- `T` — focus tag input (edit mode)
- `Ctrl+S` — save post (edit mode)
- `Delete` — delete post (edit mode)
Plus note about access keys for top navigation.

## `help_search.tpl`
**Used by**: HelpView (search section)
**Variables**: `ctx.formatClientLink`
**Structure**: Secondary navigation tabs for search subsections (General, Posts, Users, Tags, Pools). Subcontent div.

## `help_search_general.tpl`
**Used by**: HelpView (search/general subsection)
**Structure**: Explains search token types (anonymous, named `key:value`, sort styles, special tokens), ranged/composite values (`,` for OR, `..` for range), date formats, wildcards, negation with `-`, escaping with `\`. Includes an example query.

## `help_search_posts.tpl`
**Used by**: HelpView (search/posts subsection)
**Structure**: Three tables:
- **Named tokens**: id, tag, tag-category, score, uploader/submit, comment, fav, pool, pool-id, pool-category, tag-count, comment-count, fav-count, note-count, note-text, relation-count, feature-count, type, content-checksum, flag, source, file-size, image-width/height/area/aspect-ratio, creation/last-edit/comment/fav/feature-date, launch-date, media-date, safety/rating
- **Sort style tokens**: random, id, score, uploader, pool-count, tag-count, comment-count, fav-count, note-count, relation-count, feature-count, type, flag, source, file-size, image-width/height/area/aspect-ratio, creation/last-edit/comment/fav/feature-date, launch-date, media-date, safety
- **Special tokens**: liked, disliked, fav, tumbleweed

## `help_search_users.tpl`
**Used by**: HelpView (search/users subsection)
**Structure**: Named tokens (name, creation-date, last-login-date). Sort tokens (random, name, creation-date, last-login-date). No special tokens.

## `help_search_tags.tpl`
**Used by**: HelpView (search/tags subsection)
**Structure**: Named tokens (name, category, description, creation-date, last-edit-date, usages, suggestion-count, implication-count, implies, suggests). Sort tokens (random, name, category, description, creation-date, last-edit-date, usages, implication-count, suggestion-count). No special tokens.

## `help_search_pools.tpl`
**Used by**: HelpView (search/pools subsection)
**Structure**: Named tokens (name, launch-date, category, creation-date, last-edit-date, post-count). Sort tokens (random, name, launch-date, category, creation-date, last-edit-date, post-count). No special tokens.

## `help_comments.tpl`
**Used by**: HelpView (comments section)
**Structure**: Documents Markdown extensions: `@426` (post link), `#tag` (tag link), `+user` (user link), `~~strike~~`, `[spoiler]`, `[sjis]`, `[icon]`. Also documents image size syntax: `![alt](href =WIDTHx "title")`.

## `help_tos.tpl`
**Used by**: HelpView (tos section)
**Variables**: `ctx.name`
**Structure**: Terms of service text covering: AS-IS warranty disclaimer, right to delete content, right to change TOS, resource abuse, spam prohibition, content liability. Prohibited content section (NSFW, leaked media, unauthorized uploads, non-spaceflight content, personal images). Takedowns section with email. Privacy policy (IP/email disclosure, data storage, anonymous upload option, cookies).
