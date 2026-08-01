# Client Utilities

Utilities are stateless helper modules that provide cross-cutting functionality. They handle DOM manipulation, formatting, keyboard shortcuts, markdown rendering, HTTP URI construction, touch gestures, progress indication, theming, search, resize optimization, and browser polyfills.

## Utility Modules

### 1. `misc` — General Helpers

The largest utility. Provides 17 functions used across the codebase.

| Function | Purpose |
|---|---|
| `decamelize(str, sep)` | Converts camelCase to lowercase-separated (e.g., `"myVar"` → `"my-var"`) |
| `dateToRfc3339(dateStr)` | Converts `YYYY-MM-DD` to RFC 3339 UTC datetime |
| `range(start, end, step)` | Generator yielding numbers in a range |
| `formatFileSize(bytes)` | Formats bytes to human-readable (B, K, M, G using base 1024) |
| `formatRelativeTime(time)` | Formats ISO time to relative string (e.g., "3 hours ago") |
| `formatMarkdown(text)` | Delegates to `markdown.formatMarkdown()` |
| `formatInlineMarkdown(text)` | Delegates to `markdown.formatInlineMarkdown()` |
| `splitByWhitespace(str)` | Splits by whitespace, removes empties |
| `unindent(callSite)` | Removes consistent leading whitespace from template literals |
| `enableExitConfirmation()` | Sets `window.onbeforeunload` to warn before navigation |
| `disableExitConfirmation()` | Clears `window.onbeforeunload` |
| `confirmPageExit()` | Calls `window.confirm()` if exit confirmation is active |
| `makeCssName(text, suffix)` | Generates CSS-safe class name (e.g., `"tag-my_name"`) |
| `arraysDiffer(a, b, orderImportant)` | Compares arrays for differences |
| `objectsDiffer(a, b)` | Compares plain objects by sorted keys and values |
| `escapeSearchTerm(text)` | Escapes `:` and `.` for safe search queries |
| `dataURItoBlob(dataURI)` | Converts data URI to Blob |
| `getPrettyName(tag)` | Replaces underscores with spaces if setting enabled |
| `wildcardMatch(pattern, str)` | Matches string against `*` wildcard pattern |
| `matchingNames(text, names)` | Finds matching names (prefix first, then substring for text >= 3 chars) |
| `preloadPostImages(post)` | Preloads image/animation content into browser cache |

### Cross-Cutting Concerns

**Exit confirmation.** `enableExitConfirmation()`, `disableExitConfirmation()`, and `confirmPageExit()` form a system used by all edit/upload controllers. Controllers call enable on form change and disable on successful save. The router exit handler calls `confirmPageExit()` before navigation.

**Array comparison.** `arraysDiffer()` and `objectsDiffer()` are used by all diff-based save methods (Post, Tag, Pool, User, etc.) to determine which fields changed and need to be sent to the API.

**Name matching.** `matchingNames()` is used by autocomplete controls to find the best-matching tag/pool name from a list of aliases.

**CSS class generation.** `makeCssName()` is used by tags and pools modules to generate category color classes. It is also used by input controls to apply category-based styling to tag and pool list items.

---

### 2. `views` — DOM Manipulation

The largest utility by function count. Provides DOM creation, template rendering, messaging, and animation.

| Function | Purpose |
|---|---|
| `htmlToDom(html)` | Converts HTML string to DOM nodes. Handles special elements (option, legend, tr, td, etc.) |
| `getTemplate(name)` | Returns a function that renders a named template from the templates module |
| `emptyContent(target)` | Removes all child nodes |
| `replaceContent(target, source)` | Replaces content with nodes from a NodeList, Node, or null |
| `enableForm(form)` | Enables all inputs in a form |
| `disableForm(form)` | Disables all inputs in a form |
| `decorateValidator(form)` | Adds CSS validation class on submit attempt |
| `showError(target, msg)` | Displays an error message. Also adds `"! "` to page title |
| `showSuccess(target, msg)` | Displays a success message |
| `showInfo(target, msg)` | Displays an info message |
| `clearMessages(target)` | Clears all messages and restores page title |
| `syncScrollPosition()` | Scrolls to saved position from history state |
| `slideDown(element)` | CSS transition animation: 0 to natural height (500ms) |
| `slideUp(element)` | CSS transition animation: current height to 0 (500ms) |
| `monitorNodeRemoval(node, cb)` | MutationObserver that calls callback when node is removed from DOM |

**Form element generators:** `makeRadio`, `makeCheckbox`, `makeSelect`, `makeInput`, `makeButton`, `makeTextInput`, `makeTextarea`, `makePasswordInput`, `makeEmailInput`, `makeColorInput`, `makeNumericInput`, `makeDateInput`

**Link generators:** `makePostLink`, `makeTagLink`, `makePoolLink`, `makeUserLink`, `makeFlexboxAlign`, `makeThumbnail`, `makeAccessKey`, `makeFileSize`, `makeMarkdown`, `makeRelativeTime`, `makeElement`, `getPostUrl`, `getPostEditUrl`

Templates rendered through `getTemplate()` are enriched with all these generators and helper functions before the template factory is called.

---

### 3. `markdown` — Markdown Rendering

Renders markdown text to sanitized HTML using `marked` and `DOMPurify`.

