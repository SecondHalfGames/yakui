use yakui_core::{dom::Dom, layout::LayoutDom, Constraints, Vec2, Widget};

use crate::util::widget_children;
use crate::Direction;

#[derive(Debug, Clone)]
pub struct List {
    pub direction: Direction,
    pub item_spacing: f32,
}

impl List {
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Down,
            item_spacing: 0.0,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Right,
            item_spacing: 0.0,
        }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> ListResponse {
        widget_children::<ListWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ListWidget {
    props: List,
}

pub type ListResponse = ();

impl Widget for ListWidget {
    type Props = List;
    type Response = ListResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();

        let item_spacing = match self.props.direction {
            Direction::Down => Vec2::new(0.0, self.props.item_spacing),
            Direction::Right => Vec2::new(self.props.item_spacing, 0.0),
        };

        let mut constraints = match self.props.direction {
            Direction::Down => Constraints {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(input.max.x, input.max.y),
            },
            Direction::Right => Constraints {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(input.max.x, input.max.y),
            },
        };

        let mut next_pos = Vec2::new(0.0, 0.0);

        let mut size = Vec2::new(constraints.min.x, constraints.min.y);

        for (index, &child) in node.children.iter().enumerate() {
            let child_size = layout.calculate(dom, child, constraints);
            let child_node = layout.get_mut(child).unwrap();
            child_node.rect.set_pos(next_pos);

            match self.props.direction {
                Direction::Down => {
                    size.x = size.x.max(child_size.x);
                    size.y += child_size.y;

                    next_pos.y += child_size.y;

                    constraints.max.y -= child_size.y;
                    constraints.max.y = constraints.max.y.max(0.0);
                }
                Direction::Right => {
                    size.x += child_size.x;
                    size.y = size.y.max(child_size.y);

                    next_pos.x += child_size.x;

                    constraints.max.x -= child_size.x;
                    constraints.max.x = constraints.max.x.max(0.0);
                }
            }

            if index < node.children.len() - 1 {
                constraints.max -= item_spacing;
                size += item_spacing;
                next_pos += item_spacing;
            }
        }

        input.constrain(size)
    }

    fn respond(&mut self) -> Self::Response {}
}
