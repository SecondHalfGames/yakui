#[test]
fn basic() {
    let mut state = yakui::State::new();
    state.register::<yakui::List>();
    state.register::<yakui::FixedSizeBox>();

    state.start();
    my_ui([40.0, 30.0]);
    state.finish();
    println!("{state:#?}");

    state.layout(yakui::Constraints {
        min: None,
        max: Some(yakui::Vec2::new(800.0, 600.0)),
    });

    state.start();
    my_ui([2.0, 70.0]);
    state.finish();

    state.layout(yakui::Constraints {
        min: None,
        max: Some(yakui::Vec2::new(800.0, 600.0)),
    });

    println!("{state:#?}");

    panic!("show me!!");
}

fn my_ui(size: [f32; 2]) {
    yakui::vertical(|| {
        yakui::fsbox(size);
        yakui::fsbox([30.0, 20.0]);
    });
}
