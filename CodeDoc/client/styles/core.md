---
title: Core Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, core]
---

# Core Styles

## `colors.styl`
**Purpose**: Defines all color variables for the application, supporting both light and dark themes.

**Key variables**:
- `$main-color-static` = `#BC3736` (fallback red)
- `$main-color` = `var(--accent-color, #BC3736)` (CSS custom property)
- `$window-color` = `white` / `$window-color-darktheme` = `#1a1a1a`
- `$top-navigation-color` = `#F5F5F5` / `$top-navigation-color-darktheme` = `#333333`
- `$text-color` = `#111` / `$text-color-darktheme` = `#e6e6e6`
- `$inactive-link-color` = `#888` / darktheme = `#cccccc`
- `$line-color` = `#DDD`
- `$active-tab-background-color` = `rgba(0,0,0,0.06)` / darktheme = `rgba(255,255,255,0.06)`
- `$focused-tab-background-color` = `rgba(0,0,0,0.03)`

**Message colors**: `$message-info-border-color` (#BDF), `$message-error-border-color` (#FCC), `$message-success-border-color` (#D3E3D3) with corresponding backgrounds.

**Input state colors**: `$input-bad-border-color` (#FCC), `$input-good-border-color` (#D3E3D3), `$input-enabled-background-color` (#FAFAFA).

**Button colors**: `$button-enabled-background-color` = `$main-color`, `$button-disabled-background-color` = `#CCC`.

**Safety colors**: `$safety-safe` = `#88D488`, `$safety-sketchy` = `#F3D75F`, `$safety-unsafe` = `#F3985F`.

**Post thumbnail border colors**: `$post-thumbnail-border-color` = `$main-color`, `$post-thumbnail-no-tags-border-color` = `#F44` (red border for untagged posts), `$post-thumbnail-tag-me-border-color` = `#FF7801` (orange border for flagged posts).

**Tag colors**: `$default-tag-category-background-color`, `$new-tag-background-color` (#DFC), `$implied-tag-background-color` (#FFC), `$duplicate-tag-background-color` (#FDC).

**Note overlay colors**: active note overlay background/border, note background/border, edited note colors, note text colors, note point colors.

**Scrollbar**: `$scrollbar-thumb-color` = `$main-color`, `$scrollbar-bg-color` = `$input-enabled-background-color`.

**Transparency grid**: `$transparency-grid-square-color` = `#F0F0F0`.

**Root CSS custom property**: `--accent-color: #BC3736`.

---

## `mixins.styl`
**Purpose**: Defines reusable Stylus mixins.

**Mixin**: `unselectable()` — sets `user-select: none` with all vendor prefixes (`-webkit-`, `-moz-`, `-ms-`, `-o-`).

---

## `core-general.styl`
**Imports**: `colors`, `mixins`

**Purpose**: Base styles for the entire application.

**Key styles**:
- `@font-face` for Open Sans (woff2, Latin subset)
- `html`/`body` full viewport height, scrollbar always visible
- `body` background, font-family, font-size (responsive: 0.875em at ≤800px, 0.95em at ≤1200px)
- `.darktheme body` — dark colors
- Headings (`h1` 2em, `h2` 1.5em) — normal weight
- Links (`a`) — accent color, inactive class uses `$inactive-link-color`, focus outline
- `.append` — left margin 1em
- `#content-holder` — padding 1.5em (1em at ≤1000px), centering content
- `.content-wrapper` — inline-block, max-width 100%, text-align left; navigation with horizontal scrollbar
- `.transparent-container` — no background; otherwise uses `$top-navigation-color` background with padding
- **Dark theme** overrides for content-wrapper and blockquotes
- `blockquote` — left border, slight darken background
- `hr` — thin border
- **Navigation** (`nav`) — ul/li reset, button tabs with active/inactive states, hover effects
- `#top-navigation` — sticky top nav bar, mobile toggle (hidden by default, shown at ≤1000px), right-floated account/register/login/logout/settings/help items, thumbnail image in nav items, access key underline
- **Messages** (`.messages`) — `.message.info`, `.message.error`, `.message.success` with border and background colors; dark theme variants
- **Thumbnails** (`.thumbnail`) — background-cover display, `.empty` variant with transparency checkerboard
- `.flexbox-dummy` — hidden sizing element
- `.table-wrap` — horizontal scroll with custom scrollbar
- `[data-pseudo-content]:before` — CSS content hack to prevent text copying

---

## `core-forms.styl`
**Imports**: `colors`

**Purpose**: Form element styling.

**Key styles**:
- `form` — 20em default width, `.input` list-style reset with label block display and top margin
- `.horizontal` form — inline-block, side-by-side inputs/buttons, stacks vertically at ≤1000px
- **Radio/Checkbox**: Hidden native input, custom `.radio`/`.checkbox` styled labels with `:before` (box) and `:after` (dot for radio, checkmark for checkbox), accent color on checked/focus, disabled states
- **Date/Time inputs**: Border, background, color, focus accent, readonly/disabled states, dark theme
- **Regular inputs** (select, textarea, text, email, password, number): Full width, box-sizing border-box, border/background/color transitions, focus border accent, readonly/disabled, dark theme
- `.color label` — flex display with text input and color preview spans
- **Validation** (`form.show-validation`): Invalid inputs red border/background, valid inputs green border/background
- **Buttons** (`button`, `input[type=button]`, `input[type=submit]`): Accent background, white text, disabled gray, `.discourage` transparent, focus border
- **File dropper** (`.file-dropper-holder`): Styled drop zone with dashed border, active/focus accent, URL holder flex layout
- **Autocomplete** (`.autocomplete`): Positioned dropdown, accent border, list items with hover/active accent background, dark theme
- `.anticomplete` — hidden (anti-autocomplete trick)
