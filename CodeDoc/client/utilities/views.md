---
title: Views Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, views, dom, html, templates]
sources: [views.js]
---

# Views Utility

The `views.js` file provides DOM manipulation and HTML generation helpers. It creates form elements, renders templates, handles messaging, manages scrolling, and provides animation utilities.

## Global Event Listeners

Registered at module load time:

### `input` on `.color` elements

Live-updates the background and text preview elements when a color input changes value.

### `click` and `auxclick` on empty `href` links

Prevents opening links with `href=""` in new tabs (middle-click / auxclick).

## Functions

### `_imbueId(options)`

Generates a random element ID if `options.id` is not set. The ID is `"gen-"` followed by 7 random alphanumeric characters.

### `_makeLabel(options, attrs)`

Generates a `<label>` element with optional attributes if `options.text` is set. Sets `attrs.for = options.id`.

### `makeFileSize(fileSize)`

Returns `misc.formatFileSize(fileSize)`.

### `makeMarkdown(text)`

Returns `misc.formatMarkdown(text)`.

### `makeRelativeTime(time)`

Returns a `<time>` element with `datetime` and `title` attributes, containing the relative time string from `misc.formatRelativeTime(time)`.

### `makeThumbnail(url)`

Returns a `<span>` element with a thumbnail background image. If `url` is empty or null, adds the `"empty"` class. Contains a hidden `<img>` for accessibility.

### `makeRadio(options)`

Generates a labeled radio button. Options: `id`, `name`, `value`, `selectedValue`, `readonly`, `required`, `text`.

### `makeCheckbox(options)`

Generates a labeled checkbox. Options: `id`, `name`, `value`, `checked`, `readonly`, `required`, `text`.

### `makeSelect(options)`

Generates a labeled `<select>` element. Options: `id`, `name`, `readonly`, `keyValues` (key-value map), `selectedKey`.

### `makeInput(options)`

Generates a label + `<input>` element. All options are passed as attributes directly.

### `makeButton(options)`

Sets `type = "button"` and delegates to `makeInput`.

### `makeTextInput(options)`

Sets `type = "text"` and delegates to `makeInput`.

### `makeTextarea(options)`

Generates a label + `<textarea>` element with content as the text node. Removes `value` from attributes.

### `makePasswordInput(options)`

Sets `type = "password"` and delegates to `makeInput`.

### `makeEmailInput(options)`

Sets `type = "email"` and delegates to `makeInput`.

### `makeColorInput(options)`

Generates a color input with live preview swatches:
- A text input for the hex value.
- A `<div class="preview background-preview">` with border and background color.
- A `<div class="preview text-preview">` with border and text color.

### `makeNumericInput(options)`

Sets `type = "number"` and delegates to `makeInput`.

### `makeDateInput(options)`

Sets `type = "date"` and delegates to `makeInput`.

### `getPostUrl(id, parameters)`

Returns a client link to a post. Optionally includes a `query` parameter.

### `getPostEditUrl(id, parameters)`

Returns a client link to the post edit page. Optionally includes a `query` parameter.

### `makePostLink(id, includeHash)`

Returns an `<a>` element linking to a post, or escaped text if the user has no `post_view` privilege. If `includeHash` is true, prefixes the ID with `@`.

### `makeTagLink(name, includeHash, includeCount, tag)`

Returns an `<a>` or `<span>` element for a tag link. Uses the tag's category for CSS class. If the user lacks `tag_view` privilege, wraps in a `<span>` instead. Supports `#` prefix and post count suffix.

### `makePoolLink(id, includeHash, includeCount, pool, name)`

Returns an `<a>` or `<span>` element for a pool link. Uses the pool's category for CSS class. Falls back to `"pool " + id` display name. Supports `#` prefix and post count suffix.

### `makeUserLink(user)`

Returns a `<span class="user">` containing a thumbnail and username. If `user.name` exists and the user has `user_view` privilege, wraps in a link. Falls back to `"Anonymous"` text.

### `makeFlexboxAlign(options)`

Returns 20 `<li class="flexbox-dummy">` elements for flexbox grid alignment.

### `makeAccessKey(html, key)`

