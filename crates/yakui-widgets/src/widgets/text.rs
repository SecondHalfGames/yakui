use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::text_renderer::TextGlobalState;
use crate::util::widget;

#[derive(Debug, Clone)]
pub struct Text {
    pub text: Cow<'static, str>,
    pub color: Color3,
    pub font_size: f32,
}

impl Text {
    pub fn new(font_size: f32, text: Cow<'static, str>) -> Self {
        Self {
            text,
            color: Color3::WHITE,
            font_size,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            color: Color3::WHITE,
            font_size: 14.0,
        }
    }

    pub fn show(self) -> Response<TextWidget> {
        widget::<TextWidget>(self)
    }
}

pub struct TextWidget {
    props: Text,
    layout: RefCell<Layout>,
}

pub type TextResponse = ();

impl Widget for TextWidget {
    type Props = Text;
    type Response = TextResponse;

    fn new(props: Self::Props) -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props,
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let global = dom.get_global_or_init(TextGlobalState::new);

        let (max_width, max_height) = if input.is_bounded() {
            (
                Some(input.max.x * layout.scale_factor()),
                Some(input.max.y * layout.scale_factor()),
            )
        } else {
            (None, None)
        };

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            ..LayoutSettings::default()
        });

        text_layout.append(
            &[global.default_font.as_ref()],
            &TextStyle::new(
                &self.props.text,
                self.props.font_size * layout.scale_factor(),
                0,
            ),
        );

        let mut size = Vec2::ZERO;

        for glyph in text_layout.glyphs() {
            let max = Vec2::new(glyph.x + glyph.width as f32, glyph.y + glyph.height as f32)
                / layout.scale_factor();
            size = size.max(max);
        }

        input.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let global = dom.get_global_or_init(TextGlobalState::new);

        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(paint);

        let layout_node = layout.get(dom.current()).unwrap();

        for glyph in text_layout.glyphs() {
            let tex_rect = glyph_cache
                .get_or_insert(paint, &global.default_font, glyph.key)
                .as_rect()
                .div_vec2(glyph_cache.texture_size.as_vec2());

            let size = Vec2::new(glyph.width as f32, glyph.height as f32) / layout.scale_factor();
            let pos = layout_node.rect.pos() + Vec2::new(glyph.x, glyph.y) / layout.scale_factor();

            let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
            rect.color = self.props.color;
            rect.texture = Some((glyph_cache.texture.unwrap(), tex_rect));
            rect.pipeline = Pipeline::Text;
            paint.add_rect(rect);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

impl fmt::Debug for TextWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextComponent")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}
