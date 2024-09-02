use yakui_core::geometry::{Constraints, FlexFit, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::{CrossAxisAlignment, Direction, Flow, MainAxisAlignment, MainAxisSize, Response};

use crate::util::widget_children;

/**
Lays out children in a single direction. Supports flex sizing.

This is one of the most common and useful layout widgets.

Responds with [ListResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
yakui::column(|| {
    yakui::label("on top");
    yakui::label("on bottom");
});

yakui::row(|| {
    yakui::label("left");
    yakui::label("right");
});
```
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct List {
    pub direction: Direction,
    /// Added space at the end of each item.
    pub item_spacing: f32,
    pub main_axis_size: MainAxisSize,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl List {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            item_spacing: 0.0,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
        }
    }

    pub fn column() -> Self {
        Self::new(Direction::Down)
    }

    pub fn row() -> Self {
        Self::new(Direction::Right)
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ListResponse> {
        widget_children::<ListWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ListWidget {
    props: List,
}

pub type ListResponse = ();

impl Widget for ListWidget {
    type Props<'a> = List;
    type Response = ListResponse;

    fn new() -> Self {
        Self { props: List::row() }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        let flex = if self.props.cross_axis_alignment == CrossAxisAlignment::Stretch {
            1
        } else {
            0
        };

        (flex, FlexFit::Tight)
    }

    // This approach to layout is based on Flutter's Flex layout algorithm.
    //
    // https://api.flutter.dev/flutter/widgets/Flex-class.html#layout-algorithm
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let direction = self.props.direction;

        let total_item_spacing =
            self.props.item_spacing * node.children.len().saturating_sub(1) as f32;

        let mut total_main_axis_size = total_item_spacing;
        let mut max_cross_axis_size = 0.0;

        let cross_axis_max = direction.get_cross_axis(input.max);
        let cross_axis_min = match self.props.cross_axis_alignment {
            CrossAxisAlignment::Stretch => cross_axis_max,
            _ => 0.0,
        };

        let mut main_axis_max = direction.get_main_axis(input.max);
        if main_axis_max.is_infinite() {
            main_axis_max = direction.get_main_axis(input.min);
        };

        // First, we lay out any children that do not flex, giving them unbound
        // main axis constraints. This ensures that we don't unfairly squish
        // later widgets in the layout.
        //
        // Simultaneously, we'll track the total value of all flexible elements
        // so that we can divide the remaining space up later.
        let mut total_flex = 0;
        for &child_index in &node.children {
            let child = ctx.dom.get(child_index).unwrap();
            let (flex, _fit) = child.widget.flex();
            total_flex += flex;

            if flex != 0 {
                continue;
            }

            if child.widget.flow() != Flow::Inline {
                continue;
            }

            let constraints = Constraints {
                min: direction.vec2(0.0, cross_axis_min),
                max: direction.vec2(f32::INFINITY, cross_axis_max),
            };

            let size = ctx.calculate_layout(child_index, constraints);
            total_main_axis_size += direction.get_main_axis(size);
            max_cross_axis_size = f32::max(max_cross_axis_size, direction.get_cross_axis(size));
        }

        // Next, lay out all flexible elements, giving them each some portion of
        // the remaining space based on their flex factor.
        let remaining_main_axis = (main_axis_max - total_main_axis_size).max(0.0);
        for &child_index in &node.children {
            let child = ctx.dom.get(child_index).unwrap();
            let (flex, fit) = child.widget.flex();

            if flex == 0 {
                continue;
            }

            if child.widget.flow() != Flow::Inline {
                continue;
            }

            let main_axis_size = flex as f32 * remaining_main_axis / total_flex as f32;

            // The maximum main axis size is based on the portion of the
            // remaining space that we allocated to this widget.
            //
            // We pass along the maximum cross axis size from our incoming
            // constraints.
            let constraints = match fit {
                FlexFit::Loose => Constraints {
                    min: direction.vec2(0.0, cross_axis_min),
                    max: direction.vec2(main_axis_size, cross_axis_max),
                },
                FlexFit::Tight => Constraints {
                    min: direction.vec2(main_axis_size, cross_axis_min),
                    max: direction.vec2(main_axis_size, cross_axis_max),
                },
            };

            let size = ctx.calculate_layout(child_index, constraints);
            total_main_axis_size += direction.get_main_axis(size);
            max_cross_axis_size = f32::max(max_cross_axis_size, direction.get_cross_axis(size));
        }

        let cross_size = max_cross_axis_size.max(direction.get_cross_axis(input.min));

        let main_axis_size = match self.props.main_axis_size {
            MainAxisSize::Min => total_main_axis_size,
            MainAxisSize::Max => {
                let main_max = direction.get_main_axis(input.max);

                if main_max.is_finite() {
                    f32::max(total_main_axis_size, main_max)
                } else {
                    total_main_axis_size
                }
            }
            other => unimplemented!("MainAxisSize::{other:?}"),
        };

        let container_size = input.constrain(direction.vec2(main_axis_size, cross_size));

        // We can lay out all children that are not part of the layout flow at
        // this point, now that we know the total size of the container.
        for &child_id in &node.children {
            let child = ctx.dom.get(child_id).unwrap();
            let flow = child.widget.flow();

            match flow {
                Flow::Inline => (),

                Flow::Relative { anchor, offset } => {
                    ctx.calculate_layout(child_id, Constraints::none());

                    let anchor = container_size * anchor.as_vec2();
                    let offset = offset.resolve(container_size);

                    let child_layout = ctx.layout.get_mut(child_id).unwrap();
                    child_layout.rect.set_pos(anchor + offset);
                }

                other => unimplemented!("Flow::{other:?}"),
            }
        }

        // Finally, position all children based on the sizes calculated above.
        let (leading_space, mut between_space) = match self.props.main_axis_alignment {
            MainAxisAlignment::Start => (0.0, 0.0),
            MainAxisAlignment::Center => ((main_axis_size - total_main_axis_size) / 2.0, 0.0),
            MainAxisAlignment::End => (main_axis_size - total_main_axis_size, 0.0),
            MainAxisAlignment::SpaceAround => {
                // avoid division by zero
                if node.children.is_empty() {
                    (0.0, 0.0)
                } else {
                    let between_space =
                        (main_axis_size - total_main_axis_size) / node.children.len() as f32;
                    (between_space * 0.5, between_space)
                }
            }
            MainAxisAlignment::SpaceBetween => {
                if node.children.len() <= 1 {
                    // We follow CSS spec and Flutter behavior (as the Flutter doc isn't explicit)
                    // of putting the first child at the start when there is only one
                    (0.0, 0.0)
                } else {
                    let between_space = (main_axis_size - total_main_axis_size)
                        / (node.children.len() as f32 - 1.0);
                    (0.0, between_space)
                }
            }
            MainAxisAlignment::SpaceEvenly => {
                let between_space =
                    (main_axis_size - total_main_axis_size) / (node.children.len() as f32 + 1.0);
                (between_space, between_space)
            }
            other => unimplemented!("MainAxisAlignment::{other:?}"),
        };
        between_space += self.props.item_spacing;

        let mut next_main = leading_space;

        for &child_index in &node.children {
            let child = ctx.dom.get(child_index).unwrap();
            if child.widget.flow() != Flow::Inline {
                continue;
            }

            let child_layout = ctx.layout.get_mut(child_index).unwrap();
            let child_size = child_layout.rect.size();
            let child_main = direction.get_main_axis(child_size);
            let child_cross = direction.get_cross_axis(child_size);

            let cross = match self.props.cross_axis_alignment {
                CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::Center => (cross_size - child_cross) / 2.0,
                CrossAxisAlignment::End => cross_size - child_cross,
                other => unimplemented!("CrossAxisAlignment::{other:?}"),
            };
            child_layout.rect.set_pos(direction.vec2(next_main, cross));

            next_main += child_main;
            next_main += between_space;
        }

        container_size
    }
}
