#[test]
fn basic() {
    let mut state = yakui::State::new();
    state.register::<yakui::Layout, _>();
    state.register::<yakui::FixedSizeBox, _>();

    state.start();
    my_ui();
    state.finish();

    panic!("{state:?}");
}

fn my_ui() {
    yakui::vertical(|| {
        yakui::fsbox([40.0, 30.0]);
        yakui::fsbox([30.0, 20.0]);
    });
}
