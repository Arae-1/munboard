---
title: Custom Field Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, custom-fields]
---

# Custom Field Templates

## `custom_fields_page.tpl`
**Used by**: CustomFieldsPageView
**Variables**: `ctx.response.results`, `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeRelativeTime`, `ctx.customFieldDefaultColor`
**Structure**: Table with sortable columns — Name (with color styling), Description, Creation time. On mobile, description column is hidden.

---

## `custom_fields_header.tpl`
**Used by**: CustomFieldsHeaderView
**Variables**: `ctx.parameters.query`, `ctx.formatClientLink`, `ctx.makeTextInput`, `ctx.canCreate`
**Structure**: Horizontal form with search text input, submit button, syntax help link, conditional add-new link.

---

## `custom_field.tpl`
**Used by**: CustomFieldView
**Variables**: `ctx.customField` (id, name, color), `ctx.customFieldDefaultColor`, `ctx.formatClientLink`, `ctx.canEditAnything`, `ctx.canDelete`
**Structure**: Content wrapper with colored heading, tab navigation (Summary, conditional Edit/Delete), and `custom-field-content-holder` div.

---

## `custom_field_summary.tpl`
**Used by**: CustomFieldSummaryView
**Variables**: `ctx.customField` (description, color, creationTime), `ctx.makeMarkdown`, `ctx.makeRelativeTime`
**Structure**: Details section with optional description (markdown), optional color (with color swatch), and creation time.

---

## `custom_field_edit.tpl`
**Used by**: CustomFieldEditView
**Variables**: `ctx.customField`, `ctx.canEditName`, `ctx.canEditDescription`, `ctx.canEditColor`, `ctx.canEditAnything`, `ctx.makeTextInput`, `ctx.makeTextarea`
**Structure**: Form with Name (text, required), Description (textarea), Color (text, placeholder `#ff0000 or red`). Messages and save button.

---

## `custom_field_delete.tpl`
**Used by**: CustomFieldDeleteView
**Variables**: `ctx.customField` (name, color), `ctx.customFieldDefaultColor`, `ctx.makeCheckbox`
**Structure**: Warning text about cascading deletion, confirmation checkbox, messages, delete submit button.

---

## `custom_field_create.tpl`
**Used by**: CustomFieldCreateView
**Variables**: `ctx.canCreate`, `ctx.makeTextInput`, `ctx.makeTextarea`
**Structure**: Same fields as edit (Name, Description, Color). Create button.

---

## `custom_field_input.tpl`
**Used by**: CustomFieldInputControl
**Structure**: Div with name input (`cf-name`), number input (`cf-value`, min 0, step 1), Add button, and `compact-custom-fields` list.
