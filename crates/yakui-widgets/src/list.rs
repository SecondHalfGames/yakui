use yakui_core::{dom::Dom, layout::LayoutDom, Constraints, Vec2, Widget};

use crate::util::widget_children;
use crate::{CrossAxisAlignment, Direction, MainAxisAlignment, MainAxisSize};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct List {
    pub direction: Direction,
    pub main_axis_size: MainAxisSize,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl List {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
        }
    }

    pub fn vertical() -> Self {
        Self::new(Direction::Down)
    }

    pub fn horizontal() -> Self {
        Self::new(Direction::Right)
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

    // This approach to layout is based on Flutter's Flex layout algorithm.
    //
    // https://api.flutter.dev/flutter/widgets/Flex-class.html#layout-algorithm
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();
        let direction = self.props.direction;

        let mut total_main_axis_size = 0.0;
        let mut max_cross_axis_size = 0.0;

        // First, we lay out any children that do not flex, giving them unbound
        // main axis constraints. This ensures that we don't unfairly squish
        // later widgets in the layout.
        //
        // Simultaneously, we'll track the total value of all flexible elements
        // so that we can divide the remaining space up later.
        let mut total_flex = 0;
        for &child_index in &node.children {
            let child = dom.get(child_index).unwrap();
            let flex = child.widget.flex();
            total_flex += flex;

            if flex != 0 {
                continue;
            }

            let constraints = Constraints {
                min: Vec2::ZERO,
                max: Vec2::splat(f32::INFINITY),
            };

            let size = layout.calculate(dom, child_index, constraints);
            total_main_axis_size += direction.get_main_axis(size);
            max_cross_axis_size = f32::max(max_cross_axis_size, direction.get_cross_axis(size));
        }

        // Next, lay out all flexible elements, giving them each some portion of
        // the remaining space.
        let remaining_main_axis =
            (direction.get_main_axis(input.max) - total_main_axis_size).max(0.0);
        for &child_index in &node.children {
            let child = dom.get(child_index).unwrap();
            let flex = child.widget.flex();

            if flex == 0 {
                continue;
            }

            let main_axis_size = flex as f32 * remaining_main_axis / total_flex as f32;

            let constraints = Constraints {
                min: Vec2::ZERO,
                max: main_axis_size + direction.only_cross_axis(input.max),
            };

            let size = layout.calculate(dom, child_index, constraints);
            total_main_axis_size += direction.get_main_axis(size);
            max_cross_axis_size = f32::max(max_cross_axis_size, direction.get_cross_axis(size));
        }

        // Finally, position all children based on the sizes applied above.
        let mut next_pos = Vec2::ZERO;
        for &child_index in &node.children {
            let child_layout = layout.get_mut(child_index).unwrap();
            child_layout.rect.set_pos(next_pos);

            next_pos += direction.only_main_axis(child_layout.rect.size());
        }

        input.constrain(direction.vec2(total_main_axis_size, max_cross_axis_size))
    }

    fn respond(&mut self) -> Self::Response {}
}
