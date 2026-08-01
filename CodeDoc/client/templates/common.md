---
title: Common Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, common]
---

# Common Templates

## `top_navigation.tpl`
**Used by**: TopNavigationView
**Variables**: `ctx.name`, `ctx.items` (array of `{key, url, title, accessKey, imageUrl, available}`), `ctx.makeThumbnail`, `ctx.makeAccessKey`
**Structure**: Navigation bar with mobile toggle button (site name + hamburger icon). List of `<li>` items, each with a link containing an optional thumbnail image and text with access key underline. Items only render if `available` is true.

## `manual_pager.tpl`
**Used by**: ManualPageView
**Structure**: Container with `page-header-holder`, `messages`, `page-content-holder`, and `page-nav` divs.

## `manual_pager_nav.tpl`
**Used by**: ManualPageView
**Variables**: `ctx.prevPage`, `ctx.nextPage`, `ctx.currentPage`, `ctx.totalPages`, `ctx.pages` (Map), `ctx.getClientUrlForPage`
**Structure**: Button navigation with previous/next chevron links (disabled if at boundary). Page number links. Ellipsis (`…`) for gaps. Active page highlighted.

## `endless_pager.tpl`
**Used by**: EndlessPageView
**Structure**: Container with `page-header-holder`, `messages`, top `page-guard`, `pages-holder`, bottom `page-guard`.

## `endless_pager_page.tpl`
**Used by**: EndlessPageView
**Variables**: `ctx.page`, `ctx.totalPages`
**Structure**: Page div with header ("Page X of Y") and `page-content-holder` div.

## `not_found.tpl`
**Used by**: NotFoundView
**Variables**: `ctx.path`, `ctx.formatClientLink`
**Structure**: "Not found" heading, path display, link back to main page.

## `file_dropper.tpl`
**Used by**: FileDropperControl
**Variables**: `ctx.id`, `ctx.allowMultiple`, `ctx.extraText`, `ctx.allowUrls`, `ctx.urlPlaceholder`, `ctx.lock`
**Structure**: Hidden file input, styled label (drop zone with instruction text, optional extra small text), optional URL input holder with text input and button (label changes based on `lock` — "Confirm" or "Add URL").

## `expander.tpl`
**Used by**: ExpanderControl
**Variables**: `ctx.title`
**Structure**: Section with header link (title + chevron icon) and `expander-content` div.

## `home.tpl`
**Used by**: HomeView
**Variables**: `ctx.name`, `ctx.canListPosts`, `ctx.formatClientLink`, `ctx.makeTextInput`
**Structure**: Content wrapper with SVG logo (accent-colored), site name heading, conditional search form (text input + submit + "or browse all posts" link), `post-info-container` for featured post, `footer-container` for stats.

## `home_footer.tpl`
**Used by**: HomeView
**Variables**: `ctx.postCount`, `ctx.diskUsage`, `ctx.canListSnapshots`, `ctx.formatClientLink`, `ctx.makeFileSize`
**Structure**: Unordered list showing post count, disk usage, and conditional history link.

## `home_featured_post.tpl`
**Used by**: HomeView
**Variables**: `ctx.featuredPost`, `ctx.makePostLink`, `ctx.makeRelativeTime`, `ctx.makeUserLink`
**Structure**: Post container div. If featured post exists, shows an aside with post link, relative time, and user link.

## `login.tpl`
**Used by**: LoginView
**Variables**: `ctx.userNamePattern`, `ctx.passwordPattern`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.makePasswordInput`, `ctx.makeCheckbox`
**Structure**: Form with User name (pattern-validated), Password (pattern-validated), Remember me checkbox. Messages and submit button with "Forgot password?" link.

## `user_registration.tpl`
**Used by**: RegistrationView
**Variables**: `ctx.userNamePattern`, `ctx.passwordPattern`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.makePasswordInput`, `ctx.makeEmailInput`
**Structure**: Form with anti-autocomplete fields, User name (pattern-validated, placeholder "letters, digits, _, -"), Password (pattern-validated, placeholder "5+ characters"), Email (optional, placeholder "optional"). Messages and submit button. Info section below lists what registered users can do and links to guidelines and TOS.

## `password_reset.tpl`
**Used by**: PasswordResetView
**Variables**: `ctx.canSendMails`, `ctx.contactEmail`, `ctx.makeTextInput`
**Structure**: If mail enabled: form with user-name-or-email input, description text, messages, proceed button. If mail disabled: apology text and optional contact email link.
