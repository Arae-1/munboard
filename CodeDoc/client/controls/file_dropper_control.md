# FileDropperControl

## UI Element

Manages a drag-and-drop file upload area. Supports:
- File drops from the operating system.
- File selection via a hidden `<input type="file">`.
- URL text input with a confirm button.
- Paste events for files from the clipboard.

Uses the `file-dropper` template.

## Constructor

### `constructor(target, options)`
- `target`: The DOM element to replace with the dropper.
- `options`: Configuration object:
  - `extraText`: Additional instructional text.
  - `allowMultiple`: Whether multiple files can be selected.
  - `allowUrls`: Whether the URL input is shown.
  - `lock`: Whether the dropper shows selected file names and becomes locked after selection.
  - `urlPlaceholder`: Placeholder text for the URL input.

Sets up the template with a unique ID generated from `Math.random()`. Replaces the target's content with the dropper DOM.

## Public Methods

### `reset()`
Restores the dropper's inner HTML to the original template state (before any file names or URLs were shown). Dispatches a `reset` event.

## Private Methods

### `_emitFiles(files)`
Converts `files` to an array. If `options.lock` is true, replaces the dropper inner text with a comma-separated list of file names. Dispatches a `fileadd` CustomEvent with the file list.

### `_emitUrls(urls)`
Converts and trims `urls`. If `options.lock` is true, displays the URL filenames. Validates each URL with a regex (`/^https?:\/\/[^.]+\..+$/`). Shows an alert if a URL is invalid. Dispatches a `urladd` CustomEvent with the URL list.

## Event Handlers

### `_evtDragEnter(e)`
Adds the `active` class to the dropper node. Increments a counter to handle nested drag events.

### `_evtDragLeave(e)`
Decrements the counter. Removes the `active` class when the counter reaches zero.

### `_evtDragOver(e)`
Prevents the default browser behavior to allow drops.

### `_evtFileChange(e)`
Calls `_emitFiles` with the selected files from the hidden file input.

### `_evtDrop(e)`
Prevents default. Removes the `active` class. Validates:
- If no files are dropped, alerts "Only files are supported."
- If `allowMultiple` is false and more than one file is dropped, alerts "Cannot select multiple files."

Calls `_emitFiles` with the dropped files.

### `_evtPaste(e)`
Extracts file items from `clipboardData.items`. Filters for actual file objects. If `allowMultiple` is false and multiple files are pasted, alerts "Cannot select multiple files." Otherwise calls `_emitFiles`.

This handler is registered in two ways:
1. On the URL input if the page is NOT a post-upload page.
2. On `document.onpaste` if the page IS a post-upload page (`#post-upload` element exists).

### `_evtUrlInputKeyDown(e)`
If the key is "Enter", prevents default. Removes the `active` class. Splits the URL input by newlines and calls `_emitUrls`. Clears the URL input.

### `_evtUrlConfirmButtonClick(e)`
Prevents default. Removes the `active` class. Splits the URL input by newlines and calls `_emitUrls`. Clears the URL input.

## Communication

Dispatches these CustomEvents:
- `fileadd`: detail contains `{files: File[]}`
- `urladd`: detail contains `{urls: string[]}`
- `reset`: no detail

Controllers listen to these events to handle the file or URL data.

## State Management

- `_counter`: tracks nested drag enter/leave events.
- `_originalHtml`: stores the initial dropper HTML for reset.
- `_options.lock`: when true, the dropper displays the selected file/URL names.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
