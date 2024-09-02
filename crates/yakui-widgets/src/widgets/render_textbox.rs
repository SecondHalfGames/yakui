use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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
#[must_use = "yakui widgets do nothing if you don't `show` them"]
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

    pub fn show(self) -> Response<RenderTextBoxResponse> {
        widget::<RenderTextBoxWidget>(self)
    }
}

pub struct RenderTextBoxWidget {
    props: RenderTextBox,
    cursor_pos_size: RefCell<(Vec2, f32)>,
    layout: Rc<RefCell<Layout>>,
}

pub struct RenderTextBoxResponse {
    /// The fontdue text layout from this text box. This layout will be reset
    /// and updated every time the widget updates.
    pub layout: Rc<RefCell<Layout>>,
}

impl Widget for RenderTextBoxWidget {
    type Props<'a> = RenderTextBox;
    type Response = RenderTextBoxResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: RenderTextBox::new(""),
            cursor_pos_size: RefCell::new((Vec2::ZERO, 0.0)),
            layout: Rc::new(RefCell::new(layout)),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
        RenderTextBoxResponse {
            layout: self.layout.clone(),
        }
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
        text_layout.append(&[&*font], &FontdueTextStyle::new(text, font_size, 0));

        let lines = text_layout.lines().map(|x| x.as_slice()).unwrap_or(&[]);
        let glyphs = text_layout.glyphs();

        // TODO: This code doesn't account for graphemes with multiple glyphs.
        // We should accumulate the total bounding box of all glyphs that
        // contribute to a given grapheme.
        let cursor_x = if self.props.cursor >= self.props.text.len() {
            // If the cursor is after the last character, we can position it at
            // the right edge of the last glyph.
            text_layout
                .glyphs()
                .last()
                .map(|glyph| glyph.x + glyph.width as f32 + 1.0)
        } else {
            // ...otherwise, we'll position the cursor just behind the next
            // character after the cursor.
            text_layout.glyphs().iter().find_map(|glyph| {
                if glyph.byte_offset != self.props.cursor {
                    return None;
                }

                Some(glyph.x - 2.0)
            })
        };

        let cursor_line = lines
            .iter()
            .find(|line| {
                let start_byte = glyphs[line.glyph_start].byte_offset;
                let end_byte = glyphs[line.glyph_end].byte_offset;
                self.props.cursor >= start_byte && self.props.cursor <= end_byte
            })
            .or_else(|| lines.last());
        let cursor_y = cursor_line
            .map(|line| line.baseline_y - line.max_ascent)
            .unwrap_or(0.0);

        let metrics = font.vertical_line_metrics(font_size);
        let ascent = metrics.map(|m| m.ascent).unwrap_or(font_size) / ctx.layout.scale_factor();
        let cursor_size = ascent;

        let cursor_pos = Vec2::new(cursor_x.unwrap_or(0.0), cursor_y) / ctx.layout.scale_factor();
        *self.cursor_pos_size.borrow_mut() = (cursor_pos, cursor_size);

        let mut size = get_text_layout_size(&text_layout, ctx.layout.scale_factor());
        size = size.max(Vec2::new(0.0, ascent));

        input.constrain(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let text_layout = self.layout.borrow_mut();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

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
            rect.add(ctx.paint);
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
