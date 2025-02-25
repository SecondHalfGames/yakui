use crate::geometry::Rect;

/// Defines abstract sources of rects for clipping rect resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbstractRect {
    /// Represents the parent widget's clipping rect.
    ParentClip,
    /// Represents the current widget's layout rect.
    LayoutRect,
    /// Represents the entire viewport.
    Viewport,
    /// The provided rect.
    Value(Rect),
}

impl AbstractRect {
    /// Turns the abstract rect source into a concrete Rect
    pub fn to_rect(self, parent_clip: Rect, layout_rect: Rect, viewport: Rect) -> Rect {
        match self {
            AbstractRect::ParentClip => parent_clip,
            AbstractRect::LayoutRect => layout_rect,
            AbstractRect::Viewport => viewport,
            AbstractRect::Value(rect) => rect,
        }
    }
}

/// Defines the clipping logic at clipping rect resolution.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ClipLogic {
    /// No clipping logic is performed. The widget will simply reuse the parent's clipping rect.
    #[default]
    Pass,
    /// The clipping rect will be the [`constrained`][Rect::constrain] result of the two rects.
    Constrain(AbstractRect, AbstractRect),
    /// The clipping rect will be overridden to be the provided rect.
    Override(AbstractRect),
}

impl ClipLogic {
    /// Attempts to resolve the clipping rect.
    pub fn resolve(self, parent_clip: Rect, layout_rect: Rect, viewport: Rect) -> Rect {
        match self {
            ClipLogic::Pass => parent_clip,
            ClipLogic::Constrain(a, b) => {
                let a = a.to_rect(parent_clip, layout_rect, viewport);
                let b = b.to_rect(parent_clip, layout_rect, viewport);

                a.constrain(b)
            }
            ClipLogic::Override(rect) => rect.to_rect(parent_clip, layout_rect, viewport),
        }
    }
}