**Pipeline:**
1. `escapeHtml(text)` — escape HTML special characters
2. Apply `preprocess()` from 9 wrapper classes
3. `marked.parse()` with custom image renderer (supports `=WIDTHxHEIGHT` dimension syntax)
4. Apply `postprocess()` from wrappers in reverse order
5. `DOMPurify.sanitize()` — remove unsafe HTML

**Wrapper classes:**

| Wrapper | Purpose |
|---|---|
| `SjisWrapper` | Preserves `[sjis]...[/sjis]` blocks |
| `TildeWrapper` | Preserves escaped tildes `\~` |
| `TagPermalinkFixWrapper` | Prevents `#tag` from becoming markdown headings |
| `EntityPermalinkWrapper` | Converts `@1234` → post link, `+user` → user link, `#tag` → search link |
| `SearchPermalinkWrapper` | Converts `[search]query[/search]` → search link |
| `SpoilersWrapper` | Converts `[spoiler]text[/spoiler]` → `<span class="spoiler">` |
| `SmallWrapper` | Converts `[small]text[/small]` → `<small>` |
| `StrikeThroughWrapper` | Converts `~~text~~` → `<del>` |
| `FaviconWrapper` | Converts `[icon]domain[/icon]` → favicon image link |

**Two rendering modes:**
- `formatMarkdown(text)` — full markdown (block and inline, all wrappers)
- `formatInlineMarkdown(text)` — inline only, excludes SjisWrapper and TagPermalinkFixWrapper

---

### 4. `keyboard` — Keyboard Shortcut Manager

Wraps the `mousetrap` library with settings-aware binding and pause support.

| Function | Purpose |
|---|---|
| `bind(hotkey, func)` | Binds a global shortcut. Respects `keyboardShortcuts` setting |
| `bindElement(element, hotkey, func)` | Binds a shortcut to a specific DOM element |
| `unbind(hotkey)` | Unbinds a shortcut unconditionally |
| `pause()` | Suppresses all keyboard shortcuts |
| `unpause()` | Resumes keyboard shortcuts |

The `mousetrap.prototype.stopCallback` is overridden. When `paused` is true, all shortcuts are suppressed. This is used by `PostNotesOverlayControl` during note drawing to prevent global shortcuts from interfering.

---

### 5. `uri` — URL Building

| Function | Purpose |
|---|---|
| `formatApiLink(...values)` | Builds API URL path (`/api/...`). Last argument may be query object |
| `formatClientLink(...values)` | Builds client URL path. Query params joined with `;` |
| `escapeParam(text)` | URL-encodes a parameter |
| `unescapeParam(text)` | URL-decodes a parameter |
| `escapeTagName(text)` | Escapes `:` and `.` in tag names for search queries |
| `extractHostname(url)` | Extracts hostname from URL |
| `extractRootDomain(url)` | Extracts root domain (handles ccTLDs) |

---

### 6. `progress` — Loading Progress Bar

Wraps the `nprogress` library with nested request tracking.

| Function | Purpose |
|---|---|
| `start()` | Shows progress bar. Increments nesting counter |
| `done()` | Decrements counter. Advances bar. Hides when all requests complete |

The nesting counter prevents the progress bar from flickering or completing early during concurrent API requests. Called by `Api._rawRequest()` before and after each HTTP request.

---

### 7. `accent` — Theme Accent Color

| Function | Purpose |
|---|---|
| `apply(color)` | Sets `--accent-color` CSS custom property on `<html>`. Updates favicon SVG with accent color |

The favicon is a hardcoded SVG cog icon. The accent color is injected into a gradient stop-color and a path fill attribute. The SVG is serialized as a data URI and set on the favicon `<link>` element.

---

### 8. `optimized_resize` — Debounced Resize Events

Batches window resize callbacks using `requestAnimationFrame`.

| Function | Purpose |
|---|---|
| `add(callback)` | Registers a resize callback |
| `remove(callback)` | Unregisters a callback |

All callbacks fire in a single rAF frame instead of on every `resize` event. Used by `PostContentControl` to recalculate image sizes.

---

### 9. `search` — Search Input Helper

| Function | Purpose |
|---|---|
| `searchInputNodeFocusHelper(inputNode)` | Binds the `q` keyboard shortcut to focus the search input and move cursor to end |

---

### 10. `touch` — Touch Gesture Detection

Provides swipe detection for touch devices.

| Export | Purpose |
|---|---|
| `Touch(target, swipeLeft, swipeRight, swipeUp, swipeDown, getScrollElement)` | Class that attaches touch event listeners to detect directional swipes |
| `direction` object | Constants: `NONE`, `LEFT`, `RIGHT`, `DOWN`, `UP` |

Respects scroll boundaries: left/right swipes only trigger at scroll edges. Handles nested touch events via start/move/end coordinate tracking.

---

### 11. `polyfill` — Browser Polyfills

Loaded early in `main.js`. Extends native prototypes:

| Target | Extension | Purpose |
|---|---|---|
| `NodeList` | `[Symbol.iterator]` | Makes NodeLists iterable |
| `NodeList` | `querySelector()` | Searches across all nodes in a list |
| `NodeList` | `querySelectorAll()` | Searches all nodes, returns combined array |
| `Node` | `prependChild(child)` | Inserts child as first child |
| `Promise` | `always(callback)` | Executes on both resolve and reject (jQuery-compatible) |
| `Promise` | `abort()` | No-op default abort method |
| `Number` | `between(a, b, inclusive)` | Range check method |
| `Date` | `addDays(days)` | Returns new Date with days added |
