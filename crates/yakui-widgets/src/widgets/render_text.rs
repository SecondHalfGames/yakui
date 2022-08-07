use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle as FontdueTextStyle};
use yakui_core::dom::Dom;
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::font::{FontName, Fonts};
use crate::style::TextStyle;
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

/**
Renders text. You probably want to use [Text][super::Text] instead, which
supports features like padding.
*/
#[derive(Debug)]
#[non_exhaustive]
pub struct RenderText {
    pub text: Cow<'static, str>,
    pub style: TextStyle,
}

impl RenderText {
    pub fn new(size: f32, text: Cow<'static, str>) -> Self {
        let mut style = TextStyle::label();
        style.font_size = size;

        Self { text, style }
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

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            ..LayoutSettings::default()
        });

        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(
                &self.props.text,
                (self.props.style.font_size * layout.scale_factor()).ceil(),
                0,
            ),
        );

        let size = get_text_layout_size(&text_layout, layout.scale_factor());

        input.constrain_min(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let text_layout = self.layout.borrow_mut();
        let layout_node = layout.get(dom.current()).unwrap();
        paint_text(
            dom,
            layout,
            paint,
            &self.props.style.font,
            layout_node.rect.pos(),
            &text_layout,
            self.props.style.color,
        );
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
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    let width = text_layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.x + glyph.width as f32)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    Vec2::new(width, height) / scale_factor
}

pub fn paint_text(
    dom: &Dom,
    layout: &LayoutDom,
    paint: &mut PaintDom,
    font: &FontName,
    pos: Vec2,
    text_layout: &Layout,
    color: Color,
) {
    let pos = pos.round();
    let fonts = dom.get_global_or_init(Fonts::default);
    let font = match fonts.get(font) {
        Some(font) => font,
        None => return,
    };

    let text_global = dom.get_global_or_init(TextGlobalState::new_late_binding);
    let mut glyph_cache = text_global.glyph_cache.borrow_mut();

    for glyph in text_layout.glyphs() {
        let (texture_id, tex_rect) = glyph_cache.get_or_insert(paint, &font, glyph.key);

        let tex_rect = tex_rect
            .as_rect()
            .div_vec2(glyph_cache.texture_size(&font, &glyph.key).as_vec2());

        let size = Vec2::new(glyph.width as f32, glyph.height as f32) / layout.scale_factor();
        let pos = pos + Vec2::new(glyph.x, glyph.y) / layout.scale_factor();

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = color;
        rect.texture = Some((texture_id, tex_rect));
        rect.pipeline = Pipeline::Text;
        paint.add_rect(rect);
    }
}
