---
title: Comment Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, comments]
---

# Comment Views

## CommentsPageView (`js/views/comments_page_view.js`)

**Template**: `comments-page`

**Purpose**: Renders a list of posts with their comments in a global comment view.

**Constructor** (`ctx`):
- `ctx.response.results` — array of Post models (each with `.comments`)
- `ctx.hostNode` — node to render into

**Render method**: Renders template. For each post in results, finds the `.comments-container[data-for="<postId>"]` node and creates a CommentListControl. Proxies `submit`, `score`, and `delete` events from each CommentListControl.

Note: Individual comment creation, editing, and scoring is handled by the `CommentControl` (`js/controls/comment_control.js`) and `CommentListControl` (`js/controls/comment_list_control.js`), which are controls (not views).

The view layer for comments is thin — the heavy lifting is in the control layer via:
- **CommentControl** — renders a single comment form/display using the `comment` template
- **CommentListControl** — manages a list of CommentControl instances using the `comment-list` template
- **ScoreControl** — renders upvote/downvote buttons using the `score` template (used in both post sidebar and comments)
- **FavControl** — renders favorite toggle using the `fav` template
