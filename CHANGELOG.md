# yakui Changelog

## Unreleased Changes
**See <https://github.com/LPGhatguy/yakui/compare/v0.2.0...main>**

## 0.2.0 — 2022-09-17
This release features improved documentation and several major improvements to yakui's implementation and interface.

### yakui-core
* Added support for alpha blended colors.
* Added support for visual clipping.
* Changed `Widget` trait methods to accept a context object as their first parameter.
    * This enables widgets to know about their environment during lifecycle methods.
* Improved texture tracking API for renderers.

### yakui-widgets
* Added the `Scrollable`, `Draggable`, `Offset`, `NineSlice`, and `Slider` widgets.
* Added `MainAxisSize::Min` for flex containers.
* Added `CrossAxisAlignment` for flex containers.
* Added `MainAxisAlignment` for flex containers.
* Unified text styling APIs across several widgets.
* Improved many existing widgets, especially `TextBox`.
* Fixed many flexible layout cases.

### yakui-wgpu
* Add `yakui_wgpu::State::paint_with_encoder()` for encoding render commands to an existing `&mut wgpu::CommandEncoder`.

### yakui-winit
* Updated to winit 0.27.3

## 0.1.0 — 2022-07-17
* Initial release.
