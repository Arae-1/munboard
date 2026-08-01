# PostNotesOverlayControl

## UI Element

Manages an SVG overlay on top of post content for drawing and editing polygon-based notes. The SVG has a `viewBox` of "0 0 1 1" and `preserveAspectRatio` of "none", so all coordinates are in unit space (0-1). Every note is a `<polygon>` element inside a `<g>` group. Edge points are `<ellipse>` elements for grab handles.

Also manages a floating `<div class="note-text">` that shows markdown-rendered note text near the note centroid when hovering.

## SVG Namespace and Constants

| Constant | Value |
|---|---|
| `svgNS` | `"http://www.w3.org/2000/svg"` |
| `snapThreshold` | `10` (pixels) |
| `circleSize` | `10` (pixels, diameter of edge handles) |

## Helper Functions (Module Scope)

### `_getDistance(point1, point2)`
Euclidean distance between two `Point` instances.

### `_setNodeState(node, stateName)`
Sets the `data-state` attribute on an SVG element.

### `_clearEditedNote(hostNode)`
Finds the element with `data-state="editing"` and clears its state attribute. Returns true if an element was cleared.

### `_getNoteCentroid(note)`
Computes the geometric centroid of a note polygon using the polygon area algorithm. Returns a `Point`.

### `_getNoteSize(note)`
Returns a `Point` representing the width and height of the note's bounding box.

## State Machine — Base Class

### `class State`
Abstract base for all note interaction states.

#### `constructor(control, stateName)`
Sets `data-state` on the host node and text node to the state name.

#### `get canShowNoteText()`
Returns `false` by default. States that allow showing text override this to `true`.

#### Event Handlers (all no-op by default)
- `evtCanvasKeyDown(e)`
- `evtNoteMouseDown(e, hoveredNote)`
- `evtCanvasMouseDown(e)`
- `evtCanvasMouseMove(e)`
- `evtCanvasMouseUp(e)`

#### Utility Methods
- `_getScreenPoint(point)`: Converts a unit-space `Point` to pixel coordinates using `control.boundingBox`.
- `_snapPoints(targetPoint, referencePoint)`: Snaps `targetPoint` to `referencePoint` if their screen distance is below `snapThreshold`.
- `_createNote()`: Creates a new `Note` model and calls `_createPolygonNode` to create its SVG elements.
- `_getPointFromEvent(e)`: Converts a mouse event to a unit-space `Point`.

## Concrete States

### `class ReadOnlyState extends State`
State name: `"read-only"`. Notes are visible but not interactive. Shows note text on hover. Calls `_clearEditedNote` and `keyboard.unpause()`. `canShowNoteText` returns `true`.

### `class PassiveState extends State`
State name: `"passive"`. Notes are hoverable and clickable but not editable. Clicking a note transitions to `SelectedState`. `canShowNoteText` returns `true`.

- `evtNoteMouseDown`: transitions to `SelectedState`.

### `class ActiveState extends State`
Abstract base for all states that involve active note manipulation. State name passed by subclass. Calls `_clearEditedNote` and `keyboard.pause()`. If a note is provided, dispatches a `focus` event and sets `data-state="editing"` on the note's group node.

### `class SelectedState extends ActiveState`
State name: `"selected"`. A single note is selected and can be moved, scaled, or have its points moved.

#### `evtCanvasKeyDown(e)`
Moves the note with arrow keys (1px per keypress, 10px with Ctrl held). Shift+arrow scales the note instead. Prevents event propagation.

#### `evtNoteMouseDown(e, hoveredNote)`
- If Shift is held: transitions to `ScalingNoteState`.
- If clicking a different note: transitions to `SelectedState` for that note.
- Otherwise, starts a 100ms timeout. After timeout, checks if the click is near any polygon vertex (within `circleSize` px). If so, transitions to `MovingPointState`. Otherwise, transitions to `MovingNoteState`.

#### `evtCanvasMouseDown(e)`
Same logic as `evtNoteMouseDown` but with no timeout delay. If Shift, goes to `ScalingNoteState`. If near a vertex, goes to `MovingPointState`. Otherwise, goes to `PassiveState` (deselect).

#### `evtCanvasMouseMove(e)`
Highlights polygon vertices with the `nearby` class when the mouse is within `circleSize` px.

#### `evtCanvasMouseUp(e)`
Clears the click timeout.

