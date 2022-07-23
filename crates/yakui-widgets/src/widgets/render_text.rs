use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle as FontdueTextStyle,
};
use yakui_core::dom::Dom;
use yakui_core::geometry::{Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::font::Fonts;
use crate::style::{TextAlignment, TextStyle};
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

/**
Draws text.

Responds with [RenderTextResponse].

## Examples
```rust
# let _handle = yakui_widgets::DocTest::start();
# use yakui::widgets::RenderText;
yakui::label("Default text label style");

yakui::text(32.0, "Custom font size");

let mut text = RenderText::new(32.0, "Title");
text.style.color = yakui::Color3::RED;
text.show();
```
*/
#[derive(Debug)]
#[non_exhaustive]
pub struct RenderText {
    pub text: Cow<'static, str>,
    pub style: TextStyle,
}

impl RenderText {
    pub fn new<S: Into<Cow<'static, str>>>(font_size: f32, text: S) -> Self {
        let mut style = TextStyle::label();
        style.font_size = font_size;

        Self {
            text: text.into(),
            style,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            style: TextStyle::label(),
        }
    }

    pub fn show(self) -> Response<RenderTextWidget> {
        widget::<RenderTextWidget>(self)
    }
}

pub struct RenderTextWidget {
    props: RenderText,
    layout: RefCell<Layout>,
}

pub type RenderTextResponse = ();

impl Widget for RenderTextWidget {
    type Props = RenderText;
    type Response = RenderTextResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: RenderText::new(0.0, Cow::Borrowed("")),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let fonts = dom.get_global_or_init(Fonts::default);

        let font = match fonts.get(&self.props.style.font) {
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

        let fontdue_align = match self.props.style.align {
            TextAlignment::Start => HorizontalAlign::Left,
            TextAlignment::Center => HorizontalAlign::Center,
            TextAlignment::End => HorizontalAlign::Right,
        };

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            horizontal_align: fontdue_align,
            ..LayoutSettings::default()
        });

        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(
                &self.props.text,
                self.props.style.font_size * layout.scale_factor(),
                0,
            ),
        );

        let size = get_text_layout_size(&text_layout, layout.scale_factor());

        input.constrain_min(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let fonts = dom.get_global_or_init(Fonts::default);
        let global = dom.get_global_or_init(TextGlobalState::new);

        let font = match fonts.get(&self.props.style.font) {
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
            rect.color = self.props.style.color;
            rect.texture = Some((glyph_cache.texture.unwrap(), tex_rect));
            rect.pipeline = Pipeline::Text;
            paint.add_rect(rect);
        }
    }
}

impl fmt::Debug for RenderTextWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextComponent")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}

pub(crate) fn get_text_layout_size(text_layout: &Layout, scale_factor: f32) -> Vec2 {
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

    Vec2::new(width, height) / scale_factor
}
