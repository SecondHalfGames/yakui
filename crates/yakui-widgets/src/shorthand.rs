//! Contains terse defaults for the most common widgets.
//!
//! Each function in this module is easy to read in order to enable extending a
//! widget if its defaults don't work for you.

use std::borrow::Cow;

use yakui_core::geometry::{Color, Constraints, Dim2, Vec2};
use yakui_core::widget::PaintContext;
use yakui_core::{Alignment, ManagedTextureId, Pivot, Response, TextureId};

use crate::widgets::{
    Align, AlignResponse, Button, ButtonResponse, Canvas, CanvasResponse, Checkbox,
    CheckboxResponse, Circle, CircleResponse, ColoredBox, ColoredBoxResponse, ConstrainedBox,
    ConstrainedBoxResponse, CountGrid, Divider, DividerResponse, Draggable, DraggableResponse,
    Flexible, FlexibleResponse, Image, ImageResponse, List, ListResponse, MaxWidth,
    MaxWidthResponse, NineSlice, Offset, OffsetResponse, Opaque, OpaqueResponse, Pad, PadResponse,
    Reflow, ReflowResponse, Scrollable, ScrollableResponse, Slider, SliderResponse, Spacer, Stack,
    StackResponse, State, StateResponse, Text, TextBox, TextBoxResponse, TextResponse,
};

/// See [List].
#[track_caller]
pub fn column<F: FnOnce()>(children: F) -> Response<ListResponse> {
    List::column().show(children)
}

/// See [List].
#[track_caller]
pub fn row<F: FnOnce()>(children: F) -> Response<ListResponse> {
    List::row().show(children)
}

/// See [CountGrid].
#[track_caller]
pub fn countgrid_column<F: FnOnce()>(n_columns: usize, children: F) -> Response<ListResponse> {
    CountGrid::col(n_columns).show(children)
}

/// See [CountGrid].
#[track_caller]
pub fn countgrid_row<F: FnOnce()>(n_rows: usize, children: F) -> Response<ListResponse> {
    CountGrid::row(n_rows).show(children)
}

/// See [Align].
#[track_caller]
pub fn center<F: FnOnce()>(children: F) -> Response<AlignResponse> {
    Align::center().show(children)
}

/// See [Align].
#[track_caller]
pub fn align<F: FnOnce()>(alignment: Alignment, children: F) -> Response<AlignResponse> {
    Align::new(alignment).show(children)
}

/// See [Button].
#[track_caller]
pub fn button<S: Into<Cow<'static, str>>>(text: S) -> Response<ButtonResponse> {
    Button::styled(text.into()).show()
}

/// See [Circle].
#[track_caller]
pub fn colored_circle<S: Into<f32>>(color: Color, size: S) -> Response<CircleResponse> {
    Circle::new().min_radius(size).color(color).show()
}

/// See [ColoredBox].
#[track_caller]
pub fn colored_box<S: Into<Vec2>>(color: Color, size: S) -> Response<ColoredBoxResponse> {
    ColoredBox::sized(color, size.into()).show()
}

/// See [ColoredBox].
#[track_caller]
pub fn colored_box_container<F: FnOnce()>(
    color: Color,
    children: F,
) -> Response<ColoredBoxResponse> {
    ColoredBox::container(color).show_children(children)
}

/// See [Image].
#[track_caller]
pub fn image<I, S>(image: I, size: S) -> Response<ImageResponse>
where
    I: Into<TextureId>,
    S: Into<Vec2>,
{
    Image::new(image.into(), size.into()).show()
}

/// See [Pad].
#[track_caller]
pub fn pad<F: FnOnce()>(padding: Pad, children: F) -> Response<PadResponse> {
    padding.show(children)
}

/// See [Text].
#[track_caller]
pub fn text<S: Into<Cow<'static, str>>>(size: f32, text: S) -> Response<TextResponse> {
    Text::new(size, text.into()).show()
}

/// See [Text].
#[track_caller]
pub fn label<S: Into<Cow<'static, str>>>(text: S) -> Response<TextResponse> {
    Text::label(text.into()).show()
}

/// See [TextBox].
#[track_caller]
pub fn textbox<S: Into<String>>(text: S) -> Response<TextBoxResponse> {
    TextBox::new(text.into()).show()
}

/// See [Flexible].
#[track_caller]
pub fn flexible<F: FnOnce()>(flex: u32, children: F) -> Response<FlexibleResponse> {
    Flexible::new(flex).show(children)
}

/// See [Flexible].
#[track_caller]
pub fn expanded<F: FnOnce()>(children: F) -> Response<FlexibleResponse> {
    Flexible::expanded().show(children)
}

/// See [ConstrainedBox].
#[track_caller]
pub fn constrained<F: FnOnce()>(
    constraints: Constraints,
    children: F,
) -> Response<ConstrainedBoxResponse> {
    ConstrainedBox::new(constraints).show(children)
}

/// See [Checkbox].
#[track_caller]
pub fn checkbox(checked: bool) -> Response<CheckboxResponse> {
    Checkbox::new(checked).show()
}

/// See [Offset].
#[track_caller]
pub fn offset<F: FnOnce()>(offset: Vec2, children: F) -> Response<OffsetResponse> {
    Offset::new(offset).show(children)
}

/// See [Draggable].
#[track_caller]
pub fn draggable<F: FnOnce()>(children: F) -> Response<DraggableResponse> {
    Draggable::new().show(children)
}

/// See [NineSlice].
#[track_caller]
pub fn nineslice(
    texture: ManagedTextureId,
    margins: Pad,
    scale: f32,
    children: impl FnOnce(),
) -> Response<()> {
    NineSlice::new(texture, margins, scale).show(children)
}

/// See [Divider].
#[track_caller]
pub fn divider(color: Color, height: f32, thickness: f32) -> Response<DividerResponse> {
    Divider::new(color, height, thickness).show()
}

/// See [Spacer].
#[track_caller]
pub fn spacer(flex: u32) -> Response<FlexibleResponse> {
    Spacer::new(flex).show()
}

/// See [Scrollable].
#[track_caller]
pub fn scroll_vertical(children: impl FnOnce()) -> Response<ScrollableResponse> {
    Scrollable::vertical().show(children)
}

/// See [Slider].
#[track_caller]
pub fn slider(value: f64, min: f64, max: f64) -> Response<SliderResponse> {
    Slider::new(value, min, max).show()
}

/// See [Reflow].
#[track_caller]
pub fn reflow(
    anchor: Alignment,
    pivot: Pivot,
    offset: Dim2,
    children: impl FnOnce(),
) -> Response<ReflowResponse> {
    Reflow::new(anchor, pivot, offset).show(children)
}

/// See [Opaque].
#[track_caller]
pub fn opaque(children: impl FnOnce()) -> Response<OpaqueResponse> {
    Opaque::new().show(children)
}

/// See [Canvas].
#[track_caller]
pub fn canvas(paint: impl Fn(&mut PaintContext<'_>) + 'static) -> Response<CanvasResponse> {
    Canvas::new(paint).show()
}

/// See [MaxWidth].
#[track_caller]
pub fn max_width(max_width: f32, children: impl FnOnce()) -> Response<MaxWidthResponse> {
    MaxWidth::new(max_width).show(children)
}

/// See [Stack].
#[track_caller]
pub fn stack(children: impl FnOnce()) -> Response<StackResponse> {
    Stack::new().show(children)
}

#[track_caller]
pub fn use_state<F, T: 'static>(default: F) -> Response<StateResponse<T>>
where
    F: FnOnce() -> T + 'static,
{
    State::new(default).show()
}
