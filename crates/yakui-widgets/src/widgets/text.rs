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

use crate::font::{FontName, Fonts};
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

/**
Draws text.

Responds with [TextResponse].

## Examples
```rust
# let _handle = yakui_widgets::DocTest::start();
# use yakui::widgets::Text;
yakui::label("Default text label style");

yakui::text(32.0, "Custom font size");

let mut text = Text::new(32.0, "Title");
text.color = yakui::Color3::RED;
text.show();
```
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Text {
    pub text: Cow<'static, str>,
    pub color: Color3,
    pub font: FontName,
    pub font_size: f32,
}

impl Text {
    pub fn new<S: Into<Cow<'static, str>>>(font_size: f32, text: S) -> Self {
        Self {
            text: text.into(),
            color: Color3::WHITE,
            font: FontName::new("default"),
            font_size,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            color: Color3::WHITE,
            font: FontName::new("default"),
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

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: Text::new(0.0, Cow::Borrowed("")),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let fonts = dom.get_global_or_init(Fonts::default);

        let font = match fonts.get(&self.props.font) {
            Some(font) => font,
            None => {
                // TODO: Log once that we were unable to find this font.
                return input.min;
            }
        };

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
            &[&*font],
            &TextStyle::new(
                &self.props.text,
                self.props.font_size * layout.scale_factor(),
                0,
            ),
        );

        let height = text_layout
            .lines()
            .iter()
            .flat_map(|line_pos_vec| line_pos_vec.iter())
            .map(|line| line.baseline_y - line.min_descent)
            .max_by(|a, b| a.total_cmp(&b))
            .unwrap_or_default();

        let width = text_layout
            .glyphs()
            .iter()
            .map(|glyph| glyph.x + glyph.width as f32)
            .max_by(|a, b| a.total_cmp(&b))
            .unwrap_or_default();

        let size = Vec2::new(width, height) / layout.scale_factor();

        input.constrain_min(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let fonts = dom.get_global_or_init(Fonts::default);
        let global = dom.get_global_or_init(TextGlobalState::new);

        let font = match fonts.get(&self.props.font) {
            Some(font) => font,
            None => return,
        };

        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(paint);

        let layout_node = layout.get(dom.current()).unwrap();

        for glyph in text_layout.glyphs() {
            let tex_rect = glyph_cache
                .get_or_insert(paint, &font, glyph.key)
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
}

impl fmt::Debug for TextWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextComponent")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}
