use std::fmt;

use yakui_core::geometry::{Color3, Constraints, Vec2};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::colors;
use crate::util::widget;
use crate::widgets::{List, Pad};

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

    pub fn show<F: 'static + Fn()>(mut self, children: F) -> Response<WindowWidget> {
        self.children = Some(Box::new(children));
        widget::<WindowWidget>(self)
    }
}

#[derive(Debug)]
pub struct WindowWidget {
    props: Window,
    size: Vec2,
}

pub type WindowResponse = ();

impl Widget for WindowWidget {
    type Props = Window;
    type Response = WindowResponse;

    fn new(props: Self::Props) -> Self {
        Self {
            size: props.initial_size,
            props,
        }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn respond(&mut self) -> Self::Response {}

    fn children(&self) {
        crate::colored_box_container(colors::BACKGROUND_2, || {
            crate::column(|| {
                // Window Title Bar
                let constraints = Constraints::loose(self.size);
                crate::constrained(constraints, || {
                    crate::pad(Pad::all(8.0), || {
                        let row = List::horizontal();
                        row.show(|| {
                            crate::colored_box(Color3::BLUE, [16.0, 16.0]);
                            crate::expanded(|| {
                                crate::pad(Pad::balanced(8.0, 0.0), || {
                                    crate::text(16.0, "Yakui Window");
                                });
                            });
                            crate::colored_box(Color3::RED, [16.0, 16.0]);
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
