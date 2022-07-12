use std::borrow::Cow;

use yakui_core::{Color3, Constraints, Index, Vec2};

use crate::{
    Align, AlignResponse, Alignment, Button, ButtonResponse, ColoredBox, ColoredBoxResponse,
    ConstrainedBox, ConstrainedBoxResponse, Flex, FlexResponse, Image, ImageResponse, List,
    ListResponse, Pad, PadResponse, Text, TextResponse,
};

pub fn column<F: FnOnce()>(children: F) -> ListResponse {
    List::vertical().show(children)
}

pub fn row<F: FnOnce()>(children: F) -> ListResponse {
    List::horizontal().show(children)
}

pub fn center<F: FnOnce()>(children: F) -> AlignResponse {
    Align::center().show(children)
}

pub fn align<F: FnOnce()>(alignment: Alignment, children: F) -> AlignResponse {
    Align::new(alignment).show(children)
}

pub fn button<S: Into<Cow<'static, str>>>(text: S) -> ButtonResponse {
    Button::styled(text.into()).show()
}

pub fn colored_box<S: Into<Vec2>>(color: Color3, size: S) -> ColoredBoxResponse {
    ColoredBox::sized(color, size.into()).show()
}

pub fn colored_box_container<F: FnOnce()>(color: Color3, children: F) -> ColoredBoxResponse {
    ColoredBox::container(color).show_children(children)
}

pub fn image(image: Index, size: Vec2) -> ImageResponse {
    Image::new(image, size).show()
}

pub fn pad<F: FnOnce()>(padding: Pad, children: F) -> PadResponse {
    padding.show(children)
}

pub fn text<S: Into<Cow<'static, str>>>(size: f32, text: S) -> TextResponse {
    Text::new(size, text.into()).show()
}

pub fn label<S: Into<Cow<'static, str>>>(text: S) -> TextResponse {
    Text::label(text.into()).show()
}

pub fn flex<F: FnOnce()>(flex: u32, children: F) -> FlexResponse {
    Flex::new(flex).show(children)
}

pub fn expanded<F: FnOnce()>(children: F) -> FlexResponse {
    Flex::expanded().show(children)
}

pub fn constrained<F: FnOnce()>(constraints: Constraints, children: F) -> ConstrainedBoxResponse {
    ConstrainedBox::new(constraints).show(children)
}
