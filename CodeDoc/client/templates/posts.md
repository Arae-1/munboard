---
title: Post Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, posts]
---

# Post Templates

## `posts-page.tpl`
**Used by**: PostsPageView
**Variables**: `ctx.response.results`, `ctx.postFlow`, `ctx.canViewPosts`, `ctx.getPostUrl`, `ctx.makeThumbnail`, `ctx.makeFlexboxAlign`, `ctx.canBulkEditTags`, `ctx.canBulkEditSafety`, `ctx.canBulkDelete`, `ctx.parameters`
**Structure**: Flexbox list of `<li>` items, each with thumbnail wrapper (`<a>` with thumbnail, type badge, stats for score/favorites/comments), and optional bulk edit overlay with tag flipper (`+`/`-`), safety flipper (safe/sketchy/unsafe circles), and delete flipper.

---

## `posts_header.tpl`
**Used by**: PostsHeaderView
**Variables**: `ctx.parameters`, `ctx.settings`, `ctx.enableSafety`, `ctx.canBulkEditTags`, `ctx.canBulkEditSafety`, `ctx.canBulkDelete`, `ctx.formatClientLink`, `ctx.makeTextInput`
**Structure**: Horizontal form with search text input, submit button, safety toggle buttons (safe/sketchy/unsafe) with disabled states. Conditionally renders bulk-edit forms: bulk-edit-tags (open/close links, tag input, start submit), bulk-edit-safety (open/close links), bulk-edit-delete (open/close links, delete submit).

---

## `post_main.tpl`
**Used by**: PostMainView
**Variables**: `ctx.post`, `ctx.prevPostId`, `ctx.nextPostId`, `ctx.editMode`, `ctx.canEditPosts`, `ctx.canDeletePosts`, `ctx.canFeaturePosts`, `ctx.canEditPostDescription`, `ctx.canCreateComments`, `ctx.canListComments`, `ctx.getPostUrl`, `ctx.getPostEditUrl`, `ctx.makeTextarea`, `ctx.makeMarkdown`
**Structure**: Flex container with sidebar (prev/next nav, edit/back button links, sidebar-container div) and content area (post-container, optional description textarea or rendered markdown description in `<details>`, comment-form-container, comments-container). Responsive: sidebar moves below content on mobile.

---

## `post_detail.tpl`
**Used by**: PostDetailView
**Variables**: `ctx.post.id`, `ctx.canMerge`, `ctx.formatClientLink`
**Structure**: Wrapper with `<h1>` showing post number, tab navigation (Main view, conditional Merge link), and `post-content-holder` div.

---

## `post_content.tpl`
**Used by**: PostContentControl
**Variables**: `ctx.post.type`, `ctx.post.contentUrl`, `ctx.post.thumbnailUrl`, `ctx.post.canvasWidth`, `ctx.post.canvasHeight`, `ctx.post.mimeType`, `ctx.post.flags`, `ctx.autoplay`, `ctx.makeElement`
**Structure**: Div with `post-type-<type>` class. Renders differently by type:
- **image/animation**: `<img>` element
- **flash**: `<object>` with Flash params, shows error if unsupported
- **video**: `<video>` with controls, optional loop and autoplay
- **document**: `<embed>` for PDF with fallback link
- All include a `.post-overlay` div for note annotations

---

## `post_upload.tpl`
**Used by**: PostUploadView
**Variables**: `ctx.makeCheckbox`
**Structure**: Upload form with dropper-container (file dropper), control strip with submit button, skip-duplicates checkbox, always-upload-similar checkbox, pause-remain-on-error checkbox, cancel button, messages area, and `uploadables-container` list.

---

## `post_upload_row.tpl`
**Used by**: PostUploadView (per uploadable)
**Variables**: `ctx.uploadable` (type, previewUrl, mimeType, name, safety, anonymous, forceAnonymous, key, lookalikes), `ctx.makeThumbnail`, `ctx.makeRadio`, `ctx.makeCheckbox`, `ctx.makePostLink`, `ctx.canUploadAnonymously`, `ctx.canViewPosts`, `ctx.enableSafety`, `ctx.getPostUrl`
**Structure**: List item with thumbnail (image/video/empty based on type), header with move-up/move-down/remove buttons and filename, body with safety radio group, anonymous upload checkbox, messages, and lookalikes list (similar post thumbnails with copy-tags and add-relation checkboxes).

---

## `post_merge.tpl`
**Used by**: PostMergeView
**Variables**: `ctx.makeCheckbox`
**Structure**: Form with two side-by-side post containers, merge description text, confirmation checkbox, messages, and submit button.

---

## `post_merge_side.tpl`
**Used by**: PostMergeView (per side)
**Variables**: `ctx.name`, `ctx.post`, `ctx.editable`, `ctx.makeThumbnail`, `ctx.makeRadio`, `ctx.makeUserLink`, `ctx.makeRelativeTime`, `ctx.makeFileSize`
**Structure**: Header with post ID input (editable or readonly), post thumbnail, target-post radio (merge to this post), target-post-content radio (use this file), showing uploader, time, file size, MIME type, dimensions.

---

## `post_readonly_sidebar.tpl`
**Used by**: PostReadonlySidebarControl
**Variables**: `ctx.post` (all properties), `ctx.makeFileSize`, `ctx.makeUserLink`, `ctx.makeRelativeTime`, `ctx.makeThumbnail`, `ctx.makeCssName`, `ctx.getPrettyName`, `ctx.getPostUrl`, `ctx.escapeTagName`, `ctx.canViewTags`, `ctx.canEditPosts`, `ctx.canViewUsers`, `ctx.canListPosts`, `ctx.enableSafety`, `ctx.formatClientLink`, `ctx.makePostLink`, `ctx.extractRootDomain`
**Structure**: Readonly sidebar with download link (file size, MIME type, dimensions), upload info (user link, relative time), safety indicator, zoom controls (original/fit width/fit height/fit both), source links, Google Images search link, social section (score-container, fav-container), relations nav (thumbnails), tags nav (compact tag list with category colors), pool link, custom fields list, launch date, media created date.

---

## `post_edit_sidebar.tpl`
**Used by**: PostEditSidebarControl
**Variables**: `ctx.post`, `ctx.enableSafety`, `ctx.canEditPostSafety`, `ctx.canEditPostRelations`, `ctx.canEditPostFlags`, `ctx.canEditPostSource`, `ctx.canEditPostInfoDates`, `ctx.canEditPostTags`, `ctx.canEditPoolPosts`, `ctx.canEditPostCustomFields`, `ctx.canEditPostNotes`, `ctx.canEditPostContent`, `ctx.canEditPostThumbnail`, `ctx.canFeaturePosts`, `ctx.canDeletePosts`, `ctx.canMergePosts`, `ctx.hasClipboard`, `ctx.makeRadio`, `ctx.makeCheckbox`, `ctx.makeTextInput`, `ctx.makeDateInput`, `ctx.makeTextarea`
**Structure**: Edit form with save button, messages, conditional sections: safety (radio), relations (text input with post ID pattern), flags (loop/sound checkboxes for video), source (textarea), launch date (date input), media date (date input), tags (text input), pools (text input), custom-fields (text input), notes (add/delete/copy/paste links with textarea), post content (file dropper), thumbnail (file dropper + discard link), management (feature/merge/delete links).
