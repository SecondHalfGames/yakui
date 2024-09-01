use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{
    CoordinateSystem, HorizontalAlign as FontdueAlign, Layout, LayoutSettings,
    TextStyle as FontdueTextStyle,
};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::{PaintRect, Pipeline};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::font::{FontName, Fonts};
use crate::style::{TextAlignment, TextStyle};
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

/**
Renders text. You probably want to use [Text][super::Text] instead, which
supports features like padding.
*/
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
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

    pub fn show(self) -> Response<RenderTextResponse> {
        widget::<RenderTextWidget>(self)
    }
}

pub struct RenderTextWidget {
    props: RenderText,
    layout: RefCell<Layout>,
}

pub type RenderTextResponse = ();

impl Widget for RenderTextWidget {
    type Props<'a> = RenderText;
    type Response = RenderTextResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: RenderText::new(0.0, Cow::Borrowed("")),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        let font = match fonts.get(&self.props.style.font) {
            Some(font) => font,
            None => {
                // TODO: Log once that we were unable to find this font.
                panic!(
                    "font `{}` was set, but was not registered",
                    self.props.style.font
                );
            }
        };

        let max_width = input
            .max
            .x
            .is_finite()
            .then_some(input.max.x * ctx.layout.scale_factor());
        let max_height = input
            .max
            .y
            .is_finite()
            .then_some(input.max.y * ctx.layout.scale_factor());

        let horizontal_align = match self.props.style.align {
            TextAlignment::Start => FontdueAlign::Left,
            TextAlignment::Center => FontdueAlign::Center,
            TextAlignment::End => FontdueAlign::Right,
        };

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            horizontal_align,
            ..LayoutSettings::default()
        });

        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(
                &self.props.text,
                (self.props.style.font_size * ctx.layout.scale_factor()).ceil(),
                0,
            ),
        );

        let offset_x = get_text_layout_offset_x(&text_layout, ctx.layout.scale_factor());

        let size = get_text_layout_size(&text_layout, ctx.layout.scale_factor())
            - Vec2::new(offset_x, 0.0);

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let text_layout = self.layout.borrow_mut();
        let offset_x = get_text_layout_offset_x(&text_layout, ctx.layout.scale_factor());
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        paint_text(
            &mut ctx,
            &self.props.style.font,
            layout_node.rect.pos() - Vec2::new(offset_x, 0.0),
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

pub(crate) fn get_text_layout_offset_x(text_layout: &Layout, scale_factor: f32) -> f32 {
    let offset_x = text_layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.x)
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    offset_x / scale_factor
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
    ctx: &mut PaintContext<'_>,
    font: &FontName,
    pos: Vec2,
    text_layout: &Layout,
    color: Color,
) {
    let pos = pos.round();
    let fonts = ctx.dom.get_global_or_init(Fonts::default);
    let font = match fonts.get(font) {
        Some(font) => font,
        None => return,
    };

    let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);
    let mut glyph_cache = text_global.glyph_cache.borrow_mut();
    glyph_cache.ensure_texture(ctx.paint);

    for glyph in text_layout.glyphs() {
        let tex_rect = glyph_cache
            .get_or_insert(ctx.paint, &font, glyph.key)
            .as_rect()
            .div_vec2(glyph_cache.texture_size.as_vec2());

        let size = Vec2::new(glyph.width as f32, glyph.height as f32) / ctx.layout.scale_factor();
        let pos = pos + Vec2::new(glyph.x, glyph.y) / ctx.layout.scale_factor();

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = color;
        rect.texture = Some((glyph_cache.texture.unwrap().into(), tex_rect));
        rect.pipeline = Pipeline::Text;
        rect.add(ctx.paint);
    }
}
