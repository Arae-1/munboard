---
title: Custom Field Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, custom-fields]
---

# Custom Field Views

## CustomFieldsPageView (`js/views/custom_fields_page_view.js`)

**Template**: `custom-fields-page`

**Purpose**: Renders a table of custom fields with sortable columns (name, description, creation time).

**Constructor** (`ctx`):
- `ctx.response.results` — array of CustomField models
- `ctx.customFieldDefaultColor` — default color from API

**Render method**: Simple `views.replaceContent` with the template.

---

## CustomFieldsHeaderView (`js/views/custom_fields_header_view.js`)

**Template**: `custom-fields-header`

**Purpose**: Header with search form and CustomFieldAutoCompleteControl.

**Events dispatched**:
- `navigate` — on form submit with `{query, page: 1}`

---

## CustomFieldView (`js/views/custom_field_view.js`)

**Template**: `custom-field`

**Purpose**: Top-level custom field detail view with tabbed sections (summary, edit, delete).

**Constructor** (`ctx`):
- `ctx.customField` — CustomField model
- `ctx.section` — `'summary'`, `'edit'`, or `'delete'`

**Render method**: Renders template with tabs. Creates inner view:
- `summary` → CustomFieldSummaryView
- `edit` → CustomFieldEditView
- `delete` → CustomFieldDeleteView

---

## CustomFieldSummaryView (`js/views/custom_field_summary_view.js`)

**Template**: `custom-field-summary`

**Purpose**: Displays custom field details: description, color (with preview swatch), creation time.

---

## CustomFieldEditView (`js/views/custom_field_edit_view.js`)

**Template**: `custom-field-edit`

**Purpose**: Edit form for a custom field.

**Form fields**: Name (text), Description (textarea), Color (text, placeholder `#ff0000 or red`).

**Events dispatched**:
- `submit` — with `{customField, name, description, color}`
- `change` — on any input change

---

## CustomFieldDeleteView (`js/views/custom_field_delete_view.js`)

**Template**: `custom-field-delete`

**Purpose**: Delete confirmation form. Warns that deletion also removes the field from all posts.

**Events dispatched**:
- `submit` — with `{customField}`

---

## CustomFieldCreateView (`js/views/custom_field_create_view.js`)

**Template**: `custom-field-create`

**Purpose**: Form to create a new custom field.

**Form fields**: Name, Description, Color.

**Events dispatched**:
- `submit` — with `{name, description, color}`
