use yakui_core::widget::Widget;
use yakui_core::Yakui;

#[derive(Debug)]
struct TestWidget;

impl Widget for TestWidget {
    type Props = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props) -> Self::Response {}
}

/// Root cause of https://github.com/LPGhatguy/yakui/issues/38
#[test]
fn layout_nodes_leak() {
    let mut yak = Yakui::new();

    yak.start();
    yak.dom().do_widget::<TestWidget>(());
    yak.dom().do_widget::<TestWidget>(());
    yak.finish();

    let dom = yak.dom();
    let layout = yak.layout_dom();
    assert_eq!(dom.len(), 3); // Two of our nodes, plus the root node.
    assert_eq!(layout.len(), 3);

    yak.start();
    yak.finish();

    let dom = yak.dom();
    let layout = yak.layout_dom();
    assert_eq!(dom.len(), 1); // Just the root node now
    assert_eq!(layout.len(), 1);
}
