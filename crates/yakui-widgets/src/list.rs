use yakui_core::{
    context::Context,
    dom::{Dom, LayoutDom},
    draw, Component, Constraints, Index, Vec2,
};

use crate::Direction;

#[derive(Debug)]
pub struct List {
    index: Index,
    props: ListProps,
}

#[derive(Debug, Clone)]
pub struct ListProps {
    pub direction: Direction,
    pub item_spacing: f32,
}

pub type ListResponse = ();

impl Component for List {
    type Props = ListProps;
    type Response = ListResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();

        let mut constraints = Constraints {
            min: Vec2::new(input.max.x, 0.0),
            max: Vec2::new(input.max.x, input.max.y),
        };

        let mut size = Vec2::new(constraints.max.x, constraints.min.y);

        for (index, &child) in node.children.iter().enumerate() {
            let child_size = layout.calculate(dom, child, constraints);

            let child_node = layout.get_mut(child).unwrap();
            child_node.rect.set_pos(Vec2::new(0.0, size.y));

            constraints.max.y -= child_size.y;
            constraints.max.y -= self.props.item_spacing;
            constraints.max.y = constraints.max.y.max(0.0);
            size.y += child_size.y;

            if index < node.children.len() - 1 {
                size.y += self.props.item_spacing;
            }
        }

        input.constrain(size)
    }

    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut draw::Output) {
        let node = dom.get(self.index).unwrap();

        for &index in &node.children {
            let child = dom.get(index).unwrap();
            child.component.draw(dom, layout, output);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

pub fn vertical<F>(children: F) -> ListResponse
where
    F: FnOnce(),
{
    let context = Context::active();

    let index = context
        .borrow_mut()
        .dom_mut()
        .begin_component::<List>(ListProps {
            direction: Direction::Down,
            item_spacing: 8.0,
        });

    children();

    let res = context.borrow_mut().dom_mut().end_component::<List>(index);
    res
}