#### `_moveEditedNote(x, y)`
Offsets all polygon points by `x / boundingBox.width` and `y / boundingBox.height`.

#### `_scaleEditedNote(x, y)`
Scales the note around its centroid. Computes the new target size and applies a scale factor to each point relative to the centroid.

### `class MovingPointState extends ActiveState`
State name: `"moving-point"`. A single polygon vertex is being dragged.

Saves the original point position. In `evtCanvasKeyDown`, if Escape is pressed, restores the original position and transitions back to `SelectedState`. In `evtCanvasMouseMove`, updates the point position. In `evtCanvasMouseUp`, returns to `SelectedState`.

### `class MovingNoteState extends ActiveState`
State name: `"moving-note"`. The entire note is being dragged.

Saves a copy of the original polygon. In `evtCanvasKeyDown`, if Escape is pressed, restores all points and returns to `SelectedState`. In `evtCanvasMouseMove`, offsets all points by the mouse delta. In `evtCanvasMouseUp`, returns to `SelectedState`.

### `class ScalingNoteState extends ActiveState`
State name: `"scaling-note"`. The note is being scaled from an anchor point (mouse position on Shift+click).

Saves the original polygon, mouse point, and note size. In `evtCanvasKeyDown`, Escape restores the original shape. In `evtCanvasMouseMove`, scales each point relative to the anchor mouse position using the formula: `anchor.x + (originalPoint.x - anchor.x) * (1 + deltaX / originalSize.x)`. In `evtCanvasMouseUp`, returns to `SelectedState`.

### `class ReadyToDrawState extends ActiveState`
State name: `"ready-to-draw"`. The user can start drawing.

- `evtNoteMouseDown`: transitions to `SelectedState` (edit instead of draw).
- `evtCanvasMouseDown`: If Shift, transitions to `DrawingRectangleState`. Otherwise, transitions to `DrawingPolygonState`.

### `class DrawingRectangleState extends ActiveState`
State name: `"drawing-rectangle"`. Drawing a rectangle by dragging.

Creates a note with 4 vertices at the initial click point. In `evtCanvasMouseMove`, updates vertices 2 and 3 to form the rectangle. In `evtCanvasMouseUp`, if the rectangle is too small (width < 20 and height < 20 pixels), discards it and returns to `ReadyToDrawState`. Otherwise, adds the note to the model and transitions to `SelectedState`.

### `class DrawingPolygonState extends ActiveState`
State name: `"drawing-polygon"`. Drawing a freeform polygon by clicking vertices.

Creates a note with 2 points at the initial click. Each subsequent click adds a new vertex. The last vertex follows the mouse cursor.

- `evtCanvasKeyDown`: Escape removes the last point. If only 1 point remains, cancels drawing. Enter finishes the polygon.
- `evtCanvasMouseDown`: If near the first vertex (within `snapThreshold`), finishes the polygon. Otherwise, adds a new vertex.
- `evtCanvasMouseMove`: Updates the last vertex position. If Shift is held, snaps the segment to horizontal or vertical alignment (4-directional constraint). Snaps the last point to the first point if close.
- `_cancel()`: Deletes the SVG nodes and returns to `ReadyToDrawState`.
- `_finish()`: Removes the temporary last point. If fewer than 3 points remain, cancels. Otherwise, deletes the DOM node (single), adds the note to the model, and transitions to `SelectedState`.

## Main Control — PostNotesOverlayControl

### `constructor(hostNode, post)`
- `hostNode`: The container element (typically the post content area).
- `post`: The Post model.

Creates an SVG element with class `notes-overlay` and `resize-listener`. Iterates existing notes and creates polygon nodes. Appends the SVG to the host node.

Creates a floating text node (`<div class="note-text">`) appended to `document.body`. Contains a `<div class="wrapper">` for markdown-rendered content.

Listens to:
- `change` on the Post model
- `remove` and `add` on `post.notes`
- `keydown` on `document`
- `mousedown`, `mouseup`, `mousemove` on the SVG

Monitors the host node for DOM removal to clean up SVG, keyboard listener, and text node.

Starts in `ReadOnlyState`.

### Public Methods

#### `switchToPassiveEdit()`
Transitions to `PassiveState`. Notes become clickable but not editable.

#### `switchToDrawing()`
Transitions to `ReadyToDrawState`. Clicking on the canvas starts drawing a new note.

