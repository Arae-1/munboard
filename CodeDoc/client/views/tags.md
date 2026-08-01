---
title: Tag Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, tags]
---

# Tag Views

## TagsPageView (`js/views/tags_page_view.js`)

**Template**: `tags-page`

**Purpose**: Renders a table of tags with sortable columns.

**Constructor** (`ctx`):
- `ctx.hostNode` — node to render into
- `ctx.response.results` — array of Tag models

**Render method**: Simple `views.replaceContent` with the template. No event handling at this level.

---

## TagsHeaderView (`js/views/tags_header_view.js`)

**Template**: `tags-header`

**Purpose**: Header with search form and tag auto-complete.

**Constructor** (`ctx`):
- `ctx.parameters.query` — current search query

**Render method**: Renders template, creates TagAutoCompleteControl on search input. Binds form submit.

**Events dispatched**:
- `navigate` — on form submit with `{query, page: 1}`

---

## TagView (`js/views/tag_view.js`)

**Template**: `tag`

**Purpose**: Top-level tag detail view with tabbed sections (summary, edit, merge, delete).

**Constructor** (`ctx`):
- `ctx.tag` — Tag model
- `ctx.section` — `'summary'`, `'edit'`, `'merge'`, or `'delete'`

**Render method**: Renders template with tab navigation. Creates the appropriate inner view based on section:
- `summary` → TagSummaryView
- `edit` → TagEditView (if `canEditAnything`)
- `merge` → TagMergeView (if `canMerge`)
- `delete` → TagDeleteView (if `canDelete`)
- Falls back to EmptyView with error if permissions are missing

Listens to tag `change` event and re-installs.

---

## TagSummaryView (`js/views/tag_summary_view.js`)

**Template**: `tag-summary`

**Purpose**: Displays tag summary with category, aliases, implications, suggestions, description, and post count.

**Constructor** (`ctx`):
- `ctx.tag` — Tag model

**Render method**: Simple replaceContent with template. No event handling.

---

## TagEditView (`js/views/tag_edit_view.js`)

**Template**: `tag-edit`

**Purpose**: Edit form for a tag.

**Constructor** (`ctx`):
- `ctx.tag` — Tag model
- Creates TagInputControl on implications and suggestions fields
- Validates tag names against the server regex on input

**Form fields**: Names, Category (select), Implications (TagInputControl), Suggestions (TagInputControl), Description (textarea).

**Events dispatched**:
- `submit` — with `{tag, names, category, implications, suggestions, description}`
- `change` — on any form input change

---

## TagMergeView (`js/views/tag_merge_view.js`)

**Template**: `tag-merge`

**Purpose**: Merge form for a tag.

**Constructor** (`ctx`):
- `ctx.tag` — Tag model
- `ctx.tagNamePattern` — regex from API for validation
- Creates TagAutoCompleteControl on target tag field

**Events dispatched**:
- `submit` — with `{tag, targetTagName}`

---

## TagDeleteView (`js/views/tag_delete_view.js`)

**Template**: `tag-delete`

**Purpose**: Delete confirmation form for a tag.

**Events dispatched**:
- `submit` — with `{tag}`

---

## TagCreateView (`js/views/tag_create_view.js`)

**Template**: `tag-create`

**Purpose**: Form to create a new tag.

**Constructor** (`ctx`):
- `ctx.categories` — available tag categories
- Creates TagInputControl on implications and suggestions fields (with new TagList)

**Form fields**: Names, Category (select), Implications, Suggestions, Description.

**Events dispatched**:
- `submit` — with `{names, category, implications, suggestions, description}`

---

## TagCategoriesView (`js/views/tag_categories_view.js`)

**Templates**: `tag-categories`, `tag-category-row`

**Purpose**: CRUD management for tag categories.

**Constructor** (`ctx`):
- `ctx.tagCategories` — TagCategoryList model

**Render method**: Renders template, sorts existing categories (default first), creates row nodes for each. Handles add, remove events from the TagCategoryList.

**Events dispatched**:
- `submit` — on save button click
