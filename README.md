# yakui
yakui is a declarative Rust UI library intended for games.

yakui combines a layout model inspired by [Flutter] with the ease-of-use of an immediate mode UI library like [Dear Imgui].

[Flutter]: https://flutter.dev/
[Dear Imgui]: https://github.com/ocornut/imgui

## Examples
A complete demo application is contained available in [`crates/demo`](crates/demo).

You can run an example with `cargo run <example name>`.

```rust
fn app() {
    yakui::column(|| {
        yakui::text(32.0, "Hello, world!");

        if yakui::button("Click me!").clicked {
            println!("Button clicked.");
        }
    })
}
```

## Crates
* `yakui`
* `yakui-core`
* `yakui-widgets`
* `yakui-winit`
* `yakui-wgpu`

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.