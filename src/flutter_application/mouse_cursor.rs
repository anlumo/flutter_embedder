use serde::{Deserialize, Serialize};
use winit::window::CursorIcon;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MouseCursor {
    ActivateSystemCursor { device: i32, kind: MouseCursorKind },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MouseCursorKind {
    /// Hide the cursor.
    ///
    /// Any cursor other than [none] or [MouseCursor.uncontrolled] unhides the
    /// cursor.
    None,

    // STATUS
    /// The platform-dependent basic cursor.
    ///
    /// Typically the shape of an arrow.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_DEFAULT, TYPE_ARROW
    ///  * Web: default
    ///  * Windows: IDC_ARROW
    ///  * Windows UWP: CoreCursorType::Arrow
    ///  * Linux: default
    ///  * macOS: arrowCursor
    Basic,

    /// A cursor that emphasizes an element being clickable, such as a hyperlink.
    ///
    /// Typically the shape of a pointing hand.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HAND
    ///  * Web: pointer
    ///  * Windows: IDC_HAND
    ///  * Windows UWP: CoreCursorType::Hand
    ///  * Linux: pointer
    ///  * macOS: pointingHandCursor
    Click,

    /// A cursor indicating an operation that will not be carried out.
    ///
    /// Typically the shape of a circle with a diagonal line. May fall back to
    /// [noDrop].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_NO_DROP
    ///  * Web: not-allowed
    ///  * Windows: IDC_NO
    ///  * Windows UWP: CoreCursorType::UniversalNo
    ///  * Linux: not-allowed
    ///  * macOS: operationNotAllowedCursor
    ///
    /// See also:
    ///
    ///  * [noDrop], which indicates somewhere that the current item may not be
    ///    dropped.
    Forbidden,

    /// A cursor indicating the status that the program is busy and therefore
    /// can not be interacted with.
    ///
    /// Typically the shape of an hourglass or a watch.
    ///
    /// This cursor is not available as a system cursor on macOS. Although macOS
    /// displays a "spinning ball" cursor when busy, it's handled by the OS and not
    /// exposed for applications to choose.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_WAIT
    ///  * Windows: IDC_WAIT
    ///  * Web: wait
    ///  * Linux: wait
    ///
    /// See also:
    ///
    ///  * [progress], which is similar to [wait] but the program can still be
    ///    interacted with.
    Wait,

    /// A cursor indicating the status that the program is busy but can still be
    /// interacted with.
    ///
    /// Typically the shape of an arrow with an hourglass or a watch at the corner.
    /// Does *not* fall back to [wait] if unavailable.
    ///
    /// Corresponds to:
    ///
    ///  * Web: progress
    ///  * Windows: IDC_APPSTARTING
    ///  * Linux: progress
    ///
    /// See also:
    ///
    ///  * [wait], which is similar to [progress] but the program can not be
    ///    interacted with.
    Progress,

    /// A cursor indicating somewhere the user can trigger a context menu.
    ///
    /// Typically the shape of an arrow with a small menu at the corner.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_CONTEXT_MENU
    ///  * Web: context-menu
    ///  * Linux: context-menu
    ///  * macOS: contextualMenuCursor
    ContextMenu,

    /// A cursor indicating help information.
    ///
    /// Typically the shape of a question mark, or an arrow therewith.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HELP
    ///  * Windows: IDC_HELP
    ///  * Windows UWP: CoreCursorType::Help
    ///  * Web: help
    ///  * Linux: help
    Help,

    // SELECTION
    /// A cursor indicating selectable text.
    ///
    /// Typically the shape of a capital I.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TEXT
    ///  * Web: text
    ///  * Windows: IDC_IBEAM
    ///  * Windows UWP: CoreCursorType::IBeam
    ///  * Linux: text
    ///  * macOS: IBeamCursor
    Text,

    /// A cursor indicating selectable vertical text.
    ///
    /// Typically the shape of a capital I rotated to be horizontal. May fall back
    /// to [text].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_VERTICAL_TEXT
    ///  * Web: vertical-text
    ///  * Linux: vertical-text
    ///  * macOS: IBeamCursorForVerticalLayout
    VerticalText,

    /// A cursor indicating selectable table cells.
    ///
    /// Typically the shape of a hollow plus sign.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_CELL
    ///  * Web: cell
    ///  * Linux: cell
    Cell,

    /// A cursor indicating precise selection, such as selecting a pixel in a
    /// bitmap.
    ///
    /// Typically the shape of a crosshair.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_CROSSHAIR
    ///  * Web: crosshair
    ///  * Windows: IDC_CROSS
    ///  * Windows UWP: CoreCursorType::Cross
    ///  * Linux: crosshair
    ///  * macOS: crosshairCursor
    Precise,

    // DRAG-AND-DROP
    /// A cursor indicating moving something.
    ///
    /// Typically the shape of four-way arrow. May fall back to [allScroll].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_ALL_SCROLL
    ///  * Windows: IDC_SIZEALL
    ///  * Windows UWP: CoreCursorType::SizeAll
    ///  * Web: move
    ///  * Linux: move
    Move,

    /// A cursor indicating something that can be dragged.
    ///
    /// Typically the shape of an open hand.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_GRAB
    ///  * Web: grab
    ///  * Linux: grab
    ///  * macOS: openHandCursor
    Grab,

    /// A cursor indicating something that is being dragged.
    ///
    /// Typically the shape of a closed hand.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_GRABBING
    ///  * Web: grabbing
    ///  * Linux: grabbing
    ///  * macOS: closedHandCursor
    Grabbing,

    /// A cursor indicating somewhere that the current item may not be dropped.
    ///
    /// Typically the shape of a hand with a [forbidden] sign at the corner. May
    /// fall back to [forbidden].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_NO_DROP
    ///  * Web: no-drop
    ///  * Windows: IDC_NO
    ///  * Windows UWP: CoreCursorType::UniversalNo
    ///  * Linux: no-drop
    ///  * macOS: operationNotAllowedCursor
    ///
    /// See also:
    ///
    ///  * [forbidden], which indicates an action that will not be carried out.
    NoDrop,

    /// A cursor indicating that the current operation will create an alias of, or
    /// a shortcut of the item.
    ///
    /// Typically the shape of an arrow with a shortcut icon at the corner.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_ALIAS
    ///  * Web: alias
    ///  * Linux: alias
    ///  * macOS: dragLinkCursor
    Alias,

    /// A cursor indicating that the current operation will copy the item.
    ///
    /// Typically the shape of an arrow with a boxed plus sign at the corner.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_COPY
    ///  * Web: copy
    ///  * Linux: copy
    ///  * macOS: dragCopyCursor
    Copy,

    /// A cursor indicating that the current operation will result in the
    /// disappearance of the item.
    ///
    /// Typically the shape of an arrow with a cloud of smoke at the corner.
    ///
    /// Corresponds to:
    ///
    ///  * macOS: disappearingItemCursor
    Disappearing,

    // RESIZING AND SCROLLING
    /// A cursor indicating scrolling in any direction.
    ///
    /// Typically the shape of a dot surrounded by 4 arrows.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_ALL_SCROLL
    ///  * Windows: IDC_SIZEALL
    ///  * Windows UWP: CoreCursorType::SizeAll
    ///  * Web: all-scroll
    ///  * Linux: all-scroll
    ///
    /// See also:
    ///
    ///  * [move], which indicates moving in any direction.
    AllScroll,

    /// A cursor indicating resizing an object bidirectionally from its left or
    /// right edge.
    ///
    /// Typically the shape of a bidirectional arrow pointing left and right.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HORIZONTAL_DOUBLE_ARROW
    ///  * Web: ew-resize
    ///  * Windows: IDC_SIZEWE
    ///  * Windows UWP: CoreCursorType::SizeWestEast
    ///  * Linux: ew-resize
    ///  * macOS: resizeLeftRightCursor
    ResizeLeftRight,

    /// A cursor indicating resizing an object bidirectionally from its top or
    /// bottom edge.
    ///
    /// Typically the shape of a bidirectional arrow pointing up and down.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_VERTICAL_DOUBLE_ARROW
    ///  * Web: ns-resize
    ///  * Windows: IDC_SIZENS
    ///  * Windows UWP: CoreCursorType::SizeNorthSouth
    ///  * Linux: ns-resize
    ///  * macOS: resizeUpDownCursor
    ResizeUpDown,

    /// A cursor indicating resizing an object bidirectionally from its top left or
    /// bottom right corner.
    ///
    /// Typically the shape of a bidirectional arrow pointing upper left and lower right.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_LEFT_DIAGONAL_DOUBLE_ARROW
    ///  * Web: nwse-resize
    ///  * Windows: IDC_SIZENWSE
    ///  * Windows UWP: CoreCursorType::SizeNorthwestSoutheast
    ///  * Linux: nwse-resize
    ResizeUpLeftDownRight,

    /// A cursor indicating resizing an object bidirectionally from its top right or
    /// bottom left corner.
    ///
    /// Typically the shape of a bidirectional arrow pointing upper right and lower left.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_RIGHT_DIAGONAL_DOUBLE_ARROW
    ///  * Windows: IDC_SIZENESW
    ///  * Windows UWP: CoreCursorType::SizeNortheastSouthwest
    ///  * Web: nesw-resize
    ///  * Linux: nesw-resize
    ResizeUpRightDownLeft,

    /// A cursor indicating resizing an object from its top edge.
    ///
    /// Typically the shape of an arrow pointing up. May fallback to [resizeUpDown].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_VERTICAL_DOUBLE_ARROW
    ///  * Web: n-resize
    ///  * Windows: IDC_SIZENS
    ///  * Windows UWP: CoreCursorType::SizeNorthSouth
    ///  * Linux: n-resize
    ///  * macOS: resizeUpCursor
    ResizeUp,

    /// A cursor indicating resizing an object from its bottom edge.
    ///
    /// Typically the shape of an arrow pointing down. May fallback to [resizeUpDown].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_VERTICAL_DOUBLE_ARROW
    ///  * Web: s-resize
    ///  * Windows: IDC_SIZENS
    ///  * Windows UWP: CoreCursorType::SizeNorthSouth
    ///  * Linux: s-resize
    ///  * macOS: resizeDownCursor
    ResizeDown,

    /// A cursor indicating resizing an object from its left edge.
    ///
    /// Typically the shape of an arrow pointing left. May fallback to [resizeLeftRight].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HORIZONTAL_DOUBLE_ARROW
    ///  * Web: w-resize
    ///  * Windows: IDC_SIZEWE
    ///  * Windows UWP: CoreCursorType::SizeWestEast
    ///  * Linux: w-resize
    ///  * macOS: resizeLeftCursor
    ResizeLeft,

    /// A cursor indicating resizing an object from its right edge.
    ///
    /// Typically the shape of an arrow pointing right. May fallback to [resizeLeftRight].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HORIZONTAL_DOUBLE_ARROW
    ///  * Web: e-resize
    ///  * Windows: IDC_SIZEWE
    ///  * Windows UWP: CoreCursorType::SizeWestEast
    ///  * Linux: e-resize
    ///  * macOS: resizeRightCursor
    ResizeRight,

    /// A cursor indicating resizing an object from its top-left corner.
    ///
    /// Typically the shape of an arrow pointing upper left. May fallback to [resizeUpLeftDownRight].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_LEFT_DIAGONAL_DOUBLE_ARROW
    ///  * Web: nw-resize
    ///  * Windows: IDC_SIZENWSE
    ///  * Windows UWP: CoreCursorType::SizeNorthwestSoutheast
    ///  * Linux: nw-resize
    ResizeUpLeft,

    /// A cursor indicating resizing an object from its top-right corner.
    ///
    /// Typically the shape of an arrow pointing upper right. May fallback to [resizeUpRightDownLeft].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_RIGHT_DIAGONAL_DOUBLE_ARROW
    ///  * Web: ne-resize
    ///  * Windows: IDC_SIZENESW
    ///  * Windows UWP: CoreCursorType::SizeNortheastSouthwest
    ///  * Linux: ne-resize
    ResizeUpRight,

    /// A cursor indicating resizing an object from its bottom-left corner.
    ///
    /// Typically the shape of an arrow pointing lower left. May fallback to [resizeUpRightDownLeft].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_RIGHT_DIAGONAL_DOUBLE_ARROW
    ///  * Web: sw-resize
    ///  * Windows: IDC_SIZENESW
    ///  * Windows UWP: CoreCursorType::SizeNortheastSouthwest
    ///  * Linux: sw-resize
    ResizeDownLeft,

    /// A cursor indicating resizing an object from its bottom-right corner.
    ///
    /// Typically the shape of an arrow pointing lower right. May fallback to [resizeUpLeftDownRight].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_TOP_LEFT_DIAGONAL_DOUBLE_ARROW
    ///  * Web: se-resize
    ///  * Windows: IDC_SIZENWSE
    ///  * Windows UWP: CoreCursorType::SizeNorthwestSoutheast
    ///  * Linux: se-resize
    ResizeDownRight,

    /// A cursor indicating resizing a column, or an item horizontally.
    ///
    /// Typically the shape of arrows pointing left and right with a vertical bar
    /// separating them. May fallback to [resizeLeftRight].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_HORIZONTAL_DOUBLE_ARROW
    ///  * Web: col-resize
    ///  * Windows: IDC_SIZEWE
    ///  * Windows UWP: CoreCursorType::SizeWestEast
    ///  * Linux: col-resize
    ///  * macOS: resizeLeftRightCursor
    ResizeColumn,

    /// A cursor indicating resizing a row, or an item vertically.
    ///
    /// Typically the shape of arrows pointing up and down with a horizontal bar
    /// separating them. May fallback to [resizeUpDown].
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_VERTICAL_DOUBLE_ARROW
    ///  * Web: row-resize
    ///  * Windows: IDC_SIZENS
    ///  * Windows UWP: CoreCursorType::SizeNorthSouth
    ///  * Linux: row-resize
    ///  * macOS: resizeUpDownCursor
    ResizeRow,

    // OTHER OPERATIONS
    /// A cursor indicating zooming in.
    ///
    /// Typically a magnifying glass with a plus sign.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_ZOOM_IN
    ///  * Web: zoom-in
    ///  * Linux: zoom-in
    ZoomIn,

    /// A cursor indicating zooming out.
    ///
    /// Typically a magnifying glass with a minus sign.
    ///
    /// Corresponds to:
    ///
    ///  * Android: TYPE_ZOOM_OUT
    ///  * Web: zoom-out
    ///  * Linux: zoom-out
    ZoomOut,
}

impl Into<Option<CursorIcon>> for MouseCursorKind {
    fn into(self) -> Option<CursorIcon> {
        Some(match self {
            MouseCursorKind::None => return None,
            MouseCursorKind::Basic => CursorIcon::Default,
            MouseCursorKind::Click => CursorIcon::Pointer,
            MouseCursorKind::Forbidden => CursorIcon::NotAllowed,
            MouseCursorKind::Wait => CursorIcon::Wait,
            MouseCursorKind::Progress => CursorIcon::Progress,
            MouseCursorKind::ContextMenu => CursorIcon::ContextMenu,
            MouseCursorKind::Help => CursorIcon::Help,
            MouseCursorKind::Text => CursorIcon::Text,
            MouseCursorKind::VerticalText => CursorIcon::VerticalText,
            MouseCursorKind::Cell => CursorIcon::Cell,
            MouseCursorKind::Precise => CursorIcon::Crosshair,
            MouseCursorKind::Move => CursorIcon::Move,
            MouseCursorKind::Grab => CursorIcon::Grab,
            MouseCursorKind::Grabbing => CursorIcon::Grabbing,
            MouseCursorKind::NoDrop => CursorIcon::NoDrop,
            MouseCursorKind::Alias => CursorIcon::Alias,
            MouseCursorKind::Copy => CursorIcon::Copy,
            MouseCursorKind::Disappearing => unimplemented!(),
            MouseCursorKind::AllScroll => CursorIcon::AllScroll,
            MouseCursorKind::ResizeLeftRight => CursorIcon::NeResize,
            MouseCursorKind::ResizeUpDown => CursorIcon::NsResize,
            MouseCursorKind::ResizeUpLeftDownRight => CursorIcon::NwseResize,
            MouseCursorKind::ResizeUpRightDownLeft => CursorIcon::NeswResize,
            MouseCursorKind::ResizeUp => CursorIcon::NResize,
            MouseCursorKind::ResizeDown => CursorIcon::SResize,
            MouseCursorKind::ResizeLeft => CursorIcon::WResize,
            MouseCursorKind::ResizeRight => CursorIcon::EResize,
            MouseCursorKind::ResizeUpLeft => CursorIcon::NwResize,
            MouseCursorKind::ResizeUpRight => CursorIcon::NeResize,
            MouseCursorKind::ResizeDownLeft => CursorIcon::SwResize,
            MouseCursorKind::ResizeDownRight => CursorIcon::SwResize,
            MouseCursorKind::ResizeColumn => CursorIcon::ColResize,
            MouseCursorKind::ResizeRow => CursorIcon::RowResize,
            MouseCursorKind::ZoomIn => CursorIcon::ZoomIn,
            MouseCursorKind::ZoomOut => CursorIcon::ZoomOut,
        })
    }
}
