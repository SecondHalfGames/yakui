use core::cell::Cell;

use yakui::layout::{AbstractClipRect, ClipLogic};
use yakui::util::widget_children;
use yakui::widget::{LayoutContext, Widget};
use yakui::widgets::List;
use yakui::{
    button, colored_box_container, column, constrained, label, row, Alignment, Color, Constraints,
    CrossAxisAlignment, Dim2, Flow, MainAxisSize, Pivot, Rect, Response, Vec2,
};

#[derive(Debug)]
pub struct FollowCursor {
    offset: Vec2,
    pivot: Pivot,
}

impl Default for FollowCursor {
    fn default() -> Self {
        Self::new(Vec2::new(0.0, -5.0), Pivot::BOTTOM_CENTER)
    }
}

impl FollowCursor {
    pub fn new(offset: Vec2, pivot: Pivot) -> Self {
        Self { offset, pivot }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<()> {
        widget_children::<FollowCursorWidget, F>(children, self)
    }
}

#[derive(Debug)]
struct FollowCursorWidget {
    props: FollowCursor,
    mouse_pos: Cell<Option<Vec2>>,
    parent_rect: Cell<Rect>,
}

impl Widget for FollowCursorWidget {
    type Props<'a> = FollowCursor;

    type Response = ();

    fn new() -> Self {
        Self {
            props: FollowCursor::default(),
            mouse_pos: Cell::default(),
            parent_rect: Cell::new(Rect::ONE),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flow(&self) -> Flow {
        Flow::Relative {
            anchor: Alignment::TOP_LEFT,
            offset: Dim2::ZERO,
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);

        if let Some(pos) = ctx.input.mouse_pos(ctx.layout) {
            self.mouse_pos.set(Some(pos));
        }

        let node = ctx.dom.get_current();
        if let Some(parent) = ctx.layout.get(node.parent.unwrap()) {
            self.parent_rect.set(parent.rect);
        }

        let child_constraints = Constraints::loose(constraints.max);
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            size = size.max(ctx.calculate_layout(child, child_constraints));
        }

        if let Some(mouse_pos) = self.mouse_pos.get() {
            let parent_rect = self.parent_rect.get();
            let offset = mouse_pos - parent_rect.pos();
            let offset = offset - (size * self.props.pivot.as_vec2()) + self.props.offset;

            ctx.layout.set_clip_logic(
                ctx.dom,
                ClipLogic::Contain {
                    it: AbstractClipRect::LayoutRect,
                    parent: AbstractClipRect::ParentRect,
                    offset,
                },
            );

            size
        } else {
            ctx.layout.set_clip_logic(
                ctx.dom,
                ClipLogic::Override(AbstractClipRect::Value(Rect::ZERO)),
            );

            Vec2::ZERO
        }
    }
}

#[track_caller]
fn tooltip(follow_cursor: FollowCursor, text: String) {
    follow_cursor.show(|| {
        colored_box_container(Color::BLACK, || {
            label(text);
        });
    });
}

pub fn run() {
    static TOOLTIP: &str =
        "reference: 'yak song' by Vylet Pony. haha get it, yak? yakui? fine okay.";
    static SUPER_LONG_TOOLTIP: &str = "abaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaabaaba";

    tooltip(
        FollowCursor::new(Vec2::new(0.0, 60.0), Pivot::TOP_CENTER),
        "aaaa".to_string(),
    );

    column(|| {
        constrained(Constraints::tight(Vec2::new(800.0, 400.0)), || {
            colored_box_container(Color::GRAY, || {
                column(|| {
                    let mut button_hovering = false;

                    row(|| {
                        constrained(Constraints::tight(Vec2::new(200.0, 200.0)), || {
                            colored_box_container(Color::BLUE, || {
                                tooltip(FollowCursor::default(), SUPER_LONG_TOOLTIP.to_string());
                            });
                        });

                        constrained(Constraints::tight(Vec2::new(200.0, 200.0)), || {
                            colored_box_container(Color::REBECCA_PURPLE, || {
                                List::column()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .main_axis_size(MainAxisSize::Min)
                                    .show(|| {
                                        label("hiiiiiii\n\n\nhover over the button below for a helpful tooltip!!! <3");

                                        button_hovering = button(
                                            "ill never sing yak song like my father before me",
                                        )
                                        .hovering;
                                    });

                                if button_hovering {
                                    tooltip(FollowCursor::default(), TOOLTIP.to_string());
                                }
                            });
                        });
                    });

                    if !button_hovering {
                        tooltip(
                            FollowCursor::new(Vec2::new(0.0, 5.0), Pivot::TOP_CENTER),
                            "weeeeeeeeeeeeeeeeeeeeee".to_string(),
                        );
                    }
                });
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
