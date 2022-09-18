use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle as FontdueTextStyle};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::font::Fonts;
use crate::style::TextStyle;
use crate::util::widget;

use super::render_text::{get_text_layout_size, paint_text};

/**
Rendering and layout logic for a textbox, holding no state.

Responds with [RenderTextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RenderTextBox {
    pub text: String,
    pub style: TextStyle,
    pub selected: bool,
    pub cursor: usize,
}

impl RenderTextBox {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
            selected: false,
            cursor: 0,
        }
    }

    pub fn show(self) -> Response<RenderTextBoxWidget> {
        widget::<RenderTextBoxWidget>(self)
    }
}

pub struct RenderTextBoxWidget {
    props: RenderTextBox,
    cursor_pos_size: RefCell<(Vec2, f32)>,
    layout: RefCell<Layout>,
}

pub type RenderTextBoxResponse = ();

impl Widget for RenderTextBoxWidget {
    type Props = RenderTextBox;
    type Response = RenderTextBoxResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: RenderTextBox::new(""),
            cursor_pos_size: RefCell::new((Vec2::ZERO, 0.0)),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let font = match fonts.get(&self.props.style.font) {
            Some(font) => font,
            None => {
                // TODO: Log once that we were unable to find this font.
                return input.min;
            }
        };

        let text = &self.props.text;

        let (max_width, max_height) = if input.is_bounded() {
            (
                Some(input.max.x * ctx.layout.scale_factor()),
                Some(input.max.y * ctx.layout.scale_factor()),
            )
        } else {
            (None, None)
        };

        let font_size = (self.props.style.font_size * ctx.layout.scale_factor()).ceil();

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            ..LayoutSettings::default()
        });

        let before_cursor = &text[..self.props.cursor];
        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(before_cursor, font_size, 0),
        );

        let metrics = font.vertical_line_metrics(font_size);
        let ascent = metrics.map(|m| m.ascent).unwrap_or(font_size);
        let cursor_size = ascent;

        let line_height = text_layout
            .lines()
            .map(|lines| text_layout.height() / lines.len() as f32)
            .unwrap_or(0.0);
        let cursor_y = text_layout
            .lines()
            .map(|lines| (lines.len() - 1) as f32 * line_height)
            .unwrap_or(0.0);
        let cursor_x = text_layout
            .glyphs()
            .last()
            .map(|glyph| glyph.x + glyph.width as f32 + 1.0)
            .unwrap_or_default();
        let cursor_pos = Vec2::new(cursor_x, cursor_y) / ctx.layout.scale_factor();
        *self.cursor_pos_size.borrow_mut() = (cursor_pos, cursor_size);

        let after_cursor = &text[self.props.cursor..];
        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(after_cursor, font_size, 0),
        );

        let mut size = get_text_layout_size(&text_layout, ctx.layout.scale_factor());
        size = size.max(Vec2::new(0.0, ascent));

        input.constrain(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let text_layout = self.layout.borrow_mut();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        // FIXME: For some reason, these values are negative sometimes!
        let should_clip = layout_node.rect.size().x > 0.0 && layout_node.rect.size().y > 0.0;

        if should_clip {
            ctx.paint.push_clip(layout_node.rect);
        }

        paint_text(
            &mut ctx,
            &self.props.style.font,
            layout_node.rect.pos(),
            &text_layout,
            self.props.style.color,
        );

        if self.props.selected {
            let (pos, size) = *self.cursor_pos_size.borrow();

            let cursor_pos = layout_node.rect.pos() + pos;
            let cursor_size = Vec2::new(1.0, size);

            let mut rect = PaintRect::new(Rect::from_pos_size(cursor_pos, cursor_size));
            rect.color = Color::RED;
            ctx.paint.add_rect(rect);
        }

        if should_clip {
            ctx.paint.pop_clip();
        }
    }
}

impl fmt::Debug for RenderTextBoxWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderTextBoxWidget")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}
