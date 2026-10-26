use std::ops::Deref;

use crate::geometry::Rect;

use super::PaintCall;

/// Contains all of the draw calls for a single layer of the UI.
#[derive(Debug)]
pub struct PaintLayer {
    /// The draw calls that can be used to paint this layer.
    pub calls: Vec<(Rect, PaintCall)>,
}

impl PaintLayer {
    /// Create a new, empty paint layer.
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }
}

/// Contains all of the paint layers that should be drawn, as well as
/// information about which layer is currently actively being drawn to, if any.
#[derive(Debug)]
pub struct PaintLayers {
    layers: Vec<PaintLayer>,
    layer_stack: Vec<usize>,
}

impl PaintLayers {
    /// Create a new set of paint layers.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            layer_stack: Vec::new(),
        }
    }

    /// Clear all paint layers.
    pub fn clear(&mut self) {
        self.layers.clear();
        self.layer_stack.clear();
    }

    /// Returns a reference to the currently active layer, if there is one.
    pub fn current(&self) -> Option<&PaintLayer> {
        self.layer_stack
            .last()
            .and_then(|index| self.layers.get(*index))
    }

    /// Returns mutable access to the currently active layer, if there is one.
    pub fn current_mut(&mut self) -> Option<&mut PaintLayer> {
        self.layer_stack
            .last()
            .and_then(|index| self.layers.get_mut(*index))
    }

    /// Push a new paint layer into the set. Newly added layers will be drawn on
    /// top of old ones.
    pub fn push(&mut self) {
        let index = self.layers.len();
        self.layers.push(PaintLayer::new());
        self.layer_stack.push(index);
    }

    /// Pop the most recently pushed paint layer. This should always be paired
    /// with a call to `push`.
    pub fn pop(&mut self) {
        let top = self.layer_stack.pop();
        debug_assert!(
            top.is_some(),
            "cannot call PaintLayers::pop without a corresponding push call"
        );
    }
}

impl Deref for PaintLayers {
    type Target = [PaintLayer];

    fn deref(&self) -> &Self::Target {
        self.layers.deref()
    }
}
