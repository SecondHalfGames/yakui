use yakui_core::{
    context::Context,
    dom::{Dom, LayoutDom},
    draw, Color3, Component, Constraints, Index, Vec2,
};

#[derive(Debug)]
pub struct Button {
    index: Index,
    props: ButtonProps,
}

#[derive(Debug, Clone)]
pub struct ButtonProps {
    pub size: Vec2,
    pub fill: Color3,
}

#[derive(Debug)]
pub struct ButtonResponse {}

impl Component for Button {
    type Props = ButtonProps;
    type Response = ButtonResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, _dom: &Dom, _layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        constraints.constrain(self.props.size)
    }

    fn draw(&self, _dom: &Dom, layout: &LayoutDom, output: &mut draw::Output) {
        let node = layout.get(self.index).unwrap();
        let viewport = layout.viewport;
        let size = node.rect.size() / viewport.size();
        let pos = (node.rect.pos() + viewport.pos()) / viewport.size();

        output.rect(pos, size, self.props.fill);
    }

    fn respond(&mut self) -> Self::Response {
        Self::Response {}
    }
}

pub fn button(props: ButtonProps) -> ButtonResponse {
    let context = Context::active();

    let index = context
        .borrow_mut()
        .dom_mut()
        .begin_component::<Button>(props);

    let res = context
        .borrow_mut()
        .dom_mut()
        .end_component::<Button>(index);
    res
}
