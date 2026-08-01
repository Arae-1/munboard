---
title: Pool Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, pools]
---

# Pool Templates

## `pools_page.tpl`
**Used by**: PoolsPageView
**Variables**: `ctx.response.results`, `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makePoolLink`, `ctx.makeRelativeTime`
**Structure**: Table with sortable columns — names (link per pool name, comma-separated), post-count (clickable count), creation-time (relative time). Columns have bidirectional sort links. On mobile, posts column is hidden.

---

## `pools_header.tpl`
**Used by**: PoolsHeaderView
**Variables**: `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.canCreate`, `ctx.canEditPoolCategories`
**Structure**: Horizontal form with search text input, submit button, syntax help link, conditional add-new-pool link, and conditional pool-categories link.

---

## `pool.tpl`
**Used by**: PoolView
**Variables**: `ctx.getPrettyName`, `ctx.pool.names[0]`, `ctx.pool.id`, `ctx.formatClientLink`, `ctx.canEditAnything`, `ctx.canMerge`, `ctx.canDelete`
**Structure**: Content wrapper with pool name heading, tab navigation (Summary, conditional Edit/Merge/Delete tabs), and `pool-content-holder` div.

---

## `pool_summary.tpl`
**Used by**: PoolSummaryView
**Variables**: `ctx.pool` (category, names, description, postCount, searchableName, launchDate), `ctx.makeCssName`, `ctx.makePoolLink`, `ctx.makeMarkdown`, `ctx.formatClientLink`
**Structure**: Details section with category badge, aliases list. Description section with markdown. Post count link. Conditional launch date.

---

## `pool_edit.tpl`
**Used by**: PoolEditView
**Variables**: `ctx.pool`, `ctx.categories`, `ctx.canEditNames`, `ctx.canEditCategory`, `ctx.canEditDescription`, `ctx.canEditLaunchDate`, `ctx.canEditPosts`, `ctx.canEditAnything`, `ctx.makeTextInput`, `ctx.makeSelect`, `ctx.makeTextarea`, `ctx.makeDateInput`
**Structure**: Form with: Names (text, required), Category (select), Description (textarea), Launch date (date input, conditional), Posts (text input with post ID pattern). Messages and save button.

---

## `pool_merge.tpl`
**Used by**: PoolMergeView
**Variables**: `ctx.poolNamePattern`, `ctx.makeTextInput`, `ctx.makeCheckbox`
**Structure**: Target-pool text input, merge description, confirmation checkbox, messages, merge submit button.

---

## `pool_delete.tpl`
**Used by**: PoolDeleteView
**Variables**: `ctx.pool.postCount`, `ctx.pool.searchableName`, `ctx.formatClientLink`, `ctx.makeCheckbox`
**Structure**: Post count link, confirmation checkbox, messages, delete submit button.

---

## `pool_create.tpl`
**Used by**: PoolCreateView
**Variables**: `ctx.categories`, `ctx.canCreate`, `ctx.makeTextInput`, `ctx.makeSelect`, `ctx.makeTextarea`
**Structure**: Form with Names (text, required), Category (select), Description (textarea), Posts (text input). Messages and create button.

---

## `pool_categories.tpl`
**Used by**: PoolCategoriesView
**Variables**: `ctx.canCreate`, `ctx.canEditName`, `ctx.canEditColor`, `ctx.canDelete`
**Structure**: Table with Category name, CSS color, Usages columns. Conditional add link, messages, save button.

---

## `pool_category_row.tpl`
**Used by**: PoolCategoriesView (per row)
**Variables**: `ctx.poolCategory` (name, color, poolCount, isDefault), `ctx.canEditName`, `ctx.canEditColor`, `ctx.canDelete`, `ctx.canSetDefault`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.makeColorInput`
**Structure**: Similar to tag_category_row but without order column. Name/color inputs, usages link, remove link (inactive if in use), set-default link.

---

## `pool_input.tpl`
**Used by**: PoolInputControl
**Structure**: Div with text input (no button), and `compact-pools` list.
