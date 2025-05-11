use crate::geometry::Rect;

/// Defines abstract sources of [`Rect`] for clipping resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbstractClipRect {
    /// Represents the parent widget's clipping rect.
    ParentClip,
    /// Represents the current widget's layout rect.
    LayoutRect,
    /// Represents the entire viewport rect.
    Viewport,
    /// The provided rect.
    Value(Rect),
}

impl AbstractClipRect {
    /// Turns the [`AbstractClipRect`] into a concrete [`Rect`].
    pub fn to_rect(self, parent_clip: Rect, layout_rect: Rect, viewport: Rect) -> Rect {
        match self {
            AbstractClipRect::ParentClip => parent_clip,
            AbstractClipRect::LayoutRect => layout_rect,
            AbstractClipRect::Viewport => viewport,
            AbstractClipRect::Value(rect) => rect,
        }
    }
}

/// Defines the clipping logic of clipping resolution.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ClipLogic {
    /// No clipping logic is performed. The widget will simply reuse the parent's clipping rect.
    #[default]
    Pass,
    /// The clipping rect will be the [`constrained`][Rect::constrain] result of the two rects.
    Constrain(AbstractClipRect, AbstractClipRect),
    /// The clipping rect will be overridden to be the provided rect.
    Override(AbstractClipRect),
}

impl ClipLogic {
    /// Resolves the clipping rect with the provided [`ClipLogic`].
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
