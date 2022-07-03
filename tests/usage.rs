#[test]
fn basic() {
    let mut state = yakui::State::new();

    state.start();

    yakui::vertical(|| {
        yakui::fsbox([40.0, 30.0]);
        yakui::fsbox([30.0, 20.0]);
    });

    state.finish();
}
