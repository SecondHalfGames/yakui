#[test]
fn basic() {
    let mut state = yakui::State::new();
    state.register::<yakui::Layout, _>();
    state.register::<yakui::FixedSizeBox, _>();

    state.start();
    my_ui([40.0, 30.0]);
    state.finish();
    println!("{state:#?}");

    state.start();
    my_ui([2.0, 70.0]);
    state.finish();
    println!("{state:#?}");

    panic!("show me!!");
}

fn my_ui(size: [f32; 2]) {
    yakui::vertical(|| {
        yakui::fsbox(size);
        yakui::fsbox([30.0, 20.0]);
    });
}