### Getter

#### `boundingBox`
Returns `this._hostNode.getBoundingClientRect()`.

### Event Handlers (Main Control)

#### `_evtPostChange(e)`
Clears all SVG children. Updates the post reference. Recreates polygon nodes for all notes in the new post.

#### `_evtCanvasKeyDown(e)`
Ignores key events when the active element is a textarea, input, or select. Delegates to the current state's `evtCanvasKeyDown`.

#### `_evtCanvasMouseDown(e)`
Prevents default. Ignores non-left clicks. Uses `document.elementFromPoint` to determine if a note polygon was clicked. Delegates to `_state.evtNoteMouseDown` or `_state.evtCanvasMouseDown`.

#### `_evtCanvasMouseUp(e)`
Delegates to `_state.evtCanvasMouseUp`.

#### `_evtCanvasMouseMove(e)`
Delegates to `_state.evtCanvasMouseMove`.

#### `_evtNoteMouseEnter(e, note)`
If the current state allows showing text, calls `_showNoteText(note)`.

#### `_evtNoteMouseLeave(e)`
If the mouse left both the SVG and the text node, calls `_hideNoteText()`.

#### `_showNoteText(note)`
Renders `note.text` as markdown into the text node. Positions the text node at the centroid of the polygon.

#### `_hideNoteText()`
Hides the text node.

### SVG DOM Methods

#### `_updatePolygonNotePoints(note)`
Sets the `points` attribute of the SVG `<polygon>` from the note's polygon data.

#### `_createEdgeNode(point, groupNode)`
Creates an `<ellipse>` element for a polygon vertex. Sizes it at `circleSize / 2` in both axes, scaled to unit space. Stores a reference on `point.edgeNode`. Appends to the group.

#### `_deleteEdgeNode(point, note)`
Updates the polygon points attribute. Removes the edge ellipse from the DOM.

#### `_updateEdgeNode(point, note)`
Updates the polygon points and the ellipse `cx`/`cy` attributes.

#### `_deleteDomNode(note)`
Removes the note's `<g>` group from the SVG.

#### `_createPolygonNode(note)`
Creates the full SVG structure for a note:
1. A `<g>` group stored on `note.groupNode`.
2. A `<polygon>` with `vector-effect="non-scaling-stroke"`, `stroke-alignment="inside"`, and `mouseenter`/`mouseleave` handlers.
3. An `<ellipse>` edge node for each polygon vertex, via `_createEdgeNode`.
4. Event listeners on `note.polygon` for `change`, `remove`, and `add` to update or recreate edge nodes.

## Communication

- **Dispatches**: `focus`, `blur`, and `change` CustomEvents.
- **Listens to**: `change` on Post, `add`/`remove` on `post.notes`, `change`/`remove`/`add` on `note.polygon`.
- **Uses**: `keyboard.pause()` and `keyboard.unpause()` to suppress global keyboard shortcuts during drawing.

## Canvas/SVG Drawing Logic

All coordinates use a unit space (0 to 1). The SVG `viewBox` is "0 0 1 1" and `preserveAspectRatio` is "none", which stretches the SVG to fill its container. All polygon point math operates in this unit space.

The `PostNotesOverlayControl` converts between:
- **Mouse events** → **Unit coordinates**: `(clientX - boundingBox.left) / boundingBox.width`
- **Unit coordinates** → **Screen coordinates**: `point * boundingBox.width/height`

Edge handles (`<ellipse>`) use fixed pixel sizes divided by the bounding box dimensions to maintain consistent visual size regardless of image size.

## Keyboard Shortcuts (during note interaction)

| State | Key | Action |
|---|---|---|
| SelectedState | Arrow keys | Move note 1px (or 10px with Ctrl) |
| SelectedState | Shift+Arrow | Scale note |
| MovingPointState | Escape | Cancel, restore point position |
| MovingNoteState | Escape | Cancel, restore all positions |
| ScalingNoteState | Escape | Cancel, restore original shape |
| DrawingPolygonState | Escape | Remove last point / cancel |
| DrawingPolygonState | Enter | Finish polygon |

## State Management

- `_state`: The current state object. Determines all behavior.
- `_post`: The Post model reference.
- `_svgNode`: The SVG overlay element.
- `_textNode`: The floating note text element.
