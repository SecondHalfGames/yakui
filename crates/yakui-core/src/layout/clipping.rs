use glam::Vec2;

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ClipResolutionArgs {
    pub parent_clip: Rect,
    pub parent_rect: Rect,
    pub layout_rect: Rect,
    pub viewport: Rect,
    pub offset: Vec2,
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
            offset,
        }: ClipResolutionArgs,
    ) -> Rect {
        let mut rect = match self {
            AbstractClipRect::LayoutRect => layout_rect,
            AbstractClipRect::Value(rect) => rect,
            // don't offset these
            AbstractClipRect::ParentClip => return parent_clip,
            AbstractClipRect::ParentRect => return parent_rect,
            AbstractClipRect::Viewport => return viewport,
        };

        rect.set_pos(rect.pos() + offset);

        rect
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
    /// Optionally add an offset to the widget first.
    Contain {
        /// The rect of the supposed widget in question.
        it: AbstractClipRect,
        /// The rect of the supposed widget's parent.
        parent: AbstractClipRect,
        /// An optional pixel offset to apply to [`it`][ClipLogic::Contain::it] before containing it in the [`parent`][ClipLogic::Contain::parent]'s bounds.
        offset: Vec2,
    },
    /// The clipping rect will be overridden to be the provided rect.
    Override(AbstractClipRect),
}

impl ClipLogic {
    /// Resolves the *layout rect* with the provided [`ClipLogic`].
    /// Should be calculated before doing clipping rect resolution, [`None`] means "just use the original layout rect".
    pub(crate) fn resolve_layout(
        self,
        args @ ClipResolutionArgs { viewport, .. }: ClipResolutionArgs,
    ) -> Option<Rect> {
        match self {
            ClipLogic::Contain { it, parent, offset } => {
                let it = it.to_rect(args);
                let it = Rect::from_pos_size(it.pos() + offset, it.size());

                // implicitly also constrain to viewport
                let parent = parent.to_rect(args).constrain(viewport);

                let ratio = Vec2::max(it.size() / parent.size() - Vec2::ONE, Vec2::ONE);

                let offset_min = Vec2::max(parent.pos() - it.pos(), Vec2::ZERO) * ratio;
                let offset_max = Vec2::max(it.max() - parent.max(), Vec2::ZERO) * ratio;

                Some(Rect::from_pos_size(
                    it.pos() + (-offset_max + offset_min),
                    it.size(),
                ))
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
            ClipLogic::Constrain(a, b)
            | ClipLogic::Contain {
                it: a, parent: b, ..
            } => {
                let a = a.to_rect(args);
                let b = b.to_rect(args);

                a.constrain(b)
            }
            ClipLogic::Override(rect) => rect.to_rect(args),
        }
    }
}
