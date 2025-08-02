use core::cell::Cell;

use yakui::event::{EventInterest, EventResponse, WidgetEvent};
use yakui::layout::{AbstractClipRect, ClipLogic};
use yakui::util::widget;
use yakui::widget::{EventContext, LayoutContext, Widget};
use yakui::widgets::List;
use yakui::{
    button, colored_box_container, column, constrained, label, Alignment, Color, Constraints, Dim2,
    Flow, MainAxisSize, Pivot, Response, Vec2,
};

#[derive(Debug)]
struct Tooltip {
    text: String,
    show: bool,
}

impl Tooltip {
    pub fn show(self) -> Response<()> {
        widget::<TooltipWidget>(self)
    }
}

#[derive(Debug)]
struct InnerTooltipWidget {}

impl Widget for InnerTooltipWidget {
    type Props<'a> = String;

    type Response = ();

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        colored_box_container(Color::BLACK, || {
            label(props);
        });
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.set_clip_logic(
            ctx.dom,
            ClipLogic::Contain {
                it: AbstractClipRect::LayoutRect,
                parent: AbstractClipRect::ParentRect,
            },
        );

        self.default_layout(ctx, constraints)
    }
}

#[derive(Debug)]
struct TooltipWidget {
    pos: Cell<Vec2>,
    offset: Cell<Vec2>,
    size: Cell<Vec2>,
    show: bool,
}

impl Widget for TooltipWidget {
    type Props<'a> = Tooltip;

    type Response = ();

    fn new() -> Self {
        Self {
            pos: Cell::default(),
            offset: Cell::default(),
            size: Cell::default(),
            show: false,
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseMoved(Some(pos)) => {
                self.pos.set(*pos);

                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }

    fn flow(&self) -> Flow {
        let offset = self.offset.get();

        Flow::Relative {
            anchor: Alignment::TOP_LEFT,
            offset: Dim2::pixels(offset.x, offset.y),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.show = props.show;

        let mut container = List::row();
        container.main_axis_size = MainAxisSize::Min;

        container.show(|| {
            colored_box_container(Color::BLACK, || {
                label(props.text);
            });
        });
    }

    fn paint(&self, ctx: yakui::widget::PaintContext<'_>) {
        if self.show {
            self.default_paint(ctx);
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.set_clip_logic(
            ctx.dom,
            ClipLogic::Contain {
                it: AbstractClipRect::LayoutRect,
                parent: AbstractClipRect::ParentRect,
            },
        );

        {
            let size = self.size.get();
            let pos = self.pos.get();

            if size != Vec2::ZERO {
                if let Some(parent) = ctx
                    .dom
                    .get_current()
                    .parent
                    .and_then(|id| ctx.layout.get(id))
                {
                    self.offset
                        .set(pos - parent.rect.pos() - size * Pivot::TOP_CENTER.as_vec2());
                }
            }
        }

        let size = self.default_layout(ctx, constraints);
        self.size.set(size);
        size
    }
}

pub fn run() {
    static TOOLTIP: &str =
        "reference: 'yak song' by Vylet Pony. haha get it, yak? yakui? fine okay.";

    column(|| {
        constrained(Constraints::tight(Vec2::new(800.0, 400.0)), || {
            colored_box_container(Color::GRAY, || {
                column(|| {
                    label("hiiiiiii, hover over the button below for a helpful tooltip!!! <3");

                    let hovering =
                        button("ill never sing yak song like my father before me").hovering;

                    Tooltip {
                        text: TOOLTIP.to_string(),
                        show: hovering,
                    }
                    .show();
                });
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
