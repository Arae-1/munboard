---
title: Comment Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, comments]
---

# Comment Templates

## `comments_page.tpl`
**Used by**: CommentsPageView
**Variables**: `ctx.response.results`, `ctx.canViewPosts`, `ctx.formatClientLink`, `ctx.makeThumbnail`
**Structure**: List of posts with thumbnails (linked if `canViewPosts`) and a `.comments-container[data-for="<postId>"]` div for each post. On desktop, the thumbnail floats left with margin.

---

## `comment.tpl`
**Used by**: CommentControl
**Variables**: `ctx.user`, `ctx.comment`, `ctx.canViewUsers`, `ctx.canEditComment`, `ctx.canDeleteComment`, `ctx.onlyEditing`, `ctx.makeThumbnail`, `ctx.makeRelativeTime`, `ctx.makeMarkdown`, `ctx.formatClientLink`
**Structure**: Comment container with avatar thumbnail (linked to user), comment body with header containing:
- **Edit mode tabs**: Write/Preview tabs
- **Readonly header**: Nickname (linked), relative time, score container, conditional edit/delete action links
- **Body**: Preview tab with rendered markdown, Edit tab with textarea
- **Edit nav**: Messages, save button, conditional cancel button

---

## `comment_list.tpl`
**Used by**: CommentListControl
**Structure**: Simple `<div class="comments"><ul></ul></div>` container for comment items.

---

## `score.tpl`
**Used by**: ScoreControl (appears in post sidebar and comments)
**Variables**: `ctx.canScore`, `ctx.ownScore` (-1, 0, or 1), `ctx.score`
**Structure**: Upvote link (thumbs-up/thumbs-o-up based on ownScore), score value, downvote link (thumbs-down/thumbs-o-down). Inactive if `canScore` is false.

---

## `fav.tpl`
**Used by**: FavControl (appears in post sidebar)
**Variables**: `ctx.canFavorite`, `ctx.ownFavorite`, `ctx.favoriteCount`
**Structure**: Favorite toggle link (filled heart if favorited, outline if not), with favorite count.
