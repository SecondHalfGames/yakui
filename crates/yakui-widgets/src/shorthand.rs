use std::borrow::Cow;

use yakui_core::{Color3, Constraints, Response, TextureId, Vec2};

use crate::{
    Align, AlignWidget, Alignment, Button, ButtonWidget, ColoredBox, ColoredBoxWidget,
    ConstrainedBox, ConstrainedBoxWidget, Flex, FlexWidget, Image, ImageWidget, List, ListWidget,
    Pad, PadWidget, Text, TextWidget,
};

pub fn column<F: FnOnce()>(children: F) -> Response<ListWidget> {
    List::vertical().show(children)
}

pub fn row<F: FnOnce()>(children: F) -> Response<ListWidget> {
    List::horizontal().show(children)
}

pub fn center<F: FnOnce()>(children: F) -> Response<AlignWidget> {
    Align::center().show(children)
}

pub fn align<F: FnOnce()>(alignment: Alignment, children: F) -> Response<AlignWidget> {
    Align::new(alignment).show(children)
}

pub fn button<S: Into<Cow<'static, str>>>(text: S) -> Response<ButtonWidget> {
    Button::styled(text.into()).show()
}

pub fn colored_box<S: Into<Vec2>>(color: Color3, size: S) -> Response<ColoredBoxWidget> {
    ColoredBox::sized(color, size.into()).show()
}

pub fn colored_box_container<F: FnOnce()>(
    color: Color3,
    children: F,
) -> Response<ColoredBoxWidget> {
    ColoredBox::container(color).show_children(children)
}

pub fn image(image: TextureId, size: Vec2) -> Response<ImageWidget> {
    Image::new(image, size).show()
}

pub fn pad<F: FnOnce()>(padding: Pad, children: F) -> Response<PadWidget> {
    padding.show(children)
}

pub fn text<S: Into<Cow<'static, str>>>(size: f32, text: S) -> Response<TextWidget> {
    Text::new(size, text.into()).show()
}

pub fn label<S: Into<Cow<'static, str>>>(text: S) -> Response<TextWidget> {
    Text::label(text.into()).show()
}

pub fn flex<F: FnOnce()>(flex: u32, children: F) -> Response<FlexWidget> {
    Flex::new(flex).show(children)
}

pub fn expanded<F: FnOnce()>(children: F) -> Response<FlexWidget> {
    Flex::expanded().show(children)
}

pub fn constrained<F: FnOnce()>(
    constraints: Constraints,
    children: F,
) -> Response<ConstrainedBoxWidget> {
    ConstrainedBox::new(constraints).show(children)
}
