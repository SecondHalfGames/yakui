//! Defines traits for building widgets.

use std::any::{type_name, Any};
use std::collections::VecDeque;
use std::fmt;

use glam::Vec2;

use crate::dom::Dom;
use crate::event::EventResponse;
use crate::event::{EventInterest, WidgetEvent};
use crate::geometry::{Constraints, FlexFit, Rect};
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::navigation::NavDirection;
use crate::paint::PaintDom;
use crate::{Flow, WidgetId};

/// Trait that's automatically implemented for all widget props.
///
/// This trait is used by yakui to enforce that props implement `Debug`.
pub trait Props: fmt::Debug {}
impl<T> Props for T where T: fmt::Debug {}

/// Information available to a widget during the layout phase.
#[allow(missing_docs)]
pub struct LayoutContext<'dom> {
    pub dom: &'dom Dom,
    pub input: &'dom InputState,
    pub layout: &'dom mut LayoutDom,
    pub paint: &'dom PaintDom,
}

impl LayoutContext<'_> {
    /// Calculate the layout for the given widget with the given constraints.
    ///
    /// This method currently must only be called once per widget per layout
    /// phase.
    pub fn calculate_layout(&mut self, widget: WidgetId, constraints: Constraints) -> Vec2 {
        self.layout
            .calculate(self.dom, self.input, self.paint, widget, constraints)
    }

    /// Returns the current layout rectangle for the given widget relative
    /// to the given ancestor.
    pub fn rect_relative_to(&self, widget: WidgetId, ancestor: WidgetId) -> Option<Rect> {
        let layout = self.layout.get(widget)?;
        let mut rect = layout.rect;

        if widget == ancestor {
            rect.set_pos(Vec2::ZERO);
            return Some(rect);
        }

        let mut current = widget;

        loop {
            let parent = self.dom.get(current)?.parent?;
            if parent == ancestor {
                return Some(rect);
            }

            let parent_layout = self.layout.get(parent)?;
            rect.set_pos(rect.pos() + parent_layout.rect.pos());

            current = parent;
        }
    }
}

/// Information available to a widget during the paint phase.
#[allow(missing_docs)]
pub struct PaintContext<'dom> {
    pub dom: &'dom Dom,
    pub layout: &'dom LayoutDom,
    pub paint: &'dom mut PaintDom,
}

impl PaintContext<'_> {
    /// Paint the given widget.
    pub fn paint(&mut self, widget: WidgetId) {
        self.paint.paint(self.dom, self.layout, widget);
    }
}

/// Information available to a widget when it has received an event.
#[allow(missing_docs)]
pub struct EventContext<'dom> {
    pub dom: &'dom Dom,
    pub layout: &'dom LayoutDom,
    pub input: &'dom InputState,
}

/// Information available to a widget when it is being queried for navigation.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct NavigateContext<'dom> {
    pub dom: &'dom Dom,
    pub layout: &'dom LayoutDom,
    pub input: &'dom InputState,
}

impl NavigateContext<'_> {
    /// Query for navigation to the given widget or one of its descendents.
    pub fn try_navigate(&self, widget: WidgetId, dir: NavDirection) -> Option<WidgetId> {
        self.dom.enter(widget);
        let node = self.dom.get(widget).unwrap();

        log::trace!(
            "Enter Navigate {dir:?} on {widget:?} ({})",
            node.widget.type_name()
        );

        let res = node.widget.navigate(*self, dir);

        log::trace!(
            "Result of Navigate {dir:?} on {widget:?} ({}): {res:?}",
            node.widget.type_name()
        );

        self.dom.exit(widget);

        res
    }

    /// Tells whether `descendent` is a descendent of `parent`.
    pub fn contains(&self, parent: WidgetId, descendent: WidgetId) -> bool {
        let mut queue = VecDeque::new();
        queue.push_back(parent);

        while let Some(current) = queue.pop_front() {
            if current == descendent {
                return true;
            }

            let node = self.dom.get(current).unwrap();
            queue.extend(node.children.iter().copied());
        }

        false
    }

    fn nearest_in_direction(
        &self,
        origin: WidgetId,
        dir: NavDirection,
        candidates: impl IntoIterator<Item = WidgetId>,
    ) -> Option<WidgetId> {
        candidates
            .into_iter()
            .filter_map(|candidate| {
                self.directional_score(origin, candidate, dir)
                    .map(|score| (candidate, score))
            })
            .min_by(|(_, score_a), (_, score_b)| score_a.total_cmp(score_b))
            .map(|(candidate, _)| candidate)
    }

    /// Returns the directional desirebility of a candidate widget
    /// as a score where lower is better
    fn directional_score(
        &self,
        origin: WidgetId,
        candidate: WidgetId,
        dir: NavDirection,
    ) -> Option<f32> {
        let origin = self.layout.get(origin)?.rect.center();
        let candidate = self.layout.get(candidate)?.rect.center();
        let delta = candidate - origin;

        let (forward, cross) = match dir {
            NavDirection::Down => (delta.y, delta.x),
            NavDirection::Up => (-delta.y, delta.x),
            NavDirection::Left => (-delta.x, delta.y),
            NavDirection::Right => (delta.x, delta.y),
            NavDirection::Next | NavDirection::Previous => return None,
        };

        if forward <= 0.0 {
            return None;
        }

        // Prefer parallel widgets
        let score = forward * forward + 2.0 * cross * cross;
        score.is_finite().then_some(score)
    }
}

