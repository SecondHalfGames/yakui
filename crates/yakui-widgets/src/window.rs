use std::fmt;

use yakui_core::{Color3, Vec2, Widget};

use crate::util::widget;
use crate::Pad;

pub struct Window {
    pub size: Vec2,
    children: Option<Box<dyn Fn()>>,
}

impl Window {
    pub fn new<S: Into<Vec2>>(size: S) -> Self {
        Self {
            size: size.into(),
            children: None,
        }
    }

    pub fn show<F: 'static + Fn()>(mut self, children: F) -> WindowResponse {
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
    type Props = Window;
    type Response = WindowResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn respond(&mut self) -> Self::Response {}

    fn children(&self) {
        crate::colored_box_container(Color3::GRAY, || {
            crate::column(|| {
                crate::colored_box_container(Color3::rgb(92, 92, 92), || {
                    crate::pad(Pad::even(8.0), || {
                        let mut row = crate::List::horizontal();
                        row.item_spacing = 8.0;
                        row.show(|| {
                            // TODO: Expanded
                            crate::text(16.0, "Yakui Window");
                            crate::colored_box(Color3::RED, [16.0, 16.0]);
                        });
                    });
                });

                if let Some(children) = &self.props.children {
                    children()
                }
            });
        });
    }
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}
