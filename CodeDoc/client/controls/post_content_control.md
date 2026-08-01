# PostContentControl

## UI Element

Manages the main post content display area. Controls image and video sizing (fit modes). Handles viewport resizing. Moves sidebar buttons to the content area on small screens (<=800px width).

Uses the `post-content` template.

## Constructor

### `constructor(hostNode, post, viewportSizeCalculator, fitFunctionOverride)`
- `hostNode`: The DOM container for the post content.
- `post`: A `Post` model instance.
- `viewportSizeCalculator`: A function that returns `[width, height]` for the available viewport.
- `fitFunctionOverride`: Optional. A string like `"fit-both"`, `"fit-original"`, `"fit-width"`, or `"fit-height"` that overrides user settings.

Reads the user's fit mode from `settings.get().fitMode`. Maps the mode name to the corresponding fit method. Defaults to `fitBoth`.

Listens to `changeContent` events on the post model.

## Public Methods

### `disableOverlay()`
Hides the `.post-overlay` element within the host node.

### `fitWidth()`
Updates `_currentFitFunction`. Calculates the height from the post's `canvasWidth` and `canvasHeight` aspect ratio. Clamps width to the original if `upscaleSmallPosts` setting is false. Calls `_resize`.

### `fitHeight()`
Updates `_currentFitFunction`. Calculates the width from the aspect ratio. Clamps height to the original if `upscaleSmallPosts` is false. Calls `_resize`.

### `fitBoth()`
Updates `_currentFitFunction`. Compares the viewport aspect ratio to the post aspect ratio. Scales to fit either width or height, whichever constrains the image more. Clamps dimensions to original if `upscaleSmallPosts` is false.

### `fitOriginal()`
Updates `_currentFitFunction`. Calls `_resize` with the post's original canvas width and height.

## Getters

### `_viewportWidth`
Returns the first element from `viewportSizeCalculator()`.

### `_viewportHeight`
Returns the second element from `viewportSizeCalculator()`.

## Private Methods

### `_resize(width, height)`
Sets `width` and `height` inline styles on the post content node and all child elements with the class `resize-listener`.

### `_refreshSize()`
Handles responsive layout for screens <= 800px:
- Moves the `.sidebar > .buttons` element into `.content` after the `.post-container`.
- Moves `after-mobile-controls` elements accordingly.

For screens > 800px:
- Moves `.content > .buttons` back into `.sidebar`.

Then calls the current fit function.

### `_install()`
Calls `_reinstall` to render the template. Registers `_refreshSize` on `optimizedResize` for window resize events. Monitors the host node for DOM removal to call `_uninstall`.

### `_reinstall()`
Renders the `post-content` template with:
- `post`: the post model.
- `autoplay`: from `settings.get().autoplayVideos`.

Adds the `transparency-grid` class if that setting is enabled. Replaces or appends the new content node. Stores a reference to `_postContentNode`. Calls `_refreshSize`.

### `_uninstall()`
Removes the resize listener from `optimizedResize`.

## Event Handlers

### `_evtPostContentChange(e)`
Updates the local post reference from the event detail. Calls `post.mutateContentUrl()` to regenerate the content URL. Calls `_reinstall` to re-render.

## Communication

- **Listens to**: `changeContent` events from the Post model.
- **Listens to**: `optimizedResize` for window resize.
- **Uses**: `settings.get()` for `fitMode`, `upscaleSmallPosts`, `autoplayVideos`, and `transparencyGrid`.

## State Management

- `_currentFitFunction`: reference to the active fit method.
- `_postContentNode`: reference to the current content DOM node.
- `_post`: reference to the current Post model.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