Wraps the first occurrence of `key` (case-insensitive) in the HTML string with `<span class="access-key" data-accesskey="...">`.

### `_serializeElement(name, attributes)`

Serializes an element's attribute object into a string. Handles boolean attributes (present/absent), `false`/`undefined` (omitted), `0` (rendered as `"0"`), and escaped string values.

### `makeElement(name, attrs, ...content)`

Generates an HTML element string.
- With content: `<name attrs>content</name>`.
- Without content: `<name attrs/>`.

### `emptyContent(target)`

Removes all child nodes from `target` via `removeChild()` in a loop.

### `replaceContent(target, source)`

Empties `target` and appends children from `source`:
- If `source` is a `NodeList`, appends each child.
- If `source` is a `Node`, appends it directly.
- If `source` is `null`, does nothing.
- Otherwise, throws an error.

### `showMessage(target, message, className)`

Appends a styled message to the `.messages` container inside `target`:
- Creates a `<div class="message-wrapper">` with a `<div class="message <className>">` inside.
- The message text goes through innerHTML after newlines are converted to `<br/>`.
- Returns `true` if successful, `false` if no `.messages` container found.

### `appendExclamationMark()`

Prepends `"! "` to `document.title` if it does not already start with `!`. Stores the old title in `document.oldTitle`.

### `showError(target, message)`

Calls `showMessage` with `className = "error"`. Also calls `appendExclamationMark()`.

### `showSuccess(target, message)`

Calls `showMessage` with `className = "success"`.

### `showInfo(target, message)`

Calls `showMessage` with `className = "info"`.

### `clearMessages(target)`

Restores `document.title` from `document.oldTitle` if set. Empties all `.messages` containers inside `target`.

### `htmlToDom(html)`

Converts an HTML string to DOM node(s). Uses a wrap map for special HTML elements that require a parent context (`option`, `legend`, `area`, `param`, `thead`, `tr`, `td`, `col`). Uses a temporary `<div>` to parse the HTML.

Returns a single `Node` if one child was created, or a `NodeList` if multiple children were created.

### `getTemplate(templatePath)`

Returns a function `(ctx) => Node|NodeList` that renders a named template.

1. Looks up the template function in the `templates` module by `templatePath`.
2. If not found, throws an error.
3. Returns a function that:
   - Enriches `ctx` with all make* helper functions, URI functions, and format helpers.
   - Calls `templateFactory(ctx)` to get the HTML string.
   - Passes the HTML through `htmlToDom()`

### `decorateValidator(form)`

Adds form validation UI behavior:
- On click of the submit button, adds the class `"show-validation"` to the form (triggers CSS `:invalid` styling).
- On form submit, removes the `"show-validation"` class.

### `disableForm(form)`

Disables all `<input>` elements inside the form.

### `enableForm(form)`

Enables all `<input>` elements inside the form.

### `syncScrollPosition()`

Uses `requestAnimationFrame` to scroll to the position stored in `history.state.scrollX/scrollY`. If no scroll state, scrolls to `(0, 0)`.

### `slideDown(element)`

Animates an element from height 0 to its natural height using CSS transitions.

- Duration: 500ms.
- Sets `max-height: 0` and `overflow: hidden`.
- After 50ms, applies `transition` and sets `max-height` to the element's measured height.
- Resolves the promise after 500ms.

### `slideUp(element)`

Animates an element from its current height to height 0.

- Duration: 500ms.
- Sets `max-height` to the measured height, `overflow: hidden`, and applies transition.
- After 10ms, sets `max-height: 0`.
- Resolves the promise after 500ms.

### `monitorNodeRemoval(monitoredNode, callback)`

Creates a `MutationObserver` on `document.body`. Watches for `childList` mutations in the entire subtree. If `monitoredNode` is found in `mutation.removedNodes`, disconnects the observer and calls `callback()`.

## Exports

```js
{
    htmlToDom, getTemplate, emptyContent, replaceContent,
    enableForm, disableForm, decorateValidator,
    makeTagLink, makePostLink, makePoolLink,
    makeCheckbox, makeRadio, syncScrollPosition,
    slideDown, slideUp, monitorNodeRemoval,
    clearMessages, appendExclamationMark,
    showError, showSuccess, showInfo,
}
```
