---
title: Custom Field Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, custom-fields]
---

# Custom Field Styles

## `custom-field-view.styl`
**Imports**: `colors`

**Purpose**: Styles the custom field detail page.

**Key classes**:
- `#custom-field` — max-width 40em, full width. Heading word-break. Form full width.
- `.custom-field-edit textarea` — 10em height
- `.custom-field-summary` — Description section top margin, Details section (vertical-align top, padding-right, sections with 1em margin-bottom)

---

## `custom-field-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the custom field list table.

**Key classes**:
- `.custom-field-list table` — full width, row hover, header background
- Column widths: name 30%, description 62%, creation-time 8%
- Responsive at ≤800px: hide description column
- Dark theme variants for hover and header

- `.custom-field-list-header` — Search input 25em width, append links

---

## `custom-field-input-control.styl`
**Imports**: `colors`

**Purpose**: Styles the custom field input widget.

**Key classes**:
- `div.custom-field-input` — relative positioned
- `.main-control` — flex wrap with gap. `.cf-name` flex 2 (min 8em), `.cf-value` flex 1 (min 5em), button flex none
- `ul.compact-custom-fields` — full width list, `.editing` class uses `$top-navigation-color` background, items are cursor pointer (clickable)
- Usages/remove links — inactive-link-color, unselectable
- Dark theme: editing background uses `$top-navigation-color-darktheme`
