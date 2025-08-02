use glam::Vec2;

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ClipResolutionArgs {
    pub parent_clip: Rect,
    pub parent_rect: Rect,
    pub layout_rect: Rect,
    pub viewport: Rect,
}

/// Defines abstract sources of [`Rect`] for clipping resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbstractClipRect {
    /// Represents the parent widget's clipping rect.
    ParentClip,
    /// Represents the parent widget's layout rect.
    ParentRect,
    /// Represents the current widget's layout rect.
    LayoutRect,
    /// Represents the entire viewport rect.
    Viewport,
    /// The provided rect.
    Value(Rect),
}

impl AbstractClipRect {
    /// Turns the [`AbstractClipRect`] into a concrete [`Rect`].
    pub(crate) fn to_rect(
        self,
        ClipResolutionArgs {
            parent_clip,
            parent_rect,
            layout_rect,
            viewport,
        }: ClipResolutionArgs,
    ) -> Rect {
        match self {
            AbstractClipRect::ParentClip => parent_clip,
            AbstractClipRect::ParentRect => parent_rect,
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
    /// If the widget is out of the [`parent`][ClipLogic::Contain::parent]'s bounds, try to push it back in first.
    Contain {
        /// The rect of the supposed widget in question.
        it: AbstractClipRect,
        /// The rect of the supposed widget's parent.
        parent: AbstractClipRect,
    },
    /// The clipping rect will be overridden to be the provided rect.
    Override(AbstractClipRect),
}

impl ClipLogic {
    /// Resolves the offset for the *layout rect* with the provided [`ClipLogic`].
    /// Should be calculated before doing clipping rect resolution, [`None`] means "just use the original layout rect".
    pub(crate) fn resolve_offset(
        self,
        args @ ClipResolutionArgs { .. }: ClipResolutionArgs,
    ) -> Option<Vec2> {
        match self {
            ClipLogic::Contain { it, parent } => {
                let it = it.to_rect(args);
                let parent = parent.to_rect(args);

                let offset_min = Vec2::min(it.pos() - parent.pos(), Vec2::ZERO);
                let offset_max = Vec2::min(-(it.max() - parent.max()), Vec2::ZERO);

                Some(-offset_min + offset_max)
            }
            _ => None,
        }
    }

    /// Resolves the *clipping rect* with the provided [`ClipLogic`].
    pub(crate) fn resolve_clip(
        self,
        args @ ClipResolutionArgs { parent_clip, .. }: ClipResolutionArgs,
    ) -> Rect {
        match self {
            ClipLogic::Pass => parent_clip,
            ClipLogic::Constrain(a, b) | ClipLogic::Contain { it: a, parent: b } => {
                let a = a.to_rect(args);
                let b = b.to_rect(args);

                a.constrain(b)
            }
            ClipLogic::Override(rect) => rect.to_rect(args),
        }
    }
}
