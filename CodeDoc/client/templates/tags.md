---
title: Tag Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, tags]
---

# Tag Templates

## `tags_page.tpl`
**Used by**: TagsPageView
**Variables**: `ctx.response.results`, `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeTagLink`, `ctx.makeRelativeTime`, `ctx.escapeTagName`
**Structure**: Table with sortable columns — names (tag link per name), implications (comma-separated tag links or `-`), suggestions, usages (clickable post count), creation-time (relative time). Sort links toggle ascending/descending by toggling `-sort:` prefix. On mobile, implications and suggestions columns are hidden.

---

## `tags_header.tpl`
**Used by**: TagsHeaderView
**Variables**: `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.canEditTagCategories`, `ctx.canCreate`
**Structure**: Horizontal form with search text input, submit button, syntax help link, conditional tag categories link (for admins), and conditional add new tag link.

---

## `tag.tpl`
**Used by**: TagView
**Variables**: `ctx.getPrettyName`, `ctx.tag.names[0]`, `ctx.formatClientLink`, `ctx.canEditAnything`, `ctx.canMerge`, `ctx.canDelete`
**Structure**: Content wrapper with tag name heading, tab navigation (Summary, conditional Edit/Merge/Delete tabs), and `tag-content-holder` div.

---

## `tag_summary.tpl`
**Used by**: TagSummaryView
**Variables**: `ctx.tag` (category, names, implications, suggestions, description, postCount), `ctx.makeCssName`, `ctx.makeTagLink`, `ctx.makeMarkdown`, `ctx.escapeTagName`, `ctx.formatClientLink`
**Structure**: Details section with category badge, aliases list, implications list, suggestions list. Description section with markdown-rendered description and post count link.

---

## `tag_edit.tpl`
**Used by**: TagEditView
**Variables**: `ctx.tag`, `ctx.categories`, `ctx.canEditNames`, `ctx.canEditCategory`, `ctx.canEditImplications`, `ctx.canEditSuggestions`, `ctx.canEditDescription`, `ctx.canEditAnything`, `ctx.makeTextInput`, `ctx.makeSelect`, `ctx.makeTextarea`
**Structure**: Form with fields: Names (text, required), Category (select), Implications (text input), Suggestions (text input), Description (textarea). Messages and save button.

---

## `tag_merge.tpl`
**Used by**: TagMergeView
**Variables**: `ctx.tagNamePattern`, `ctx.makeTextInput`, `ctx.makeCheckbox`
**Structure**: Form with target-tag text input (pattern from API), description text about what gets merged, confirmation checkbox, messages, and merge submit button.

---

## `tag_delete.tpl`
**Used by**: TagDeleteView
**Variables**: `ctx.tag.postCount`, `ctx.escapeTagName`, `ctx.formatClientLink`, `ctx.makeCheckbox`
**Structure**: Shows tag usage count with link, confirmation checkbox, messages, delete submit button.

---

## `tag_create.tpl`
**Used by**: TagCreateView
**Variables**: `ctx.categories`, `ctx.canCreate`, `ctx.makeTextInput`, `ctx.makeSelect`, `ctx.makeTextarea`
**Structure**: Same fields as tag edit (Names, Category select, Implications, Suggestions, Description), with create button.

---

## `tag_categories.tpl`
**Used by**: TagCategoriesView
**Variables**: `ctx.canCreate`, `ctx.canEditName`, `ctx.canEditColor`, `ctx.canEditOrder`, `ctx.canDelete`
**Structure**: Page with table (Category name, CSS color, Order, Usages columns), `<tbody>` for rows, conditional "Add new category" link, messages, and save button.

---

## `tag_category_row.tpl`
**Used by**: TagCategoriesView (per row)
**Variables**: `ctx.tagCategory` (name, color, order, tagCount, isDefault), `ctx.canEditName`, `ctx.canEditColor`, `ctx.canEditOrder`, `ctx.canDelete`, `ctx.canSetDefault`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.makeColorInput`, `ctx.makeNumericInput`
**Structure**: Table row with default class if default. Editable name/color/order fields, usages link (if category has name), conditional remove link (inactive if in use), and set-default link.

---

## `tag_input.tpl`
**Used by**: TagInputControl
**Variables**: (none from ctx — pure HTML structure)
**Structure**: Div with text input + Add button, tag-suggestions panel (with visibility toggle and close button, `<ul>` for suggestion items), and `compact-tags` list.