bitflags::bitflags! {
    /// A bitfield representing the focus policy of a widget.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    pub struct FocusPolicy: u8 {
        /// Focusable through sequential navigation.
        const SEQUENTIAL = 1 << 0;

        /// Focusable through directional navigation.
        const DIRECTIONAL = 1 << 1;

        /// Automatically focused when this widget or one of its
        /// descendants sinks a primary pointer click.
        const POINTER = 1 << 2;
    }
}

/// A yakui widget. Implement this trait to create a custom widget if composing
/// existing widgets does not solve your use case.
pub trait Widget: 'static + fmt::Debug {
    /// The props that this widget needs to be created or updated. Props define
    /// all of the values that a widget's user can specify every render.
    type Props<'a>: Props;

    /// The type that the widget will return to the user when it is created or
    /// updated. This type should contain information like whether the widget
    /// was clicked, had keyboard input, or other info that might be useful.
    type Response;

    /// Create the widget.
    fn new() -> Self;

    /// Update the widget with new props.
    fn update(&mut self, props: Self::Props<'_>) -> Self::Response;

    /// Returns whether this widget should grow to fill a flexible layout, and
    /// if so, what weight should be applied to it if other widgets also want to
    /// grow.
    ///
    /// A value of `0` indicates that this widget should not grow, while `1`
    /// means that it should stretch to take the available space.
    fn flex(&self) -> (u32, FlexFit) {
        (0, FlexFit::Loose)
    }

    /// Returns the behavior that this widget should have when part of a layout.
    ///
    /// By default, widgets participate in layout using [`Flow::Inline`].
    fn flow(&self) -> Flow {
        Flow::Inline
    }

    /// Calculate this widget's layout with the given constraints and return its
    /// size. The returned size must fit within the given constraints, which can
    /// be done using `constraints.constrain(size)`.
    ///
    /// The default implementation will lay out all of this widget's children on
    /// top of each other, and fit the widget tightly around them.
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        self.default_layout(ctx, constraints)
    }

    /// A convenience method that always performs the default layout strategy
    /// for a widget. This method is intended to be called from custom widget's
    /// `layout` methods.
    #[inline]
    fn default_layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain_min(size)
    }

    /// Paint the widget based on its current state.
    ///
    /// The default implementation will paint all of the widget's children.
    fn paint(&self, ctx: PaintContext<'_>) {
        self.default_paint(ctx);
    }

    /// A convenience method that always performs the default painting operation
    /// for a widget. This method is intended to be called from custom widget's
    /// `paint` methods.
    #[inline]
    fn default_paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }
    }

    /// Tells which events the widget is interested in receiving.
    ///
    /// The default implementation will register interest in no events.
    fn event_interest(&self) -> EventInterest {
        EventInterest::empty()
    }

    /// Handle the given event and update the widget's state.
    ///
    /// The default implementation will bubble all events.
    #[allow(unused)]
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        EventResponse::Bubble
    }

    /// Returns the focus policy for this widget.
    ///
    /// The default implementation implies no focus behavior.
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::empty()
    }

    /// Tell which widget should be navigated to if the user navigates in a
    /// given direction.
    fn navigate(&self, ctx: NavigateContext<'_>, dir: NavDirection) -> Option<WidgetId> {
        self.default_navigate(ctx, dir)
    }

    /// Perform the default navigation based on focus and position.
    fn default_navigate(&self, ctx: NavigateContext<'_>, dir: NavDirection) -> Option<WidgetId> {
        let node_id = ctx.dom.current();
        let node = ctx.dom.get_current();

        let focus = ctx.input.focus();
        let mut current_index = None;

        for (index, &child) in node.children.iter().enumerate() {
            if focus.is_some_and(|focus| ctx.contains(child, focus)) {
                current_index = Some(index);
                break;
            }
        }

        if let Some(index) = current_index {
            // The navigation is originating from inside this widget. This
            // widget should find the next focusable child, or return None.

            match dir {
                NavDirection::Next => {
                    for &child in node.children.iter().skip(index + 1) {
                        if let Some(id) = ctx.try_navigate(child, NavDirection::Next) {
                            return Some(id);
                        }
                    }
                }

                NavDirection::Previous => {
                    if let Some(prev_index) = index.checked_sub(1) {
                        let skip = node.children.len() - prev_index - 1;
                        for &child in node.children.iter().rev().skip(skip) {
                            if let Some(id) = ctx.try_navigate(child, NavDirection::Previous) {
                                return Some(id);
                            }
                        }
                    }
                }

                NavDirection::Down
                | NavDirection::Up
                | NavDirection::Left
                | NavDirection::Right => {
                    let focus = focus?;

                    let candidates = node
                        .children
                        .iter()
                        .enumerate()
                        .filter(|(child_index, _)| *child_index != index)
                        .filter_map(|(_, &child)| ctx.try_navigate(child, dir));
                    return ctx.nearest_in_direction(focus, dir, candidates);
                }
            }

            None
        } else {
            // The navigation is originating from outside this widget. This code
            // should pick the widget that's nearest to the given navigation
            // direction that's focusable.

            if focus != Some(node_id) {
                let policy = self.focus_policy();

                let can_focus = match dir {
                    NavDirection::Next | NavDirection::Previous => {
                        policy.contains(FocusPolicy::SEQUENTIAL)
                    }
                    NavDirection::Down
                    | NavDirection::Up
                    | NavDirection::Left
                    | NavDirection::Right => policy.contains(FocusPolicy::DIRECTIONAL),
                };

                if can_focus {
                    return Some(node_id);
                }
            }

            match dir {
                NavDirection::Next => {
                    for &child in &node.children {
                        if let Some(id) = ctx.try_navigate(child, NavDirection::Next) {
                            return Some(id);
                        }
                    }

                    None
                }

                NavDirection::Previous => {
                    for &child in node.children.iter().rev() {
                        if let Some(id) = ctx.try_navigate(child, NavDirection::Previous) {
                            return Some(id);
                        }
                    }

                    None
                }

                NavDirection::Down
                | NavDirection::Up
                | NavDirection::Left
                | NavDirection::Right => {
                    let focus = focus?;

                    let candidates = node
                        .children
                        .iter()
                        .filter_map(|&child| ctx.try_navigate(child, dir));
                    ctx.nearest_in_direction(focus, dir, candidates)
                }
            }
        }
    }
}

