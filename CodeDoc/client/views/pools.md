---
title: Pool Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, pools]
---

# Pool Views

## PoolsPageView (`js/views/pools_page_view.js`)

**Template**: `pools-page`

**Purpose**: Renders a table of pools with sortable columns (names, post count, creation time).

**Constructor** (`ctx`):
- `ctx.hostNode` — node to render into
- `ctx.response.results` — array of Pool models

**Render method**: Simple `views.replaceContent` with the template.

---

## PoolsHeaderView (`js/views/pools_header_view.js`)

**Template**: `pools-header`

**Purpose**: Header with search form and pool auto-complete.

**Constructor** (`ctx`):
- `ctx.parameters.query` — current search query

**Render method**: Renders template, creates PoolAutoCompleteControl on search input. Binds form submit.

**Events dispatched**:
- `navigate` — on form submit with `{query, page: 1}`

---

## PoolView (`js/views/pool_view.js`)

**Template**: `pool`

**Purpose**: Top-level pool detail view with tabbed sections (summary, edit, merge, delete).

**Constructor** (`ctx`):
- `ctx.pool` — Pool model
- `ctx.section` — `'summary'`, `'edit'`, `'merge'`, or `'delete'`

**Render method**: Renders template with tab navigation. Creates inner view based on section. Falls back to EmptyView if permissions are missing. Listens to pool `change` event.

---

## PoolSummaryView (`js/views/pool_summary_view.js`)

**Template**: `pool-summary`

**Purpose**: Displays pool summary: category, aliases, description, post count, launch date.

---

## PoolEditView (`js/views/pool_edit_view.js`)

**Template**: `pool-edit`

**Purpose**: Edit form for a pool.

**Form fields**: Names (validated against pool name regex), Category (select), Description (textarea), Posts (space-separated IDs, validated as integers), Launch date (date input).

**Events dispatched**:
- `submit` — with `{pool, names, category, description, posts, launchDate}`
- `change` — on any form input change

---

## PoolMergeView (`js/views/pool_merge_view.js`)

**Template**: `pool-merge`

**Purpose**: Merge form for a pool with PoolAutoCompleteControl on target pool field.

**Events dispatched**:
- `submit` — with `{pool, targetPoolId}`

---

## PoolDeleteView (`js/views/pool_delete_view.js`)

**Template**: `pool-delete`

**Purpose**: Delete confirmation form for a pool.

**Events dispatched**:
- `submit` — with `{pool}`

---

## PoolCreateView (`js/views/pool_create_view.js`)

**Template**: `pool-create`

**Purpose**: Form to create a new pool.

**Form fields**: Names, Category, Description, Posts (space-separated IDs).

**Events dispatched**:
- `submit` — with `{names, category, description, posts}`

---

## PoolCategoriesView (`js/views/pool_categories_view.js`)

**Templates**: `pool-categories`, `pool-category-row`

**Purpose**: CRUD management for pool categories.

**Constructor** (`ctx`):
- `ctx.poolCategories` — PoolCategoryList model

**Render method**: Same pattern as TagCategoriesView. Sorts categories (default first), creates rows with name/color inputs, remove and set-default links.
