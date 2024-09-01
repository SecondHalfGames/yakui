use std::fmt;

use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::colors;
use crate::util::widget;
use crate::widgets::Pad;

/**
A floating window within the application.

Responds with [WindowResponse].
*/
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Window {
    pub initial_size: Vec2,
    children: Option<Box<dyn Fn()>>,
}

impl Window {
    pub fn new<S: Into<Vec2>>(initial_size: S) -> Self {
        Self {
            initial_size: initial_size.into(),
            children: None,
        }
    }

    pub fn show<F: 'static + Fn()>(mut self, children: F) -> Response<WindowResponse> {
        self.children = Some(Box::new(children));
        widget::<WindowWidget>(self)
    }
}

#[derive(Debug)]
pub struct WindowWidget {
    props: Window,
}

pub type WindowResponse = ();

impl Widget for WindowWidget {
    type Props<'a> = Window;
    type Response = WindowResponse;

    fn new() -> Self {
        Self {
            props: Window::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        crate::colored_box_container(colors::BACKGROUND_2, || {
            crate::column(|| {
                // Window Title Bar
                let constraints = Constraints::loose(self.props.initial_size);
                crate::constrained(constraints, || {
                    crate::pad(Pad::all(8.0), || {
                        crate::row(|| {
                            crate::colored_box(Color::BLUE, [16.0, 16.0]);
                            crate::expanded(|| {
                                crate::pad(Pad::balanced(8.0, 0.0), || {
                                    crate::text(16.0, "Yakui Window");
                                });
                            });
                            crate::colored_box(Color::RED, [16.0, 16.0]);
                        });
                    });
                });

                // Window Contents
                crate::constrained(constraints, || {
                    if let Some(children) = &self.props.children {
                        children();
                    }
                });
            });
        });
    }
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("size", &self.initial_size)
            .finish_non_exhaustive()
    }
}
