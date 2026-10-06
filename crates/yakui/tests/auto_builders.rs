/// Test to make sure our auto_builders! helper macro works outside of `yakui-widgets` (aka have proper hygiene)
#[test]
fn test_auto_builders_hygiene() {
    #[allow(unused)]
    struct Foo {
        woof: String,
        meow: f32,
        chirp: u32,
        squeak: &'static str,
        beep: std::borrow::Cow<'static, bool>,
        boop: isize,
        hiss: String,
    }

    yakui_widgets::auto_builders!(Foo {
        woof: String,
        meow: f32,
        chirp: u32,
        squeak: &'static str,
        beep: std::borrow::Cow<'static, bool>,
        boop: isize,
        hiss: String,
    });
}