/// A type-erased version of [`Widget`].
pub trait ErasedWidget: Any + fmt::Debug {
    /// See [`Widget::layout`].
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2;

    /// See [`Widget::flex`].
    fn flex(&self) -> (u32, FlexFit);

    /// See [`Widget::flow`].
    fn flow(&self) -> Flow;

    /// See [`Widget::paint`].
    fn paint(&self, ctx: PaintContext<'_>);

    /// See [`Widget::event_interest`].
    fn event_interest(&self) -> EventInterest;

    /// See [`Widget::event`].
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse;

    /// Returns the type name of the widget, usable only for debugging.
    fn type_name(&self) -> &'static str;

    /// See [`Widget::focus_policy`].
    fn focus_policy(&self) -> FocusPolicy;

    /// See [`Widget::navigate`].
    fn navigate(&self, ctx: NavigateContext<'_>, dir: NavDirection) -> Option<WidgetId>;
}

impl<T> ErasedWidget for T
where
    T: Widget,
{
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        <T as Widget>::layout(self, ctx, constraints)
    }

    fn flex(&self) -> (u32, FlexFit) {
        <T as Widget>::flex(self)
    }

    fn flow(&self) -> Flow {
        <T as Widget>::flow(self)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        <T as Widget>::paint(self, ctx)
    }

    fn event_interest(&self) -> EventInterest {
        <T as Widget>::event_interest(self)
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        log::debug!("Event on {}: {event:?}", type_name::<T>());

        <T as Widget>::event(self, ctx, event)
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn focus_policy(&self) -> FocusPolicy {
        <T as Widget>::focus_policy(self)
    }

    fn navigate(&self, ctx: NavigateContext<'_>, dir: NavDirection) -> Option<WidgetId> {
        <T as Widget>::navigate(self, ctx, dir)
    }
}

impl dyn ErasedWidget {
    /// Casts a dyn [`ErasedWidget`] into a dyn [`Any`]
    pub fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    /// Casts a mutable dyn [`ErasedWidget`] into a mutable dyn [`Any`]
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}
